//! Entry point of the client core. Exposes a narrow API to the Flutter bridge.

use crate::frb_generated::StreamSink;
use std::sync::Mutex;
use veltrunox_client_adapters::{EventConnection, HttpServerLink, connect_events};
use veltrunox_client_app::{ConnectionMonitor, ConnectionStatus, load_library};

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
pub enum ConnectionState {
    Connected,
    Disconnected,
}

/// The one real-time channel of this client; a new `watch_connection` replaces it.
/// `flutter_rust_bridge` drops values returned from stream functions, so the
/// connection has to be owned here to stay open.
#[cfg(not(target_arch = "wasm32"))]
static CHANNEL: Mutex<Option<EventConnection>> = Mutex::new(None);

// The browser connection is not `Send`; the browser runs on one thread anyway.
#[cfg(target_arch = "wasm32")]
thread_local! {
    static CHANNEL: std::cell::RefCell<Option<EventConnection>> = const { std::cell::RefCell::new(None) };
}

#[cfg(not(target_arch = "wasm32"))]
fn keep_open(connection: EventConnection) -> Result<(), String> {
    let mut channel = CHANNEL.lock().map_err(|error| error.to_string())?;
    *channel = Some(connection);
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn keep_open(connection: EventConnection) -> Result<(), String> {
    CHANNEL.with(|channel| *channel.borrow_mut() = Some(connection));
    Ok(())
}

/// Opens the real-time channel and pushes each distinct state change to `states`.
// Async on purpose: the browser runs async functions on the main thread, while
// synchronous ones need a worker pool, which needs threads.
pub async fn watch_connection(
    server_url: String,
    states: StreamSink<ConnectionState>,
) -> Result<(), String> {
    let monitor = Mutex::new(ConnectionMonitor::new(move |status| {
        // A closed sink means the UI stopped listening; nothing is left to report.
        let _ = states.add(ConnectionState::from(status));
    }));
    let connection = connect_events(&server_url, move |event| {
        if let Ok(mut monitor) = monitor.lock() {
            monitor.push(event);
        }
    })
    .map_err(|error| error.to_string())?;
    keep_open(connection)
}

impl From<ConnectionStatus> for ConnectionState {
    fn from(status: ConnectionStatus) -> Self {
        match status {
            ConnectionStatus::Connected => Self::Connected,
            ConnectionStatus::Disconnected => Self::Disconnected,
        }
    }
}
