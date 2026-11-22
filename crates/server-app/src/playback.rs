//! Use case: the shared playback state, changed by client intents.

use crate::ports::{Clock, PlaybackBroadcast, RecordingLengths, RecordingRepository, StorageError};
use std::sync::{Arc, Mutex, PoisonError};
use veltrunox_core::{
    Intent, Millis, PlaybackState, TransportError, end_playback, end_region, pause, play,
    play_region, seek, select_recording, set_key, set_play_mode, set_speed, set_tuning, set_volume,
};

#[derive(Debug, thiserror::Error)]
pub enum IntentError {
    #[error("recording does not exist")]
    UnknownRecording,
    #[error("position {position} ms is beyond the end of the recording ({length} ms)")]
    BeyondEnd { position: u64, length: u64 },
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
    lengths: Arc<dyn RecordingLengths>,
    clock: Arc<dyn Clock>,
    broadcast: Arc<dyn PlaybackBroadcast>,
}

impl PlaybackService {
    /// Starts from `initial`, the state clients already know (see `PlaybackState::initial`).
    pub fn new(
        initial: PlaybackState,
        recordings: Arc<dyn RecordingRepository>,
        lengths: Arc<dyn RecordingLengths>,
        clock: Arc<dyn Clock>,
        broadcast: Arc<dyn PlaybackBroadcast>,
    ) -> Self {
        Self {
            state: Mutex::new(initial),
            recordings,
            lengths,
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
        let next = self.transition(&guard, intent)?;
        if changes_settings(intent)
            && let Some(recording) = next.recording
        {
            self.recordings.save_settings(recording, next.settings)?;
        }
        Ok(self.publish(&mut guard, next))
    }

    /// Pauses at the end of the recording once playback has reached it; returns
    /// the published state, or `None` if there was nothing to do. Meant to be
    /// called regularly: the server, not a client, ends the playback.
    pub fn stop_at_end(&self) -> Option<PlaybackState> {
        let mut guard = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        let length = self.lengths.length(guard.recording?)?;
        let ended = end_playback(&guard, length, self.clock.now())?;
        Some(self.publish(&mut guard, ended))
    }

    /// How long until playback reaches the end of the play range and has to stop; `None` when
    /// it does not stop there. Changes with every published state, so the caller asks again
    /// after each one.
    pub fn region_stop_in(&self) -> Option<Millis> {
        let at = self
            .state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .region_stop_at()?;
        Some(at.saturating_sub(self.clock.now()))
    }

    /// Pauses at the end of the play range once playback has reached it; returns the published
    /// state, or `None` if there was nothing to do. Meant to be called after `region_stop_in`.
    pub fn stop_at_region_end(&self) -> Option<PlaybackState> {
        let mut guard = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        let ended = end_region(&guard, self.clock.now())?;
        Some(self.publish(&mut guard, ended))
    }

    fn publish(&self, current: &mut PlaybackState, mut next: PlaybackState) -> PlaybackState {
        next.revision = current.revision + 1;
        *current = next;
        self.broadcast.state_changed(&next);
        next
    }

    /// A position behind the end, also the end of a play range, is refused once the length is known. While the
    /// audio is still being decoded it is not, and `stop_at_end` catches up.
    fn check_position(&self, state: &PlaybackState, position: Millis) -> Result<(), IntentError> {
        let length = state.recording.and_then(|id| self.lengths.length(id));
        match length {
            Some(length) if position > length => Err(IntentError::BeyondEnd {
                position: position.value(),
                length: length.value(),
            }),
            _ => Ok(()),
        }
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
            } => {
                self.check_position(state, position)?;
                seek(state, position, scrubbing, now)?
            }
            Intent::SetVolume(volume) => set_volume(state, volume),
            Intent::SetSpeed(speed) => set_speed(state, speed, now)?,
            Intent::SetTuning(tuning) => set_tuning(state, tuning)?,
            Intent::SetKey(key) => set_key(state, key)?,
            Intent::PlayRegion(range) => {
                self.check_position(state, range.end())?;
                play_region(state, range, now)?
            }
            Intent::SetPlayMode(mode) => set_play_mode(state, mode, now)?,
        })
    }
}

