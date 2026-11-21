//! What the audio side reports back to the server: which recordings are how
//! long, and whether a sound device is working.

use crate::hub::Hub;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};
use veltrunox_core::{Millis, RecordingId};
use veltrunox_server_app::RecordingLengths;

/// Lengths learned from decoding; unknown until a recording has been decoded once.
#[derive(Default)]
pub struct LengthCache(Mutex<HashMap<RecordingId, Millis>>);

impl LengthCache {
    pub fn insert(&self, id: RecordingId, length: Millis) {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(id, length);
    }
}

impl RecordingLengths for LengthCache {
    fn length(&self, id: RecordingId) -> Option<Millis> {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&id)
            .copied()
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

    #[test]
    fn a_length_is_unknown_until_reported() {
        let cache = LengthCache::default();
        let id = RecordingId::new(3);
        assert_eq!(cache.length(id), None);
        cache.insert(id, Millis::new(5_000));
        assert_eq!(cache.length(id), Some(Millis::new(5_000)));
    }
}
