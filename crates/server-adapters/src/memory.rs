use veltrunox_core::Recording;
use veltrunox_server_app::{RecordingRepository, StorageError};

/// Temporary storage for the vertical slice; replaced by persistence later.
pub struct InMemoryRecordingRepository {
    recordings: Vec<Recording>,
}

impl InMemoryRecordingRepository {
    pub fn new(recordings: Vec<Recording>) -> Self {
        Self { recordings }
    }
}

impl RecordingRepository for InMemoryRecordingRepository {
    fn list(&self) -> Result<Vec<Recording>, StorageError> {
        Ok(self.recordings.clone())
    }
}
