//! Entry point of the client core. Exposes a narrow API to the Flutter bridge.
//!
//! All functions are `async` on purpose: the browser runs async functions on the
//! main thread, while synchronous ones need a worker pool, which needs threads.

use crate::clock::local_now;
use crate::frb_generated::StreamSink;
use std::sync::{Arc, LazyLock, Mutex, MutexGuard, PoisonError};
use veltrunox_client_adapters::{EventChannel, HttpServerLink, SystemTimer, open_channel};
use veltrunox_client_app::{
    ChannelEvent, ConnectionMonitor, ConnectionStatus, LinkEvent, ServerEvent, SessionView,
    ViewChange, load_library,
};
use veltrunox_core::{Intent, KeyShift, Millis, RecordingId, Speed, TuningHz};

/// A recording as the user interface needs it.
pub struct RecordingItem {
    pub id: u64,
    pub title: String,
}

/// Loads the recording list from the server at `server_url`.
pub async fn list_recordings(server_url: String) -> Result<Vec<RecordingItem>, String> {
    let link = HttpServerLink::new(&server_url);
    let recordings = load_library(&link)
        .await
        .map_err(|error| error.to_string())?;
    Ok(recordings
        .into_iter()
        .map(|recording| RecordingItem {
            id: recording.id.value(),
            title: recording.title.as_str().to_owned(),
        })
        .collect())
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
}

/// Everything the real-time channel tells the user interface.
#[derive(Clone)]
pub enum SessionUpdate {
    Connection(ConnectionState),
    Playback(PlaybackView),
    BookmarksChanged { recording_id: u64 },
    Rejected { reason: String },
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
        }
    }
}

/// The playback position now, extrapolated from the latest state; `None` before the first state.
pub async fn playback_position_ms() -> Option<u64> {
    view().position_now(local_now()).map(Millis::value)
}

/// Sends a command to the shared player. Out-of-range values are rejected here
/// and the server rejects them again; the answer arrives as a session update.
pub async fn send_command(command: PlayerCommand) -> Result<(), String> {
    let intent = command.into_intent().map_err(|error| error.to_string())?;
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
