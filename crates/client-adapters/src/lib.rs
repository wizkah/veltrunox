//! HTTP client adapter; compiles for native targets and `wasm32`.

mod access;
mod channel;
mod events;
mod timer;

pub use access::rights_from_dto;
pub use channel::{EventChannel, open_channel};
pub use events::{EventConnection, connect_events};
pub use timer::SystemTimer;

use access::{managed_user_from_dto, signed_in_from_dto};
use reqwest::{Client, RequestBuilder, Response, StatusCode, header::RETRY_AFTER};
use serde::de::DeserializeOwned;
use veltrunox_client_app::{
    AccessStatus, Credentials, LinkError, ManagedUser, NewUser, RecordingWaveform, Rights,
    ServerLink, SignedIn, UserChange,
};
use veltrunox_core::{
    BookmarkId, BookmarkName, Bookmarks, Label, LibraryEntry, Millis, RecordingId, Region,
    TimeRange, UserId, Version, Waveform, WaveformLevel,
};
use veltrunox_protocol::{
    ANALYSIS_PATH, BOOKMARKS_PATH, BookmarksDto, ConflictDto, CredentialsDto, GUEST_ACCESS_PATH,
    GuestAccessDto, IdentityDto, LABEL_PATH, LABEL_TO_REGION_PATH, LABELS_PATH, LOGIN_PATH,
    LOGOUT_PATH, LabelDto, ME_PATH, NewLabelDto, NewRegionDto, NewUserDto, RECORDINGS_PATH,
    REGION_PATH, REGION_TO_LABEL_PATH, REGIONS_PATH, RecordingListDto, RegionDto, SETUP_PATH,
    STATUS_PATH, SessionDto, StatusDto, USER_PATH, USERS_PATH, UpdateLabelDto, UpdateRegionDto,
    UpdateUserDto, UserDto, WAVEFORM_LEVEL_PATH, WAVEFORM_PATH, WaveformInfoDto,
    bookmarks_from_dto, label_from_dto, library_entry_from_dto, peaks_from_bytes, range_to_dto,
    region_from_dto, role_to_dto,
};

pub struct HttpServerLink {
    client: Client,
    base_url: String,
    /// Sent as bearer token with every request; without it the server sees a guest.
    token: Option<String>,
}

impl HttpServerLink {
    pub fn new(base_url: &str) -> Self {
        Self::with_token(base_url, None)
    }

    /// A link that signs its requests with `token`.
    pub fn with_token(base_url: &str, token: Option<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_owned(),
            token,
        }
    }

    fn signed(&self, request: RequestBuilder) -> RequestBuilder {
        match &self.token {
            Some(token) => request.bearer_auth(token),
            None => request,
        }
    }

    fn get(&self, url: String) -> RequestBuilder {
        self.signed(self.client.get(url))
    }

    fn post(&self, url: String) -> RequestBuilder {
        self.signed(self.client.post(url))
    }

    fn put(&self, url: String) -> RequestBuilder {
        self.signed(self.client.put(url))
    }

    fn delete(&self, url: String) -> RequestBuilder {
        self.signed(self.client.delete(url))
    }
}

/// Passes a successful answer on; maps a refusal of sign-in or rights.
fn checked(response: Response) -> Result<Response, LinkError> {
    match response.status() {
        status if status.is_success() => Ok(response),
        StatusCode::UNAUTHORIZED => Err(LinkError::Unauthorized),
        StatusCode::FORBIDDEN => Err(LinkError::Forbidden),
        status => Err(LinkError::Unreachable(format!("server answered {status}"))),
    }
}

impl HttpServerLink {
    async fn waveform_info(&self, recording: RecordingId) -> Result<WaveformInfoDto, LinkError> {
        let path = WAVEFORM_PATH.replace("{id}", &recording.value().to_string());
        self.get(format!("{}{}", self.base_url, path))
            .send()
            .await
            .map_err(|error| LinkError::Unreachable(error.to_string()))
            .and_then(checked)?
            .json()
            .await
            .map_err(|error| LinkError::InvalidResponse(error.to_string()))
    }

