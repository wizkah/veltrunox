//! The output device (cpal) and the thread that drives playback.

use super::decode::decode_file;
use super::engine::{Engine, Pcm};
use super::player::{Command, CommandSink, PlayerCore};
use crate::hooks::AudioHooks;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SampleFormat, SizedSample, Stream, StreamConfig};
use std::sync::mpsc::{Receiver, Sender, SyncSender, channel, sync_channel};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;
use veltrunox_core::{Millis, PlaybackState, RecordingId};
use veltrunox_server_app::{Clock, MediaCatalog, PlaybackBroadcast};

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
        })
    }
}

impl CommandSink for DeviceOutput {
    fn output_rate(&self) -> u32 {
        self.rate
    }

    fn send(&mut self, command: Command) {
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
}

impl Renderer {
    fn new(commands: Receiver<Command>, retire: SyncSender<Box<Engine>>, channels: usize) -> Self {
        Self {
            commands,
            retire,
            engine: None,
            channels: channels.max(1),
            stereo: vec![0.0; RENDER_FRAMES * 2],
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
            command => {
                if let Some(engine) = &mut self.engine {
                    match command {
                        Command::Running(running) => engine.set_running(running),
                        Command::Speed(speed) => engine.set_speed(speed),
                        Command::Pitch(semitones) => engine.set_pitch_semitones(semitones),
                        Command::Loop(range) => engine.set_loop_ms(range),
                        Command::Seek(position) => engine.seek_ms(position),
                        Command::Replace(_) | Command::Unload => {}
                    }
                }
            }
        }
    }
}

/// Plays the shared state on the server's audio device. Without a device the
/// shared state keeps running silently.
pub struct AudioPlayback {
    states: Mutex<Sender<PlaybackState>>,
}

impl AudioPlayback {
    /// Starts the playback thread; it opens the device itself and reports the outcome on stderr.
    pub fn start(
        device: Option<String>,
        catalog: Arc<dyn MediaCatalog>,
        clock: Arc<dyn Clock>,
        hooks: AudioHooks,
    ) -> Self {
        let (states, incoming) = channel();
        thread::spawn(move || run(device.as_deref(), &*catalog, &*clock, &hooks, &incoming));
        Self {
            states: Mutex::new(states),
        }
    }
}

impl PlaybackBroadcast for AudioPlayback {
    fn state_changed(&self, state: &PlaybackState) {
        let sender = self.states.lock().unwrap_or_else(PoisonError::into_inner);
        // A stopped thread means the server is shutting down.
        let _ = sender.send(*state);
    }

    fn bookmarks_changed(&self, _recording: RecordingId) {}
}

fn run(
    device: Option<&str>,
    catalog: &dyn MediaCatalog,
    clock: &dyn Clock,
    hooks: &AudioHooks,
    incoming: &Receiver<PlaybackState>,
) {
    let mut load = |id: RecordingId| {
        let pcm = load_audio(catalog, id)?;
        hooks.recording_length(id, Millis::new(pcm.duration_ms()));
        Some(pcm)
    };
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
    let mut core = PlayerCore::default();
    while let Ok(mut state) = incoming.recv() {
        // Every state is complete; only the newest matters.
        while let Ok(newer) = incoming.try_recv() {
            state = newer;
        }
        core.apply(&state, clock.now(), &mut output, &mut load);
    }
}

/// Without a device nothing is played, but the length of each selected
/// recording is still needed to end the playback and to check positions.
fn follow_silently(
    incoming: &Receiver<PlaybackState>,
    load: &mut dyn FnMut(RecordingId) -> Option<Arc<Pcm>>,
) {
    let mut last = None;
    while let Ok(state) = incoming.recv() {
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
