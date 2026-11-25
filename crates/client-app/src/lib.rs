//! Client use cases and the ports they need. Free of network and UI types.

mod access;
mod connection;
mod exchange;
mod reconnect;
mod session;
mod waveform;

pub use access::{
    AccessStatus, ChannelAccess, Credentials, ManagedUser, NewUser, Rights, SignedIn, UserChange,
};
pub use connection::{ConnectionMonitor, ConnectionStatus, LinkEvent};
pub use exchange::{
    ApiToken, CreatedApiToken, ExchangeFormat, ImportCounts, ImportIssue, ImportOptions,
    ImportReport, ImportStatus, RecordingImport,
};
pub use reconnect::{CLOCK_SYNC_INTERVAL, ReconnectPolicy, Timer};
pub use session::{ScrubThrottle, ServerEvent, SessionView, ViewChange};
pub use waveform::{RecordingWaveform, WaveformCache, cached_waveform};

/// What the real-time channel reports: its own lifecycle or a message from the server.
#[derive(Debug, Clone, PartialEq)]
pub enum ChannelEvent {
    Link(LinkEvent),
    Server(ServerEvent),
}

use std::future::Future;
use veltrunox_core::{
    ApiTokenId, BookmarkId, BookmarkName, Bookmarks, Label, LibraryEntry, Millis, PasswordPolicy,
    Reason, RecordingId, RecordingTitle, Region, TimeRange, TokenScope, UserId, Version,
};

#[derive(Debug, thiserror::Error)]
pub enum LinkError {
    #[error("server unreachable: {0}")]
    Unreachable(String),
    #[error("unexpected server response: {0}")]
    InvalidResponse(String),
    #[error("not connected")]
    NotConnected,
    /// The bookmark was changed by someone else; `current` is its stored version.
    #[error("the bookmark changed meanwhile (now at version {})", .current.value())]
    Conflict { current: Version },
    /// The server refused the values or the file; the reason has a code the interface words.
    #[error("the server refused it: {}", .0.message)]
    Refused(Reason),
    /// Not signed in, or the token is no longer valid.
    #[error("sign-in required")]
    Unauthorized,
    /// Signed in, but the role may not do this.
    #[error("not allowed")]
    Forbidden,
    /// Too many failed sign-ins; the server asks to wait.
    #[error("too many failed sign-ins; try again in {retry_after_seconds} seconds")]
    TooManyAttempts { retry_after_seconds: u64 },
}

/// What the server made of an uploaded file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UploadedRecording {
    pub id: RecordingId,
    pub title: RecordingTitle,
    /// The same audio was in the library already; nothing was added.
    pub existing: bool,
}

