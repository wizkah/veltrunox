//! Shared helpers: a real server on a free port and clients that collect events.

// Helper module of integration tests: not every test file uses every helper.
#![allow(dead_code, clippy::unwrap_used, clippy::expect_used)]

use std::sync::{Arc, mpsc};
use std::time::Duration;
use veltrunox_client_adapters::{EventConnection, connect_events};
use veltrunox_client_app::{ChannelEvent, LinkEvent, ServerEvent};
use veltrunox_core::{PlaybackState, RecordingTitle};
use veltrunox_server_adapters::{
    AppState, Argon2Hasher, RandomTokens, SqliteStore, SystemClock, SystemWallClock, router,
    watch_end,
};
use veltrunox_server_app::{AccountService, RecordingRepository};

pub const WAIT: Duration = Duration::from_secs(5);

/// Starts the server on a free local port over `store`; returns its base URL.
pub async fn start_server(store: Arc<SqliteStore>) -> String {
    let app = AppState::new(store, Arc::new(SystemClock::new()));
    watch_end(&app);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router(app)).await.unwrap() });
    format!("http://{address}")
}

/// Like `start_server`, but with sign-in and rights checks switched on.
pub async fn secured_server() -> String {
    let store = store_with_recording();
    let accounts = Arc::new(
        AccountService::new(
            store.clone(),
            store.clone(),
            Arc::new(Argon2Hasher::new()),
            Arc::new(RandomTokens),
            Arc::new(SystemWallClock),
        )
        .unwrap(),
    );
    let app = AppState::new(store, Arc::new(SystemClock::new())).with_accounts(accounts);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router(app)).await.unwrap() });
    format!("http://{address}")
}

pub fn store_with_recording() -> Arc<SqliteStore> {
    let store = Arc::new(SqliteStore::in_memory().unwrap());
    store
        .add(RecordingTitle::new("Soundcheck").unwrap())
        .unwrap();
    store
}

/// A client on the real-time channel whose events can be waited for.
pub struct TestClient {
    pub connection: EventConnection,
    events: mpsc::Receiver<ChannelEvent>,
}

impl TestClient {
    /// Connects and waits for the greeting.
    pub fn connect(base_url: &str) -> Self {
        let (sender, events) = mpsc::channel();
        let connection = connect_events(base_url, move |event| {
            let _ = sender.send(event);
        })
        .unwrap();
        let mut client = Self { connection, events };
        client.next_matching(|event| {
            matches!(event, ChannelEvent::Link(LinkEvent::Greeted)).then_some(())
        });
        client
    }

    /// Waits for the first event `pick` accepts; earlier ones are skipped.
    pub fn next_matching<T>(&mut self, pick: impl Fn(&ChannelEvent) -> Option<T>) -> T {
        loop {
            let event = self.events.recv_timeout(WAIT).expect("event in time");
            if let Some(found) = pick(&event) {
                return found;
            }
        }
    }

    /// Waits for a state that satisfies `wanted`.
    pub fn state_where(&mut self, wanted: impl Fn(&PlaybackState) -> bool) -> PlaybackState {
        self.next_matching(|event| match event {
            ChannelEvent::Server(ServerEvent::State(state)) if wanted(state) => Some(*state),
            _ => None,
        })
    }
}
