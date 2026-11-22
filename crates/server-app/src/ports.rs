//! Ports the server use cases need; adapters implement them.

use std::path::{Path, PathBuf};
use veltrunox_core::{
    BookmarkId, BookmarkName, Bookmarks, Label, Millis, PlaybackSettings, PlaybackState, Recording,
    RecordingAnalysis, RecordingId, RecordingTitle, Region, TimeRange, Version, Waveform,
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
pub trait RecordingAnalyzer: Send + Sync {
    fn scan(&self, path: &Path) -> Result<RecordingScan, AnalysisFailure>;
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
    /// Replaces a label by a new region in one step; the region gets a new id.
    fn convert_label_to_region(
        &self,
        id: BookmarkId,
        expected: Version,
        name: BookmarkName,
        range: TimeRange,
    ) -> Result<Owned<Region>, BookmarkError>;
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
}
