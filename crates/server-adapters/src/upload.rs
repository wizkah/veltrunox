//! Upload of recordings (ADR-012): the file store and the endpoint.

use crate::{
    access::Caller,
    audio::probe_file,
    http::{ApiError, blocking},
    state::AppState,
};
use axum::{
    Json,
    body::Body,
    extract::State,
    http::{HeaderMap, StatusCode, header},
};
use futures_util::StreamExt;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::io::AsyncWriteExt;
use veltrunox_core::{Capability, RecordingTitle};
use veltrunox_protocol::{FILE_NAME_HEADER, UploadedRecordingDto};
use veltrunox_server_app::{MediaCatalog, StorageError};

/// Prefix of files that are still being received; they are never part of the library.
const TEMPORARY_PREFIX: &str = ".upload-";

/// Directory of the uploaded audio files and the size limit of one upload. Files are named by
/// the hash of their content.
#[derive(Debug, Clone)]
pub struct UploadStore {
    directory: PathBuf,
    max_bytes: u64,
}

impl UploadStore {
    /// Creates the directory if needed and removes what no recording refers to any more:
    /// leftovers of interrupted uploads and files of deleted recordings.
    pub fn open(
        directory: &Path,
        max_bytes: u64,
        catalog: &dyn MediaCatalog,
    ) -> Result<Self, StorageError> {
        fs::create_dir_all(directory).map_err(|error| StorageError(error.to_string()))?;
        let store = Self {
            directory: directory.to_owned(),
            max_bytes,
        };
        store.sweep(catalog)?;
        Ok(store)
    }

    pub fn max_bytes(&self) -> u64 {
        self.max_bytes
    }

    /// Whether the file lies in this store, so removing it is ours to do.
    pub fn owns(&self, path: &Path) -> bool {
        path.parent() == Some(self.directory.as_path())
    }

    fn sweep(&self, catalog: &dyn MediaCatalog) -> Result<usize, StorageError> {
        let referenced = catalog.media_paths()?;
        let entries =
            fs::read_dir(&self.directory).map_err(|error| StorageError(error.to_string()))?;
        let mut removed = 0;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && !referenced.contains(&path) && fs::remove_file(&path).is_ok() {
                removed += 1;
            }
        }
        Ok(removed)
    }

    fn temporary_path(&self, extension: Option<&str>) -> PathBuf {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_nanos());
        let name = format!(
            "{TEMPORARY_PREFIX}{nanos:x}-{}",
            COUNTER.fetch_add(1, Ordering::Relaxed)
        );
        self.directory.join(match extension {
            Some(extension) => format!("{name}.{extension}"),
            None => name,
        })
    }

    fn final_path(&self, hash: &str, extension: Option<&str>) -> PathBuf {
        self.directory.join(match extension {
            Some(extension) => format!("{hash}.{extension}"),
            None => hash.to_owned(),
        })
    }
}

/// Removes the file when dropped, unless it was handed over to the store.
struct TemporaryFile(Option<PathBuf>);

impl TemporaryFile {
    fn path(&self) -> &Path {
        self.0.as_deref().unwrap_or(Path::new(""))
    }

    fn keep(&mut self) {
        self.0 = None;
    }
}

impl Drop for TemporaryFile {
    fn drop(&mut self) {
        if let Some(path) = self.0.take() {
            let _ = fs::remove_file(path);
        }
    }
}

fn refused(status: StatusCode, message: &str) -> ApiError {
    ApiError::Refused(status, message.to_owned())
}

/// Receives one audio file as the raw request body.
pub(crate) async fn upload(
    State(app): State<AppState>,
    caller: Caller,
    headers: HeaderMap,
    body: Body,
) -> Result<(StatusCode, Json<UploadedRecordingDto>), ApiError> {
    caller.require(Capability::Upload)?;
    let store = app.uploads.clone().ok_or_else(|| {
        refused(
            StatusCode::SERVICE_UNAVAILABLE,
            "uploading is not set up on this server",
        )
    })?;
    let (title, extension) = file_name_of(&headers)?;
    let declared = headers
        .get(header::CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok());
    if declared.is_some_and(|size| size > store.max_bytes) {
        return Err(too_large(store.max_bytes));
    }

    let mut temporary = TemporaryFile(Some(store.temporary_path(extension.as_deref())));
    let hash = receive(body, &store, temporary.path()).await?;

    let probed = temporary.path().to_owned();
    let checked = blocking(move || {
        probe_file(&probed).map_err(|error| {
            refused(
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                &format!("this is not audio the server can read: {error}"),
            )
        })
    })
    .await;
    checked?;

    let catalog = app.media.clone();
    let final_path = store.final_path(&hash, extension.as_deref());
    let source = temporary.path().to_owned();
    let (recording, existing) = blocking(move || {
        if let Some(known) = catalog
            .recording_by_hash(&hash)
            .map_err(|_| ApiError::Internal)?
        {
            return Ok((known, true));
        }
        fs::rename(&source, &final_path).map_err(|_| ApiError::Internal)?;
        match catalog
            .add_media(title, &final_path, &hash)
            .map_err(|_| ApiError::Internal)?
        {
            Some(added) => Ok((added, false)),
            // Another upload of the same audio finished first and owns the file now.
            None => catalog
                .recording_by_hash(&hash)
                .map_err(|_| ApiError::Internal)?
                .map(|known| (known, true))
                .ok_or(ApiError::Internal),
        }
    })
    .await?;
    if !existing {
        // The file now lives in the store; a duplicate's temporary file is removed on drop.
        temporary.keep();
        app.analysis_queue.enqueue(recording.id);
    }
    let status = if existing {
        StatusCode::OK
    } else {
        StatusCode::CREATED
    };
    Ok((
        status,
        Json(UploadedRecordingDto {
            id: recording.id.value(),
            title: recording.title.as_str().to_owned(),
            existing,
        }),
    ))
}

