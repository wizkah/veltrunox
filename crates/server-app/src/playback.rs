//! Use case: the shared playback state, changed by client intents.

use crate::ports::{Clock, PlaybackBroadcast, RecordingRepository, StorageError};
use std::sync::{Arc, Mutex, PoisonError};
use veltrunox_core::{
    Intent, PlaybackState, TransportError, pause, play, seek, select_recording, set_key, set_loop,
    set_speed, set_tuning,
};

#[derive(Debug, thiserror::Error)]
pub enum IntentError {
    #[error("recording does not exist")]
    UnknownRecording,
    #[error(transparent)]
    Transport(#[from] TransportError),
    #[error(transparent)]
    Storage(#[from] StorageError),
}

/// Holds the one playback state. Intents are applied one at a time in arrival
/// order, so every client sees the same result.
pub struct PlaybackService {
    state: Mutex<PlaybackState>,
    recordings: Arc<dyn RecordingRepository>,
    clock: Arc<dyn Clock>,
    broadcast: Arc<dyn PlaybackBroadcast>,
}

impl PlaybackService {
    pub fn new(
        recordings: Arc<dyn RecordingRepository>,
        clock: Arc<dyn Clock>,
        broadcast: Arc<dyn PlaybackBroadcast>,
    ) -> Self {
        let initial = PlaybackState::initial(clock.now());
        Self {
            state: Mutex::new(initial),
            recordings,
            clock,
            broadcast,
        }
    }

    pub fn current(&self) -> PlaybackState {
        *self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Applies the intent, stores changed settings and publishes the new state.
    /// A rejected intent leaves the state untouched.
    pub fn apply(&self, intent: Intent) -> Result<PlaybackState, IntentError> {
        let mut guard = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        let mut next = self.transition(&guard, intent)?;
        if changes_settings(intent)
            && let Some(recording) = next.recording
        {
            self.recordings.save_settings(recording, next.settings)?;
        }
        next.revision = guard.revision + 1;
        *guard = next;
        self.broadcast.state_changed(&next);
        Ok(next)
    }

    fn transition(
        &self,
        state: &PlaybackState,
        intent: Intent,
    ) -> Result<PlaybackState, IntentError> {
        let now = self.clock.now();
        Ok(match intent {
            Intent::SelectRecording(id) => {
                let settings = self
                    .recordings
                    .settings(id)?
                    .ok_or(IntentError::UnknownRecording)?;
                select_recording(state, id, settings, now)
            }
            Intent::Play => play(state, now)?,
            Intent::Pause => pause(state, now)?,
            Intent::Seek {
                position,
                scrubbing,
            } => seek(state, position, scrubbing, now)?,
            Intent::SetSpeed(speed) => set_speed(state, speed, now)?,
            Intent::SetTuning(tuning) => set_tuning(state, tuning)?,
            Intent::SetKey(key) => set_key(state, key)?,
            Intent::SetLoop(range) => set_loop(state, range, now)?,
        })
    }
}

/// Speed, tuning and key are stored with the recording; selecting one only loads them.
fn changes_settings(intent: Intent) -> bool {
    matches!(
        intent,
        Intent::SetSpeed(_) | Intent::SetTuning(_) | Intent::SetKey(_)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{FakeBroadcast, FakeClock, FakeRecordings};
    use veltrunox_core::{Millis, RecordingId, Speed};

    fn service() -> (
        PlaybackService,
        Arc<FakeRecordings>,
        Arc<FakeBroadcast>,
        Arc<FakeClock>,
    ) {
        let recordings = Arc::new(FakeRecordings::with_one());
        let clock = Arc::new(FakeClock::default());
        let broadcast = Arc::new(FakeBroadcast::default());
        let service = PlaybackService::new(recordings.clone(), clock.clone(), broadcast.clone());
        (service, recordings, broadcast, clock)
    }

    #[test]
    fn every_applied_intent_gets_the_next_revision_and_is_published() {
        let (service, _, broadcast, _) = service();
        service
            .apply(Intent::SelectRecording(RecordingId::new(1)))
            .unwrap();
        service.apply(Intent::Play).unwrap();
        let revisions: Vec<u64> = broadcast
            .states()
            .iter()
            .map(|state| state.revision)
            .collect();
        assert_eq!(revisions, [1, 2]);
        assert_eq!(service.current().revision, 2);
    }

    #[test]
    fn a_rejected_intent_changes_and_publishes_nothing() {
        let (service, _, broadcast, _) = service();
        assert!(matches!(
            service.apply(Intent::Play),
            Err(IntentError::Transport(TransportError::NoRecording))
        ));
        assert!(matches!(
            service.apply(Intent::SelectRecording(RecordingId::new(99))),
            Err(IntentError::UnknownRecording)
        ));
        assert!(broadcast.states().is_empty());
        assert_eq!(service.current().revision, 0);
    }

    #[test]
    fn speed_is_stored_with_the_recording_and_restored_on_select() {
        let (service, recordings, _, _) = service();
        let id = RecordingId::new(1);
        service.apply(Intent::SelectRecording(id)).unwrap();
        service
            .apply(Intent::SetSpeed(Speed::new(0.75).unwrap()))
            .unwrap();
        assert_eq!(recordings.stored(id).speed.factor(), 0.75);

        service.apply(Intent::SelectRecording(id)).unwrap();
        assert_eq!(service.current().settings.speed.factor(), 0.75);
    }

    #[test]
    fn the_clock_drives_the_position_reference() {
        let (service, _, _, clock) = service();
        service
            .apply(Intent::SelectRecording(RecordingId::new(1)))
            .unwrap();
        clock.set(Millis::new(1_000));
        let state = service.apply(Intent::Play).unwrap();
        assert_eq!(state.reference.server_time, Millis::new(1_000));
    }
}
