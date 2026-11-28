//! Use cases: the stem set of a recording (ADR-015).

use crate::bookmarks::BookmarkService;
use crate::ports::{
    BookmarkError, MediaCatalog, PlaybackBroadcast, RecordingLengths, StemRepository, StemSet,
    StemSources, StorageError, WaveformRepository,
};
use std::sync::Arc;
use veltrunox_core::{Millis, Recording, RecordingId, Section, StemSource, section_regions};

/// How far the length of a stem may differ from the length of its recording.
const LENGTH_TOLERANCE_MS: u64 = 1_000;

#[derive(Debug, thiserror::Error)]
pub enum StemError {
    #[error("the recording of these stems is not in the library")]
    UnknownRecording,
    #[error("a stem is not as long as the recording")]
    LengthMismatch,
    #[error("the stems do not have the sample rate of the recording")]
    RateMismatch,
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
            Self::RateMismatch => "stems_rate_mismatch",
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
    waveforms: Arc<dyn WaveformRepository>,
}

impl StemService {
    pub fn new(
        media: Arc<dyn MediaCatalog>,
        stems: Arc<dyn StemRepository>,
        lengths: Arc<dyn RecordingLengths>,
        bookmarks: Arc<BookmarkService>,
        broadcast: Arc<dyn PlaybackBroadcast>,
        waveforms: Arc<dyn WaveformRepository>,
    ) -> Self {
        Self {
            media,
            stems,
            lengths,
            bookmarks,
            broadcast,
            waveforms,
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
        // The engine mixes the stems frame by frame in place of the recording.
        if let Some(original) = self.waveforms.waveform(recording.id)?
            && set
                .stems
                .iter()
                .any(|stem| stem.waveform.sample_rate != original.sample_rate)
        {
            return Err(StemError::RateMismatch);
        }
        self.stems.put_stem_set(recording.id, set)?;
        self.broadcast.stems_changed(recording.id);
        let regions_added = self.add_sections(recording.id, structure, length)?;
        self.broadcast.library_changed();
        Ok(ImportedStems {
            recording,
            stems: set.stems.len(),
            regions_added,
        })
    }

    /// Adds the sections of the structure of `set` as regions and keeps `analysis`, so they
    /// can be added again without finding them anew. Returns how many regions are new.
    pub fn add_structure(
        &self,
        id: RecordingId,
        set: &StemSet,
        structure: &Structure,
        analysis: &str,
    ) -> Result<usize, StemError> {
        let length = checked_length(set, self.lengths.length(id))?;
        self.stems.set_stem_analysis(id, &set.source, analysis)?;
        let added = self.add_sections(id, structure, length)?;
        self.broadcast.library_changed();
        Ok(added)
    }

    fn add_sections(
        &self,
        id: RecordingId,
        structure: &Structure,
        length: Millis,
    ) -> Result<usize, StemError> {
        let existing = self.bookmarks.list(id)?.regions;
        let added = section_regions(&structure.sections, &structure.downbeats, length, &existing);
        let count = added.len();
        for region in added {
            self.bookmarks.add_region(id, region.name, region.range)?;
        }
        Ok(count)
    }

    pub fn recordings_with_stems(&self) -> Result<Vec<RecordingId>, StorageError> {
        self.stems.recordings_with_stems()
    }

    pub fn stem_sources(&self, id: RecordingId) -> Result<StemSources, StorageError> {
        self.stems.stem_sources(id)
    }

    /// `false` if the recording has no set of this source.
    pub fn select(&self, id: RecordingId, source: &StemSource) -> Result<bool, StorageError> {
        let found = self.stems.select_stem_set(id, source)?;
        if found {
            self.broadcast.stems_changed(id);
            self.broadcast.library_changed();
        }
        Ok(found)
    }

    /// Removes the set of this source, or all of them.
    pub fn remove(&self, id: RecordingId, source: Option<&StemSource>) -> Result<(), StorageError> {
        self.stems.remove_stem_sets(id, source)?;
        self.broadcast.stems_changed(id);
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