fn too_large(limit: u64) -> ApiError {
    refused(
        StatusCode::PAYLOAD_TOO_LARGE,
        &format!("the file is larger than the limit of {} MB", limit >> 20),
    )
}

/// Writes the body to `path` and returns the SHA-256 of what was received, hex encoded.
async fn receive(body: Body, store: &UploadStore, path: &Path) -> Result<String, ApiError> {
    let mut file = tokio::fs::File::create(path)
        .await
        .map_err(|_| ApiError::Internal)?;
    let mut hasher = Sha256::new();
    let mut received = 0u64;
    let mut chunks = body.into_data_stream();
    while let Some(chunk) = chunks.next().await {
        let chunk =
            chunk.map_err(|_| refused(StatusCode::BAD_REQUEST, "the upload was interrupted"))?;
        received += chunk.len() as u64;
        if received > store.max_bytes {
            return Err(too_large(store.max_bytes));
        }
        hasher.update(&chunk);
        file.write_all(&chunk)
            .await
            .map_err(|_| ApiError::Internal)?;
    }
    file.flush().await.map_err(|_| ApiError::Internal)?;
    if received == 0 {
        return Err(refused(StatusCode::BAD_REQUEST, "the file is empty"));
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

/// Title and extension from the file name header (percent-encoded UTF-8, no directories).
fn file_name_of(headers: &HeaderMap) -> Result<(RecordingTitle, Option<String>), ApiError> {
    let encoded = headers
        .get(FILE_NAME_HEADER)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| refused(StatusCode::BAD_REQUEST, "the file name is missing"))?;
    let decoded = percent_decode(encoded)
        .ok_or_else(|| refused(StatusCode::BAD_REQUEST, "the file name is not valid"))?;
    let name = decoded
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or_default()
        .to_owned();
    let (stem, extension) = match name.rsplit_once('.') {
        Some((stem, extension)) if !stem.is_empty() => (stem, Some(extension)),
        _ => (name.as_str(), None),
    };
    let title = RecordingTitle::new(stem)
        .map_err(|_| refused(StatusCode::BAD_REQUEST, "the file name is empty"))?;
    // Only a plain extension is kept: it names the file in the store and hints the decoder.
    let extension = extension
        .filter(|extension| {
            (1..=8).contains(&extension.len())
                && extension
                    .chars()
                    .all(|letter| letter.is_ascii_alphanumeric())
        })
        .map(str::to_ascii_lowercase);
    Ok((title, extension))
}

fn percent_decode(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let pair = text.get(index + 1..index + 3)?;
            decoded.push(u8::from_str_radix(pair, 16).ok()?);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    fn named(value: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(FILE_NAME_HEADER, HeaderValue::from_str(value).unwrap());
        headers
    }

    fn title_and_extension(value: &str) -> (String, Option<String>) {
        match file_name_of(&named(value)) {
            Ok((title, extension)) => (title.as_str().to_owned(), extension),
            Err(_) => panic!("{value} should be accepted"),
        }
    }

    #[test]
    fn the_title_is_the_name_without_extension() {
        assert_eq!(
            title_and_extension("Blue%20Monday.MP3"),
            ("Blue Monday".to_owned(), Some("mp3".to_owned()))
        );
        assert_eq!(
            title_and_extension("Gr%C3%BC%C3%9Fe.flac").0,
            "Grüße".to_owned()
        );
    }

    #[test]
    fn directories_and_odd_extensions_are_dropped() {
        assert_eq!(
            title_and_extension("..%2F..%2Fetc%2Fpasswd.wav"),
            ("passwd".to_owned(), Some("wav".to_owned()))
        );
        assert_eq!(title_and_extension("take.one two").1, None);
        assert_eq!(
            title_and_extension("noextension"),
            ("noextension".to_owned(), None)
        );
        assert_eq!(title_and_extension(".hidden").1, None);
    }

    #[test]
    fn missing_or_empty_names_are_refused() {
        assert!(file_name_of(&HeaderMap::new()).is_err());
        assert!(file_name_of(&named("%20.wav")).is_err());
        assert!(file_name_of(&named("%zz.wav")).is_err());
        assert!(file_name_of(&named("%ff.wav")).is_err());
    }
}
