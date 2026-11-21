//! Vertical slice: HTTP client adapter against the real server router.

use std::sync::Arc;
use veltrunox_client_adapters::HttpServerLink;
use veltrunox_client_app::load_library;
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
