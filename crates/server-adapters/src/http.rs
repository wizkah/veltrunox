//! HTTP endpoints: library, upload, recording administration, waveforms and bookmarks.

use crate::{
    access::{AccessError, Caller},
    accounts_http,
    events::events,
    exchange_http,
    state::AppState,
    tokens_http,
    upload::upload,
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post, put},
};
use veltrunox_core::RecordingTitle;
use veltrunox_core::{
    BookmarkId, BookmarkName, Capability, InvalidValue, Millis, Reason, RecordingId, Refusal,
    Version,
};
use veltrunox_protocol::{
    ANALYSIS_PATH, BOOKMARKS_PATH, BookmarksDto, ConflictDto, EVENTS_PATH, ExpectedVersionQuery,
    LABEL_PATH, LABEL_TO_REGION_PATH, LABELS_PATH, LabelDto, MERGE_REGIONS_PATH, MergeRegionsDto,
    NewLabelDto, NewRegionDto, RECORDING_PATH, RECORDINGS_PATH, REGION_PATH, REGION_TO_LABEL_PATH,
    REGIONS_PATH, ReasonDto, RecordingListDto, RegionDto, RenameRecordingDto, ReplaceBookmarksDto,
    UpdateLabelDto, UpdateRegionDto, WAVEFORM_LEVEL_PATH, WAVEFORM_PATH, WaveformInfoDto,
    bookmarks_to_dto, label_to_dto, library_entry_to_dto, new_bookmarks_from_dto, peaks_to_bytes,
    range_from_dto, region_to_dto, waveform_info_to_dto,
};
use veltrunox_server_app::{AccountError, BookmarkError, PlaybackBroadcast, list_recordings};

/// Builds the HTTP routes of the server.
pub fn router(app: AppState) -> Router {
    Router::new()
        .route(RECORDINGS_PATH, get(recordings).post(upload))
        .route(
            RECORDING_PATH,
            put(rename_recording).delete(delete_recording),
        )
        .route(EVENTS_PATH, get(events))
        .route(ANALYSIS_PATH, post(analyse_again))
        .route(WAVEFORM_PATH, get(waveform_info))
        .route(WAVEFORM_LEVEL_PATH, get(waveform_level))
        .route(BOOKMARKS_PATH, get(bookmarks).put(replace_bookmarks))
        .route(LABELS_PATH, post(add_label))
        .route(REGIONS_PATH, post(add_region))
        .route(LABEL_PATH, put(update_label).delete(delete_label))
        .route(REGION_PATH, put(update_region).delete(delete_region))
        .route(MERGE_REGIONS_PATH, post(merge_regions))
        .route(LABEL_TO_REGION_PATH, post(label_to_region))
        .route(REGION_TO_LABEL_PATH, post(region_to_label))
        .merge(accounts_http::routes())
        .merge(exchange_http::routes())
        .merge(tokens_http::routes())
        .with_state(app)
}

