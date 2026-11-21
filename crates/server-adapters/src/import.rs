//! Reads audio files from a directory into the library until upload exists.

use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::Path;
use veltrunox_core::RecordingTitle;
use veltrunox_server_app::{MediaCatalog, StorageError};

/// Formats the decoder is expected to read (ADR-006).
const EXTENSIONS: &[&str] = &["wav", "mp3", "flac", "aif", "aiff", "m4a", "ogg"];

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("cannot read the media directory: {0}")]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Storage(#[from] StorageError),
}

/// Adds one recording per audio file whose content is not known yet; returns how many.
/// Files are taken in name order, so the library order is stable.
pub fn import_directory(
    directory: &Path,
    catalog: &dyn MediaCatalog,
) -> Result<usize, ImportError> {
    let mut files: Vec<_> = fs::read_dir(directory)?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && has_audio_extension(path))
        .collect();
    files.sort();
    let mut added = 0;
    for path in files {
        let Some(title) = title_of(&path) else {
            continue;
        };
        let hash = content_hash(&path)?;
        let absolute = fs::canonicalize(&path)?;
        if catalog.add_media(title, &absolute, &hash)?.is_some() {
            added += 1;
        }
    }
    Ok(added)
}

fn has_audio_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str()))
}

fn title_of(path: &Path) -> Option<RecordingTitle> {
    RecordingTitle::new(path.file_stem()?.to_str()?).ok()
}

fn content_hash(path: &Path) -> std::io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut block = vec![0; 1 << 16];
    loop {
        let read = file.read(&mut block)?;
        if read == 0 {
            break;
        }
        hasher.update(&block[..read]);
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SqliteStore;
    use veltrunox_server_app::RecordingRepository;

    fn directory(name: &str) -> std::path::PathBuf {
        let path =
            std::env::temp_dir().join(format!("veltrunox-import-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn imports_audio_files_once_and_ignores_the_rest() {
        let dir = directory("once");
        fs::write(dir.join("b-song.wav"), b"two").unwrap();
        fs::write(dir.join("a-song.MP3"), b"one").unwrap();
        fs::write(dir.join("notes.txt"), b"text").unwrap();
        fs::write(dir.join("copy.flac"), b"one").unwrap();
        let store = SqliteStore::in_memory().unwrap();

        assert_eq!(import_directory(&dir, &store).unwrap(), 2);
        let titles: Vec<_> = store
            .list()
            .unwrap()
            .iter()
            .map(|recording| recording.title.as_str().to_owned())
            .collect();
        assert_eq!(titles, ["a-song", "b-song"]);

        assert_eq!(import_directory(&dir, &store).unwrap(), 0);
        let first = store.list().unwrap()[0].id;
        let path = store.media_path(first).unwrap().unwrap();
        assert!(path.ends_with("a-song.MP3"));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_missing_directory_is_an_error() {
        let store = SqliteStore::in_memory().unwrap();
        assert!(import_directory(Path::new("/nonexistent-veltrunox"), &store).is_err());
    }
}
