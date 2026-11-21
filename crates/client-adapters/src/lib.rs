//! HTTP client adapter; compiles for native targets and `wasm32`.

mod channel;
mod events;
mod timer;

pub use channel::{EventChannel, open_channel};
pub use events::{EventConnection, connect_events};
pub use timer::SystemTimer;

use reqwest::Client;
use veltrunox_client_app::{LinkError, ServerLink};
use veltrunox_core::{Bookmarks, Recording, RecordingId, RecordingTitle};
use veltrunox_protocol::{
    BOOKMARKS_PATH, BookmarksDto, RECORDINGS_PATH, RecordingListDto, RecordingSummaryDto,
    bookmarks_from_dto,
};

pub struct HttpServerLink {
    client: Client,
    base_url: String,
}

impl HttpServerLink {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_owned(),
        }
    }
}

impl ServerLink for HttpServerLink {
    async fn fetch_recordings(&self) -> Result<Vec<Recording>, LinkError> {
        let url = format!("{}{}", self.base_url, RECORDINGS_PATH);
        let response = self
            .client
            .get(url)
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|error| LinkError::Unreachable(error.to_string()))?;
        let list: RecordingListDto = response
            .json()
            .await
            .map_err(|error| LinkError::InvalidResponse(error.to_string()))?;
        list.recordings.into_iter().map(into_recording).collect()
    }

    async fn fetch_bookmarks(&self, recording: RecordingId) -> Result<Bookmarks, LinkError> {
        let path = BOOKMARKS_PATH.replace("{id}", &recording.value().to_string());
        let response = self
            .client
            .get(format!("{}{}", self.base_url, path))
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|error| LinkError::Unreachable(error.to_string()))?;
        let dto: BookmarksDto = response
            .json()
            .await
            .map_err(|error| LinkError::InvalidResponse(error.to_string()))?;
        bookmarks_from_dto(&dto).map_err(|error| LinkError::InvalidResponse(error.to_string()))
    }
}

fn into_recording(dto: RecordingSummaryDto) -> Result<Recording, LinkError> {
    let title = RecordingTitle::new(&dto.title)
        .map_err(|error| LinkError::InvalidResponse(error.to_string()))?;
    Ok(Recording {
        id: RecordingId::new(dto.id),
        title,
    })
}
