//! The shared playback state and the pure operations that change it.

use crate::recording::RecordingId;
use crate::units::{KeyShift, Millis, Speed, TimeRange, TuningHz, Volume};

/// Settings stored with a recording and restored when it is loaded.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PlaybackSettings {
    pub speed: Speed,
    pub tuning: TuningHz,
    pub key: KeyShift,
}

impl Default for Speed {
    fn default() -> Self {
        Self::NORMAL
    }
}

impl Default for TuningHz {
    fn default() -> Self {
        Self::CONCERT_PITCH
    }
}

impl Default for Volume {
    fn default() -> Self {
        Self::FULL
    }
}

impl Default for KeyShift {
    fn default() -> Self {
        Self::NONE
    }
}

/// A position that is valid at a given server time; while running, the
/// position at a later time is extrapolated from it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PositionReference {
    pub position: Millis,
    pub server_time: Millis,
}

/// What every client sees. The server is the only writer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlaybackState {
    /// Strictly increasing; clients ignore anything older than what they hold.
    pub revision: u64,
    pub recording: Option<RecordingId>,
    pub reference: PositionReference,
    pub running: bool,
    pub settings: PlaybackSettings,
    pub loop_range: Option<TimeRange>,
    pub scrubbing: bool,
    /// Output volume of the server; belongs to the room, not to a recording.
    pub volume: Volume,
}

impl PlaybackState {
    /// The state after a server start: nothing selected, paused.
    pub fn initial(now: Millis) -> Self {
        Self {
            revision: 0,
            recording: None,
            reference: PositionReference {
                position: Millis::ZERO,
                server_time: now,
            },
            running: false,
            settings: PlaybackSettings::default(),
            loop_range: None,
            scrubbing: false,
            volume: Volume::default(),
        }
    }

    /// The playback position at `now`, following speed and loop range.
    pub fn position_at(&self, now: Millis) -> Millis {
        let start = self.reference.position;
        if !self.running {
            return start;
        }
        let elapsed = now.saturating_sub(self.reference.server_time).value() as f64;
        let advanced = (elapsed * self.settings.speed.factor()).round() as u64;
        let position = start.saturating_add(Millis::new(advanced));
        match self.loop_range {
            Some(range) if range.contains(start) => wrap_into(range, position),
            _ => position,
        }
    }
}

/// Folds a position that ran past the end of `range` back into it.
fn wrap_into(range: TimeRange, position: Millis) -> Millis {
    if position < range.end() {
        return position;
    }
    let overshoot = position.value() - range.start().value();
    Millis::new(range.start().value() + overshoot % range.length().value())
}

/// What a client asks the server to do.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Intent {
    SelectRecording(RecordingId),
    Play,
    Pause,
    Seek { position: Millis, scrubbing: bool },
    SetSpeed(Speed),
    SetTuning(TuningHz),
    SetKey(KeyShift),
    SetLoop(Option<TimeRange>),
    SetVolume(Volume),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum TransportError {
    #[error("no recording is selected")]
    NoRecording,
}

/// The state with its reference moved to `now`, so a change of speed or loop
/// range applies from this moment on.
fn rebased(state: &PlaybackState, now: Millis) -> PlaybackState {
    PlaybackState {
        reference: PositionReference {
            position: state.position_at(now),
            server_time: now,
        },
        ..*state
    }
}

fn require_recording(state: &PlaybackState) -> Result<(), TransportError> {
    state
        .recording
        .map(|_| ())
        .ok_or(TransportError::NoRecording)
}

/// Loads a recording with its stored settings; playback starts paused at the beginning.
pub fn select_recording(
    state: &PlaybackState,
    recording: RecordingId,
    settings: PlaybackSettings,
    now: Millis,
) -> PlaybackState {
    PlaybackState {
        recording: Some(recording),
        reference: PositionReference {
            position: Millis::ZERO,
            server_time: now,
        },
        running: false,
        settings,
        loop_range: None,
        scrubbing: false,
        ..*state
    }
}

