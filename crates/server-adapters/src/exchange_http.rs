//! HTTP endpoints to move bookmarks in and out: export of the library, import with preview.

use crate::{
    access::Caller,
    bookmark_csv::{export_to_csv, import_from_csv},
    http::{ApiError, blocking},
    state::AppState,
};
use axum::{
    Json, Router,
    body::Bytes,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::Deserialize;
use veltrunox_core::{Capability, ConflictKind, EntryRef, Reason, RecordingId, Refusal};
use veltrunox_protocol::{
    EXPORT_FORMAT_VERSION, EXPORT_PATH, ExportDto, ExportedRecordingDto, IMPORT_PATH,
    IMPORT_SCHEMA_PATH, ImportDto, ImportIssueDto, ImportOutcomeDto, ImportResultDto,
    ImportStatusDto, RECORDING_EXPORT_PATH, ReasonDto, bookmarks_to_dto, entry_path, import_schema,
    imported_bookmarks_from_dto, library_entry_to_dto,
};
use veltrunox_server_app::{BookmarkError, ImportMode, ImportOutcome, list_recordings};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(EXPORT_PATH, get(export_library))
        .route(RECORDING_EXPORT_PATH, get(export_recording))
        .route(IMPORT_PATH, post(import_bookmarks))
        .route(IMPORT_SCHEMA_PATH, get(schema))
}

#[derive(Deserialize)]
struct ExportQuery {
    /// `json` (default) or `csv`.
    format: Option<String>,
}

#[derive(Deserialize)]
struct ImportQuery {
    #[serde(default)]
    dry_run: bool,
    #[serde(default)]
    overwrite: bool,
}

fn bad_request(code: &str, message: &str, args: Vec<u64>) -> ApiError {
    ApiError::Refused(
        StatusCode::BAD_REQUEST,
        Reason::with_args(&message, code, args),
    )
}

/// Every recording with its details and bookmarks.
async fn export_library(
    State(app): State<AppState>,
    caller: Caller,
    Query(query): Query<ExportQuery>,
) -> Result<Response, ApiError> {
    let _ = &caller;
    let recordings = blocking(move || exported_recordings(&app, None)).await?;
    export_response(recordings, query.format.as_deref())
}

/// One recording with its details and bookmarks.
async fn export_recording(
    State(app): State<AppState>,
    caller: Caller,
    Path(recording): Path<u64>,
    Query(query): Query<ExportQuery>,
) -> Result<Response, ApiError> {
    let _ = &caller;
    let id = RecordingId::new(recording);
    let recordings = blocking(move || exported_recordings(&app, Some(id))).await?;
    if recordings.is_empty() {
        return Err(ApiError::NotFound);
    }
    export_response(recordings, query.format.as_deref())
}

fn export_response(
    recordings: Vec<ExportedRecordingDto>,
    format: Option<&str>,
) -> Result<Response, ApiError> {
    let export = ExportDto {
        format_version: EXPORT_FORMAT_VERSION,
        recordings,
    };
    match format {
        None | Some("json") => Ok(Json(export).into_response()),
        Some("csv") => Ok((
            [(header::CONTENT_TYPE, "text/csv; charset=utf-8")],
            export_to_csv(&export),
        )
            .into_response()),
        Some(_) => Err(bad_request(
            "export_format_unknown",
            "the export format must be json or csv",
            Vec::new(),
        )),
    }
}

fn exported_recordings(
    app: &AppState,
    only: Option<RecordingId>,
) -> Result<Vec<ExportedRecordingDto>, ApiError> {
    let mut listed = list_recordings(app.recordings.as_ref()).map_err(|_| ApiError::Internal)?;
    listed.retain(|recording| only.is_none_or(|id| recording.id == id));
    let entries = app
        .analysis
        .entries(listed)
        .map_err(|_| ApiError::Internal)?;
    entries
        .iter()
        .map(|entry| {
            let bookmarks = app.bookmarks.list(entry.recording.id)?;
            Ok(ExportedRecordingDto {
                recording: library_entry_to_dto(entry),
                bookmarks: bookmarks_to_dto(&bookmarks),
            })
        })
        .collect()
}

