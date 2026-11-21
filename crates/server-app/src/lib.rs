//! Server use cases and the ports they need.

use veltrunox_core::Recording;

#[derive(Debug, thiserror::Error)]
#[error("recording storage failed: {0}")]
pub struct StorageError(pub String);

/// Port: read access to stored recordings.
pub trait RecordingRepository: Send + Sync {
    fn list(&self) -> Result<Vec<Recording>, StorageError>;
}

/// Use case: list the recordings of the library.
pub fn list_recordings(
    repository: &dyn RecordingRepository,
) -> Result<Vec<Recording>, StorageError> {
    repository.list()
}

#[cfg(test)]
mod tests {
    use super::*;
    use veltrunox_core::{RecordingId, RecordingTitle};

    struct Fixed(Vec<Recording>);

    impl RecordingRepository for Fixed {
        fn list(&self) -> Result<Vec<Recording>, StorageError> {
            Ok(self.0.clone())
        }
    }

    #[test]
    fn returns_what_the_repository_holds() {
        let recording = Recording {
            id: RecordingId::new(1),
            title: RecordingTitle::new("Intro").unwrap(),
        };
        let listed = list_recordings(&Fixed(vec![recording.clone()])).unwrap();
        assert_eq!(listed, vec![recording]);
    }
}
