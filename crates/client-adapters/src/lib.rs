//! HTTP client adapter; compiles for native targets and `wasm32`.

mod access;
mod channel;
mod diagnostics;
mod events;
mod exchange;
mod timer;

pub use access::rights_from_dto;
pub use channel::{EventChannel, open_channel};
pub use events::{EventConnection, connect_events};
pub use timer::SystemTimer;

use access::{managed_user_from_dto, signed_in_from_dto};
use exchange::{
    api_token_from_dto, content_type, created_api_token_from_dto, format_name,
    import_report_from_dto,
};
use reqwest::{
    Client, RequestBuilder, Response, StatusCode,
    header::{CONTENT_TYPE, RETRY_AFTER},
};
use serde::de::DeserializeOwned;
use veltrunox_client_app::{
    AccessStatus, ApiToken, CreatedApiToken, Credentials, ExchangeFormat, ImportOptions,
    ImportReport, LinkError, ManagedUser, NewUser, RecordingWaveform, Rights, ServerLink, SignedIn,
    UploadedRecording, UserChange,
};
use veltrunox_core::{
    ApiTokenId, BookmarkId, BookmarkName, Bookmarks, Label, LibraryEntry, Millis, PasswordPolicy,
    Reason, RecordingId, RecordingTitle, Region, TimeRange, TokenScope, UserId, Version, Waveform,
    WaveformLevel,
};
use veltrunox_protocol::{
    ANALYSIS_PATH, ApiTokenListDto, BOOKMARKS_PATH, BookmarksDto, ChangePasswordDto, ConflictDto,
    CreatedApiTokenDto, CredentialsDto, EXPORT_PATH, FILE_NAME_HEADER, GUEST_ACCESS_PATH,
    GuestAccessDto, IMPORT_PATH, IdentityDto, ImportResultDto, LABEL_PATH, LABEL_TO_REGION_PATH,
    LABELS_PATH, LOGIN_PATH, LOGOUT_PATH, LabelDto, ME_PASSWORD_PATH, ME_PATH, MERGE_REGIONS_PATH,
    MergeRegionsDto, NewApiTokenDto, NewLabelDto, NewRegionDto, NewUserDto, PASSWORD_POLICY_PATH,
    RECORDING_EXPORT_PATH, RECORDING_PATH, RECORDINGS_PATH, REGION_PATH, REGION_TO_LABEL_PATH,
    REGIONS_PATH, ReasonDto, RecordingListDto, RegionDto, RegionRefDto, RenameRecordingDto,
    SETUP_PATH, SPLIT_REGION_PATH, STATUS_PATH, SessionDto, SetPasswordPolicyDto, SplitRegionDto,
    StatusDto, TOKEN_PATH, TOKENS_PATH, UPLOAD_ID_HEADER, USER_PATH, USERS_PATH, UpdateLabelDto,
    UpdateRegionDto, UpdateUserDto, UploadedRecordingDto, UserDto, WAVEFORM_LEVEL_PATH,
    WAVEFORM_PATH, WaveformInfoDto, bookmarks_from_dto, label_from_dto, library_entry_from_dto,
    password_policy_from_dto, password_policy_to_dto, peaks_from_bytes, range_to_dto,
    region_from_dto, role_to_dto, token_scope_to_dto,
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

/// Text safe in a header value: everything but unreserved ASCII as `%XX` of its UTF-8 bytes.
fn percent_encode(text: &str) -> String {
    text.bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                char::from(byte).to_string()
            }
            _ => format!("%{byte:02X}"),
        })
        .collect()
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
            StatusCode::BAD_REQUEST
            | StatusCode::UNPROCESSABLE_ENTITY
            | StatusCode::PAYLOAD_TOO_LARGE
            | StatusCode::UNSUPPORTED_MEDIA_TYPE => Err(refusal_of(response).await),
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
            StatusCode::BAD_REQUEST | StatusCode::CONFLICT | StatusCode::NOT_FOUND => {
                Err(refusal_of(response).await)
            }
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

