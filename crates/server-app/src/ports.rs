//! Ports the server use cases need; adapters implement them.

use std::path::{Path, PathBuf};
use veltrunox_core::{
    BookmarkId, BookmarkName, Bookmarks, Label, Millis, PlaybackSettings, PlaybackState, Recording,
    RecordingId, RecordingTitle, Region, TimeRange, Version,
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
}

#[derive(Debug, thiserror::Error)]
pub enum BookmarkError {
    #[error("bookmark or recording not found")]
    NotFound,
    #[error("bookmark was changed meanwhile; current version is {}", .0.value())]
    Conflict(Version),
    #[error(transparent)]
    Storage(#[from] StorageError),
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
}
