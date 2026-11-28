//! Ports the server use cases need; adapters implement them.

use std::path::{Path, PathBuf};
use veltrunox_core::{
    ApiTokenId, BookmarkId, BookmarkName, Bookmarks, InvalidValue, Label, Millis, NewBookmarks,
    PasswordPolicy, PlaybackSettings, PlaybackState, Recording, RecordingAnalysis, RecordingId,
    RecordingTitle, Region, Role, SplitModel, StemName, StemSource, TimeRange, TokenName,
    TokenScope, UserId, UserName, Version, Waveform,
};

#[derive(Debug, thiserror::Error)]
#[error("storage failed: {0}")]
pub struct StorageError(pub String);

/// Port: stored recordings and the playback settings kept with them.
pub trait RecordingRepository: Send + Sync {
    fn list(&self) -> Result<Vec<Recording>, StorageError>;
    fn add(&self, title: RecordingTitle) -> Result<Recording, StorageError>;
    /// `None` if the recording does not exist.
    fn settings(&self, id: RecordingId) -> Result<Option<PlaybackSettings>, StorageError>;
    fn save_settings(
        &self,
        id: RecordingId,
        settings: PlaybackSettings,
    ) -> Result<(), StorageError>;
    /// `false` if the recording does not exist.
    fn rename(&self, id: RecordingId, title: RecordingTitle) -> Result<bool, StorageError>;
    /// Removes the recording with its bookmarks, analysis and waveform; `false` if it does
    /// not exist. The audio file stays: the caller decides what happens to it.
    fn delete(&self, id: RecordingId) -> Result<bool, StorageError>;
}

/// Port: the audio files behind the recordings. Files are known by the hash
/// of their content, so the same audio is never listed twice.
pub trait MediaCatalog: Send + Sync {
    /// `None` if the recording does not exist or has no audio file.
    fn media_path(&self, id: RecordingId) -> Result<Option<PathBuf>, StorageError>;
    /// Adds a recording for the file; `None` if audio with this hash is already known.
    fn add_media(
        &self,
        title: RecordingTitle,
        path: &Path,
        content_hash: &str,
    ) -> Result<Option<Recording>, StorageError>;
    /// The recording whose audio has this hash.
    fn recording_by_hash(&self, content_hash: &str) -> Result<Option<Recording>, StorageError>;
    /// The hash of the recording's audio; `None` without an audio file.
    fn content_hash(&self, id: RecordingId) -> Result<Option<String>, StorageError>;
    /// Every audio file a recording refers to.
    fn media_paths(&self) -> Result<Vec<PathBuf>, StorageError>;
}

/// One stored stem of a recording.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stem {
    pub name: StemName,
    pub path: PathBuf,
    pub length: Millis,
    pub waveform: Waveform,
}

/// The stems of a recording, in their order, with the analysis they came with.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StemSet {
    pub source: StemSource,
    pub stems: Vec<Stem>,
    /// The analysis part of the package as it came, kept to export it again.
    pub analysis: Option<String>,
}

/// The sets a recording has, by where they came from, and the one in use.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StemSources {
    pub all: Vec<StemSource>,
    pub selected: Option<StemSource>,
}

/// Port: the stem sets of the recordings. A recording keeps one set per source, one of
/// them selected. Whatever changes which stems are selected resets the stored mix.
/// Deleting a recording removes its sets.
pub trait StemRepository: Send + Sync {
    /// The selected set; empty if the recording has no stems or does not exist.
    fn stem_set(&self, id: RecordingId) -> Result<StemSet, StorageError>;
    fn stem_sources(&self, id: RecordingId) -> Result<StemSources, StorageError>;
    /// Adds the set or replaces the one of its source, and selects it.
    fn put_stem_set(&self, id: RecordingId, set: &StemSet) -> Result<(), StorageError>;
    /// `false` if there is no set of this source.
    fn select_stem_set(&self, id: RecordingId, source: &StemSource) -> Result<bool, StorageError>;
    /// Removes the set of this source, or all; another one is selected if one remains.
    fn remove_stem_sets(
        &self,
        id: RecordingId,
        source: Option<&StemSource>,
    ) -> Result<(), StorageError>;
    /// Keeps the analysis of an existing set; the stems and their mix stay.
    fn set_stem_analysis(
        &self,
        id: RecordingId,
        source: &StemSource,
        analysis: &str,
    ) -> Result<(), StorageError>;
    /// The recordings that have stems.
    fn recordings_with_stems(&self) -> Result<Vec<RecordingId>, StorageError>;
    /// Every stem file a recording refers to.
    fn stem_paths(&self) -> Result<Vec<PathBuf>, StorageError>;
}