/// The reason in the body of a refusal; a body that is no reason still says something.
async fn refusal_of(response: Response) -> LinkError {
    let status = response.status();
    let text = response.text().await.unwrap_or_default();
    let reason = match serde_json::from_str::<ReasonDto>(&text) {
        Ok(dto) => dto.into(),
        Err(_) => Reason::coded("refused", &format!("the server answered {status}: {text}")),
    };
    LinkError::Refused(reason)
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

    async fn upload_recording(
        &self,
        file_name: &str,
        upload_id: &str,
        content: Vec<u8>,
    ) -> Result<UploadedRecording, LinkError> {
        let request = self
            .post(format!("{}{}", self.base_url, RECORDINGS_PATH))
            .header(FILE_NAME_HEADER, percent_encode(file_name))
            .header(UPLOAD_ID_HEADER, upload_id)
            .body(content);
        let dto: UploadedRecordingDto = Self::send_write(request)
            .await?
            .json()
            .await
            .map_err(|error| LinkError::InvalidResponse(error.to_string()))?;
        let title = RecordingTitle::new(&dto.title)
            .map_err(|error| LinkError::InvalidResponse(error.to_string()))?;
        Ok(UploadedRecording {
            id: RecordingId::new(dto.id),
            title,
            existing: dto.existing,
        })
    }

    async fn rename_recording(
        &self,
        recording: RecordingId,
        title: &RecordingTitle,
    ) -> Result<(), LinkError> {
        let request =
            self.put(self.url(RECORDING_PATH, recording.value()))
                .json(&RenameRecordingDto {
                    title: title.as_str().to_owned(),
                });
        Self::send_write(request).await.map(|_| ())
    }

    async fn remove_recording(&self, recording: RecordingId) -> Result<(), LinkError> {
        let request = self.delete(self.url(RECORDING_PATH, recording.value()));
        Self::send_write(request).await.map(|_| ())
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

    async fn split_region(
        &self,
        id: BookmarkId,
        expected: Version,
        names: [&BookmarkName; 2],
        position: Millis,
    ) -> Result<(), LinkError> {
        let body = SplitRegionDto {
            expected_version: expected.value(),
            position_ms: position.value(),
            first_name: names[0].as_str().to_owned(),
            second_name: names[1].as_str().to_owned(),
        };
        let _: Vec<RegionDto> = Self::written(
            self.post(self.url(SPLIT_REGION_PATH, id.value()))
                .json(&body),
        )
        .await?;
        Ok(())
    }

    async fn regions_into_region(
        &self,
        recording: RecordingId,
        regions: &[(BookmarkId, Version)],
        name: &BookmarkName,
        range: TimeRange,
    ) -> Result<Region, LinkError> {
        let body = MergeRegionsDto {
            regions: regions
                .iter()
                .map(|(id, expected)| RegionRefDto {
                    id: id.value(),
                    expected_version: expected.value(),
                })
                .collect(),
            name: name.as_str().to_owned(),
            range: range_to_dto(range),
        };
        let dto: RegionDto = Self::written(
            self.post(self.url(MERGE_REGIONS_PATH, recording.value()))
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
    async fn fetch_export(
        &self,
        recording: Option<RecordingId>,
        format: ExchangeFormat,
    ) -> Result<String, LinkError> {
        let path = match recording {
            Some(recording) => {
                RECORDING_EXPORT_PATH.replace("{id}", &recording.value().to_string())
            }
            None => EXPORT_PATH.to_owned(),
        };
        let url = format!("{}{}?format={}", self.base_url, path, format_name(format));
        Self::account_call(self.get(url))
            .await?
            .text()
            .await
            .map_err(|error| LinkError::InvalidResponse(error.to_string()))
    }

    async fn send_import(
        &self,
        content: &str,
        format: ExchangeFormat,
        options: ImportOptions,
    ) -> Result<ImportReport, LinkError> {
        let url = format!(
            "{}{}?dry_run={}&overwrite={}",
            self.base_url, IMPORT_PATH, options.dry_run, options.overwrite
        );
        let request = self
            .post(url)
            .header(CONTENT_TYPE, content_type(format))
            .body(content.to_owned());
        let dto: ImportResultDto = Self::written(request).await?;
        import_report_from_dto(&dto).map_err(LinkError::InvalidResponse)
    }

    async fn fetch_api_tokens(&self) -> Result<Vec<ApiToken>, LinkError> {
        let dto: ApiTokenListDto =
            Self::account_json(self.get(format!("{}{}", self.base_url, TOKENS_PATH))).await?;
        Ok(dto.tokens.iter().map(api_token_from_dto).collect())
    }

    async fn create_api_token(
        &self,
        name: &str,
        scope: TokenScope,
    ) -> Result<CreatedApiToken, LinkError> {
        let request =
            self.post(format!("{}{}", self.base_url, TOKENS_PATH))
                .json(&NewApiTokenDto {
                    name: name.to_owned(),
                    scope: token_scope_to_dto(scope),
                });
        let dto: CreatedApiTokenDto = Self::account_json(request).await?;
        Ok(created_api_token_from_dto(&dto))
    }

    async fn revoke_api_token(&self, id: ApiTokenId) -> Result<(), LinkError> {
        Self::account_call(self.delete(self.url(TOKEN_PATH, id.value())))
            .await
            .map(|_| ())
    }

    async fn fetch_access_status(&self) -> Result<AccessStatus, LinkError> {
        let dto: StatusDto =
            Self::account_json(self.get(format!("{}{}", self.base_url, STATUS_PATH))).await?;
        Ok(AccessStatus {
            setup_needed: dto.setup_needed,
            guest_enabled: dto.guest_enabled,
            password_policy: password_policy_from_dto(dto.password_policy),
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

    async fn send_own_password(&self, current: &str, new: &str) -> Result<(), LinkError> {
        let body = ChangePasswordDto {
            current_password: current.to_owned(),
            new_password: new.to_owned(),
        };
        Self::account_call(
            self.put(format!("{}{}", self.base_url, ME_PASSWORD_PATH))
                .json(&body),
        )
        .await
        .map(drop)
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

    async fn switch_password_policy(&self, policy: PasswordPolicy) -> Result<(), LinkError> {
        Self::account_call(
            self.put(format!("{}{}", self.base_url, PASSWORD_POLICY_PATH))
                .json(&SetPasswordPolicyDto {
                    policy: password_policy_to_dto(policy),
                }),
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

#[cfg(test)]
mod tests {
    use super::percent_encode;

    #[test]
    fn file_names_are_encoded_for_a_header() {
        assert_eq!(percent_encode("Blue Monday.mp3"), "Blue%20Monday.mp3");
        assert_eq!(percent_encode("Grüße/x.wav"), "Gr%C3%BC%C3%9Fe%2Fx.wav");
    }
}
