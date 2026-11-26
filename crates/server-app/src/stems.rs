//! Use cases: the stem set of a recording (ADR-015).

use crate::bookmarks::BookmarkService;
use crate::ports::{
    BookmarkError, MediaCatalog, PlaybackBroadcast, RecordingLengths, StemRepository, StemSet,
    StorageError,
};
use std::sync::Arc;
use veltrunox_core::{Millis, Recording, RecordingId, Section, section_regions};

/// How far the length of a stem may differ from the length of its recording.
const LENGTH_TOLERANCE_MS: u64 = 1_000;

#[derive(Debug, thiserror::Error)]
pub enum StemError {
    #[error("the recording of these stems is not in the library")]
    UnknownRecording,
    #[error("a stem is not as long as the recording")]
    LengthMismatch,
    #[error(transparent)]
    Storage(#[from] StorageError),
    #[error(transparent)]
    Bookmark(#[from] BookmarkError),
}

impl veltrunox_core::Refusal for StemError {
    fn reason(&self) -> veltrunox_core::Reason {
        let code = match self {
            Self::UnknownRecording => "stems_unknown_recording",
            Self::LengthMismatch => "stems_length_mismatch",
            Self::Storage(_) | Self::Bookmark(_) => "storage_failed",
        };
        veltrunox_core::Reason::new(self, code)
    }
}

/// What an import came to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedStems {
    pub recording: Recording,
    pub stems: usize,
    pub regions_added: usize,
}

/// Song structure that arrives with a stem set.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Structure {
    pub sections: Vec<Section>,
    pub downbeats: Vec<Millis>,
}

pub struct StemService {
    media: Arc<dyn MediaCatalog>,
    stems: Arc<dyn StemRepository>,
    lengths: Arc<dyn RecordingLengths>,
    bookmarks: Arc<BookmarkService>,
    broadcast: Arc<dyn PlaybackBroadcast>,
}

impl StemService {
    pub fn new(
        media: Arc<dyn MediaCatalog>,
        stems: Arc<dyn StemRepository>,
        lengths: Arc<dyn RecordingLengths>,
        bookmarks: Arc<BookmarkService>,
        broadcast: Arc<dyn PlaybackBroadcast>,
    ) -> Self {
        Self {
            media,
            stems,
            lengths,
            bookmarks,
            broadcast,
        }
    }

    pub fn stem_set(&self, id: RecordingId) -> Result<StemSet, StorageError> {
        self.stems.stem_set(id)
    }

    /// Gives the recording with this audio hash the stems, replacing earlier ones, and adds
    /// its sections as regions.
    pub fn install(
        &self,
        content_hash: &str,
        set: &StemSet,
        structure: &Structure,
    ) -> Result<ImportedStems, StemError> {
        let recording = self
            .media
            .recording_by_hash(content_hash)?
            .ok_or(StemError::UnknownRecording)?;
        let length = checked_length(set, self.lengths.length(recording.id))?;
        self.stems.replace_stem_set(recording.id, set)?;
        let existing = self.bookmarks.list(recording.id)?.regions;
        let added = section_regions(&structure.sections, &structure.downbeats, length, &existing);
        let regions_added = added.len();
        for region in added {
            self.bookmarks
                .add_region(recording.id, region.name, region.range)?;
        }
        self.broadcast.library_changed();
        Ok(ImportedStems {
            recording,
            stems: set.stems.len(),
            regions_added,
        })
    }

    pub fn remove(&self, id: RecordingId) -> Result<(), StorageError> {
        self.stems.replace_stem_set(id, &StemSet::default())?;
        self.broadcast.library_changed();
        Ok(())
    }
}

/// The length the stems are checked against: the recording's, or the first stem's while that
/// is not known.
fn checked_length(set: &StemSet, recording: Option<Millis>) -> Result<Millis, StemError> {
    let length = recording
        .or_else(|| set.stems.first().map(|stem| stem.length))
        .ok_or(StemError::LengthMismatch)?;
    let fits = set
        .stems
        .iter()
        .all(|stem| stem.length.value().abs_diff(length.value()) <= LENGTH_TOLERANCE_MS);
    fits.then_some(length).ok_or(StemError::LengthMismatch)
}
