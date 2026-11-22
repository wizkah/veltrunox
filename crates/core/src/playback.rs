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

/// What happens at the end of the play range. Without a play range it has no effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayMode {
    /// The range is ignored; playback runs to the end of the recording.
    #[default]
    ToEnd,
    /// Playback stops at the end of the range.
    ToRegionEnd,
    /// The position wraps from the end of the range to its start.
    Loop,
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
    /// The region being played; ends with the position leaving it.
    pub play_range: Option<TimeRange>,
    /// Belongs to the room, not to a recording.
    pub play_mode: PlayMode,
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
            play_range: None,
            play_mode: PlayMode::ToEnd,
            scrubbing: false,
            volume: Volume::default(),
        }
    }

    /// The playback position at `now`, following speed, play range and play mode.
    pub fn position_at(&self, now: Millis) -> Millis {
        let start = self.reference.position;
        // While the position handle is dragged the position is the one under it.
        if !self.running || self.scrubbing {
            return start;
        }
        let position = start.saturating_add(self.advanced_by(now));
        match (self.active_range(), self.play_mode) {
            (Some(range), PlayMode::Loop) => wrap_into(range, position),
            (Some(range), PlayMode::ToRegionEnd) => position.min(range.end()),
            _ => position,
        }
    }

    /// The server time at which a running playback reaches the end of the play range and
    /// has to stop; `None` when it does not stop there.
    pub fn region_stop_at(&self) -> Option<Millis> {
        if !self.running || self.scrubbing || self.play_mode != PlayMode::ToRegionEnd {
            return None;
        }
        let range = self.active_range()?;
        let remaining = range.end().value() - self.reference.position.value();
        let wait = (remaining as f64 / self.settings.speed.factor()).ceil() as u64;
        Some(self.reference.server_time.saturating_add(Millis::new(wait)))
    }

    /// The range that governs the position: the play range, while the reference lies inside it.
    fn active_range(&self) -> Option<TimeRange> {
        self.play_range
            .filter(|range| range.contains(self.reference.position))
    }

    fn advanced_by(&self, now: Millis) -> Millis {
        let elapsed = now.saturating_sub(self.reference.server_time).value() as f64;
        Millis::new((elapsed * self.settings.speed.factor()).round() as u64)
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
    Seek {
        position: Millis,
        scrubbing: bool,
    },
    SetSpeed(Speed),
    SetTuning(TuningHz),
    SetKey(KeyShift),
    /// Sets the play range, moves the position to its start and plays, in one step.
    PlayRegion(TimeRange),
    SetPlayMode(PlayMode),
    SetVolume(Volume),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum TransportError {
    #[error("no recording is selected")]
    NoRecording,
}

/// The state with its reference moved to `now`, so a change of speed or play
/// mode applies from this moment on.
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
        play_range: None,
        play_mode: PlayMode::ToEnd,
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
    // The play range ends with the position leaving it.
    let play_range = state.play_range.filter(|range| range.contains(position));
    Ok(PlaybackState {
        reference: PositionReference {
            position,
            server_time: now,
        },
        scrubbing,
        play_range,
        ..*state
    })
}

