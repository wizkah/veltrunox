//! HTTP endpoints: library and bookmarks.

use crate::{events::events, state::AppState};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post, put},
};
use veltrunox_core::{
    BookmarkId, BookmarkName, InvalidValue, Millis, Recording, RecordingId, Version,
};
use veltrunox_protocol::{
    BOOKMARKS_PATH, BookmarksDto, ConflictDto, EVENTS_PATH, ExpectedVersionQuery, LABEL_PATH,
    LABELS_PATH, LabelDto, NewLabelDto, NewRegionDto, RECORDINGS_PATH, REGION_PATH, REGIONS_PATH,
    RecordingListDto, RecordingSummaryDto, RegionDto, UpdateLabelDto, UpdateRegionDto,
    bookmarks_to_dto, label_to_dto, range_from_dto, region_to_dto,
};
use veltrunox_server_app::{BookmarkError, list_recordings};

/// Builds the HTTP routes of the server.
pub fn router(app: AppState) -> Router {
    Router::new()
        .route(RECORDINGS_PATH, get(recordings))
        .route(EVENTS_PATH, get(events))
        .route(BOOKMARKS_PATH, get(bookmarks))
        .route(LABELS_PATH, post(add_label))
        .route(REGIONS_PATH, post(add_region))
        .route(LABEL_PATH, put(update_label).delete(delete_label))
        .route(REGION_PATH, put(update_region).delete(delete_region))
        .with_state(app)
}

/// Why a request failed, mapped to an HTTP status in one place.
enum ApiError {
    Invalid(InvalidValue),
    Bookmark(BookmarkError),
    Internal,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            Self::Invalid(error) => (StatusCode::BAD_REQUEST, error.to_string()).into_response(),
            Self::Bookmark(BookmarkError::NotFound) => StatusCode::NOT_FOUND.into_response(),
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

impl From<InvalidValue> for ApiError {
    fn from(error: InvalidValue) -> Self {
        Self::Invalid(error)
    }
}

impl From<BookmarkError> for ApiError {
    fn from(error: BookmarkError) -> Self {
        Self::Bookmark(error)
    }
}

/// Runs blocking storage work off the async runtime threads.
async fn blocking<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, ApiError> + Send + 'static,
) -> Result<T, ApiError> {
    tokio::task::spawn_blocking(work)
        .await
        .map_err(|_| ApiError::Internal)?
}

async fn recordings(State(app): State<AppState>) -> Result<Json<RecordingListDto>, ApiError> {
    let listed =
        blocking(move || list_recordings(app.recordings.as_ref()).map_err(|_| ApiError::Internal))
            .await?;
    Ok(Json(RecordingListDto {
        recordings: listed.into_iter().map(into_dto).collect(),
    }))
}

fn into_dto(recording: Recording) -> RecordingSummaryDto {
    RecordingSummaryDto {
        id: recording.id.value(),
        title: recording.title.as_str().to_owned(),
    }
}

async fn bookmarks(
    State(app): State<AppState>,
    Path(recording): Path<u64>,
) -> Result<Json<BookmarksDto>, ApiError> {
    let found = blocking(move || Ok(app.bookmarks.list(RecordingId::new(recording))?)).await?;
    Ok(Json(bookmarks_to_dto(&found)))
}

async fn add_label(
    State(app): State<AppState>,
    Path(recording): Path<u64>,
    Json(body): Json<NewLabelDto>,
) -> Result<(StatusCode, Json<LabelDto>), ApiError> {
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
    Path(id): Path<u64>,
    Json(body): Json<UpdateLabelDto>,
) -> Result<Json<LabelDto>, ApiError> {
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
    Path(id): Path<u64>,
    Query(query): Query<ExpectedVersionQuery>,
) -> Result<StatusCode, ApiError> {
    let expected = Version::new(query.expected_version);
    blocking(move || Ok(app.bookmarks.delete_label(BookmarkId::new(id), expected)?)).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn add_region(
    State(app): State<AppState>,
    Path(recording): Path<u64>,
    Json(body): Json<NewRegionDto>,
) -> Result<(StatusCode, Json<RegionDto>), ApiError> {
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
    Path(id): Path<u64>,
    Json(body): Json<UpdateRegionDto>,
) -> Result<Json<RegionDto>, ApiError> {
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
    Path(id): Path<u64>,
    Query(query): Query<ExpectedVersionQuery>,
) -> Result<StatusCode, ApiError> {
    let expected = Version::new(query.expected_version);
    blocking(move || Ok(app.bookmarks.delete_region(BookmarkId::new(id), expected)?)).await?;
    Ok(StatusCode::NO_CONTENT)
}
