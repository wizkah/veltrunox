//! The output device (cpal) and the thread that drives playback.

use super::decode::decode_file;
use super::engine::{Engine, Pcm};
use super::player::{Command, CommandSink, PlayerCore};
use super::volume::{SystemMixer, engine_gain};
use crate::hooks::AudioHooks;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SampleFormat, SizedSample, Stream, StreamConfig};
use std::sync::mpsc::{Receiver, Sender, SyncSender, channel, sync_channel};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;
use veltrunox_core::{Millis, PlaybackState, RecordingId, TuningHz, Volume};
use veltrunox_server_app::{AnalysisRepository, Clock, MediaCatalog, PlaybackBroadcast};

/// Commands waiting for the audio thread; a full queue drops the newest, which
/// a complete state makes up for with the next one.
const COMMAND_QUEUE: usize = 64;
/// Frames the render buffer holds without growing.
const RENDER_FRAMES: usize = 8_192;

#[derive(Debug, thiserror::Error)]
#[error("audio output: {0}")]
pub struct OutputError(String);

fn failed(error: impl std::fmt::Display) -> OutputError {
    OutputError(error.to_string())
}

/// Names of the output devices, for the admin to choose from.
pub fn list_output_devices() -> Vec<String> {
    let Ok(devices) = cpal::default_host().devices() else {
        return Vec::new();
    };
    devices
        .filter(|device| device.default_output_config().is_ok())
        .filter_map(|device| device.id().ok().map(|id| id.to_string()))
        .collect()
}

/// An open stream plus the way to reach its audio thread.
struct DeviceOutput {
    _stream: Stream,
    rate: u32,
    commands: SyncSender<Command>,
    retired: Receiver<Box<Engine>>,
    /// The sound card's own volume control; without one the render thread scales the audio.
    mixer: Option<SystemMixer>,
}

impl DeviceOutput {
    /// Opens the named device, or the default one.
    fn open(name: Option<&str>) -> Result<Self, OutputError> {
        let host = cpal::default_host();
        let device = match name {
            Some(wanted) => host
                .devices()
                .map_err(failed)?
                .find(|device| device.id().is_ok_and(|id| id.to_string() == wanted))
                .ok_or_else(|| failed(format!("device {wanted} not found")))?,
            None => host
                .default_output_device()
                .ok_or_else(|| failed("no output device"))?,
        };
        let config = device.default_output_config().map_err(failed)?;
        let (commands, command_queue) = sync_channel(COMMAND_QUEUE);
        let (retire, retired) = sync_channel(4);
        let rate = config.sample_rate();
        let renderer = Renderer::new(command_queue, retire, usize::from(config.channels()));
        let stream = match config.sample_format() {
            SampleFormat::F32 => build::<f32>(&device, config.into(), renderer),
            SampleFormat::I16 => build::<i16>(&device, config.into(), renderer),
            SampleFormat::I32 => build::<i32>(&device, config.into(), renderer),
            SampleFormat::U16 => build::<u16>(&device, config.into(), renderer),
            format => Err(failed(format!("unsupported sample format {format}"))),
        }?;
        stream.play().map_err(failed)?;
        Ok(Self {
            _stream: stream,
            rate,
            commands,
            retired,
            mixer: SystemMixer::open(name),
        })
    }

    /// Applies the volume of the room: on the card's mixer if it has one, else as gain.
    fn set_volume(&mut self, volume: Volume) {
        if let Some(mixer) = &self.mixer
            && mixer.set(volume).is_ok()
        {
            return;
        }
        self.send(Command::Gain(engine_gain(volume)));
    }
}

impl CommandSink for DeviceOutput {
    fn output_rate(&self) -> u32 {
        self.rate
    }

    fn send(&mut self, command: Command) {
        if let Command::Volume(volume) = command {
            return self.set_volume(volume);
        }
        // Engines the audio thread is done with are freed here, not there.
        while self.retired.try_recv().is_ok() {}
        // A full queue only happens when the device stalls; the next state repairs it.
        let _ = self.commands.try_send(command);
    }
}

fn build<T>(
    device: &cpal::Device,
    config: StreamConfig,
    mut renderer: Renderer,
) -> Result<Stream, OutputError>
where
    T: SizedSample + FromSample<f32>,
{
    device
        .build_output_stream(
            config,
            move |data: &mut [T], _| renderer.fill(data),
            |error| eprintln!("audio stream: {error}"),
            None,
        )
        .map_err(failed)
}

/// Lives in the audio callback: applies commands, renders, maps to the device channels.
struct Renderer {
    commands: Receiver<Command>,
    retire: SyncSender<Box<Engine>>,
    engine: Option<Box<Engine>>,
    channels: usize,
    stereo: Vec<f32>,
    /// Amplitude factor applied to everything rendered; changes are ramped over one buffer.
    gain: f32,
    target_gain: f32,
}