/// Why a request failed, mapped to an HTTP status in one place.
pub(crate) enum ApiError {
    Invalid(InvalidValue),
    Access(AccessError),
    Account(AccountError),
    Bookmark(BookmarkError),
    /// Refused with a reason the client can word.
    Refused(StatusCode, Reason),
    NotFound,
    Internal,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            Self::Invalid(error) => refusal(StatusCode::BAD_REQUEST, &error.reason()),
            Self::Access(error) => error.into_response(),
            Self::Account(error) => accounts_http::account_error_response(error),
            Self::Refused(status, reason) => refusal(status, &reason),
            Self::Bookmark(BookmarkError::NotFound) | Self::NotFound => {
                StatusCode::NOT_FOUND.into_response()
            }
            Self::Bookmark(BookmarkError::Invalid(error)) => {
                refusal(StatusCode::BAD_REQUEST, &error.reason())
            }
            Self::Bookmark(BookmarkError::Conflict(current)) => (
                StatusCode::CONFLICT,
                Json(ConflictDto {
                    current_version: current.value(),
                }),
            )
                .into_response(),
            Self::Bookmark(BookmarkError::Storage(_)) | Self::Internal => {
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

/// An error answer: the status with the reason as JSON.
pub(crate) fn refusal(status: StatusCode, reason: &Reason) -> Response {
    (status, Json(ReasonDto::from(reason))).into_response()
}

impl From<InvalidValue> for ApiError {
    fn from(error: InvalidValue) -> Self {
        Self::Invalid(error)
    }
}

impl From<AccessError> for ApiError {
    fn from(error: AccessError) -> Self {
        Self::Access(error)
    }
}

impl From<AccountError> for ApiError {
    fn from(error: AccountError) -> Self {
        Self::Account(error)
    }
}

impl From<BookmarkError> for ApiError {
    fn from(error: BookmarkError) -> Self {
        Self::Bookmark(error)
    }
}

/// Runs blocking storage work off the async runtime threads.
pub(crate) async fn blocking<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, ApiError> + Send + 'static,
) -> Result<T, ApiError> {
    tokio::task::spawn_blocking(work)
        .await
        .map_err(|_| ApiError::Internal)?
}

async fn recordings(
    State(app): State<AppState>,
    caller: Caller,
) -> Result<Json<RecordingListDto>, ApiError> {
    let _ = &caller;
    let entries = blocking(move || {
        let listed = list_recordings(app.recordings.as_ref()).map_err(|_| ApiError::Internal)?;
        app.analysis.entries(listed).map_err(|_| ApiError::Internal)
    })
    .await?;
    Ok(Json(RecordingListDto {
        recordings: entries.iter().map(library_entry_to_dto).collect(),
    }))
}

async fn rename_recording(
    State(app): State<AppState>,
    caller: Caller,
    Path(recording): Path<u64>,
    Json(body): Json<RenameRecordingDto>,
) -> Result<StatusCode, ApiError> {
    caller.require(Capability::ManageRecordings)?;
    let title = RecordingTitle::new(&body.title)
        .map_err(|error| ApiError::Refused(StatusCode::BAD_REQUEST, error.reason()))?;
    let id = RecordingId::new(recording);
    let hub = app.hub.clone();
    blocking(move || match app.recordings.rename(id, title) {
        Ok(true) => Ok(()),
        Ok(false) => Err(ApiError::NotFound),
        Err(_) => Err(ApiError::Internal),
    })
    .await?;
    // The library entry changed; clients read it again.
    hub.library_changed();
    Ok(StatusCode::NO_CONTENT)
}

/// Removes a recording with everything that belongs to it. If it is loaded, playback stops and
/// nothing stays selected. The audio file goes too when the upload store owns it; files of an
/// imported directory are never touched.
async fn delete_recording(
    State(app): State<AppState>,
    caller: Caller,
    Path(recording): Path<u64>,
) -> Result<StatusCode, ApiError> {
    caller.require(Capability::ManageRecordings)?;
    let id = RecordingId::new(recording);
    let hub = app.hub.clone();
    blocking(move || {
        let path = app.media.media_path(id).map_err(|_| ApiError::Internal)?;
        // Unload first: no client may start the recording once it is going away.
        app.playback.unload(id);
        if !app.recordings.delete(id).map_err(|_| ApiError::Internal)? {
            return Err(ApiError::NotFound);
        }
        if let (Some(path), Some(uploads)) = (path, app.uploads.as_ref())
            && uploads.owns(&path)
        {
            let _ = std::fs::remove_file(path);
        }
        Ok(())
    })
    .await?;
    hub.library_changed();
    Ok(StatusCode::NO_CONTENT)
}

/// Starts the analysis of a recording again; the result arrives through the library.
async fn analyse_again(
    State(app): State<AppState>,
    caller: Caller,
    Path(recording): Path<u64>,
) -> Result<StatusCode, ApiError> {
    caller.require(Capability::ManageRecordings)?;
    let id = RecordingId::new(recording);
    let exists = blocking(move || {
        let listed = list_recordings(app.recordings.as_ref()).map_err(|_| ApiError::Internal)?;
        Ok((listed.iter().any(|known| known.id == id), app))
    })
    .await?;
    match exists {
        (true, app) => {
            app.analysis_queue.enqueue(id);
            Ok(StatusCode::ACCEPTED)
        }
        (false, _) => Ok(StatusCode::NOT_FOUND),
    }
}

/// Levels and length of the waveform; 404 until the recording has been scanned.
async fn waveform_info(
    State(app): State<AppState>,
    caller: Caller,
    Path(recording): Path<u64>,
) -> Result<Json<WaveformInfoDto>, ApiError> {
    let _ = &caller;
    let id = RecordingId::new(recording);
    let (length, waveform) = blocking(move || stored_waveform(&app, id)).await?;
    Ok(Json(waveform_info_to_dto(length, &waveform)))
}

/// The peaks of one level, coarsest level 0.
async fn waveform_level(
    State(app): State<AppState>,
    caller: Caller,
    Path((recording, level)): Path<(u64, usize)>,
) -> Result<impl IntoResponse, ApiError> {
    let _ = &caller;
    let id = RecordingId::new(recording);
    let (_, waveform) = blocking(move || stored_waveform(&app, id)).await?;
    let found = waveform.levels.get(level).ok_or(ApiError::NotFound)?;
    Ok((
        [(header::CONTENT_TYPE, "application/octet-stream")],
        peaks_to_bytes(&found.peaks),
    ))
}

fn stored_waveform(
    app: &AppState,
    id: RecordingId,
) -> Result<(Millis, veltrunox_core::Waveform), ApiError> {
    let waveform = app.waveforms.waveform(id).map_err(|_| ApiError::Internal)?;
    let length = app
        .waveforms
        .stored_length(id)
        .map_err(|_| ApiError::Internal)?;
    waveform
        .zip(length)
        .map(|(w, l)| (l, w))
        .ok_or(ApiError::NotFound)
}

async fn bookmarks(
    State(app): State<AppState>,
    caller: Caller,
    Path(recording): Path<u64>,
) -> Result<Json<BookmarksDto>, ApiError> {
    let _ = &caller;
    let found = blocking(move || Ok(app.bookmarks.list(RecordingId::new(recording))?)).await?;
    Ok(Json(bookmarks_to_dto(&found)))
}

/// Replaces all labels and regions of a recording, or none of them.
async fn replace_bookmarks(
    State(app): State<AppState>,
    caller: Caller,
    Path(recording): Path<u64>,
    Json(body): Json<ReplaceBookmarksDto>,
) -> Result<Json<BookmarksDto>, ApiError> {
    caller.require(Capability::EditBookmarks)?;
    let replacement = new_bookmarks_from_dto(&body)?;
    let stored = blocking(move || {
        Ok(app
            .bookmarks
            .replace_all(RecordingId::new(recording), &replacement)?)
    })
    .await?;
    Ok(Json(bookmarks_to_dto(&stored)))
}

async fn add_label(
    State(app): State<AppState>,
    caller: Caller,
    Path(recording): Path<u64>,
    Json(body): Json<NewLabelDto>,
) -> Result<(StatusCode, Json<LabelDto>), ApiError> {
    caller.require(Capability::EditBookmarks)?;
    let name = BookmarkName::new(&body.name)?;
    let position = Millis::position(body.position_ms)?;
    let label = blocking(move || {
        Ok(app
            .bookmarks
            .add_label(RecordingId::new(recording), name, position)?)
    })
    .await?;
    Ok((StatusCode::CREATED, Json(label_to_dto(&label))))
}

async fn update_label(
    State(app): State<AppState>,
    caller: Caller,
    Path(id): Path<u64>,
    Json(body): Json<UpdateLabelDto>,
) -> Result<Json<LabelDto>, ApiError> {
    caller.require(Capability::EditBookmarks)?;
    let name = BookmarkName::new(&body.name)?;
    let position = Millis::position(body.position_ms)?;
    let expected = Version::new(body.expected_version);
    let label = blocking(move || {
        Ok(app
            .bookmarks
            .update_label(BookmarkId::new(id), expected, name, position)?)
    })
    .await?;
    Ok(Json(label_to_dto(&label)))
}

async fn delete_label(
    State(app): State<AppState>,
    caller: Caller,
    Path(id): Path<u64>,
    Query(query): Query<ExpectedVersionQuery>,
) -> Result<StatusCode, ApiError> {
    caller.require(Capability::EditBookmarks)?;
    let expected = Version::new(query.expected_version);
    blocking(move || Ok(app.bookmarks.delete_label(BookmarkId::new(id), expected)?)).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn add_region(
    State(app): State<AppState>,
    caller: Caller,
    Path(recording): Path<u64>,
    Json(body): Json<NewRegionDto>,
) -> Result<(StatusCode, Json<RegionDto>), ApiError> {
    caller.require(Capability::EditBookmarks)?;
    let name = BookmarkName::new(&body.name)?;
    let range = range_from_dto(body.range)?;
    let region = blocking(move || {
        Ok(app
            .bookmarks
            .add_region(RecordingId::new(recording), name, range)?)
    })
    .await?;
    Ok((StatusCode::CREATED, Json(region_to_dto(&region))))
}

async fn update_region(
    State(app): State<AppState>,
    caller: Caller,
    Path(id): Path<u64>,
    Json(body): Json<UpdateRegionDto>,
) -> Result<Json<RegionDto>, ApiError> {
    caller.require(Capability::EditBookmarks)?;
    let name = BookmarkName::new(&body.name)?;
    let range = range_from_dto(body.range)?;
    let expected = Version::new(body.expected_version);
    let region = blocking(move || {
        Ok(app
            .bookmarks
            .update_region(BookmarkId::new(id), expected, name, range)?)
    })
    .await?;
    Ok(Json(region_to_dto(&region)))
}

async fn delete_region(
    State(app): State<AppState>,
    caller: Caller,
    Path(id): Path<u64>,
    Query(query): Query<ExpectedVersionQuery>,
) -> Result<StatusCode, ApiError> {
    caller.require(Capability::EditBookmarks)?;
    let expected = Version::new(query.expected_version);
    blocking(move || Ok(app.bookmarks.delete_region(BookmarkId::new(id), expected)?)).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn label_to_region(
    State(app): State<AppState>,
    caller: Caller,
    Path(id): Path<u64>,
    Json(body): Json<UpdateRegionDto>,
) -> Result<Json<RegionDto>, ApiError> {
    caller.require(Capability::EditBookmarks)?;
    let name = BookmarkName::new(&body.name)?;
    let range = range_from_dto(body.range)?;
    let expected = Version::new(body.expected_version);
    let region = blocking(move || {
        Ok(app
            .bookmarks
            .convert_label_to_region(BookmarkId::new(id), expected, name, range)?)
    })
    .await?;
    Ok(Json(region_to_dto(&region)))
}

async fn merge_regions(
    State(app): State<AppState>,
    caller: Caller,
    Path(recording): Path<u64>,
    Json(body): Json<MergeRegionsDto>,
) -> Result<Json<RegionDto>, ApiError> {
    caller.require(Capability::EditBookmarks)?;
    let name = BookmarkName::new(&body.name)?;
    let range = range_from_dto(body.range)?;
    let regions: Vec<_> = body
        .regions
        .iter()
        .map(|region| {
            (
                BookmarkId::new(region.id),
                Version::new(region.expected_version),
            )
        })
        .collect();
    let region = blocking(move || {
        Ok(app
            .bookmarks
            .merge_regions(RecordingId::new(recording), &regions, name, range)?)
    })
    .await?;
    Ok(Json(region_to_dto(&region)))
}

async fn region_to_label(
    State(app): State<AppState>,
    caller: Caller,
    Path(id): Path<u64>,
    Json(body): Json<UpdateLabelDto>,
) -> Result<Json<LabelDto>, ApiError> {
    caller.require(Capability::EditBookmarks)?;
    let name = BookmarkName::new(&body.name)?;
    let position = Millis::position(body.position_ms)?;
    let expected = Version::new(body.expected_version);
    let label = blocking(move || {
        Ok(app
            .bookmarks
            .convert_region_to_label(BookmarkId::new(id), expected, name, position)?)
    })
    .await?;
    Ok(Json(label_to_dto(&label)))
}
