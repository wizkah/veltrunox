//! Endpoints for the stems of a recording (ADR-015).

use crate::{
    access::Caller,
    http::{ApiError, blocking},
    state::AppState,
    stem_analysis::{Job, JobState, StemTask},
    stem_package::{PackageError, StemStore, read_structure},
};
use axum::{
    Json, Router,
    body::Body,
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::IntoResponse,
    routing::{get, post},
};
use futures_util::StreamExt;
use serde::Deserialize;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use veltrunox_core::SplitStems;
use veltrunox_core::{Capability, Reason, RecordingId, Refusal};
use veltrunox_protocol::{
    ImportedStemsDto, PackageRecordingDto, STEM_ANALYSIS_PATH, STEM_PACKAGE_PATH,
    STEM_PACKAGES_PATH, STEM_REGIONS_PATH, STEM_WAVEFORM_LEVEL_PATH, STEM_WAVEFORM_PATH,
    STEMS_PATH, StemAnalysisDto, StemDto, StemListDto, WaveformInfoDto, peaks_to_bytes,
    waveform_info_to_dto,
};
use veltrunox_server_app::{Stem, StemError, list_recordings};

pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route(STEM_PACKAGES_PATH, post(import_package))
        .route(STEMS_PATH, get(stems).delete(remove_stems))
        .route(STEM_PACKAGE_PATH, get(export_package))
        .route(STEM_ANALYSIS_PATH, post(analyse))
        .route(STEM_REGIONS_PATH, post(create_regions))
        .route(STEM_WAVEFORM_PATH, get(waveform_info))
        .route(STEM_WAVEFORM_LEVEL_PATH, get(waveform_level))
}

fn store_of(app: &AppState) -> Result<Arc<StemStore>, ApiError> {
    app.stem_store.clone().ok_or_else(|| {
        ApiError::Refused(
            StatusCode::SERVICE_UNAVAILABLE,
            Reason::coded("stems_unavailable", "stems are not set up on this server"),
        )
    })
}

impl From<PackageError> for ApiError {
    fn from(error: PackageError) -> Self {
        match error {
            PackageError::Invalid(_) => Self::Refused(
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                Reason::coded("stems_package_invalid", &error.to_string()),
            ),
            PackageError::Io(_) => Self::Internal,
        }
    }
}

impl From<StemError> for ApiError {
    fn from(error: StemError) -> Self {
        let status = match error {
            StemError::UnknownRecording => StatusCode::NOT_FOUND,
            StemError::LengthMismatch | StemError::RateMismatch => StatusCode::UNPROCESSABLE_ENTITY,
            StemError::Storage(_) | StemError::Bookmark(_) => return Self::Internal,
        };
        Self::Refused(status, error.reason())
    }
}

async fn import_package(
    State(app): State<AppState>,
    caller: Caller,
    body: Body,
) -> Result<Json<ImportedStemsDto>, ApiError> {
    caller.require(Capability::ManageRecordings)?;
    let store = store_of(&app)?;
    let received = store.temporary_file("zip");
    receive(body, received.path(), store.max_bytes()).await?;
    blocking(move || {
        let package = store.read(received.path())?;
        let imported = {
            let _guard = store.lock();
            let set = store.keep(&package)?;
            app.stems
                .install(&package.content_hash, &set, &package.structure)
        };
        // Replaced stems, or the files of a refused package, are not referred to any more.
        let _ = store.sweep(app.stem_repository.as_ref());
        let imported = imported?;
        app.playback
            .reload_mix(imported.recording.id)
            .map_err(|_| ApiError::Internal)?;
        Ok(Json(ImportedStemsDto {
            recording: imported.recording.id.value(),
            title: imported.recording.title.as_str().to_owned(),
            stems: imported.stems as u32,
            regions_added: imported.regions_added as u32,
        }))
    })
    .await
}