/// Port: separates an audio file into stems and finds the structure of a recording from its
/// stems (ADR-015). Both run for minutes and block.
pub trait StemSeparator: Send + Sync {
    /// `report` is told how far it has come in percent. `Err` describes what went wrong,
    /// for the log.
    fn separate(
        &self,
        audio: &Path,
        title: &str,
        model: SplitModel,
        package: &Path,
        report: &dyn Fn(u8),
    ) -> Result<(), String>;

    /// Writes what it finds in the named stem files to `output`, as the analysis part of a
    /// package manifest. Reports and fails like `separate`.
    fn find_structure(
        &self,
        stems: &[(&str, &Path)],
        output: &Path,
        report: &dyn Fn(u8),
    ) -> Result<(), String>;
}

/// Port: stored analysis results. A new result replaces the old one.
pub trait AnalysisRepository: Send + Sync {
    /// `None` if the recording has no result (yet) or does not exist.
    fn analysis(&self, id: RecordingId) -> Result<Option<RecordingAnalysis>, StorageError>;
    fn analyses(&self) -> Result<Vec<(RecordingId, RecordingAnalysis)>, StorageError>;
    fn save_analysis(
        &self,
        id: RecordingId,
        analysis: RecordingAnalysis,
    ) -> Result<(), StorageError>;
    /// Recordings with an audio file that were not scanned completely yet:
    /// no stored analysis result, length or waveform.
    fn unanalysed(&self) -> Result<Vec<RecordingId>, StorageError>;
}

/// Port: stored waveform and length, found by decoding the audio once.
pub trait WaveformRepository: Send + Sync {
    /// Replaces the stored waveform and length of the recording.
    fn save_waveform(
        &self,
        id: RecordingId,
        length: Millis,
        waveform: &Waveform,
    ) -> Result<(), StorageError>;
    /// `None` if the recording has none (yet) or does not exist.
    fn waveform(&self, id: RecordingId) -> Result<Option<Waveform>, StorageError>;
    /// `None` if the length is not known (yet) or the recording does not exist.
    fn stored_length(&self, id: RecordingId) -> Result<Option<Millis>, StorageError>;
}

/// Why an analysis failed: the code of its reason (see `veltrunox_core::Reason`).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct AnalysisFailure(pub String);

/// What decoding one audio file yields. The waveform and length exist even if
/// the analysis found nothing (for example in silence).
#[derive(Debug, Clone, PartialEq)]
pub struct RecordingScan {
    pub length: Millis,
    pub waveform: Waveform,
    pub analysis: Result<RecordingAnalysis, AnalysisFailure>,
}

/// Port: decodes an audio file and derives waveform, length, tuning, key and
/// tempo from it. CPU-bound and blocking. `Err` if the file cannot be decoded.
/// `report` is told how far the scan has come in percent (0 to 100), rising.
pub trait RecordingAnalyzer: Send + Sync {
    fn scan(&self, path: &Path, report: &dyn Fn(u8)) -> Result<RecordingScan, AnalysisFailure>;
}