/// Port: requests to the server. The future is not required to be `Send`
/// because the browser target runs single-threaded.
pub trait ServerLink {
    fn fetch_recordings(&self) -> impl Future<Output = Result<Vec<LibraryEntry>, LinkError>>;
    fn fetch_bookmarks(
        &self,
        recording: RecordingId,
    ) -> impl Future<Output = Result<Bookmarks, LinkError>>;
    /// Asks the server to analyse the recording again.
    fn request_analysis(
        &self,
        recording: RecordingId,
    ) -> impl Future<Output = Result<(), LinkError>>;
    /// The length of the recording as stored by its scan; an error while there is none.
    fn fetch_length(
        &self,
        recording: RecordingId,
    ) -> impl Future<Output = Result<Millis, LinkError>>;
    /// The length and the peaks of the recording as stored by its scan; an error while there is none.
    fn fetch_waveform(
        &self,
        recording: RecordingId,
    ) -> impl Future<Output = Result<RecordingWaveform, LinkError>>;
    fn create_label(
        &self,
        recording: RecordingId,
        name: &BookmarkName,
        position: Millis,
    ) -> impl Future<Output = Result<Label, LinkError>>;
    fn change_label(
        &self,
        id: BookmarkId,
        expected: Version,
        name: &BookmarkName,
        position: Millis,
    ) -> impl Future<Output = Result<Label, LinkError>>;
    fn remove_label(
        &self,
        id: BookmarkId,
        expected: Version,
    ) -> impl Future<Output = Result<(), LinkError>>;
    fn create_region(
        &self,
        recording: RecordingId,
        name: &BookmarkName,
        range: TimeRange,
    ) -> impl Future<Output = Result<Region, LinkError>>;
    fn change_region(
        &self,
        id: BookmarkId,
        expected: Version,
        name: &BookmarkName,
        range: TimeRange,
    ) -> impl Future<Output = Result<Region, LinkError>>;
    fn remove_region(
        &self,
        id: BookmarkId,
        expected: Version,
    ) -> impl Future<Output = Result<(), LinkError>>;
    /// Replaces a label by a region in one step on the server.
    fn label_into_region(
        &self,
        id: BookmarkId,
        expected: Version,
        name: &BookmarkName,
        range: TimeRange,
    ) -> impl Future<Output = Result<Region, LinkError>>;
    /// Replaces a region by a label in one step on the server.
    fn region_into_label(
        &self,
        id: BookmarkId,
        expected: Version,
        name: &BookmarkName,
        position: Millis,
    ) -> impl Future<Output = Result<Label, LinkError>>;
    /// Sends an audio file; the server checks and stores it and queues its analysis. While it
    /// receives the file, the server reports the bytes received under `upload_id` over the
    /// real-time channel.
    fn upload_recording(
        &self,
        file_name: &str,
        upload_id: &str,
        content: Vec<u8>,
    ) -> impl Future<Output = Result<UploadedRecording, LinkError>>;
    fn rename_recording(
        &self,
        recording: RecordingId,
        title: &RecordingTitle,
    ) -> impl Future<Output = Result<(), LinkError>>;
    /// Removes the recording with its bookmarks and analysis.
    fn remove_recording(
        &self,
        recording: RecordingId,
    ) -> impl Future<Output = Result<(), LinkError>>;
    /// The bookmarks of one recording, or of all recordings, as the text of a file.
    fn fetch_export(
        &self,
        recording: Option<RecordingId>,
        format: ExchangeFormat,
    ) -> impl Future<Output = Result<String, LinkError>>;
    /// Sends the text of an export file to the server, which replaces the bookmarks of the
    /// recordings it lists (or, with `dry_run`, only says what it would do).
    fn send_import(
        &self,
        content: &str,
        format: ExchangeFormat,
        options: ImportOptions,
    ) -> impl Future<Output = Result<ImportReport, LinkError>>;
    /// The API tokens of the signed-in user.
    fn fetch_api_tokens(&self) -> impl Future<Output = Result<Vec<ApiToken>, LinkError>>;
    fn create_api_token(
        &self,
        name: &str,
        scope: TokenScope,
    ) -> impl Future<Output = Result<CreatedApiToken, LinkError>>;
    fn revoke_api_token(&self, id: ApiTokenId) -> impl Future<Output = Result<(), LinkError>>;
    /// What may be known without signing in.
    fn fetch_access_status(&self) -> impl Future<Output = Result<AccessStatus, LinkError>>;
    /// Creates the first admin and signs in; only works while no user exists.
    fn setup_first_admin(
        &self,
        credentials: &Credentials,
    ) -> impl Future<Output = Result<SignedIn, LinkError>>;
    fn sign_in(
        &self,
        credentials: &Credentials,
    ) -> impl Future<Output = Result<SignedIn, LinkError>>;
    /// Ends the session of this link's token.
    fn sign_out(&self) -> impl Future<Output = Result<(), LinkError>>;
    /// The rights of this link's token, or of a guest while guests are let in.
    fn fetch_rights(&self) -> impl Future<Output = Result<Rights, LinkError>>;
    fn fetch_users(&self) -> impl Future<Output = Result<Vec<ManagedUser>, LinkError>>;
    fn create_user(&self, user: &NewUser) -> impl Future<Output = Result<ManagedUser, LinkError>>;
    fn change_user(
        &self,
        id: UserId,
        change: &UserChange,
    ) -> impl Future<Output = Result<(), LinkError>>;
    fn remove_user(&self, id: UserId) -> impl Future<Output = Result<(), LinkError>>;
    fn switch_guest_access(&self, enabled: bool) -> impl Future<Output = Result<(), LinkError>>;
    fn switch_password_policy(
        &self,
        policy: PasswordPolicy,
    ) -> impl Future<Output = Result<(), LinkError>>;
}

