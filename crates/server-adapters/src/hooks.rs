//! What the audio side reports back to the server: which recordings are how
//! long, and whether a sound device is working.

use crate::hub::Hub;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};
use veltrunox_core::{Millis, RecordingId};
use veltrunox_server_app::{RecordingLengths, WaveformRepository};

/// Lengths learned from decoding. Unknown until a recording has been decoded
/// once, by playback or by the scan whose result the store keeps.
pub struct LengthCache {
    decoded: Mutex<HashMap<RecordingId, Millis>>,
    stored: Arc<dyn WaveformRepository>,
}

impl LengthCache {
    pub fn new(stored: Arc<dyn WaveformRepository>) -> Self {
        Self {
            decoded: Mutex::default(),
            stored,
        }
    }

    pub fn insert(&self, id: RecordingId, length: Millis) {
        self.decoded
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(id, length);
    }
}

impl RecordingLengths for LengthCache {
    fn length(&self, id: RecordingId) -> Option<Millis> {
        let decoded = self
            .decoded
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&id)
            .copied();
        // A store that cannot answer means "not known yet", like a missing length.
        decoded.or_else(|| self.stored.stored_length(id).ok().flatten())
    }
}

/// Handed to the audio playback by `AppState::with_audio`.
#[derive(Clone)]
pub struct AudioHooks {
    pub(crate) hub: Arc<Hub>,
    pub(crate) lengths: Arc<LengthCache>,
}

impl AudioHooks {
    /// The clients are told whether there is sound.
    pub fn output_changed(&self, available: bool) {
        self.hub.set_audio_output(available);
    }

    pub fn recording_length(&self, id: RecordingId, length: Millis) {
        self.lengths.insert(id, length);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use veltrunox_core::Waveform;
    use veltrunox_server_app::StorageError;

    struct StoredLength(Option<Millis>);

    impl WaveformRepository for StoredLength {
        fn save_waveform(
            &self,
            _: RecordingId,
            _: Millis,
            _: &Waveform,
        ) -> Result<(), StorageError> {
            Ok(())
        }

        fn waveform(&self, _: RecordingId) -> Result<Option<Waveform>, StorageError> {
            Ok(None)
        }

        fn stored_length(&self, _: RecordingId) -> Result<Option<Millis>, StorageError> {
            Ok(self.0)
        }
    }

    #[test]
    fn a_length_is_unknown_until_reported() {
        let cache = LengthCache::new(Arc::new(StoredLength(None)));
        let id = RecordingId::new(3);
        assert_eq!(cache.length(id), None);
        cache.insert(id, Millis::new(5_000));
        assert_eq!(cache.length(id), Some(Millis::new(5_000)));
    }

    #[test]
    fn a_stored_length_is_known_without_decoding() {
        let cache = LengthCache::new(Arc::new(StoredLength(Some(Millis::new(7_000)))));
        assert_eq!(cache.length(RecordingId::new(3)), Some(Millis::new(7_000)));
    }
}
