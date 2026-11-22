//! Use cases: edit the bookmarks of a recording and tell the clients.

use crate::ports::{BookmarkError, BookmarkRepository, PlaybackBroadcast};
use std::sync::Arc;
use veltrunox_core::{
    BookmarkId, BookmarkName, Bookmarks, Label, Millis, RecordingId, Region, TimeRange, Version,
};

/// Each successful change sends a notice so connected clients reload the bookmarks.
pub struct BookmarkService {
    repository: Arc<dyn BookmarkRepository>,
    broadcast: Arc<dyn PlaybackBroadcast>,
}

impl BookmarkService {
    pub fn new(
        repository: Arc<dyn BookmarkRepository>,
        broadcast: Arc<dyn PlaybackBroadcast>,
    ) -> Self {
        Self {
            repository,
            broadcast,
        }
    }

    pub fn list(&self, recording: RecordingId) -> Result<Bookmarks, BookmarkError> {
        self.repository.list(recording)
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