/// Speed, tuning and key are stored with the recording; selecting one only loads them.
/// The volume belongs to the room and is not stored here.
fn changes_settings(intent: Intent) -> bool {
    matches!(
        intent,
        Intent::SetSpeed(_) | Intent::SetTuning(_) | Intent::SetKey(_)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{FakeBroadcast, FakeClock, FakeLengths, FakeRecordings};
    use veltrunox_core::{
        Millis, PlayMode, PlaybackSettings, RecordingId, Speed, TimeRange, Volume,
    };

    fn service() -> (
        PlaybackService,
        Arc<FakeRecordings>,
        Arc<FakeBroadcast>,
        Arc<FakeClock>,
        Arc<FakeLengths>,
    ) {
        let recordings = Arc::new(FakeRecordings::with_one());
        let clock = Arc::new(FakeClock::default());
        let broadcast = Arc::new(FakeBroadcast::default());
        let lengths = Arc::new(FakeLengths::default());
        let service = PlaybackService::new(
            PlaybackState::initial(clock.now()),
            recordings.clone(),
            lengths.clone(),
            clock.clone(),
            broadcast.clone(),
        );
        (service, recordings, broadcast, clock, lengths)
    }

    #[test]
    fn every_applied_intent_gets_the_next_revision_and_is_published() {
        let (service, _, broadcast, _, _) = service();
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
    fn volume_is_published_to_everyone_but_not_stored_with_a_recording() {
        let (service, recordings, broadcast, _, _) = service();
        let quiet = Volume::new(25).unwrap();
        let state = service.apply(Intent::SetVolume(quiet)).unwrap();
        assert_eq!(state.volume, quiet);
        assert_eq!(
            broadcast.states().last().map(|state| state.volume),
            Some(quiet)
        );
        assert_eq!(
            recordings.stored(RecordingId::new(1)),
            PlaybackSettings::default()
        );
    }

    #[test]
    fn a_rejected_intent_changes_and_publishes_nothing() {
        let (service, _, broadcast, _, _) = service();
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
        let (service, recordings, _, _, _) = service();
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
        let (service, _, _, clock, _) = service();
        service
            .apply(Intent::SelectRecording(RecordingId::new(1)))
            .unwrap();
        clock.set(Millis::new(1_000));
        let state = service.apply(Intent::Play).unwrap();
        assert_eq!(state.reference.server_time, Millis::new(1_000));
    }

    #[test]
    fn a_region_reaching_beyond_the_known_length_is_refused() {
        let (service, _, _, _, lengths) = service();
        let range = |end| TimeRange::new(Millis::new(1_000), Millis::new(end)).unwrap();
        // Nothing selected.
        assert!(matches!(
            service.apply(Intent::PlayRegion(range(2_000))),
            Err(IntentError::Transport(TransportError::NoRecording))
        ));
        service
            .apply(Intent::SelectRecording(RecordingId::new(1)))
            .unwrap();
        // Length not known yet: accepted.
        service.apply(Intent::PlayRegion(range(90_000))).unwrap();
        lengths.set(Millis::new(60_000));
        assert!(matches!(
            service.apply(Intent::PlayRegion(range(60_001))),
            Err(IntentError::BeyondEnd {
                position: 60_001,
                length: 60_000
            })
        ));
        service.apply(Intent::PlayRegion(range(60_000))).unwrap();
    }

    #[test]
    fn a_region_starts_with_one_published_state() {
        let (service, _, broadcast, clock, _) = service();
        service
            .apply(Intent::SelectRecording(RecordingId::new(1)))
            .unwrap();
        clock.set(Millis::new(500));
        let range = TimeRange::new(Millis::new(4_000), Millis::new(6_000)).unwrap();
        let state = service.apply(Intent::PlayRegion(range)).unwrap();
        assert!(state.running);
        assert_eq!(state.play_range, Some(range));
        assert_eq!(state.position_at(Millis::new(500)), Millis::new(4_000));
        assert_eq!(broadcast.states().len(), 2);
    }

    #[test]
    fn playback_pauses_at_the_region_end_at_the_computed_instant() {
        let (service, _, broadcast, clock, _) = service();
        service
            .apply(Intent::SelectRecording(RecordingId::new(1)))
            .unwrap();
        service
            .apply(Intent::SetPlayMode(PlayMode::ToRegionEnd))
            .unwrap();
        clock.set(Millis::new(1_000));
        let range = TimeRange::new(Millis::new(4_000), Millis::new(6_000)).unwrap();
        service.apply(Intent::PlayRegion(range)).unwrap();
        assert_eq!(service.region_stop_in(), Some(Millis::new(2_000)));

        clock.set(Millis::new(2_900));
        assert!(service.stop_at_region_end().is_none());
        clock.set(Millis::new(3_000));
        let ended = service.stop_at_region_end().unwrap();
        assert!(!ended.running);
        assert_eq!(ended.play_range, None);
        assert_eq!(ended.position_at(Millis::new(99_000)), Millis::new(6_000));
        assert_eq!(service.region_stop_in(), None);
        assert!(service.stop_at_region_end().is_none());
        assert_eq!(broadcast.states().len(), 4);
    }

    #[test]
    fn a_seek_beyond_the_known_length_is_refused() {
        let (service, _, broadcast, _, lengths) = service();
        service
            .apply(Intent::SelectRecording(RecordingId::new(1)))
            .unwrap();
        let seek_to = |ms| Intent::Seek {
            position: Millis::new(ms),
            scrubbing: false,
        };
        // Length not known yet: accepted.
        service.apply(seek_to(99_000)).unwrap();
        lengths.set(Millis::new(60_000));
        assert!(matches!(
            service.apply(seek_to(60_001)),
            Err(IntentError::BeyondEnd {
                position: 60_001,
                length: 60_000
            })
        ));
        service.apply(seek_to(60_000)).unwrap();
        assert_eq!(broadcast.states().len(), 3);
    }

    #[test]
    fn playback_pauses_at_the_end_and_publishes_it_once() {
        let (service, _, broadcast, clock, lengths) = service();
        lengths.set(Millis::new(10_000));
        service
            .apply(Intent::SelectRecording(RecordingId::new(1)))
            .unwrap();
        service.apply(Intent::Play).unwrap();
        clock.set(Millis::new(9_000));
        assert!(service.stop_at_end().is_none());

        clock.set(Millis::new(10_300));
        let ended = service.stop_at_end().unwrap();
        assert!(!ended.running);
        assert_eq!(ended.position_at(Millis::new(99_000)), Millis::new(10_000));
        assert_eq!(ended.revision, 3);
        assert!(service.stop_at_end().is_none());
        assert_eq!(broadcast.states().len(), 3);
    }
}
