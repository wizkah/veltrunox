//! Server use cases and the ports they need.

mod bookmarks;
mod playback;
mod ports;
#[cfg(test)]
mod testing;

pub use bookmarks::BookmarkService;
pub use playback::{IntentError, PlaybackService};
pub use ports::{
    BookmarkError, BookmarkRepository, Clock, MediaCatalog, Owned, PlaybackBroadcast,
    RecordingRepository, StorageError,
};

use veltrunox_core::Recording;

/// Use case: list the recordings of the library.
pub fn list_recordings(
    repository: &dyn RecordingRepository,
) -> Result<Vec<Recording>, StorageError> {
    repository.list()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::FakeRecordings;

    #[test]
    fn lists_what_the_repository_holds() {
        let listed = list_recordings(&FakeRecordings::with_one()).unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].title.as_str(), "Intro");
    }
}