/// Use case: load the recording list from the server.
pub async fn load_library(link: &impl ServerLink) -> Result<Vec<LibraryEntry>, LinkError> {
    link.fetch_recordings().await
}

/// Use case: load the bookmarks of a recording, for example after a change notice.
pub async fn load_bookmarks(
    link: &impl ServerLink,
    recording: RecordingId,
) -> Result<Bookmarks, LinkError> {
    link.fetch_bookmarks(recording).await
}

/// Use case: start the analysis of a recording again; the result arrives as a notice.
pub async fn analyse_again(
    link: &impl ServerLink,
    recording: RecordingId,
) -> Result<(), LinkError> {
    link.request_analysis(recording).await
}

/// Use case: add a recording from an audio file; the upload progress (under `upload_id`) and
/// the analysis result arrive as notices.
pub async fn add_recording(
    link: &impl ServerLink,
    file_name: &str,
    upload_id: &str,
    content: Vec<u8>,
) -> Result<UploadedRecording, LinkError> {
    link.upload_recording(file_name, upload_id, content).await
}

/// Use case: give a recording another title.
pub async fn rename_recording(
    link: &impl ServerLink,
    recording: RecordingId,
    title: &RecordingTitle,
) -> Result<(), LinkError> {
    link.rename_recording(recording, title).await
}

/// Use case: delete a recording for good.
pub async fn delete_recording(
    link: &impl ServerLink,
    recording: RecordingId,
) -> Result<(), LinkError> {
    link.remove_recording(recording).await
}

/// Use case: the length of a recording, for the position control.
pub async fn load_length(
    link: &impl ServerLink,
    recording: RecordingId,
) -> Result<Millis, LinkError> {
    link.fetch_length(recording).await
}

/// Use case: the waveform of a recording, from memory once it was fetched.
pub async fn load_waveform(
    link: &impl ServerLink,
    cache: &std::sync::Mutex<WaveformCache>,
    recording: RecordingId,
) -> Result<std::sync::Arc<RecordingWaveform>, LinkError> {
    cached_waveform(cache, recording, async || {
        link.fetch_waveform(recording).await
    })
    .await
}

/// Use case: add a label at a position.
pub async fn add_label(
    link: &impl ServerLink,
    recording: RecordingId,
    name: &BookmarkName,
    position: Millis,
) -> Result<Label, LinkError> {
    link.create_label(recording, name, position).await
}

/// Use case: rename or move a label; `expected` is the version the user saw.
pub async fn edit_label(
    link: &impl ServerLink,
    id: BookmarkId,
    expected: Version,
    name: &BookmarkName,
    position: Millis,
) -> Result<Label, LinkError> {
    link.change_label(id, expected, name, position).await
}

/// Use case: delete a label.
pub async fn delete_label(
    link: &impl ServerLink,
    id: BookmarkId,
    expected: Version,
) -> Result<(), LinkError> {
    link.remove_label(id, expected).await
}

/// Use case: add a region.
pub async fn add_region(
    link: &impl ServerLink,
    recording: RecordingId,
    name: &BookmarkName,
    range: TimeRange,
) -> Result<Region, LinkError> {
    link.create_region(recording, name, range).await
}

/// Use case: rename or resize a region; `expected` is the version the user saw.
pub async fn edit_region(
    link: &impl ServerLink,
    id: BookmarkId,
    expected: Version,
    name: &BookmarkName,
    range: TimeRange,
) -> Result<Region, LinkError> {
    link.change_region(id, expected, name, range).await
}

/// Use case: delete a region.
pub async fn delete_region(
    link: &impl ServerLink,
    id: BookmarkId,
    expected: Version,
) -> Result<(), LinkError> {
    link.remove_region(id, expected).await
}

