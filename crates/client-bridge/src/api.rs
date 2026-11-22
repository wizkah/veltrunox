//! Entry point of the client core. Exposes a narrow API to the Flutter bridge.
//!
//! All functions are `async` on purpose: the browser runs async functions on the
//! main thread, while synchronous ones need a worker pool, which needs threads.

use crate::clock::local_now;
use crate::frb_generated::StreamSink;
use std::sync::{Arc, LazyLock, Mutex, MutexGuard, PoisonError};
use veltrunox_client_adapters::{EventChannel, HttpServerLink, SystemTimer, open_channel};
use veltrunox_client_app::{
    ChannelEvent, ConnectionMonitor, ConnectionStatus, LinkEvent, ScrubThrottle, ServerEvent,
    SessionView, ViewChange, analyse_again, load_length, load_library,
};
use veltrunox_core::{
    AnalysisProgress, Bpm, Intent, KeyMode, KeyShift, LibraryEntry, Millis, RecordingId, Speed,
    TargetSpeed, TuningHz, Volume, speed_for_target_tempo,
};

/// A recording as the user interface needs it.
pub struct RecordingItem {
    pub id: u64,
    pub title: String,
    pub analysis_state: AnalysisState,
    /// Why the analysis failed; only set in the state `Failed`.
    pub analysis_failure: Option<String>,
    /// The estimated values; only set once the analysis is done.
    pub analysis: Option<AnalysisValues>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AnalysisState {
    None,
    Queued,
    Running,
    Failed,
    Done,
}

/// Tuning, key and tempo as estimated from the audio.
#[derive(Clone)]
pub struct AnalysisValues {
    pub tuning_hz: f64,
    /// Pitch class of the key's tonic, 0 = C … 11 = B.
    pub key_tonic: u8,
    pub key_minor: bool,
    pub tempo_bpm: f64,
}

/// Loads the recording list from the server at `server_url`.
pub async fn list_recordings(server_url: String) -> Result<Vec<RecordingItem>, String> {
    let link = HttpServerLink::new(&server_url);
    let entries = load_library(&link)
        .await
        .map_err(|error| error.to_string())?;
    Ok(entries.iter().map(RecordingItem::from).collect())
}

impl From<&LibraryEntry> for RecordingItem {
    fn from(entry: &LibraryEntry) -> Self {
        let (analysis_state, analysis_failure) = match &entry.analysis.progress {
            AnalysisProgress::None => (AnalysisState::None, None),
            AnalysisProgress::Queued => (AnalysisState::Queued, None),
            AnalysisProgress::Running => (AnalysisState::Running, None),
            AnalysisProgress::Failed(reason) => (AnalysisState::Failed, Some(reason.clone())),
            AnalysisProgress::Done => (AnalysisState::Done, None),
        };
        Self {
            id: entry.recording.id.value(),
            title: entry.recording.title.as_str().to_owned(),
            analysis_state,
            analysis_failure,
            analysis: entry.analysis.result.map(|found| AnalysisValues {
                tuning_hz: found.tuning.hertz(),
                key_tonic: found.key.tonic.index(),
                key_minor: found.key.mode == KeyMode::Minor,
                tempo_bpm: found.tempo.value(),
            }),
        }
    }
}

/// Asks the server to analyse a recording again; the result arrives as a session update.
pub async fn reanalyse_recording(server_url: String, recording_id: u64) -> Result<(), String> {
    let link = HttpServerLink::new(&server_url);
    analyse_again(&link, RecordingId::new(recording_id))
        .await
        .map_err(|error| error.to_string())
}

/// The length of a recording in milliseconds from its stored scan; `None` while there is none
/// (not scanned yet) or the server cannot be reached.
pub async fn recording_length_ms(server_url: String, recording_id: u64) -> Option<u64> {
    let link = HttpServerLink::new(&server_url);
    load_length(&link, RecordingId::new(recording_id))
        .await
        .ok()
        .map(Millis::value)
}

/// The speed that plays a recording of `recording_bpm` at `target_bpm`.
pub struct TargetTempo {
    pub speed: f64,
    /// `false` if the speed lies outside the supported range; it must not be set then.
    pub supported: bool,
}

pub async fn speed_for_target_bpm(
    target_bpm: f64,
    recording_bpm: f64,
) -> Result<TargetTempo, String> {
    target_tempo(target_bpm, recording_bpm)
}

fn target_tempo(target_bpm: f64, recording_bpm: f64) -> Result<TargetTempo, String> {
    let target = Bpm::new(target_bpm).map_err(|error| error.to_string())?;
    let recording = Bpm::new(recording_bpm).map_err(|error| error.to_string())?;
    Ok(match speed_for_target_tempo(target, recording) {
        TargetSpeed::Supported(speed) => TargetTempo {
            speed: speed.factor(),
            supported: true,
        },
        TargetSpeed::OutOfRange { factor } => TargetTempo {
            speed: factor,
            supported: false,
        },
    })
}

/// Whether the server accepts this tuning; it must not be sent otherwise.
pub async fn tuning_supported(tuning_hz: f64) -> bool {
    tuning_in_range(tuning_hz)
}

fn tuning_in_range(tuning_hz: f64) -> bool {
    TuningHz::new(tuning_hz).is_ok()
}

/// State of the real-time channel as the user interface needs it.
#[derive(Clone)]
pub enum ConnectionState {
    Connected,
    Disconnected,
}

/// The shared playback state as the user interface needs it.
#[derive(Clone)]
pub struct PlaybackView {
    pub revision: u64,
    pub recording_id: Option<u64>,
    /// Position at the moment this view was made; use `playback_position_ms` while running.
    pub position_ms: u64,
    pub running: bool,
    pub speed: f64,
    pub tuning_hz: f64,
    pub key_shift_semitones: i8,
    pub scrubbing: bool,
    pub volume_percent: u8,
}

/// Everything the real-time channel tells the user interface.
#[derive(Clone)]
pub enum SessionUpdate {
    Connection(ConnectionState),
    Playback(PlaybackView),
    BookmarksChanged {
        recording_id: u64,
    },
    /// The analysis of this recording changed; load the library again.
    AnalysisChanged {
        recording_id: u64,
    },
    Rejected {
        reason: String,
    },
}

/// What the user asks the shared player to do.
pub enum PlayerCommand {
    SelectRecording { recording_id: u64 },
    Play,
    Pause,
    Seek { position_ms: u64 },
    SetSpeed { speed: f64 },
    SetTuning { tuning_hz: f64 },
    SetKey { key_shift_semitones: i8 },
    SetVolume { volume_percent: u8 },
}

/// The latest server state and clock estimate of this client; shared between the
/// channel callback and the calls from the interface.
static VIEW: LazyLock<Mutex<SessionView>> = LazyLock::new(|| Mutex::new(SessionView::new()));

fn view() -> MutexGuard<'static, SessionView> {
    VIEW.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The one real-time channel of this client; a new `watch_session` replaces it.
/// `flutter_rust_bridge` drops values returned from stream functions, so the
/// channel has to be owned here to stay open.
static CHANNEL: Mutex<Option<EventChannel>> = Mutex::new(None);

fn with_channel<T>(action: impl FnOnce(&mut Option<EventChannel>) -> T) -> T {
    action(&mut CHANNEL.lock().unwrap_or_else(PoisonError::into_inner))
}

/// Opens the real-time channel and pushes connection changes, playback state,
/// bookmark notices and rejections to `updates`. The channel reconnects by itself
/// and measures the clock offset when connected and from time to time.
pub async fn watch_session(
    server_url: String,
    updates: StreamSink<SessionUpdate>,
) -> Result<(), String> {
    // Close the previous channel first so it cannot report into the new session.
    with_channel(|channel| *channel = None);
    *view() = SessionView::new();
    let status_sink = updates.clone();
    let monitor = Mutex::new(ConnectionMonitor::new(move |status| {
        // A closed sink means the UI stopped listening; nothing is left to report.
        let _ = status_sink.add(SessionUpdate::Connection(ConnectionState::from(status)));
    }));
    let channel = open_channel(
        &server_url,
        Arc::new(SystemTimer),
        local_now,
        move |event| match event {
            ChannelEvent::Link(link_event) => {
                if link_event == LinkEvent::Greeted {
                    // The server sends the current state next; its revisions may
                    // have started over if it restarted.
                    view().forget_state();
                }
                if let Ok(mut monitor) = monitor.lock() {
                    monitor.push(link_event);
                }
            }
            ChannelEvent::Server(server_event) => forward(server_event, &updates),
        },
    );
    with_channel(|slot| *slot = Some(channel));
    Ok(())
}

fn forward(event: ServerEvent, updates: &StreamSink<SessionUpdate>) {
    let now = local_now();
    let mut view = view();
    let update = match view.handle(event, now) {
        ViewChange::Nothing => return,
        ViewChange::PlaybackState => view.state().map(|state| {
            SessionUpdate::Playback(PlaybackView::from_state(state, view.server_time(now)))
        }),
        ViewChange::BookmarksChanged(recording) => Some(SessionUpdate::BookmarksChanged {
            recording_id: recording.value(),
        }),
        ViewChange::AnalysisChanged(recording) => Some(SessionUpdate::AnalysisChanged {
            recording_id: recording.value(),
        }),
        ViewChange::Rejected(reason) => Some(SessionUpdate::Rejected { reason }),
        // Not shown yet: the interface needs a place for it first (GUI work).
        ViewChange::AudioOutput => None,
    };
    if let Some(update) = update {
        let _ = updates.add(update);
    }
}

impl PlaybackView {
    fn from_state(state: &veltrunox_core::PlaybackState, server_now: Millis) -> Self {
        Self {
            revision: state.revision,
            recording_id: state.recording.map(RecordingId::value),
            position_ms: state.position_at(server_now).value(),
            running: state.running,
            speed: state.settings.speed.factor(),
            tuning_hz: state.settings.tuning.hertz(),
            key_shift_semitones: state.settings.key.semitones(),
            scrubbing: state.scrubbing,
            volume_percent: state.volume.percent(),
        }
    }
}

/// The playback position now, extrapolated from the latest state; `None` before the first state.
pub async fn playback_position_ms() -> Option<u64> {
    view().position_now(local_now()).map(Millis::value)
}

/// Limits the seek intents of the position handle while it is dragged.
static SCRUB: Mutex<Option<ScrubThrottle>> = Mutex::new(None);

fn with_scrub<T>(action: impl FnOnce(&mut ScrubThrottle) -> T) -> T {
    let mut guard = SCRUB.lock().unwrap_or_else(PoisonError::into_inner);
    action(guard.get_or_insert_with(|| ScrubThrottle::new(ScrubThrottle::DEFAULT_INTERVAL)))
}

/// The position handle is dragged to `position_ms`; the server hears it at most about 20 times a
/// second and keeps the sound muted meanwhile.
pub async fn scrub_to(position_ms: u64) -> Result<(), String> {
    let position = Millis::position(position_ms).map_err(|error| error.to_string())?;
    match with_scrub(|throttle| throttle.drag(position, local_now())) {
        Some(intent) => send_intent(intent),
        None => Ok(()),
    }
}

/// The position handle is let go at `position_ms`: always sent, so the last position is never lost.
pub async fn end_scrub(position_ms: u64) -> Result<(), String> {
    let position = Millis::position(position_ms).map_err(|error| error.to_string())?;
    send_intent(with_scrub(|throttle| throttle.release(position)))
}

/// Sends a command to the shared player. Out-of-range values are rejected here
/// and the server rejects them again; the answer arrives as a session update.
pub async fn send_command(command: PlayerCommand) -> Result<(), String> {
    send_intent(command.into_intent().map_err(|error| error.to_string())?)
}

fn send_intent(intent: Intent) -> Result<(), String> {
    with_channel(|channel| match channel {
        Some(channel) => channel
            .send_intent(intent)
            .map_err(|error| error.to_string()),
        None => Err("not connected".to_owned()),
    })
}

impl PlayerCommand {
    fn into_intent(self) -> Result<Intent, veltrunox_core::InvalidValue> {
        Ok(match self {
            Self::SelectRecording { recording_id } => {
                Intent::SelectRecording(RecordingId::new(recording_id))
            }
            Self::Play => Intent::Play,
            Self::Pause => Intent::Pause,
            Self::Seek { position_ms } => Intent::Seek {
                position: Millis::position(position_ms)?,
                scrubbing: false,
            },
            Self::SetSpeed { speed } => Intent::SetSpeed(Speed::new(speed)?),
            Self::SetTuning { tuning_hz } => Intent::SetTuning(TuningHz::new(tuning_hz)?),
            Self::SetKey {
                key_shift_semitones,
            } => Intent::SetKey(KeyShift::new(key_shift_semitones)?),
            Self::SetVolume { volume_percent } => Intent::SetVolume(Volume::new(volume_percent)?),
        })
    }
}

impl From<ConnectionStatus> for ConnectionState {
    fn from(status: ConnectionStatus) -> Self {
        match status {
            ConnectionStatus::Connected => Self::Connected,
            ConnectionStatus::Disconnected => Self::Disconnected,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tuning_is_supported_between_300_and_500_hertz() {
        assert!(tuning_in_range(300.0) && tuning_in_range(432.5) && tuning_in_range(500.0));
        assert!(!tuning_in_range(299.9) && !tuning_in_range(600.0) && !tuning_in_range(f64::NAN));
    }

    #[test]
    fn commands_become_intents() {
        let intent = PlayerCommand::Seek { position_ms: 1_500 }.into_intent();
        assert_eq!(
            intent,
            Ok(Intent::Seek {
                position: Millis::new(1_500),
                scrubbing: false
            })
        );
    }

    #[test]
    fn a_volume_command_is_checked_and_becomes_an_intent() {
        assert_eq!(
            PlayerCommand::SetVolume { volume_percent: 40 }.into_intent(),
            Ok(Intent::SetVolume(Volume::new(40).unwrap()))
        );
        assert!(
            PlayerCommand::SetVolume {
                volume_percent: 101
            }
            .into_intent()
            .is_err()
        );
    }

    #[test]
    fn a_target_tempo_gives_the_speed() {
        let target = target_tempo(90.0, 120.0).unwrap();
        assert!(target.supported);
        assert!((target.speed - 0.75).abs() < 1e-12);
        let too_fast = target_tempo(300.0, 100.0).unwrap();
        assert!(!too_fast.supported);
        assert!(target_tempo(0.0, 100.0).is_err());
    }

    #[test]
    fn out_of_range_values_are_refused_before_sending() {
        assert!(
            PlayerCommand::SetSpeed { speed: 3.0 }
                .into_intent()
                .is_err()
        );
        assert!(
            PlayerCommand::SetKey {
                key_shift_semitones: 20
            }
            .into_intent()
            .is_err()
        );
    }
}