/// Stops playback at the end of the play range once it has been reached: paused at the range
/// end, play range cleared. `None` before that, so the caller can ask at any time.
pub fn end_region(state: &PlaybackState, now: Millis) -> Option<PlaybackState> {
    let range = state.play_range?;
    if state.region_stop_at()? > now {
        return None;
    }
    Some(PlaybackState {
        running: false,
        scrubbing: false,
        play_range: None,
        reference: PositionReference {
            position: range.end(),
            server_time: now,
        },
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

/// Plays `range` from its start; the play mode stays as it is.
pub fn play_region(
    state: &PlaybackState,
    range: TimeRange,
    now: Millis,
) -> Result<PlaybackState, TransportError> {
    require_recording(state)?;
    Ok(PlaybackState {
        running: true,
        scrubbing: false,
        play_range: Some(range),
        reference: PositionReference {
            position: range.start(),
            server_time: now,
        },
        ..*state
    })
}

/// Changes the play mode, also while playing; it applies from this moment on.
pub fn set_play_mode(
    state: &PlaybackState,
    play_mode: PlayMode,
    now: Millis,
) -> Result<PlaybackState, TransportError> {
    require_recording(state)?;
    Ok(PlaybackState {
        play_mode,
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

    fn region(start: u64, end: u64) -> TimeRange {
        TimeRange::new(ms(start), ms(end)).unwrap()
    }

    fn playing_region(mode: PlayMode) -> PlaybackState {
        let state = set_play_mode(&loaded(0), mode, ms(0)).unwrap();
        play_region(&state, region(10_000, 12_000), ms(0)).unwrap()
    }

    #[test]
    fn play_region_sets_range_position_and_start_in_one_step() {
        let state = play_region(&loaded(0), region(10_000, 12_000), ms(5)).unwrap();
        assert!(state.running && !state.scrubbing);
        assert_eq!(state.play_range, Some(region(10_000, 12_000)));
        assert_eq!(state.position_at(ms(5)), ms(10_000));
        assert_eq!(state.play_mode, PlayMode::ToEnd);
        assert_eq!(
            play_region(&PlaybackState::initial(ms(0)), region(1, 2), ms(0)),
            Err(TransportError::NoRecording)
        );
    }

    #[test]
    fn running_position_wraps_inside_a_loop() {
        let state = playing_region(PlayMode::Loop);
        assert_eq!(state.position_at(ms(1_500)), ms(11_500));
        assert_eq!(state.position_at(ms(2_500)), ms(10_500));
        assert_eq!(state.position_at(ms(4_500)), ms(10_500));
    }

    #[test]
    fn to_end_ignores_the_range() {
        let state = playing_region(PlayMode::ToEnd);
        assert_eq!(state.position_at(ms(5_000)), ms(15_000));
        assert_eq!(state.region_stop_at(), None);
    }

    #[test]
    fn to_region_end_holds_the_position_at_the_range_end() {
        let state = playing_region(PlayMode::ToRegionEnd);
        assert_eq!(state.position_at(ms(1_500)), ms(11_500));
        assert_eq!(state.position_at(ms(5_000)), ms(12_000));
        assert_eq!(state.region_stop_at(), Some(ms(2_000)));
    }

    #[test]
    fn the_stop_instant_follows_the_speed() {
        let state = playing_region(PlayMode::ToRegionEnd);
        let slow = set_speed(&state, Speed::new(0.5).unwrap(), ms(1_000)).unwrap();
        // 1000 ms played, 1000 ms of the range left at half speed.
        assert_eq!(slow.region_stop_at(), Some(ms(3_000)));
        assert_eq!(slow.position_at(ms(3_000)), ms(12_000));
    }

    #[test]
    fn the_region_ends_paused_at_its_end_and_clears_the_range() {
        let state = playing_region(PlayMode::ToRegionEnd);
        assert_eq!(end_region(&state, ms(1_999)), None);
        let ended = end_region(&state, ms(2_030)).unwrap();
        assert!(!ended.running);
        assert_eq!(ended.play_range, None);
        assert_eq!(ended.position_at(ms(60_000)), ms(12_000));
        // A new play runs on to the end of the recording.
        let again = play(&ended, ms(3_000)).unwrap();
        assert_eq!(again.position_at(ms(4_000)), ms(13_000));
        assert_eq!(end_region(&again, ms(60_000)), None);
    }

    #[test]
    fn the_play_mode_can_change_while_playing() {
        let state = playing_region(PlayMode::ToEnd);
        let state = set_play_mode(&state, PlayMode::ToRegionEnd, ms(1_000)).unwrap();
        assert_eq!(state.position_at(ms(1_000)), ms(11_000));
        assert_eq!(state.region_stop_at(), Some(ms(2_000)));
        let state = set_play_mode(&state, PlayMode::Loop, ms(1_500)).unwrap();
        assert_eq!(state.position_at(ms(2_500)), ms(10_500));
    }

    #[test]
    fn the_range_ends_with_the_position_leaving_it() {
        let state = playing_region(PlayMode::Loop);
        let inside = seek(&state, ms(11_000), false, ms(10)).unwrap();
        assert_eq!(inside.play_range, Some(region(10_000, 12_000)));
        let outside = seek(&state, ms(20_000), false, ms(10)).unwrap();
        assert_eq!(outside.play_range, None);
        assert_eq!(outside.play_mode, PlayMode::Loop);
        let paused = pause(&state, ms(500)).unwrap();
        assert_eq!(paused.play_range, Some(region(10_000, 12_000)));
    }

    #[test]
    fn selecting_resets_range_and_mode() {
        let state = playing_region(PlayMode::Loop);
        let state = select_recording(&state, RecordingId::new(2), Default::default(), ms(5));
        assert_eq!(state.play_range, None);
        assert_eq!(state.play_mode, PlayMode::ToEnd);
    }

    #[test]
    fn a_position_outside_the_range_runs_on() {
        let state = set_play_mode(&loaded(0), PlayMode::Loop, ms(0)).unwrap();
        let state = play_region(&state, region(10_000, 12_000), ms(0)).unwrap();
        let state = seek(&state, ms(20_000), false, ms(0)).unwrap();
        let state = play(&state, ms(0)).unwrap();
        assert_eq!(state.position_at(ms(5_000)), ms(25_000));
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
        let looping = set_play_mode(&loaded(0), PlayMode::Loop, ms(0)).unwrap();
        let looping = play_region(&looping, region(1_000, 2_000), ms(0)).unwrap();
        assert_eq!(end_playback(&looping, ms(10_000), ms(60_000)), None);
    }

    #[test]
    fn the_position_holds_while_scrubbing_and_runs_on_after_the_release() {
        let playing = play(&loaded(0), ms(0)).unwrap();
        let scrubbing = seek(&playing, ms(5_000), true, ms(100)).unwrap();
        assert_eq!(scrubbing.position_at(ms(900)), ms(5_000));
        let released = seek(&scrubbing, ms(5_000), false, ms(1_000)).unwrap();
        assert_eq!(released.position_at(ms(1_200)), ms(5_200));
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
