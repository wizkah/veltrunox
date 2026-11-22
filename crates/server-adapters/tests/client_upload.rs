//! The client core uploads, renames and deletes recordings against a real server.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;
use veltrunox_client_adapters::HttpServerLink;
use veltrunox_client_app::{
    LinkError, add_recording, delete_recording, load_library, rename_recording,
};
use veltrunox_core::RecordingTitle;
use veltrunox_server_adapters::{AppState, SqliteStore, SystemClock, UploadStore, router};

fn wav(hertz: f64) -> Vec<u8> {
    let rate = 8_000_u32;
    let samples: Vec<i16> = (0..rate)
        .map(|frame| {
            let time = f64::from(frame) / f64::from(rate);
            (0.3 * (2.0 * std::f64::consts::PI * hertz * time).sin() * f64::from(i16::MAX)) as i16
        })
        .collect();
    let data_bytes = (samples.len() * 2) as u32;
    let mut bytes = Vec::new();
    bytes.extend(b"RIFF");
    bytes.extend((36 + data_bytes).to_le_bytes());
    bytes.extend(b"WAVEfmt ");
    bytes.extend(16_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(rate.to_le_bytes());
    bytes.extend((rate * 2).to_le_bytes());
    bytes.extend(2_u16.to_le_bytes());
    bytes.extend(16_u16.to_le_bytes());
    bytes.extend(b"data");
    bytes.extend(data_bytes.to_le_bytes());
    bytes.extend(samples.iter().flat_map(|sample| sample.to_le_bytes()));
    bytes
}

async fn server(name: &str, max: u64) -> (String, std::path::PathBuf) {
    let store = Arc::new(SqliteStore::in_memory().unwrap());
    let directory = std::env::temp_dir().join(format!(
        "veltrunox-client-upload-{name}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&directory);
    let uploads = UploadStore::open(&directory, max, store.as_ref()).unwrap();
    let app = AppState::new(store, Arc::new(SystemClock::new())).with_uploads(uploads);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router(app)).await.unwrap() });
    (format!("http://{address}"), directory)
}

#[tokio::test]
async fn upload_rename_and_delete_go_through_the_client_core() {
    let (url, directory) = server("flow", 1 << 20).await;
    let link = HttpServerLink::new(&url);

    let added = add_recording(&link, "Grüße von Ünï.wav", wav(440.0))
        .await
        .unwrap();
    assert_eq!(added.title.as_str(), "Grüße von Ünï");
    assert!(!added.existing);
    let again = add_recording(&link, "copy.wav", wav(440.0)).await.unwrap();
    assert!(again.existing);
    assert_eq!(again.id, added.id);
    assert_eq!(load_library(&link).await.unwrap().len(), 1);

    rename_recording(&link, added.id, &RecordingTitle::new("Intro").unwrap())
        .await
        .unwrap();
    let library = load_library(&link).await.unwrap();
    assert_eq!(library[0].recording.title.as_str(), "Intro");

    delete_recording(&link, added.id).await.unwrap();
    assert!(load_library(&link).await.unwrap().is_empty());
    assert!(matches!(
        delete_recording(&link, added.id).await,
        Err(LinkError::Unreachable(_))
    ));
    std::fs::remove_dir_all(directory).unwrap();
}

#[tokio::test]
async fn refused_files_come_back_with_the_servers_reason() {
    let (url, directory) = server("small", 500).await;
    let link = HttpServerLink::new(&url);

    let too_large = add_recording(&link, "big.wav", wav(440.0)).await;
    assert!(matches!(&too_large, Err(LinkError::Refused(reason)) if reason.contains("limit")));
    let (url, other) = server("normal", 1 << 20).await;
    let link = HttpServerLink::new(&url);
    let text = add_recording(&link, "notes.wav", b"not audio".to_vec()).await;
    assert!(matches!(&text, Err(LinkError::Refused(reason)) if reason.contains("not audio")));
    std::fs::remove_dir_all(directory).unwrap();
    std::fs::remove_dir_all(other).unwrap();
}
