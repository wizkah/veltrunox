//! Translates the shared playback state into engine commands. Pure decisions:
//! the device, the clock and the file loading are passed in.

use super::engine::{Engine, Pcm};
use std::sync::Arc;
use veltrunox_core::{Millis, PlaybackState, RecordingId, TuningHz, pitch_shift_semitones};

/// A position that differs from the expected one by more than this is a jump
/// and seeks the engine; smaller differences are timing noise.
const SEEK_THRESHOLD_MS: u64 = 80;

/// What the audio thread is told to do.
pub enum Command {
    Replace(Box<Engine>),
    Unload,
    Running(bool),
    Speed(f64),
    Pitch(f64),
    Loop(Option<(u64, u64)>),
    Seek(u64),
}

/// Where commands go: the audio thread of a device, or a recorder in tests.
pub trait CommandSink {
    fn output_rate(&self) -> u32;
    fn send(&mut self, command: Command);
}

#[derive(Default)]
pub struct PlayerCore {
    loaded: Option<RecordingId>,
    engine_ready: bool,
    last: Option<PlaybackState>,
    /// The pitch shift the engine was last told, so a changed tuning reference
    /// (a finished analysis) reaches it although the state did not change.
    pitch: Option<f64>,
}

impl PlayerCore {
    /// The newest state applied, for applying it again after the tuning reference changed.
    pub fn last_state(&self) -> Option<PlaybackState> {
        self.last
    }

    /// Brings the engine in line with `state`. `load` supplies the audio of a
    /// recording (`None` if it has no playable file); `tuning_of` its own tuning.
    pub fn apply(
        &mut self,
        state: &PlaybackState,
        now: Millis,
        sink: &mut dyn CommandSink,
        load: &mut dyn FnMut(RecordingId) -> Option<Arc<Pcm>>,
        tuning_of: &dyn Fn(RecordingId) -> TuningHz,
    ) {
        let switched = state.recording != self.loaded;
        if switched {
            self.switch_recording(state.recording, sink, load);
        }
        if self.engine_ready {
            self.follow(state, now, switched, sink, tuning_of);
        }
        self.last = Some(*state);
    }

    fn switch_recording(
        &mut self,
        recording: Option<RecordingId>,
        sink: &mut dyn CommandSink,
        load: &mut dyn FnMut(RecordingId) -> Option<Arc<Pcm>>,
    ) {
        self.loaded = recording;
        self.engine_ready = false;
        self.pitch = None;
        match recording.and_then(load) {
            Some(pcm) => {
                sink.send(Command::Replace(Box::new(Engine::new(
                    pcm,
                    sink.output_rate(),
                ))));
                self.engine_ready = true;
            }
            None => sink.send(Command::Unload),
        }
    }