    async fn waveform_level(
        &self,
        recording: RecordingId,
        level: usize,
    ) -> Result<Vec<veltrunox_core::Peak>, LinkError> {
        let path = WAVEFORM_LEVEL_PATH
            .replace("{id}", &recording.value().to_string())
            .replace("{level}", &level.to_string());
        let bytes = self
            .get(format!("{}{}", self.base_url, path))
            .send()
            .await
            .map_err(|error| LinkError::Unreachable(error.to_string()))
            .and_then(checked)?
            .bytes()
            .await
            .map_err(|error| LinkError::Unreachable(error.to_string()))?;
        peaks_from_bytes(&bytes).ok_or_else(|| {
            LinkError::InvalidResponse("waveform level is not whole blocks".to_owned())
        })
    }

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
            StatusCode::UNAUTHORIZED => Err(LinkError::Unauthorized),
            StatusCode::FORBIDDEN => Err(LinkError::Forbidden),
            StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY => Err(LinkError::Refused(
                response.text().await.unwrap_or_default(),
            )),
            status => Err(LinkError::Unreachable(format!("server answered {status}"))),
        }
    }

    /// Sends a sign-in or administration request and maps its refusals: the
    /// server's reason for a refused value or a conflict, the wait after too many sign-ins.
    async fn account_call(request: RequestBuilder) -> Result<Response, LinkError> {
        let response = request
            .send()
            .await
            .map_err(|error| LinkError::Unreachable(error.to_string()))?;
        match response.status() {
            status if status.is_success() => Ok(response),
            StatusCode::UNAUTHORIZED => Err(LinkError::Unauthorized),
            StatusCode::FORBIDDEN => Err(LinkError::Forbidden),
            StatusCode::TOO_MANY_REQUESTS => {
                let seconds = response
                    .headers()
                    .get(RETRY_AFTER)
                    .and_then(|value| value.to_str().ok())
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(1);
                Err(LinkError::TooManyAttempts {
                    retry_after_seconds: seconds,
                })
            }
            StatusCode::BAD_REQUEST | StatusCode::CONFLICT => Err(LinkError::Refused(
                response.text().await.unwrap_or_default(),
            )),
            StatusCode::NOT_FOUND => Err(LinkError::Refused("not found".to_owned())),
            status => Err(LinkError::Unreachable(format!("server answered {status}"))),
        }
    }

    async fn account_json<T: DeserializeOwned>(request: RequestBuilder) -> Result<T, LinkError> {
        Self::account_call(request)
            .await?
            .json()
            .await
            .map_err(|error| LinkError::InvalidResponse(error.to_string()))
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
            .get(url)
            .send()
            .await
            .map_err(|error| LinkError::Unreachable(error.to_string()))
            .and_then(checked)?;
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
            .get(format!("{}{}", self.base_url, path))
            .send()
            .await
            .map_err(|error| LinkError::Unreachable(error.to_string()))
            .and_then(checked)?;
        let dto: BookmarksDto = response
            .json()
            .await
            .map_err(|error| LinkError::InvalidResponse(error.to_string()))?;
        bookmarks_from_dto(&dto).map_err(|error| LinkError::InvalidResponse(error.to_string()))
    }

    async fn request_analysis(&self, recording: RecordingId) -> Result<(), LinkError> {
        let path = ANALYSIS_PATH.replace("{id}", &recording.value().to_string());
        self.post(format!("{}{}", self.base_url, path))
            .send()
            .await
            .map_err(|error| LinkError::Unreachable(error.to_string()))
            .and_then(checked)?;
        Ok(())
    }

    async fn fetch_length(&self, recording: RecordingId) -> Result<Millis, LinkError> {
        let info = self.waveform_info(recording).await?;
        Millis::position(info.length_ms)
            .map_err(|error| LinkError::InvalidResponse(error.to_string()))
    }

    async fn fetch_waveform(&self, recording: RecordingId) -> Result<RecordingWaveform, LinkError> {
        let info = self.waveform_info(recording).await?;
        let length = Millis::position(info.length_ms)
            .map_err(|error| LinkError::InvalidResponse(error.to_string()))?;
        let mut levels = Vec::with_capacity(info.levels.len());
        for (index, described) in info.levels.iter().enumerate() {
            let peaks = self.waveform_level(recording, index).await?;
            if peaks.len() != described.blocks as usize {
                return Err(LinkError::InvalidResponse(format!(
                    "waveform level {index} has {} blocks, not {}",
                    peaks.len(),
                    described.blocks
                )));
            }
            levels.push(WaveformLevel {
                frames_per_block: described.frames_per_block,
                peaks,
            });
        }
        Ok(RecordingWaveform {
            length,
            waveform: Waveform {
                sample_rate: info.sample_rate,
                levels,
            },
        })
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
            self.post(self.url(LABELS_PATH, recording.value()))
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
        let dto: LabelDto =
            Self::written(self.put(self.url(LABEL_PATH, id.value())).json(&body)).await?;
        label_from_dto(&dto).map_err(|error| LinkError::InvalidResponse(error.to_string()))
    }

    async fn remove_label(&self, id: BookmarkId, expected: Version) -> Result<(), LinkError> {
        let url = format!(
            "{}?expected_version={}",
            self.url(LABEL_PATH, id.value()),
            expected.value()
        );
        Self::send_write(self.delete(url)).await.map(drop)
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
            self.post(self.url(REGIONS_PATH, recording.value()))
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
        let dto: RegionDto =
            Self::written(self.put(self.url(REGION_PATH, id.value())).json(&body)).await?;
        region_from_dto(&dto).map_err(|error| LinkError::InvalidResponse(error.to_string()))
    }

    async fn remove_region(&self, id: BookmarkId, expected: Version) -> Result<(), LinkError> {
        let url = format!(
            "{}?expected_version={}",
            self.url(REGION_PATH, id.value()),
            expected.value()
        );
        Self::send_write(self.delete(url)).await.map(drop)
    }

    async fn label_into_region(
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
            self.post(self.url(LABEL_TO_REGION_PATH, id.value()))
                .json(&body),
        )
        .await?;
        region_from_dto(&dto).map_err(|error| LinkError::InvalidResponse(error.to_string()))
    }

    async fn region_into_label(
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
            self.post(self.url(REGION_TO_LABEL_PATH, id.value()))
                .json(&body),
        )
        .await?;
        label_from_dto(&dto).map_err(|error| LinkError::InvalidResponse(error.to_string()))
    }
    async fn fetch_access_status(&self) -> Result<AccessStatus, LinkError> {
        let dto: StatusDto =
            Self::account_json(self.get(format!("{}{}", self.base_url, STATUS_PATH))).await?;
        Ok(AccessStatus {
            setup_needed: dto.setup_needed,
            guest_enabled: dto.guest_enabled,
        })
    }

    async fn setup_first_admin(&self, credentials: &Credentials) -> Result<SignedIn, LinkError> {
        let dto: SessionDto = Self::account_json(
            self.post(format!("{}{}", self.base_url, SETUP_PATH))
                .json(&credentials_to_dto(credentials)),
        )
        .await?;
        Ok(signed_in_from_dto(&dto))
    }

    async fn sign_in(&self, credentials: &Credentials) -> Result<SignedIn, LinkError> {
        let dto: SessionDto = Self::account_json(
            self.post(format!("{}{}", self.base_url, LOGIN_PATH))
                .json(&credentials_to_dto(credentials)),
        )
        .await?;
        Ok(signed_in_from_dto(&dto))
    }

    async fn sign_out(&self) -> Result<(), LinkError> {
        Self::account_call(self.post(format!("{}{}", self.base_url, LOGOUT_PATH)))
            .await
            .map(drop)
    }

    async fn fetch_rights(&self) -> Result<Rights, LinkError> {
        let dto: IdentityDto =
            Self::account_json(self.get(format!("{}{}", self.base_url, ME_PATH))).await?;
        Ok(access::rights_from_dto(&dto))
    }

    async fn fetch_users(&self) -> Result<Vec<ManagedUser>, LinkError> {
        let dtos: Vec<UserDto> =
            Self::account_json(self.get(format!("{}{}", self.base_url, USERS_PATH))).await?;
        Ok(dtos.iter().map(managed_user_from_dto).collect())
    }

    async fn create_user(&self, user: &NewUser) -> Result<ManagedUser, LinkError> {
        let body = NewUserDto {
            user_name: user.user_name.clone(),
            password: user.password.clone(),
            role: role_to_dto(user.role),
        };
        let dto: UserDto = Self::account_json(
            self.post(format!("{}{}", self.base_url, USERS_PATH))
                .json(&body),
        )
        .await?;
        Ok(managed_user_from_dto(&dto))
    }

    async fn change_user(&self, id: UserId, change: &UserChange) -> Result<(), LinkError> {
        let body = UpdateUserDto {
            role: change.role.map(role_to_dto),
            active: change.active,
            password: change.password.clone(),
        };
        Self::account_call(self.put(self.url(USER_PATH, id.value())).json(&body))
            .await
            .map(drop)
    }

    async fn remove_user(&self, id: UserId) -> Result<(), LinkError> {
        Self::account_call(self.delete(self.url(USER_PATH, id.value())))
            .await
            .map(drop)
    }

    async fn switch_guest_access(&self, enabled: bool) -> Result<(), LinkError> {
        Self::account_call(
            self.put(format!("{}{}", self.base_url, GUEST_ACCESS_PATH))
                .json(&GuestAccessDto { enabled }),
        )
        .await
        .map(drop)
    }
}

fn credentials_to_dto(credentials: &Credentials) -> CredentialsDto {
    CredentialsDto {
        user_name: credentials.user_name.clone(),
        password: credentials.password.clone(),
    }
}