async fn receive(body: Body, path: &std::path::Path, limit: u64) -> Result<(), ApiError> {
    let mut file = tokio::fs::File::create(path)
        .await
        .map_err(|_| ApiError::Internal)?;
    let mut received = 0_u64;
    let mut chunks = body.into_data_stream();
    while let Some(chunk) = chunks.next().await {
        let chunk = chunk.map_err(|_| {
            ApiError::Refused(
                StatusCode::BAD_REQUEST,
                Reason::coded("upload_interrupted", "the upload was interrupted"),
            )
        })?;
        received += chunk.len() as u64;
        if received > limit {
            let megabytes = limit >> 20;
            return Err(ApiError::Refused(
                StatusCode::PAYLOAD_TOO_LARGE,
                Reason::with_args(
                    &format!("the file is larger than the limit of {megabytes} MB"),
                    "file_too_large",
                    vec![megabytes],
                ),
            ));
        }
        file.write_all(&chunk)
            .await
            .map_err(|_| ApiError::Internal)?;
    }
    file.flush().await.map_err(|_| ApiError::Internal)
}

async fn stems(
    State(app): State<AppState>,
    caller: Caller,
    Path(recording): Path<u64>,
) -> Result<Json<StemListDto>, ApiError> {
    let _ = &caller;
    let id = RecordingId::new(recording);
    let jobs = app.stem_jobs.clone();
    let set = blocking(move || app.stems.stem_set(id).map_err(|_| ApiError::Internal)).await?;
    let job = jobs.as_ref().and_then(|jobs| jobs.state(id));
    let creating_regions = matches!(
        job,
        Some(Job {
            task: StemTask::Regions,
            ..
        })
    );
    let analysis = match job.map(|job| job.state) {
        None => StemAnalysisDto::None,
        Some(JobState::Queued) => StemAnalysisDto::Queued,
        Some(JobState::Running(percent)) => StemAnalysisDto::Running { percent },
        Some(JobState::Failed(code)) => StemAnalysisDto::Failed {
            code: code.to_owned(),
        },
    };
    Ok(Json(StemListDto {
        analysis_available: jobs.is_some(),
        analysis,
        creating_regions,
        stems: set
            .stems
            .iter()
            .map(|stem| StemDto {
                name: stem.name.as_str().to_owned(),
                key: stem
                    .path
                    .file_stem()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_default(),
                length_ms: stem.length.value(),
            })
            .collect(),
    }))
}

#[derive(Deserialize)]
struct AnalyseQuery {
    /// Into how many stems; four if not said.
    stems: Option<u8>,
}

impl AnalyseQuery {
    fn split(&self) -> Result<SplitStems, ApiError> {
        let Some(count) = self.stems else {
            return Ok(SplitStems::default());
        };
        SplitStems::from_count(count).ok_or_else(|| {
            ApiError::Refused(
                StatusCode::BAD_REQUEST,
                Reason::coded("stems_count_unsupported", "split into 4 or 6 stems"),
            )
        })
    }
}

async fn analyse(
    State(app): State<AppState>,
    caller: Caller,
    Path(recording): Path<u64>,
    Query(query): Query<AnalyseQuery>,
) -> Result<StatusCode, ApiError> {
    caller.require(Capability::ManageRecordings)?;
    let stems = query.split()?;
    let jobs = app.stem_jobs.clone().ok_or_else(no_program)?;
    let id = RecordingId::new(recording);
    let known = blocking(move || {
        Ok(list_recordings(app.recordings.as_ref())
            .map_err(|_| ApiError::Internal)?
            .iter()
            .any(|recording| recording.id == id))
    })
    .await?;
    if !known {
        return Err(ApiError::NotFound);
    }
    jobs.enqueue(id, StemTask::Split(stems));
    Ok(StatusCode::ACCEPTED)
}