impl Renderer {
    fn new(commands: Receiver<Command>, retire: SyncSender<Box<Engine>>, channels: usize) -> Self {
        Self {
            commands,
            retire,
            engine: None,
            channels: channels.max(1),
            stereo: vec![0.0; RENDER_FRAMES * 2],
            gain: 1.0,
            target_gain: 1.0,
        }
    }

    fn fill<T: SizedSample + FromSample<f32>>(&mut self, data: &mut [T]) {
        while let Ok(command) = self.commands.try_recv() {
            self.apply(command);
        }
        let frames = data.len() / self.channels;
        if self.stereo.len() < frames * 2 {
            self.stereo.resize(frames * 2, 0.0); // only for unusually large buffers
        }
        let stereo = &mut self.stereo[..frames * 2];
        match &mut self.engine {
            Some(engine) => engine.render(stereo),
            None => stereo.fill(0.0),
        }
        ramp_gain(stereo, self.gain, self.target_gain);
        self.gain = self.target_gain;
        for (frame, pair) in data.chunks_mut(self.channels).zip(stereo.chunks(2)) {
            for (channel, sample) in frame.iter_mut().enumerate() {
                *sample = T::from_sample(match (self.channels, channel) {
                    (1, _) => (pair[0] + pair[1]) / 2.0,
                    (_, 0) => pair[0],
                    (_, 1) => pair[1],
                    _ => 0.0,
                });
            }
        }
    }

    fn apply(&mut self, command: Command) {
        match command {
            Command::Replace(engine) => {
                if let Some(old) = self.engine.replace(engine) {
                    let _ = self.retire.try_send(old);
                }
            }
            Command::Unload => {
                if let Some(old) = self.engine.take() {
                    let _ = self.retire.try_send(old);
                }
            }
            Command::Gain(gain) => self.target_gain = gain,
            command => {
                if let Some(engine) = &mut self.engine {
                    match command {
                        Command::Running(running) => engine.set_running(running),
                        Command::Speed(speed) => engine.set_speed(speed),
                        Command::Pitch(semitones) => engine.set_pitch_semitones(semitones),
                        Command::Loop(range) => engine.set_loop_ms(range),
                        Command::Stop(position) => engine.set_stop_ms(position),
                        Command::Seek(position) => engine.seek_ms(position),
                        Command::Scrub(position) => engine.scrub_ms(position),
                        Command::Replace(_)
                        | Command::Unload
                        | Command::Volume(_)
                        | Command::Gain(_) => {}
                    }
                }
            }
        }
    }
}

/// Scales interleaved stereo frames, moving the factor from `from` to `to` over
/// the buffer so that a change of volume does not click.
fn ramp_gain(stereo: &mut [f32], from: f32, to: f32) {
    if from == 1.0 && to == 1.0 {
        return;
    }
    let frames = (stereo.len() / 2).max(1) as f32;
    for (index, pair) in stereo.chunks_mut(2).enumerate() {
        let gain = from + (to - from) * (index as f32 + 1.0) / frames;
        pair.iter_mut().for_each(|sample| *sample *= gain);
    }
}

/// Plays the shared state on the server's audio device. Without a device the
/// shared state keeps running silently.
pub struct AudioPlayback {
    inputs: Mutex<Sender<Input>>,
}

/// What the playback thread reacts to.
enum Input {
    State(PlaybackState),
    /// The analysis of this recording changed: its tuning may be new.
    Analysis(RecordingId),
}

impl AudioPlayback {
    /// Starts the playback thread; it opens the device itself and reports the outcome on stderr.
    pub fn start(
        device: Option<String>,
        volume: Volume,
        catalog: Arc<dyn MediaCatalog>,
        analyses: Arc<dyn AnalysisRepository>,
        clock: Arc<dyn Clock>,
        hooks: AudioHooks,
    ) -> Self {
        let (inputs, incoming) = channel();
        thread::spawn(move || {
            run(
                device.as_deref(),
                volume,
                &*catalog,
                &*analyses,
                &*clock,
                &hooks,
                &incoming,
            );
        });
        Self {
            inputs: Mutex::new(inputs),
        }
    }
}

impl AudioPlayback {
    fn send(&self, input: Input) {
        let sender = self.inputs.lock().unwrap_or_else(PoisonError::into_inner);
        // A stopped thread means the server is shutting down.
        let _ = sender.send(input);
    }
}

impl PlaybackBroadcast for AudioPlayback {
    fn state_changed(&self, state: &PlaybackState) {
        self.send(Input::State(*state));
    }