pub fn play(state: &PlaybackState, now: Millis) -> Result<PlaybackState, TransportError> {
    require_recording(state)?;
    Ok(PlaybackState {
        running: true,
        scrubbing: false,
        ..rebased(state, now)
    })
}

pub fn pause(state: &PlaybackState, now: Millis) -> Result<PlaybackState, TransportError> {
    require_recording(state)?;
    Ok(PlaybackState {
        running: false,
        scrubbing: false,
        ..rebased(state, now)
    })
}

pub fn seek(
    state: &PlaybackState,
    position: Millis,
    scrubbing: bool,
    now: Millis,
) -> Result<PlaybackState, TransportError> {
    require_recording(state)?;
    Ok(PlaybackState {
        reference: PositionReference {
            position,
            server_time: now,
        },
        scrubbing,
        ..*state
    })
}

/// Stops playback at the end of the recording: paused at `length`. `None` while
/// playback has not reached the end (or is not running), so the caller can
/// ask at any time.
pub fn end_playback(state: &PlaybackState, length: Millis, now: Millis) -> Option<PlaybackState> {
    if !state.running || state.position_at(now) < length {
        return None;
    }
    Some(PlaybackState {
        running: false,
        scrubbing: false,
        reference: PositionReference {
            position: length,
            server_time: now,
        },
        ..*state
    })
}

pub fn set_speed(
    state: &PlaybackState,
    speed: Speed,
    now: Millis,
) -> Result<PlaybackState, TransportError> {
    require_recording(state)?;
    let moved = rebased(state, now);
    Ok(PlaybackState {
        settings: PlaybackSettings {
            speed,
            ..state.settings
        },
        ..moved
    })
}

pub fn set_tuning(
    state: &PlaybackState,
    tuning: TuningHz,
) -> Result<PlaybackState, TransportError> {
    require_recording(state)?;
    Ok(PlaybackState {
        settings: PlaybackSettings {
            tuning,
            ..state.settings
        },
        ..*state
    })
}

pub fn set_key(state: &PlaybackState, key: KeyShift) -> Result<PlaybackState, TransportError> {
    require_recording(state)?;
    Ok(PlaybackState {
        settings: PlaybackSettings {
            key,
            ..state.settings
        },
        ..*state
    })
}

/// Changes the output volume; possible without a selected recording.
pub fn set_volume(state: &PlaybackState, volume: Volume) -> PlaybackState {
    PlaybackState { volume, ..*state }
}

