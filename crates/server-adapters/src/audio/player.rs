//! Translates the shared playback state into engine commands. Pure decisions:
//! the device, the clock and the file loading are passed in.

use super::engine::{Engine, Pcm};
use std::sync::Arc;
use veltrunox_core::{
    Millis, PlayMode, PlaybackState, RecordingId, StemMix, TuningHz, Volume, pitch_shift_semitones,
};

/// The audio of a recording, ready to play.
pub struct Audio {
    pub original: Arc<Pcm>,
    /// Empty unless they were asked for and the recording has them.
    pub stems: Vec<Arc<Pcm>>,
}

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
    /// The range that wraps; `None` plays on.
    Loop(Option<(u64, u64)>),
    /// The position where playback ends; `None` plays on.
    Stop(Option<u64>),
    Seek(u64),
    /// Which of original and stems sound, and how loud.
    Mix(Box<StemMix>),
    /// Plays a short piece at the position, for the dragged position handle.
    Scrub(u64),
    /// The output volume of the room; the output decides how it is applied.
    Volume(Volume),
    /// Amplitude factor of the render thread (the volume where there is no system mixer).
    Gain(f32),
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
    /// Whether the loaded audio was asked for with its stems. They are only decoded once a
    /// mix uses them, since each costs as much memory as the recording.
    with_stems: bool,
    last: Option<PlaybackState>,
    /// The pitch shift the engine was last told, so a changed tuning reference
    /// (a finished analysis) reaches it although the state did not change.
    pitch: Option<f64>,
    /// The volume the output has; a state with another one changes it.
    volume: Volume,
}

impl PlayerCore {
    /// A player whose output starts at `volume`, which the first state carries too.
    pub fn with_volume(volume: Volume) -> Self {
        Self {
            volume,
            ..Self::default()
        }
    }

    /// The newest state applied, for applying it again after the tuning reference changed.
    pub fn last_state(&self) -> Option<PlaybackState> {
        self.last
    }

    /// Forgets the loaded audio of the recording, so that the next state loads it again.
    pub fn stems_changed(&mut self, recording: RecordingId) {
        if self.loaded == Some(recording) {
            self.loaded = None;
        }
    }

    /// Brings the engine in line with `state`. `load` supplies the audio of a
    /// recording (`None` if it has no playable file); `tuning_of` its own tuning.
    pub fn apply(
        &mut self,
        state: &PlaybackState,
        now: Millis,
        sink: &mut dyn CommandSink,
        load: &mut dyn FnMut(RecordingId, bool) -> Option<Audio>,
        tuning_of: &dyn Fn(RecordingId) -> TuningHz,
    ) {
        // The volume is the room's, so it follows even without a loaded recording.
        if self.volume != state.volume {
            self.volume = state.volume;
            sink.send(Command::Volume(state.volume));
        }
        let wants_stems = state.settings.mix.enabled;
        let switched = state.recording != self.loaded || (wants_stems && !self.with_stems);
        if switched {
            self.with_stems = wants_stems;
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
        load: &mut dyn FnMut(RecordingId, bool) -> Option<Audio>,
    ) {
        self.loaded = recording;
        self.engine_ready = false;
        self.pitch = None;
        let with_stems = self.with_stems;
        match recording.and_then(|id| load(id, with_stems)) {
            Some(audio) => {
                let engine = Engine::new(audio.original, sink.output_rate());
                sink.send(Command::Replace(Box::new(engine.with_stems(audio.stems))));
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
        if previous.is_none_or(|last| last.settings.mix != state.settings.mix) {
            sink.send(Command::Mix(Box::new(state.settings.mix)));
        }
        let own_tuning = state.recording.map_or(TuningHz::CONCERT_PITCH, tuning_of);
        let pitch = pitch_shift_semitones(&state.settings, own_tuning);
        if self.pitch != Some(pitch) {
            self.pitch = Some(pitch);
            sink.send(Command::Pitch(pitch));
        }
        if previous.is_none_or(|last| last.settings.speed != state.settings.speed) {
            sink.send(Command::Speed(state.settings.speed.factor()));
        }
        if previous.is_none_or(|last| looping(&last) != looping(state)) {
            sink.send(Command::Loop(looping(state)));
        }
        if previous.and_then(|last| stopping(&last)) != stopping(state) {
            sink.send(Command::Stop(stopping(state)));
        }
        let target = state.position_at(now);
        let jumped = previous.is_none_or(|last| {
            last.position_at(now).value().abs_diff(target.value()) > SEEK_THRESHOLD_MS
        });
        let scrubbing = state.scrubbing;
        // The sound of a drag is a short piece per position (below); on the release the
        // engine is put on the final position, wherever the last piece left it.
        let released = previous.is_some_and(|last| last.scrubbing) && !scrubbing;
        if (jumped && !scrubbing) || released {
            sink.send(Command::Seek(target.value()));
        }
        let audible = state.running && !scrubbing;
        if previous.is_none_or(|last| (last.running && !last.scrubbing) != audible)
            || (jumped && !scrubbing)
            || released
        {
            sink.send(Command::Running(audible));
        }
        let dragged = scrubbing
            && previous.is_none_or(|last| {
                !last.scrubbing || last.reference.position != state.reference.position
            });
        if dragged {
            sink.send(Command::Scrub(target.value()));
        }
        // The engine stops a little after the shared position froze. Putting it on that
        // position keeps repeated pauses from moving the sound ahead of the shared state.
        let paused = previous.is_some_and(|last| last.running && !last.scrubbing) && !audible;
        if paused && !jumped && !state.scrubbing {
            sink.send(Command::Seek(target.value()));
        }
    }
}

/// The range the engine wraps in: the play range in loop mode.
fn looping(state: &PlaybackState) -> Option<(u64, u64)> {
    state
        .play_range
        .filter(|_| state.play_mode == PlayMode::Loop)
        .map(|range| (range.start().value(), range.end().value()))
}

/// The position the engine stops at: the end of the play range in to-region-end mode.
fn stopping(state: &PlaybackState) -> Option<u64> {
    state
        .play_range
        .filter(|_| state.play_mode == PlayMode::ToRegionEnd)
        .map(|range| range.end().value())
}

#[cfg(test)]
mod tests {
    use super::*;
    use veltrunox_core::{
        KeyShift, Speed, TimeRange, end_region, pause, play, play_region, seek, select_recording,
        set_key, set_play_mode, set_speed,
    };

    #[derive(Debug, PartialEq)]
    enum Seen {
        Replace,
        Unload,
        Running(bool),
        Speed(f64),
        Pitch(f64),
        Loop(Option<(u64, u64)>),
        Stop(Option<u64>),
        Seek(u64),
        Mix(bool),
        Scrub(u64),
        Volume(u8),
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
                Command::Stop(position) => Seen::Stop(position),
                Command::Seek(position) => Seen::Seek(position),
                Command::Mix(mix) => Seen::Mix(mix.enabled),
                Command::Scrub(position) => Seen::Scrub(position),
                Command::Volume(volume) => Seen::Volume(volume.percent()),
                Command::Gain(_) => unreachable!("only the output sends a gain"),
            });
        }
    }