/// The JSON schema of an import.
async fn schema(caller: Caller) -> impl IntoResponse {
    let _ = &caller;
    Json(import_schema())
}

/// Imports bookmarks as JSON (`ImportDto`) or, with the content type `text/csv`, as a table.
/// Every recording is its own unit: one that is invalid, in conflict or unknown is reported
/// and left as it was, the others are applied (or only previewed with `dry_run`).
async fn import_bookmarks(
    State(app): State<AppState>,
    caller: Caller,
    Query(query): Query<ImportQuery>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<ImportResultDto>, ApiError> {
    caller.require(Capability::EditBookmarks)?;
    let import = read_import(&headers, &body)?;
    let mode = ImportMode {
        dry_run: query.dry_run,
        overwrite: query.overwrite,
    };
    let recordings = blocking(move || {
        import
            .recordings
            .iter()
            .map(|entry| import_one(&app, entry, mode))
            .collect()
    })
    .await?;
    Ok(Json(ImportResultDto {
        dry_run: mode.dry_run,
        recordings,
    }))
}

fn read_import(headers: &HeaderMap, body: &[u8]) -> Result<ImportDto, ApiError> {
    let is_csv = headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.trim_start().starts_with("text/csv"));
    let import = if is_csv {
        let text = std::str::from_utf8(body)
            .map_err(|_| bad_request("import_malformed", "the table is not UTF-8 text", vec![]))?;
        import_from_csv(text)
            .map_err(|error| bad_request("csv_invalid", &error.to_string(), vec![error.line]))?
    } else {
        serde_json::from_slice(body)
            .map_err(|error| bad_request("import_malformed", &error.to_string(), vec![]))?
    };
    if import.format_version != EXPORT_FORMAT_VERSION {
        return Err(bad_request(
            "import_format_unsupported",
            "this format version is not supported",
            vec![u64::from(import.format_version)],
        ));
    }
    Ok(import)
}

fn import_one(
    app: &AppState,
    entry: &veltrunox_protocol::ImportRecordingDto,
    mode: ImportMode,
) -> Result<ImportOutcomeDto, ApiError> {
    let recording_id = entry.recording.id;
    let outcome = |status, issues| ImportOutcomeDto {
        recording_id,
        status,
        added: 0,
        changed: 0,
        unchanged: 0,
        removed: 0,
        issues,
    };
    let issue = |entry: EntryRef, reason: Reason| ImportIssueDto {
        entry: entry_path(entry),
        reason: ReasonDto::from(&reason),
    };
    let incoming = match imported_bookmarks_from_dto(&entry.bookmarks) {
        Ok(incoming) => incoming,
        Err(issues) => {
            let issues = issues
                .into_iter()
                .map(|found| issue(found.entry, found.error.reason()))
                .collect();
            return Ok(outcome(ImportStatusDto::Invalid, issues));
        }
    };
    match app
        .bookmarks
        .import(RecordingId::new(recording_id), &incoming, mode)
    {
        Err(BookmarkError::NotFound) => Ok(outcome(ImportStatusDto::UnknownRecording, vec![])),
        Err(error) => Err(error.into()),
        Ok(ImportOutcome::Outside(entries)) => {
            let reason = veltrunox_core::InvalidValue::OutsideRecording.reason();
            let issues = entries
                .into_iter()
                .map(|found| issue(found, reason.clone()))
                .collect();
            Ok(outcome(ImportStatusDto::Invalid, issues))
        }
        Ok(ImportOutcome::Conflict(conflict)) => {
            let reason = match conflict.kind {
                ConflictKind::Missing => BookmarkError::NotFound.reason(),
                ConflictKind::Stale(current) => BookmarkError::Conflict(current).reason(),
            };
            Ok(outcome(
                ImportStatusDto::Conflict,
                vec![issue(conflict.entry, reason)],
            ))
        }
        Ok(ImportOutcome::Done { diff, .. }) => Ok(ImportOutcomeDto {
            added: count(diff.added),
            changed: count(diff.changed),
            unchanged: count(diff.unchanged),
            removed: count(diff.removed),
            ..outcome(ImportStatusDto::Ok, vec![])
        }),
    }
}

fn count(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}