/// Use case: turn a label into a region; the label is replaced atomically by a new region.
pub async fn convert_label_to_region(
    link: &impl ServerLink,
    id: BookmarkId,
    expected: Version,
    name: &BookmarkName,
    range: TimeRange,
) -> Result<Region, LinkError> {
    link.label_into_region(id, expected, name, range).await
}

/// Use case: turn a region into a label; the region is replaced atomically by a new label.
pub async fn convert_region_to_label(
    link: &impl ServerLink,
    id: BookmarkId,
    expected: Version,
    name: &BookmarkName,
    position: Millis,
) -> Result<Label, LinkError> {
    link.region_into_label(id, expected, name, position).await
}

/// Use case: what may be known before signing in (first start, guest access).
pub async fn load_access_status(link: &impl ServerLink) -> Result<AccessStatus, LinkError> {
    link.fetch_access_status().await
}

/// Use case: set up the first admin at the first start.
pub async fn set_up_first_admin(
    link: &impl ServerLink,
    credentials: &Credentials,
) -> Result<SignedIn, LinkError> {
    link.setup_first_admin(credentials).await
}

/// Use case: sign in with user name and password.
pub async fn sign_in(
    link: &impl ServerLink,
    credentials: &Credentials,
) -> Result<SignedIn, LinkError> {
    link.sign_in(credentials).await
}

/// Use case: sign out; the token stops working on the server.
pub async fn sign_out(link: &impl ServerLink) -> Result<(), LinkError> {
    link.sign_out().await
}

/// Use case: learn what the current sign-in (or guest access) may do.
pub async fn load_rights(link: &impl ServerLink) -> Result<Rights, LinkError> {
    link.fetch_rights().await
}

/// Use case (admin): list all users.
pub async fn load_users(link: &impl ServerLink) -> Result<Vec<ManagedUser>, LinkError> {
    link.fetch_users().await
}

/// Use case (admin): add a user.
pub async fn add_user(link: &impl ServerLink, user: &NewUser) -> Result<ManagedUser, LinkError> {
    link.create_user(user).await
}

/// Use case (admin): change role, activity or password of a user.
pub async fn edit_user(
    link: &impl ServerLink,
    id: UserId,
    change: &UserChange,
) -> Result<(), LinkError> {
    link.change_user(id, change).await
}

/// Use case (admin): delete a user.
pub async fn delete_user(link: &impl ServerLink, id: UserId) -> Result<(), LinkError> {
    link.remove_user(id).await
}

/// Use case (admin): choose the rules for new passwords.
pub async fn set_password_policy(
    link: &impl ServerLink,
    policy: PasswordPolicy,
) -> Result<(), LinkError> {
    link.switch_password_policy(policy).await
}

/// Use case (admin): let guests in or not.
pub async fn set_guest_access(link: &impl ServerLink, enabled: bool) -> Result<(), LinkError> {
    link.switch_guest_access(enabled).await
}

/// Use case: the bookmarks of a recording, or of the whole library, as the text of a file.
pub async fn export_bookmarks(
    link: &impl ServerLink,
    recording: Option<RecordingId>,
    format: ExchangeFormat,
) -> Result<String, LinkError> {
    link.fetch_export(recording, format).await
}

/// Use case: import bookmarks from the text of a file. A first call with `dry_run` shows what
/// would change; the same call without it applies it.
pub async fn import_bookmarks(
    link: &impl ServerLink,
    content: &str,
    format: ExchangeFormat,
    options: ImportOptions,
) -> Result<ImportReport, LinkError> {
    link.send_import(content, format, options).await
}

/// Use case: the API tokens of the signed-in user.
pub async fn load_api_tokens(link: &impl ServerLink) -> Result<Vec<ApiToken>, LinkError> {
    link.fetch_api_tokens().await
}

/// Use case: make an API token; its text is in the answer and is not shown again.
pub async fn make_api_token(
    link: &impl ServerLink,
    name: &str,
    scope: TokenScope,
) -> Result<CreatedApiToken, LinkError> {
    link.create_api_token(name, scope).await
}

/// Use case: end an API token of the signed-in user.
pub async fn end_api_token(link: &impl ServerLink, id: ApiTokenId) -> Result<(), LinkError> {
    link.revoke_api_token(id).await
}