#[derive(Debug, thiserror::Error)]
pub enum BookmarkError {
    #[error("bookmark or recording not found")]
    NotFound,
    #[error("bookmark was changed meanwhile; current version is {}", .0.value())]
    Conflict(Version),
    #[error(transparent)]
    Invalid(#[from] InvalidValue),
    #[error(transparent)]
    Storage(#[from] StorageError),
}

impl veltrunox_core::Refusal for BookmarkError {
    fn reason(&self) -> veltrunox_core::Reason {
        use veltrunox_core::Reason;
        match self {
            Self::NotFound => Reason::new(self, "bookmark_not_found"),
            Self::Conflict(version) => {
                Reason::with_args(self, "bookmark_conflict", vec![version.value()])
            }
            Self::Invalid(error) => error.reason(),
            Self::Storage(_) => Reason::new(self, "storage_failed"),
        }
    }
}

/// A stored item together with the recording it belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Owned<T> {
    pub recording: RecordingId,
    pub item: T,
}

/// Port: bookmarks with per-record versions. Writes state the version they saw.
pub trait BookmarkRepository: Send + Sync {
    fn list(&self, recording: RecordingId) -> Result<Bookmarks, BookmarkError>;
    /// Replaces all labels and regions of the recording in one transaction: either the new
    /// set is stored as it is (with new ids and first versions) or nothing changes.
    fn replace_all(
        &self,
        recording: RecordingId,
        bookmarks: &NewBookmarks,
    ) -> Result<Bookmarks, BookmarkError>;
    fn add_label(
        &self,
        recording: RecordingId,
        name: BookmarkName,
        position: Millis,
    ) -> Result<Label, BookmarkError>;
    fn update_label(
        &self,
        id: BookmarkId,
        expected: Version,
        name: BookmarkName,
        position: Millis,
    ) -> Result<Owned<Label>, BookmarkError>;
    fn delete_label(&self, id: BookmarkId, expected: Version)
    -> Result<RecordingId, BookmarkError>;
    fn add_region(
        &self,
        recording: RecordingId,
        name: BookmarkName,
        range: TimeRange,
    ) -> Result<Region, BookmarkError>;
    fn update_region(
        &self,
        id: BookmarkId,
        expected: Version,
        name: BookmarkName,
        range: TimeRange,
    ) -> Result<Owned<Region>, BookmarkError>;
    fn delete_region(
        &self,
        id: BookmarkId,
        expected: Version,
    ) -> Result<RecordingId, BookmarkError>;
    /// Replaces a label by a new region in one step; the region gets a new id.
    fn convert_label_to_region(
        &self,
        id: BookmarkId,
        expected: Version,
        name: BookmarkName,
        range: TimeRange,
    ) -> Result<Owned<Region>, BookmarkError>;
    /// Replaces the given regions (with the versions seen) of a recording by one new region
    /// in one step; the region gets a new id. If any of them is missing, belongs to another
    /// recording or has changed, nothing changes.
    fn merge_regions(
        &self,
        recording: RecordingId,
        regions: &[(BookmarkId, Version)],
        name: BookmarkName,
        range: TimeRange,
    ) -> Result<Region, BookmarkError>;
    /// Replaces a region by two that meet at `position` in one step; both get new ids.
    /// The position must lie inside the region.
    fn split_region(
        &self,
        id: BookmarkId,
        expected: Version,
        names: [BookmarkName; 2],
        position: Millis,
    ) -> Result<Owned<[Region; 2]>, BookmarkError>;
    /// Replaces a region by a new label in one step; the label gets a new id.
    fn convert_region_to_label(
        &self,
        id: BookmarkId,
        expected: Version,
        name: BookmarkName,
        position: Millis,
    ) -> Result<Owned<Label>, BookmarkError>;
}

/// Port: how long the recordings are, as far as the server knows yet. A length
/// is known once the audio has been decoded.
pub trait RecordingLengths: Send + Sync {
    fn length(&self, id: RecordingId) -> Option<Millis>;
}

/// Port: controllable time source in server milliseconds.
pub trait Clock: Send + Sync {
    fn now(&self) -> Millis;
}

/// Port: tells connected clients what changed.
pub trait PlaybackBroadcast: Send + Sync {
    fn state_changed(&self, state: &PlaybackState);
    /// Clients reload the bookmarks of this recording.
    fn bookmarks_changed(&self, recording: RecordingId);
    /// The analysis result or status of this recording changed; clients reload the library.
    fn analysis_changed(&self, recording: RecordingId);
    /// Recordings were added, renamed or removed; clients reload the library.
    fn library_changed(&self);
    /// The stem set of this recording was replaced or removed.
    fn stems_changed(&self, _recording: RecordingId) {}
}

/// Port: wall-clock time in seconds since 1970, for session lifetimes and
/// login delays (the `Clock` above counts from the server start).
pub trait WallClock: Send + Sync {
    fn unix_seconds(&self) -> u64;
}

/// A stored user account. Only admins and users have accounts; the guest has none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserRecord {
    pub id: UserId,
    pub name: UserName,
    pub role: Role,
    pub password_hash: String,
    pub active: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum UserStoreError {
    #[error("a user with this name exists already")]
    NameTaken,
    #[error("user not found")]
    NotFound,
    #[error(transparent)]
    Storage(#[from] StorageError),
}

/// Port: users, their roles and password hashes, and the guest-access switch.
pub trait UserRepository: Send + Sync {
    fn count(&self) -> Result<u64, StorageError>;
    /// Creates the user only if no user exists yet, in one step; `None` otherwise.
    fn create_first_admin(
        &self,
        name: &UserName,
        password_hash: &str,
    ) -> Result<Option<UserRecord>, UserStoreError>;
    fn create(
        &self,
        name: &UserName,
        role: Role,
        password_hash: &str,
    ) -> Result<UserRecord, UserStoreError>;
    /// Looks up by `UserName::key`.
    fn find_by_name(&self, key: &str) -> Result<Option<UserRecord>, StorageError>;
    fn find(&self, id: UserId) -> Result<Option<UserRecord>, StorageError>;
    fn list(&self) -> Result<Vec<UserRecord>, StorageError>;
    fn set_role(&self, id: UserId, role: Role) -> Result<(), UserStoreError>;
    fn set_active(&self, id: UserId, active: bool) -> Result<(), UserStoreError>;
    fn set_password_hash(&self, id: UserId, password_hash: &str) -> Result<(), UserStoreError>;
    /// Removes the user and their sessions.
    fn delete(&self, id: UserId) -> Result<(), UserStoreError>;
    fn guest_enabled(&self) -> Result<bool, StorageError>;
    fn set_guest_enabled(&self, enabled: bool) -> Result<(), StorageError>;
    /// The rules for new passwords; `Standard` until the admin changes them.
    fn password_policy(&self) -> Result<PasswordPolicy, StorageError>;
    fn set_password_policy(&self, policy: PasswordPolicy) -> Result<(), StorageError>;
}

/// A stored API token; the token itself is never kept, only its digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiTokenRecord {
    pub id: ApiTokenId,
    pub user: UserId,
    pub name: TokenName,
    pub scope: TokenScope,
    pub created_at: u64,
    /// `None` until the token was used.
    pub last_used: Option<u64>,
}

/// Port: API tokens of users, known by the digest of their token only.
pub trait ApiTokenRepository: Send + Sync {
    fn create(
        &self,
        user: UserId,
        name: &TokenName,
        scope: TokenScope,
        digest: &str,
        now: u64,
    ) -> Result<ApiTokenRecord, StorageError>;
    /// The tokens of one user, oldest first.
    fn list_for_user(&self, user: UserId) -> Result<Vec<ApiTokenRecord>, StorageError>;
    /// The active user a token belongs to and the token's reach; a hit counts as use.
    fn user_of(
        &self,
        digest: &str,
        now: u64,
    ) -> Result<Option<(UserRecord, TokenScope)>, StorageError>;
    /// Removes the token if it belongs to `user`; `false` if there is none.
    fn revoke(&self, user: UserId, id: ApiTokenId) -> Result<bool, StorageError>;
}

/// Port: login sessions, known by the digest of their token only.
pub trait SessionRepository: Send + Sync {
    fn save(&self, digest: &str, user: UserId, now: u64) -> Result<(), StorageError>;
    /// The active user of a session that was used within `max_idle` seconds;
    /// a hit counts as use. Older sessions are removed.
    fn user_of(
        &self,
        digest: &str,
        now: u64,
        max_idle: u64,
    ) -> Result<Option<UserRecord>, StorageError>;
    fn end(&self, digest: &str) -> Result<(), StorageError>;
    fn delete_for_user(&self, user: UserId) -> Result<(), StorageError>;
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct HashFailure(pub String);

/// Port: hashes and checks passwords with a deliberately slow algorithm.
pub trait PasswordHasher: Send + Sync {
    fn hash(&self, password: &str) -> Result<String, HashFailure>;
    /// `false` for a wrong password and for an unreadable hash.
    fn verify(&self, password: &str, hash: &str) -> bool;
}

/// Port: creates session tokens and the digests under which they are stored.
pub trait SessionTokens: Send + Sync {
    /// A new unguessable token.
    fn issue(&self) -> String;
    fn digest(&self, token: &str) -> String;
}
