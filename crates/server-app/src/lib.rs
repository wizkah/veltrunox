//! Server use cases and the ports they need.

mod accounts;
mod analysis;
mod api_tokens;
mod bookmarks;
mod playback;
mod ports;
mod stems;
#[cfg(test)]
mod testing;

pub use accounts::{AccessStatus, AccountError, AccountService, Identity, Session};
pub use analysis::AnalysisService;
pub use api_tokens::{API_TOKEN_PREFIX, ApiTokenService, CreatedToken, TokenError};
pub use bookmarks::{BookmarkService, ImportMode, ImportOutcome};
pub use playback::{IntentError, PlaybackService};
pub use ports::{
    AnalysisFailure, AnalysisRepository, ApiTokenRecord, ApiTokenRepository, BookmarkError,
    BookmarkRepository, Clock, HashFailure, MediaCatalog, Owned, PasswordHasher, PlaybackBroadcast,
    RecordingAnalyzer, RecordingLengths, RecordingRepository, RecordingScan, SessionRepository,
    SessionTokens, Stem, StemRepository, StemSeparator, StemSet, StorageError, UserRecord,
    UserRepository, UserStoreError, WallClock, WaveformRepository,
};
pub use stems::{ImportedStems, StemError, StemService, Structure};

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
