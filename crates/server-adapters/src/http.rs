use crate::events::events;
use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use std::sync::Arc;
use veltrunox_core::Recording;
use veltrunox_protocol::{EVENTS_PATH, RECORDINGS_PATH, RecordingListDto, RecordingSummaryDto};
use veltrunox_server_app::{RecordingRepository, list_recordings};

type SharedRepository = Arc<dyn RecordingRepository>;

/// Builds the HTTP routes of the server.
pub fn router(repository: SharedRepository) -> Router {
    Router::new()
        .route(RECORDINGS_PATH, get(recordings))
        .route(EVENTS_PATH, get(events))
        .with_state(repository)
}

async fn recordings(
    State(repository): State<SharedRepository>,
) -> Result<Json<RecordingListDto>, StatusCode> {
    let listed =
        list_recordings(repository.as_ref()).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
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