    const ID: RecordingId = RecordingId::new(1);

    fn audio() -> Option<Audio> {
        Some(Audio {
            original: Arc::new(Pcm::new(vec![0.0; 44_100 * 2], 44_100)),
            stems: Vec::new(),
        })
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
            &mut |_, _| audio(),
            &|_| TuningHz::CONCERT_PITCH,
        );
        recorder.0
    }

    #[test]
    fn stems_are_loaded_only_once_the_mix_uses_them() {
        let mut core = PlayerCore::default();
        let mut asked = Vec::new();
        let mut apply = |core: &mut PlayerCore, state: &PlaybackState| {
            core.apply(
                state,
                Millis::ZERO,
                &mut Recorder::default(),
                &mut |_, with_stems| {
                    asked.push(with_stems);
                    audio()
                },
                &|_| TuningHz::CONCERT_PITCH,
            );
        };
        let off = selected();
        let mut on = off;
        on.settings.mix.enabled = true;
        apply(&mut core, &off);
        apply(&mut core, &on);
        apply(&mut core, &off);
        apply(&mut core, &on);
        core.stems_changed(ID);
        apply(&mut core, &on);
        assert_eq!(asked, [false, true, true]);
    }

    #[test]
    fn selecting_loads_the_engine_and_its_settings() {
        let mut core = PlayerCore::default();
        let seen = apply(&mut core, &selected(), 0);
        assert_eq!(
            seen,
            [
                Seen::Replace,
                Seen::Mix(false),
                Seen::Pitch(0.0),
                Seen::Speed(1.0),
                Seen::Loop(None),
                Seen::Seek(0),
                Seen::Running(false)
            ]
        );
    }

    #[test]
    fn play_starts_without_a_seek_and_pause_aligns_the_engine() {
        let mut core = PlayerCore::default();
        let state = selected();
        apply(&mut core, &state, 0);
        let playing = play(&state, Millis::new(100)).unwrap();
        assert_eq!(apply(&mut core, &playing, 100), [Seen::Running(true)]);
        let paused = pause(&playing, Millis::new(600)).unwrap();
        // Playing since 100 ms at speed 1: the frozen position is 500 ms.
        assert_eq!(
            apply(&mut core, &paused, 600),
            [Seen::Running(false), Seen::Seek(500)]
        );
        // Playing again continues from there without a further seek.
        let again = play(&paused, Millis::new(700)).unwrap();
        assert_eq!(apply(&mut core, &again, 700), [Seen::Running(true)]);
    }

    #[test]
    fn a_volume_change_reaches_the_output_even_without_a_recording() {
        use veltrunox_core::{Volume, set_volume};
        let mut core = PlayerCore::default();
        let initial = PlaybackState::initial(Millis::ZERO);
        assert!(apply(&mut core, &initial, 0).is_empty());
        let quiet = set_volume(&initial, Volume::new(30).unwrap());
        assert_eq!(apply(&mut core, &quiet, 10), [Seen::Volume(30)]);
        assert!(apply(&mut core, &quiet, 20).is_empty());
    }

    #[test]
    fn the_first_volume_change_after_the_start_is_applied_too() {
        use veltrunox_core::{Volume, set_volume};
        let device_volume = Volume::new(60).unwrap();
        let mut core = PlayerCore::with_volume(device_volume);
        let initial = set_volume(&PlaybackState::initial(Millis::ZERO), device_volume);
        assert!(apply(&mut core, &initial, 0).is_empty());
        let louder = set_volume(&initial, Volume::new(80).unwrap());
        assert_eq!(apply(&mut core, &louder, 10), [Seen::Volume(80)]);
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
    fn scrubbing_plays_a_piece_per_position_and_the_release_resumes() {
        let mut core = PlayerCore::default();
        let state = play(&selected(), Millis::ZERO).unwrap();
        apply(&mut core, &state, 0);
        let scrub = seek(&state, Millis::new(5_000), true, Millis::new(100)).unwrap();
        let seen = apply(&mut core, &scrub, 100);
        assert_eq!(seen, [Seen::Running(false), Seen::Scrub(5_000)]);
        // Held still: nothing new; moved: the next piece, the running state stays untouched.
        assert!(apply(&mut core, &scrub, 400).is_empty());
        let moved = seek(&scrub, Millis::new(9_000), true, Millis::new(500)).unwrap();
        assert_eq!(apply(&mut core, &moved, 500), [Seen::Scrub(9_000)]);
        let released = seek(&moved, Millis::new(9_000), false, Millis::new(600)).unwrap();
        let seen = apply(&mut core, &released, 600);
        assert_eq!(seen, [Seen::Seek(9_000), Seen::Running(true)]);
    }

    #[test]
    fn scrubbing_while_paused_ends_paused() {
        let mut core = PlayerCore::default();
        let state = selected();
        apply(&mut core, &state, 0);
        let scrub = seek(&state, Millis::new(2_000), true, Millis::new(100)).unwrap();
        assert_eq!(apply(&mut core, &scrub, 100), [Seen::Scrub(2_000)]);
        let released = seek(&scrub, Millis::new(2_000), false, Millis::new(200)).unwrap();
        assert_eq!(
            apply(&mut core, &released, 200),
            [Seen::Seek(2_000), Seen::Running(false)]
        );
    }

    #[test]
    fn speed_key_and_play_mode_changes_reach_the_engine() {
        let mut core = PlayerCore::default();
        let state = selected();
        apply(&mut core, &state, 0);
        let faster = set_speed(&state, Speed::new(1.5).unwrap(), Millis::ZERO).unwrap();
        assert_eq!(apply(&mut core, &faster, 0), [Seen::Speed(1.5)]);
        let keyed = set_key(&faster, KeyShift::new(2).unwrap()).unwrap();
        assert_eq!(apply(&mut core, &keyed, 0), [Seen::Pitch(2.0)]);
    }

    #[test]
    fn a_region_reaches_the_engine_as_loop_or_stop_by_mode() {
        let mut core = PlayerCore::default();
        let state = selected();
        apply(&mut core, &state, 0);
        let range = TimeRange::new(Millis::new(1_000), Millis::new(2_000)).unwrap();
        // To end: the range is ignored, the engine only moves to the start.
        let region = play_region(&state, range, Millis::ZERO).unwrap();
        assert_eq!(
            apply(&mut core, &region, 0),
            [Seen::Seek(1_000), Seen::Running(true)]
        );
        let looped = set_play_mode(&region, PlayMode::Loop, Millis::ZERO).unwrap();
        assert_eq!(
            apply(&mut core, &looped, 0),
            [Seen::Loop(Some((1_000, 2_000)))]
        );
        let stopping = set_play_mode(&looped, PlayMode::ToRegionEnd, Millis::ZERO).unwrap();
        assert_eq!(
            apply(&mut core, &stopping, 0),
            [Seen::Loop(None), Seen::Stop(Some(2_000))]
        );
        // The server pauses at the region end and clears the range.
        let ended = end_region(&stopping, Millis::new(1_000)).unwrap();
        assert_eq!(
            apply(&mut core, &ended, 1_000),
            [Seen::Stop(None), Seen::Running(false), Seen::Seek(2_000)]
        );
    }

    #[test]
    fn the_recordings_own_tuning_sets_the_pitch_and_a_late_analysis_corrects_it() {
        let mut core = PlayerCore::default();
        let state = selected();
        let mut recorder = Recorder::default();
        let load = &mut |_: RecordingId, _: bool| audio();
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
        core.apply(
            &state,
            Millis::ZERO,
            &mut recorder,
            &mut |_, _| None,
            &|_| TuningHz::CONCERT_PITCH,
        );
        assert_eq!(recorder.0, [Seen::Unload]);
    }
}
