//! Vertical slice: HTTP client adapter against the real server router.

use std::sync::Arc;
use veltrunox_client_adapters::{HttpServerLink, connect_events};
use veltrunox_client_app::{ConnectionMonitor, ConnectionStatus, load_library};
use veltrunox_core::{Recording, RecordingId, RecordingTitle};
use veltrunox_server_adapters::{InMemoryRecordingRepository, router};

#[tokio::test]
async fn client_lists_recordings_served_by_the_server() {
    let recording = Recording {
        id: RecordingId::new(7),
        title: RecordingTitle::new("Soundcheck").unwrap(),
    };
    let repository = Arc::new(InMemoryRecordingRepository::new(vec![recording.clone()]));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router(repository)).await.unwrap() });

    let link = HttpServerLink::new(&format!("http://{address}"));
    let listed = load_library(&link).await.unwrap();

    assert_eq!(listed, vec![recording]);
}

// The test thread blocks while waiting, so the server needs its own worker thread.
#[tokio::test(flavor = "multi_thread")]
async fn client_becomes_connected_after_the_servers_greeting() {
    let repository = Arc::new(InMemoryRecordingRepository::new(Vec::new()));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router(repository)).await.unwrap() });

    let (statuses, received) = std::sync::mpsc::channel();
    let monitor = std::sync::Mutex::new(ConnectionMonitor::new(move |status| {
        statuses.send(status).unwrap();
    }));
    let _connection = connect_events(&format!("http://{address}"), move |event| {
        monitor.lock().unwrap().push(event);
    })
    .unwrap();

    let status = received
        .recv_timeout(std::time::Duration::from_secs(5))
        .unwrap();
    assert_eq!(status, ConnectionStatus::Connected);
}