pub fn set_loop(
    state: &PlaybackState,
    loop_range: Option<TimeRange>,
    now: Millis,
) -> Result<PlaybackState, TransportError> {
    require_recording(state)?;
    Ok(PlaybackState {
        loop_range,
        ..rebased(state, now)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ms(value: u64) -> Millis {
        Millis::new(value)
    }

    fn loaded(now: u64) -> PlaybackState {
        select_recording(
            &PlaybackState::initial(ms(now)),
            RecordingId::new(1),
            PlaybackSettings::default(),
            ms(now),
        )
    }

    #[test]
    fn nothing_works_without_a_recording() {
        let state = PlaybackState::initial(ms(0));
        assert_eq!(play(&state, ms(1)), Err(TransportError::NoRecording));
        assert_eq!(
            seek(&state, ms(5), false, ms(1)),
            Err(TransportError::NoRecording)
        );
    }

    #[test]
    fn position_stands_still_while_paused() {
        let state = seek(&loaded(0), ms(4_000), false, ms(10)).unwrap();
        assert_eq!(state.position_at(ms(9_999)), ms(4_000));
    }

    #[test]
    fn position_advances_with_speed_while_running() {
        let state = play(&loaded(0), ms(100)).unwrap();
        let state = set_speed(&state, Speed::new(0.5).unwrap(), ms(1_100)).unwrap();
        assert_eq!(state.position_at(ms(1_100)), ms(1_000));
        assert_eq!(state.position_at(ms(3_100)), ms(2_000));
    }

    #[test]
    fn pause_freezes_the_current_position() {
        let state = play(&loaded(0), ms(0)).unwrap();
        let state = pause(&state, ms(2_500)).unwrap();
        assert!(!state.running);
        assert_eq!(state.position_at(ms(60_000)), ms(2_500));
    }

    #[test]
    fn volume_changes_without_a_recording_and_survives_selecting() {
        let quiet = Volume::new(30).unwrap();
        let state = set_volume(&PlaybackState::initial(ms(0)), quiet);
        assert_eq!(state.volume, quiet);
        let selected = select_recording(&state, RecordingId::new(1), Default::default(), ms(5));
        assert_eq!(selected.volume, quiet);
    }

    #[test]
    fn selecting_applies_stored_settings_and_starts_paused() {
        let settings = PlaybackSettings {
            speed: Speed::new(0.8).unwrap(),
            tuning: TuningHz::new(412.0).unwrap(),
            key: KeyShift::new(-2).unwrap(),
        };
        let playing = play(&loaded(0), ms(0)).unwrap();
        let state = select_recording(&playing, RecordingId::new(2), settings, ms(50));
        assert_eq!(state.recording, Some(RecordingId::new(2)));
        assert_eq!(state.settings, settings);
        assert!(!state.running);
        assert_eq!(state.position_at(ms(50)), Millis::ZERO);
    }

    #[test]
    fn running_position_wraps_inside_a_loop() {
        let range = TimeRange::new(ms(10_000), ms(12_000)).unwrap();
        let state = seek(&loaded(0), ms(10_000), false, ms(0)).unwrap();
        let state = set_loop(&state, Some(range), ms(0)).unwrap();
        let state = play(&state, ms(0)).unwrap();
        assert_eq!(state.position_at(ms(1_500)), ms(11_500));
        assert_eq!(state.position_at(ms(2_500)), ms(10_500));
        assert_eq!(state.position_at(ms(4_500)), ms(10_500));
    }

    #[test]
    fn a_position_outside_the_loop_runs_on() {
        let range = TimeRange::new(ms(10_000), ms(12_000)).unwrap();
        let state = set_loop(&loaded(0), Some(range), ms(0)).unwrap();
        let state = play(&state, ms(0)).unwrap();
        assert_eq!(state.position_at(ms(20_000)), ms(20_000));
    }

    #[test]
    fn playback_stops_paused_at_the_end() {
        let state = play(&loaded(0), ms(0)).unwrap();
        assert_eq!(end_playback(&state, ms(10_000), ms(9_999)), None);
        let ended = end_playback(&state, ms(10_000), ms(10_200)).unwrap();
        assert!(!ended.running);
        assert_eq!(ended.position_at(ms(60_000)), ms(10_000));
    }

    #[test]
    fn a_paused_or_looping_state_never_ends() {
        let paused = seek(&loaded(0), ms(10_000), false, ms(0)).unwrap();
        assert_eq!(end_playback(&paused, ms(10_000), ms(50)), None);
        let range = TimeRange::new(ms(1_000), ms(2_000)).unwrap();
        let state = seek(&loaded(0), ms(1_000), false, ms(0)).unwrap();
        let state = set_loop(&state, Some(range), ms(0)).unwrap();
        let state = play(&state, ms(0)).unwrap();
        assert_eq!(end_playback(&state, ms(10_000), ms(60_000)), None);
    }

    #[test]
    fn scrubbing_flag_follows_seek_and_ends_with_play_or_pause() {
        let scrubbing = seek(&loaded(0), ms(500), true, ms(1)).unwrap();
        assert!(scrubbing.scrubbing);
        let released = seek(&scrubbing, ms(600), false, ms(2)).unwrap();
        assert!(!released.scrubbing);
        assert!(!play(&scrubbing, ms(3)).unwrap().scrubbing);
    }

    #[test]
    fn tuning_and_key_change_only_their_setting() {
        let state = set_tuning(&loaded(0), TuningHz::new(432.0).unwrap()).unwrap();
        let state = set_key(&state, KeyShift::new(3).unwrap()).unwrap();
        assert_eq!(state.settings.tuning.hertz(), 432.0);
        assert_eq!(state.settings.key.semitones(), 3);
        assert_eq!(state.settings.speed, Speed::NORMAL);
    }
}