/// Creates regions from the structure of the stems. A structure found before is used again.
async fn create_regions(
    State(app): State<AppState>,
    caller: Caller,
    Path(recording): Path<u64>,
) -> Result<StatusCode, ApiError> {
    caller.require(Capability::ManageRecordings)?;
    let id = RecordingId::new(recording);
    let jobs = app.stem_jobs.clone();
    let added = blocking(move || {
        let set = app.stems.stem_set(id).map_err(|_| ApiError::Internal)?;
        if set.stems.is_empty() {
            return Err(ApiError::NotFound);
        }
        let Some(analysis) = set.analysis else {
            return Ok(false);
        };
        let structure = read_structure(&analysis).map_err(|_| ApiError::Internal)?;
        if structure.sections.is_empty() {
            return Ok(false);
        }
        app.stems.add_structure(id, &structure, &analysis)?;
        Ok(true)
    })
    .await?;
    if added {
        return Ok(StatusCode::NO_CONTENT);
    }
    jobs.ok_or_else(no_program)?.enqueue(id, StemTask::Regions);
    Ok(StatusCode::ACCEPTED)
}

fn no_program() -> ApiError {
    ApiError::Refused(
        StatusCode::SERVICE_UNAVAILABLE,
        Reason::coded(
            "stems_analysis_unavailable",
            "no analysis program is set up on this server",
        ),
    )
}

async fn remove_stems(
    State(app): State<AppState>,
    caller: Caller,
    Path(recording): Path<u64>,
) -> Result<StatusCode, ApiError> {
    caller.require(Capability::ManageRecordings)?;
    let store = store_of(&app)?;
    blocking(move || {
        let id = RecordingId::new(recording);
        app.stems.remove(id).map_err(|_| ApiError::Internal)?;
        app.playback
            .reload_mix(id)
            .map_err(|_| ApiError::Internal)?;
        let _ = store.sweep(app.stem_repository.as_ref());
        Ok(StatusCode::NO_CONTENT)
    })
    .await
}

async fn export_package(
    State(app): State<AppState>,
    caller: Caller,
    Path(recording): Path<u64>,
) -> Result<impl IntoResponse, ApiError> {
    let _ = &caller;
    let store = store_of(&app)?;
    let id = RecordingId::new(recording);
    let bytes = blocking(move || {
        let title = list_recordings(app.recordings.as_ref())
            .map_err(|_| ApiError::Internal)?
            .into_iter()
            .find(|known| known.id == id)
            .ok_or(ApiError::NotFound)?
            .title;
        let content_hash = app
            .media
            .content_hash(id)
            .map_err(|_| ApiError::Internal)?
            .ok_or(ApiError::NotFound)?;
        // Held so that no replaced file is swept away while it is read.
        let _guard = store.lock();
        let set = app.stems.stem_set(id).map_err(|_| ApiError::Internal)?;
        if set.stems.is_empty() {
            return Err(ApiError::NotFound);
        }
        let written = store.write(
            PackageRecordingDto {
                content_hash,
                title: title.as_str().to_owned(),
            },
            &set,
        )?;
        std::fs::read(written.path()).map_err(|_| ApiError::Internal)
    })
    .await?;
    Ok(([(header::CONTENT_TYPE, "application/zip")], bytes))
}

fn stem_at(app: &AppState, recording: u64, index: usize) -> Result<Stem, ApiError> {
    app.stems
        .stem_set(RecordingId::new(recording))
        .map_err(|_| ApiError::Internal)?
        .stems
        .into_iter()
        .nth(index)
        .ok_or(ApiError::NotFound)
}

async fn waveform_info(
    State(app): State<AppState>,
    caller: Caller,
    Path((recording, index)): Path<(u64, usize)>,
) -> Result<Json<WaveformInfoDto>, ApiError> {
    let _ = &caller;
    let stem = blocking(move || stem_at(&app, recording, index)).await?;
    Ok(Json(waveform_info_to_dto(stem.length, &stem.waveform)))
}

async fn waveform_level(
    State(app): State<AppState>,
    caller: Caller,
    Path((recording, index, level)): Path<(u64, usize, usize)>,
) -> Result<impl IntoResponse, ApiError> {
    let _ = &caller;
    let stem = blocking(move || stem_at(&app, recording, index)).await?;
    let found = stem.waveform.levels.get(level).ok_or(ApiError::NotFound)?;
    Ok((
        [(header::CONTENT_TYPE, "application/octet-stream")],
        peaks_to_bytes(&found.peaks),
    ))
}
