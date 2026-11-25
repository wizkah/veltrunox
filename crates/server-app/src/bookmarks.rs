//! Use cases: edit the bookmarks of a recording and tell the clients.

use crate::ports::{BookmarkError, BookmarkRepository, PlaybackBroadcast, RecordingLengths};
use std::sync::Arc;
use veltrunox_core::{
    BookmarkId, BookmarkName, Bookmarks, EntryRef, ImportConflict, ImportDiff, ImportedBookmarks,
    Label, Millis, NewBookmarks, RecordingId, Region, TimeRange, Version, plan_import,
};

/// How an import is carried out.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ImportMode {
    /// Report what would change and write nothing.
    pub dry_run: bool,
    /// Do not stop at entries whose stored original changed or is gone.
    pub overwrite: bool,
}

/// What an import of one recording came to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportOutcome {
    /// The change; `applied` is false for a dry run.
    Done { diff: ImportDiff, applied: bool },
    /// Entries of the import lie outside the recording; nothing was written.
    Outside(Vec<EntryRef>),
    /// An entry's stored original changed or is gone; nothing was written.
    Conflict(ImportConflict),
}

/// Each successful change sends a notice so connected clients reload the bookmarks.
pub struct BookmarkService {
    repository: Arc<dyn BookmarkRepository>,
    broadcast: Arc<dyn PlaybackBroadcast>,
    lengths: Arc<dyn RecordingLengths>,
}

impl BookmarkService {
    pub fn new(
        repository: Arc<dyn BookmarkRepository>,
        broadcast: Arc<dyn PlaybackBroadcast>,
        lengths: Arc<dyn RecordingLengths>,
    ) -> Self {
        Self {
            repository,
            broadcast,
            lengths,
        }
    }

    pub fn list(&self, recording: RecordingId) -> Result<Bookmarks, BookmarkError> {
        self.repository.list(recording)
    }

    /// Replaces all bookmarks of the recording by `bookmarks`, or changes nothing. As long as
    /// the length of the recording is not known, only the shape of the data is checked.
    pub fn replace_all(
        &self,
        recording: RecordingId,
        bookmarks: &NewBookmarks,
    ) -> Result<Bookmarks, BookmarkError> {
        if let Some(length) = self.lengths.length(recording) {
            bookmarks.check_within(length)?;
        }
        let stored = self.repository.replace_all(recording, bookmarks)?;
        self.broadcast.bookmarks_changed(recording);
        Ok(stored)
    }

    /// Imports the bookmarks of one recording: checks them against its length and against what
    /// is stored, and, unless it is a dry run, replaces the stored set. The stored set is read
    /// before it is replaced, so a change in between is not noticed.
    pub fn import(
        &self,
        recording: RecordingId,
        incoming: &ImportedBookmarks,
        mode: ImportMode,
    ) -> Result<ImportOutcome, BookmarkError> {
        let stored = self.repository.list(recording)?;
        let replacement = incoming.to_new();
        if let Some(length) = self.lengths.length(recording) {
            let outside = replacement.entries_outside(length);
            if !outside.is_empty() {
                return Ok(ImportOutcome::Outside(outside));
            }
        }
        let diff = match plan_import(&stored, incoming, mode.overwrite) {
            Ok(diff) => diff,
            Err(conflict) => return Ok(ImportOutcome::Conflict(conflict)),
        };
        if !mode.dry_run {
            self.replace_all(recording, &replacement)?;
        }
        Ok(ImportOutcome::Done {
            diff,
            applied: !mode.dry_run,
        })
    }

    pub fn add_label(
        &self,
        recording: RecordingId,
        name: BookmarkName,
        position: Millis,
    ) -> Result<Label, BookmarkError> {
        let label = self.repository.add_label(recording, name, position)?;
        self.broadcast.bookmarks_changed(recording);
        Ok(label)
    }

    pub fn update_label(
        &self,
        id: BookmarkId,
        expected: Version,
        name: BookmarkName,
        position: Millis,
    ) -> Result<Label, BookmarkError> {
        let stored = self.repository.update_label(id, expected, name, position)?;
        self.broadcast.bookmarks_changed(stored.recording);
        Ok(stored.item)
    }

    pub fn delete_label(&self, id: BookmarkId, expected: Version) -> Result<(), BookmarkError> {
        let recording = self.repository.delete_label(id, expected)?;
        self.broadcast.bookmarks_changed(recording);
        Ok(())
    }

    pub fn add_region(
        &self,
        recording: RecordingId,
        name: BookmarkName,
        range: TimeRange,
    ) -> Result<Region, BookmarkError> {
        let region = self.repository.add_region(recording, name, range)?;
        self.broadcast.bookmarks_changed(recording);
        Ok(region)
    }

    pub fn update_region(
        &self,
        id: BookmarkId,
        expected: Version,
        name: BookmarkName,
        range: TimeRange,
    ) -> Result<Region, BookmarkError> {
        let stored = self.repository.update_region(id, expected, name, range)?;
        self.broadcast.bookmarks_changed(stored.recording);
        Ok(stored.item)
    }

    pub fn delete_region(&self, id: BookmarkId, expected: Version) -> Result<(), BookmarkError> {
        let recording = self.repository.delete_region(id, expected)?;
        self.broadcast.bookmarks_changed(recording);
        Ok(())
    }

    /// Turns a label into a region; the label disappears, the region has a new id.
    pub fn convert_label_to_region(
        &self,
        id: BookmarkId,
        expected: Version,
        name: BookmarkName,
        range: TimeRange,
    ) -> Result<Region, BookmarkError> {
        let stored = self
            .repository
            .convert_label_to_region(id, expected, name, range)?;
        self.broadcast.bookmarks_changed(stored.recording);
        Ok(stored.item)
    }

    /// Replaces regions of a recording by one region; all or nothing.
    pub fn merge_regions(
        &self,
        recording: RecordingId,
        regions: &[(BookmarkId, Version)],
        name: BookmarkName,
        range: TimeRange,
    ) -> Result<Region, BookmarkError> {
        let region = self
            .repository
            .merge_regions(recording, regions, name, range)?;
        self.broadcast.bookmarks_changed(recording);
        Ok(region)
    }

    /// Turns a region into a label; the region disappears, the label has a new id.
    pub fn convert_region_to_label(
        &self,
        id: BookmarkId,
        expected: Version,
        name: BookmarkName,
        position: Millis,
    ) -> Result<Label, BookmarkError> {
        let stored = self
            .repository
            .convert_region_to_label(id, expected, name, position)?;
        self.broadcast.bookmarks_changed(stored.recording);
        Ok(stored.item)
    }
}