    fn bookmarks_changed(&self, _recording: RecordingId) {}

    fn analysis_changed(&self, recording: RecordingId) {
        self.send(Input::Analysis(recording));
    }

    fn library_changed(&self) {}
}

fn run(
    device: Option<&str>,
    volume: Volume,
    catalog: &dyn MediaCatalog,
    analyses: &dyn AnalysisRepository,
    clock: &dyn Clock,
    hooks: &AudioHooks,
    incoming: &Receiver<Input>,
) {
    let mut load = |id: RecordingId| {
        let pcm = load_audio(catalog, id)?;
        hooks.recording_length(id, Millis::new(pcm.duration_ms()));
        Some(pcm)
    };
    let tuning_of = |id: RecordingId| recording_tuning(analyses, id);
    let mut output = match DeviceOutput::open(device) {
        Ok(output) => {
            println!("audio output at {} Hz", output.rate);
            hooks.output_changed(true);
            output
        }
        Err(error) => {
            eprintln!("{error}; playback state runs without sound");
            hooks.output_changed(false);
            return follow_silently(incoming, &mut load);
        }
    };
    let mut core = PlayerCore::with_volume(volume);
    while let Ok(input) = incoming.recv() {
        let Some(state) = newest_state(input, incoming, &core) else {
            continue;
        };
        core.apply(&state, clock.now(), &mut output, &mut load, &tuning_of);
    }
}

/// Every state is complete, so only the newest matters. A change of analysis
/// applies the last state again, for the sake of a new tuning reference; one
/// for another recording than the loaded one changes nothing.
fn newest_state(
    first: Input,
    incoming: &Receiver<Input>,
    core: &PlayerCore,
) -> Option<PlaybackState> {
    let mut newest = None;
    let mut analysed = Vec::new();
    for input in std::iter::once(first).chain(std::iter::from_fn(|| incoming.try_recv().ok())) {
        match input {
            Input::State(state) => newest = Some(state),
            Input::Analysis(id) => analysed.push(id),
        }
    }
    newest.or_else(|| {
        core.last_state()
            .filter(|state| state.recording.is_some_and(|id| analysed.contains(&id)))
    })
}

/// The tuning found by the analysis; concert pitch until there is one.
fn recording_tuning(analyses: &dyn AnalysisRepository, id: RecordingId) -> TuningHz {
    match analyses.analysis(id) {
        Ok(Some(found)) => found.tuning,
        Ok(None) => TuningHz::CONCERT_PITCH,
        Err(error) => {
            eprintln!("recording {id}: {error}");
            TuningHz::CONCERT_PITCH
        }
    }
}

/// Without a device nothing is played, but the length of each selected
/// recording is still needed to end the playback and to check positions.
fn follow_silently(
    incoming: &Receiver<Input>,
    load: &mut dyn FnMut(RecordingId) -> Option<Arc<Pcm>>,
) {
    let mut last = None;
    while let Ok(input) = incoming.recv() {
        let Input::State(state) = input else {
            continue;
        };
        if state.recording != last {
            last = state.recording;
            if let Some(id) = last {
                load(id);
            }
        }
    }
}

fn load_audio(catalog: &dyn MediaCatalog, id: RecordingId) -> Option<Arc<Pcm>> {
    let path = match catalog.media_path(id) {
        Ok(Some(path)) => path,
        Ok(None) => return None,
        Err(error) => {
            eprintln!("recording {id}: {error}");
            return None;
        }
    };
    match decode_file(&path) {
        Ok(pcm) => Some(Arc::new(pcm)),
        Err(error) => {
            eprintln!("recording {id} ({}): {error}", path.display());
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ramp_gain;

    #[test]
    fn a_steady_gain_scales_every_sample() {
        let mut samples = [1.0; 8];
        ramp_gain(&mut samples, 0.5, 0.5);
        assert!(samples.iter().all(|sample| (sample - 0.5).abs() < 1e-6));
    }

    #[test]
    fn a_change_of_gain_moves_gradually_and_ends_on_the_target() {
        let mut samples = [1.0; 8];
        ramp_gain(&mut samples, 1.0, 0.0);
        assert!(samples.windows(2).all(|pair| pair[0] >= pair[1]));
        assert_eq!(samples[6], 0.0);
        assert_eq!(samples[7], 0.0);
        assert!(samples[0] > 0.5);
    }

    #[test]
    fn unity_gain_leaves_the_audio_untouched() {
        let mut samples = [0.25, -0.5, 0.75, 1.0];
        ramp_gain(&mut samples, 1.0, 1.0);
        assert_eq!(samples, [0.25, -0.5, 0.75, 1.0]);
    }
}
