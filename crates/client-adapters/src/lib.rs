//! HTTP client adapter; compiles for native targets and `wasm32`.

mod channel;
mod events;
mod timer;

pub use channel::{EventChannel, open_channel};
pub use events::{EventConnection, connect_events};
pub use timer::SystemTimer;

use reqwest::{Client, RequestBuilder, Response, StatusCode};
use serde::de::DeserializeOwned;
use veltrunox_client_app::{LinkError, ServerLink};
use veltrunox_core::{
    BookmarkId, BookmarkName, Bookmarks, Label, LibraryEntry, Millis, RecordingId, Region,
    TimeRange, Version,
};
use veltrunox_protocol::{
    ANALYSIS_PATH, BOOKMARKS_PATH, BookmarksDto, ConflictDto, LABEL_PATH, LABELS_PATH, LabelDto,
    NewLabelDto, NewRegionDto, RECORDINGS_PATH, REGION_PATH, REGIONS_PATH, RecordingListDto,
    RegionDto, UpdateLabelDto, UpdateRegionDto, WAVEFORM_PATH, WaveformInfoDto, bookmarks_from_dto,
    label_from_dto, library_entry_from_dto, range_to_dto, region_from_dto,
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

impl HttpServerLink {
    fn url(&self, template: &str, id: u64) -> String {
        format!(
            "{}{}",
            self.base_url,
            template.replace("{id}", &id.to_string())
        )
    }

    /// Sends a bookmark write and maps the answers a write can get: a stale
    /// version is a conflict, a refused value carries the server's reason.
    async fn send_write(request: RequestBuilder) -> Result<Response, LinkError> {
        let response = request
            .send()
            .await
            .map_err(|error| LinkError::Unreachable(error.to_string()))?;
        match response.status() {
            status if status.is_success() => Ok(response),
            StatusCode::CONFLICT => {
                let conflict: ConflictDto = response
                    .json()
                    .await
                    .map_err(|error| LinkError::InvalidResponse(error.to_string()))?;
                Err(LinkError::Conflict {
                    current: Version::new(conflict.current_version),
                })
            }
            StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY => Err(LinkError::Refused(
                response.text().await.unwrap_or_default(),
            )),
            status => Err(LinkError::Unreachable(format!("server answered {status}"))),
        }
    }

    async fn written<T: DeserializeOwned>(request: RequestBuilder) -> Result<T, LinkError> {
        Self::send_write(request)
            .await?
            .json()
            .await
            .map_err(|error| LinkError::InvalidResponse(error.to_string()))
    }
}

impl ServerLink for HttpServerLink {
    async fn fetch_recordings(&self) -> Result<Vec<LibraryEntry>, LinkError> {
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
        list.recordings
            .iter()
            .map(|dto| {
                library_entry_from_dto(dto)
                    .map_err(|error| LinkError::InvalidResponse(error.to_string()))
            })
            .collect()
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

    async fn request_analysis(&self, recording: RecordingId) -> Result<(), LinkError> {
        let path = ANALYSIS_PATH.replace("{id}", &recording.value().to_string());
        self.client
            .post(format!("{}{}", self.base_url, path))
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|error| LinkError::Unreachable(error.to_string()))?;
        Ok(())
    }

    async fn fetch_length(&self, recording: RecordingId) -> Result<Millis, LinkError> {
        let path = WAVEFORM_PATH.replace("{id}", &recording.value().to_string());
        let response = self
            .client
            .get(format!("{}{}", self.base_url, path))
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|error| LinkError::Unreachable(error.to_string()))?;
        let info: WaveformInfoDto = response
            .json()
            .await
            .map_err(|error| LinkError::InvalidResponse(error.to_string()))?;
        Millis::position(info.length_ms)
            .map_err(|error| LinkError::InvalidResponse(error.to_string()))
    }

    async fn create_label(
        &self,
        recording: RecordingId,
        name: &BookmarkName,
        position: Millis,
    ) -> Result<Label, LinkError> {
        let body = NewLabelDto {
            name: name.as_str().to_owned(),
            position_ms: position.value(),
        };
        let dto: LabelDto = Self::written(
            self.client
                .post(self.url(LABELS_PATH, recording.value()))
                .json(&body),
        )
        .await?;
        label_from_dto(&dto).map_err(|error| LinkError::InvalidResponse(error.to_string()))
    }

    async fn change_label(
        &self,
        id: BookmarkId,
        expected: Version,
        name: &BookmarkName,
        position: Millis,
    ) -> Result<Label, LinkError> {
        let body = UpdateLabelDto {
            expected_version: expected.value(),
            name: name.as_str().to_owned(),
            position_ms: position.value(),
        };
        let dto: LabelDto = Self::written(
            self.client
                .put(self.url(LABEL_PATH, id.value()))
                .json(&body),
        )
        .await?;
        label_from_dto(&dto).map_err(|error| LinkError::InvalidResponse(error.to_string()))
    }

    async fn remove_label(&self, id: BookmarkId, expected: Version) -> Result<(), LinkError> {
        let url = format!(
            "{}?expected_version={}",
            self.url(LABEL_PATH, id.value()),
            expected.value()
        );
        Self::send_write(self.client.delete(url)).await.map(drop)
    }

    async fn create_region(
        &self,
        recording: RecordingId,
        name: &BookmarkName,
        range: TimeRange,
    ) -> Result<Region, LinkError> {
        let body = NewRegionDto {
            name: name.as_str().to_owned(),
            range: range_to_dto(range),
        };
        let dto: RegionDto = Self::written(
            self.client
                .post(self.url(REGIONS_PATH, recording.value()))
                .json(&body),
        )
        .await?;
        region_from_dto(&dto).map_err(|error| LinkError::InvalidResponse(error.to_string()))
    }

    async fn change_region(
        &self,
        id: BookmarkId,
        expected: Version,
        name: &BookmarkName,
        range: TimeRange,
    ) -> Result<Region, LinkError> {
        let body = UpdateRegionDto {
            expected_version: expected.value(),
            name: name.as_str().to_owned(),
            range: range_to_dto(range),
        };
        let dto: RegionDto = Self::written(
            self.client
                .put(self.url(REGION_PATH, id.value()))
                .json(&body),
        )
        .await?;
        region_from_dto(&dto).map_err(|error| LinkError::InvalidResponse(error.to_string()))
    }

    async fn remove_region(&self, id: BookmarkId, expected: Version) -> Result<(), LinkError> {
        let url = format!(
            "{}?expected_version={}",
            self.url(REGION_PATH, id.value()),
            expected.value()
        );
        Self::send_write(self.client.delete(url)).await.map(drop)
    }
}