    fn follow(
        &mut self,
        state: &PlaybackState,
        now: Millis,
        switched: bool,
        sink: &mut dyn CommandSink,
        tuning_of: &dyn Fn(RecordingId) -> TuningHz,
    ) {
        let previous = self.last.filter(|_| !switched);
        let own_tuning = state.recording.map_or(TuningHz::CONCERT_PITCH, tuning_of);
        let pitch = pitch_shift_semitones(&state.settings, own_tuning);
        if self.pitch != Some(pitch) {
            self.pitch = Some(pitch);
            sink.send(Command::Pitch(pitch));
        }
        if previous.is_none_or(|last| last.settings.speed != state.settings.speed) {
            sink.send(Command::Speed(state.settings.speed.factor()));
        }
        if previous.is_none_or(|last| last.loop_range != state.loop_range) {
            sink.send(Command::Loop(
                state
                    .loop_range
                    .map(|range| (range.start().value(), range.end().value())),
            ));
        }
        let target = state.position_at(now);
        let jumped = previous.is_none_or(|last| {
            last.position_at(now).value().abs_diff(target.value()) > SEEK_THRESHOLD_MS
        });
        // While scrubbing the room hears nothing (ADR-005 fallback) but the
        // engine follows, so the release lands on the right place.
        if jumped || state.scrubbing {
            sink.send(Command::Seek(target.value()));
        }
        let audible = state.running && !state.scrubbing;
        if previous.is_none_or(|last| (last.running && !last.scrubbing) != audible) || jumped {
            sink.send(Command::Running(audible));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use veltrunox_core::{
        KeyShift, Speed, TimeRange, pause, play, seek, select_recording, set_key, set_loop,
        set_speed,
    };

    #[derive(Debug, PartialEq)]
    enum Seen {
        Replace,
        Unload,
        Running(bool),
        Speed(f64),
        Pitch(f64),
        Loop(Option<(u64, u64)>),
        Seek(u64),
    }

    #[derive(Default)]
    struct Recorder(Vec<Seen>);

    impl CommandSink for Recorder {
        fn output_rate(&self) -> u32 {
            44_100
        }
        fn send(&mut self, command: Command) {
            self.0.push(match command {
                Command::Replace(_) => Seen::Replace,
                Command::Unload => Seen::Unload,
                Command::Running(running) => Seen::Running(running),
                Command::Speed(speed) => Seen::Speed(speed),
                Command::Pitch(pitch) => Seen::Pitch(pitch),
                Command::Loop(range) => Seen::Loop(range),
                Command::Seek(position) => Seen::Seek(position),
            });
        }
    }

    const ID: RecordingId = RecordingId::new(1);

    fn audio() -> Option<Arc<Pcm>> {
        Some(Arc::new(Pcm::new(vec![0.0; 44_100 * 2], 44_100)))
    }

    fn selected() -> PlaybackState {
        let state = PlaybackState::initial(Millis::ZERO);
        select_recording(&state, ID, Default::default(), Millis::ZERO)
    }

    fn apply(core: &mut PlayerCore, state: &PlaybackState, now: u64) -> Vec<Seen> {
        let mut recorder = Recorder::default();
        core.apply(
            state,
            Millis::new(now),
            &mut recorder,
            &mut |_| audio(),
            &|_| TuningHz::CONCERT_PITCH,
        );
        recorder.0
    }

    #[test]
    fn selecting_loads_the_engine_and_its_settings() {
        let mut core = PlayerCore::default();
        let seen = apply(&mut core, &selected(), 0);
        assert_eq!(
            seen,
            [
                Seen::Replace,
                Seen::Pitch(0.0),
                Seen::Speed(1.0),
                Seen::Loop(None),
                Seen::Seek(0),
                Seen::Running(false)
            ]
        );
    }

    #[test]
    fn play_and_pause_start_and_stop_without_a_seek() {
        let mut core = PlayerCore::default();
        let state = selected();
        apply(&mut core, &state, 0);
        let playing = play(&state, Millis::new(100)).unwrap();
        assert_eq!(apply(&mut core, &playing, 100), [Seen::Running(true)]);
        let paused = pause(&playing, Millis::new(600)).unwrap();
        assert_eq!(apply(&mut core, &paused, 600), [Seen::Running(false)]);
    }

    #[test]
    fn a_jump_seeks_and_a_continuing_position_does_not() {
        let mut core = PlayerCore::default();
        let state = play(&selected(), Millis::ZERO).unwrap();
        apply(&mut core, &state, 0);
        let jumped = seek(&state, Millis::new(30_000), false, Millis::new(1_000)).unwrap();
        let seen = apply(&mut core, &jumped, 1_000);
        assert!(seen.contains(&Seen::Seek(30_000)));
        // Same timeline a moment later: nothing to do.
        assert!(apply(&mut core, &jumped, 2_000).is_empty());
    }

    #[test]
    fn scrubbing_follows_silently_and_the_release_resumes() {
        let mut core = PlayerCore::default();
        let state = play(&selected(), Millis::ZERO).unwrap();
        apply(&mut core, &state, 0);
        let scrub = seek(&state, Millis::new(5_000), true, Millis::new(100)).unwrap();
        let seen = apply(&mut core, &scrub, 100);
        assert_eq!(seen, [Seen::Seek(5_000), Seen::Running(false)]);
        let released = seek(&scrub, Millis::new(5_040), false, Millis::new(150)).unwrap();
        let seen = apply(&mut core, &released, 150);
        assert_eq!(seen, [Seen::Running(true)]);
    }

    #[test]
    fn speed_key_and_loop_changes_reach_the_engine() {
        let mut core = PlayerCore::default();
        let state = selected();
        apply(&mut core, &state, 0);
        let faster = set_speed(&state, Speed::new(1.5).unwrap(), Millis::ZERO).unwrap();
        assert_eq!(apply(&mut core, &faster, 0), [Seen::Speed(1.5)]);
        let keyed = set_key(&faster, KeyShift::new(2).unwrap()).unwrap();
        assert_eq!(apply(&mut core, &keyed, 0), [Seen::Pitch(2.0)]);
        let range = TimeRange::new(Millis::new(1_000), Millis::new(2_000)).unwrap();
        let looped = set_loop(&keyed, Some(range), Millis::ZERO).unwrap();
        assert_eq!(
            apply(&mut core, &looped, 0),
            [Seen::Loop(Some((1_000, 2_000)))]
        );
    }

    #[test]
    fn the_recordings_own_tuning_sets_the_pitch_and_a_late_analysis_corrects_it() {
        let mut core = PlayerCore::default();
        let state = selected();
        let mut recorder = Recorder::default();
        let load = &mut |_: RecordingId| audio();
        // Settings ask for 440 Hz; the recording is analysed at 466.16 Hz (a semitone up).
        let tuned = |_: RecordingId| TuningHz::new(466.1638).unwrap();
        core.apply(&state, Millis::ZERO, &mut recorder, load, &tuned);
        let Some(Seen::Pitch(shift)) = recorder.0.into_iter().find(|s| matches!(s, Seen::Pitch(_)))
        else {
            panic!("no pitch command");
        };
        assert!((shift + 1.0).abs() < 1e-3, "shift {shift}");

        // The analysis finishes later: the same state is applied again and only the pitch moves.
        let mut later = Recorder::default();
        core.apply(&state, Millis::ZERO, &mut later, load, &|_| {
            TuningHz::CONCERT_PITCH
        });
        assert_eq!(later.0, [Seen::Pitch(0.0)]);
    }

    #[test]
    fn a_recording_without_audio_unloads_and_ignores_the_rest() {
        let mut core = PlayerCore::default();
        let mut recorder = Recorder::default();
        let state = play(&selected(), Millis::ZERO).unwrap();
        core.apply(&state, Millis::ZERO, &mut recorder, &mut |_| None, &|_| {
            TuningHz::CONCERT_PITCH
        });
        assert_eq!(recorder.0, [Seen::Unload]);
    }
}
