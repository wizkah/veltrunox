//! Stem packages, checked through the real endpoints (ADR-015).

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use reqwest::{Client, StatusCode};
use serde_json::{Value, json};
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use support::TestClient;
use veltrunox_client_adapters::HttpServerLink;
use veltrunox_client_app::{LinkError, StemLink};
use veltrunox_core::{Intent, RecordingId};
use veltrunox_server_adapters::{
    AppState, SqliteStore, StemStore, SystemClock, UploadStore, router,
};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

const LOW: &[u8] = include_bytes!("fixtures/stem-110.ogg");
const HIGH: &[u8] = include_bytes!("fixtures/stem-880.ogg");
/// The rate of the stem fixtures: stems must have the rate of their recording.
const RATE: u32 = 22_050;

/// Two seconds of silence as mono 16-bit WAV: the recording the stems belong to.
fn recording() -> Vec<u8> {
    let data = RATE * 2 * 2;
    let mut bytes = Vec::new();
    bytes.extend(b"RIFF");
    bytes.extend((36 + data).to_le_bytes());
    bytes.extend(b"WAVEfmt ");
    bytes.extend(16_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(RATE.to_le_bytes());
    bytes.extend((RATE * 2).to_le_bytes());
    bytes.extend(2_u16.to_le_bytes());
    bytes.extend(16_u16.to_le_bytes());
    bytes.extend(b"data");
    bytes.extend(data.to_le_bytes());
    bytes.resize(bytes.len() + data as usize, 0);
    bytes
}

struct Server {
    url: String,
    stems: PathBuf,
    id: u64,
    hash: String,
}

async fn server(name: &str) -> Server {
    let root = std::env::temp_dir().join(format!("veltrunox-stems-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let store = Arc::new(SqliteStore::in_memory().unwrap());
    let stems = root.join("stems");
    let app = AppState::new(store.clone(), Arc::new(SystemClock::new()))
        .with_uploads(UploadStore::open(&root.join("media"), 1 << 20, store.as_ref()).unwrap())
        .with_stems(StemStore::open(&stems, 1 << 20, store.as_ref()).unwrap());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, router(app)).await.unwrap() });

    let uploaded: Value = Client::new()
        .post(format!("{url}/api/v1/recordings"))
        .header("x-file-name", "song.wav")
        .body(recording())
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    // The upload stores the file under the hash of its content.
    let file = std::fs::read_dir(root.join("media"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let hash = file.file_stem().unwrap().to_str().unwrap().to_owned();
    Server {
        url,
        stems,
        id: uploaded["id"].as_u64().unwrap(),
        hash,
    }
}

fn package(hash: &str, files: &[(&str, &str, &[u8])], analysis: Value) -> Vec<u8> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default();
    for (_, file, bytes) in files {
        writer.start_file(*file, options).unwrap();
        writer.write_all(bytes).unwrap();
    }
    let stems: Vec<Value> = files
        .iter()
        .map(|(name, file, _)| json!({"name": name, "file": file}))
        .collect();
    let manifest = json!({
        "format_version": 1,
        "recording": {"content_hash": hash, "title": "song"},
        "stems": stems,
        "analysis": analysis,
    });
    writer.start_file("manifest.json", options).unwrap();
    writer.write_all(manifest.to_string().as_bytes()).unwrap();
    writer.finish().unwrap().into_inner()
}

fn two_stems(hash: &str) -> Vec<u8> {
    package(
        hash,
        &[("bass", "stems/a.ogg", LOW), ("other", "stems/b.ogg", HIGH)],
        json!({
            "downbeats_ms": [0, 1000],
            "sections": [
                {"start_ms": 0, "end_ms": 900, "label": "intro"},
                {"start_ms": 900, "end_ms": 2000, "label": "verse"},
            ],
        }),
    )
}

async fn send(url: &str, body: Vec<u8>) -> (StatusCode, Value) {
    let response = Client::new()
        .post(format!("{url}/api/v1/stem-packages"))
        .body(body)
        .send()
        .await
        .unwrap();
    let status = response.status();
    (status, response.json().await.unwrap_or(Value::Null))
}

async fn get(url: String) -> (StatusCode, Vec<u8>) {
    let response = Client::new().get(url).send().await.unwrap();
    (response.status(), response.bytes().await.unwrap().to_vec())
}

fn json_of(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).unwrap()
}

fn count_files(directory: &Path) -> usize {
    std::fs::read_dir(directory).unwrap().count()
}

#[tokio::test]
async fn a_package_gives_the_recording_stems_and_section_regions() {
    let server = server("import").await;
    let (status, body) = send(&server.url, two_stems(&server.hash)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["recording"], server.id);
    assert_eq!(body["stems"], 2);
    assert_eq!(body["regions_added"], 2);

    let base = format!("{}/api/v1/recordings/{}", server.url, server.id);
    let (_, stems) = get(format!("{base}/stems")).await;
    let stems = json_of(&stems);
    assert_eq!(stems["stems"][0]["name"], "bass");
    assert_eq!(stems["stems"][1]["name"], "other");
    let length = stems["stems"][0]["length_ms"].as_u64().unwrap();
    assert!((1_950..=2_050).contains(&length), "{length}");

    let (status, info) = get(format!("{base}/stems/1/waveform")).await;
    assert_eq!(status, StatusCode::OK);
    assert!(!json_of(&info)["levels"].as_array().unwrap().is_empty());
    let (status, peaks) = get(format!("{base}/stems/1/waveform/0")).await;
    assert_eq!(status, StatusCode::OK);
    assert!(!peaks.is_empty());

    let (_, bookmarks) = get(format!("{base}/bookmarks")).await;
    let regions = json_of(&bookmarks)["regions"].clone();
    assert_eq!(regions[0]["name"], "Intro 1");
    assert_eq!(regions[1]["name"], "Verse 1");

    // Importing again replaces the files and adds no region twice.
    let (_, again) = send(&server.url, two_stems(&server.hash)).await;
    assert_eq!(again["regions_added"], 0);
    assert_eq!(count_files(&server.stems), 2);
}

#[tokio::test]
async fn an_exported_package_can_be_imported_again() {
    let server = server("export").await;
    send(&server.url, two_stems(&server.hash)).await;
    let base = format!("{}/api/v1/recordings/{}", server.url, server.id);
    let (status, exported) = get(format!("{base}/stems/package")).await;
    assert_eq!(status, StatusCode::OK);

    let mut archive = ZipArchive::new(Cursor::new(exported.clone())).unwrap();
    let mut manifest = String::new();
    archive
        .by_name("manifest.json")
        .unwrap()
        .read_to_string(&mut manifest)
        .unwrap();
    let manifest = json_of(manifest.as_bytes());
    assert_eq!(manifest["recording"]["content_hash"], server.hash);
    assert_eq!(manifest["analysis"]["sections"][1]["label"], "verse");

    let removed = Client::new()
        .delete(format!("{base}/stems"))
        .send()
        .await
        .unwrap();
    assert_eq!(removed.status(), StatusCode::NO_CONTENT);
    assert_eq!(count_files(&server.stems), 0);
    let (status, _) = get(format!("{base}/stems/package")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, body) = send(&server.url, exported).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["stems"], 2);
}

#[tokio::test(flavor = "multi_thread")]
async fn the_mix_is_shared_stored_and_reset_by_new_stems() {
    let server = server("mix").await;
    send(&server.url, two_stems(&server.hash)).await;
    let mut first = TestClient::connect(&server.url);
    let mut second = TestClient::connect(&server.url);
    first
        .connection
        .send_intent(Intent::SelectRecording(RecordingId::new(server.id)));
    first.connection.send_intent(Intent::SetStemsEnabled(true));
    first.connection.send_intent(Intent::SetStemMuted {
        stem: 1,
        muted: true,
    });
    second.state_where(|state| state.settings.mix.channels[1].muted);

    send(&server.url, two_stems(&server.hash)).await;
    second.state_where(|state| !state.settings.mix.channels[1].muted);
}

#[tokio::test]
async fn stems_of_another_rate_than_the_recording_are_refused() {
    let server = server("rate").await;
    // The scan tells the rate of the recording.
    let waveform = format!("{}/api/v1/recordings/{}/waveform", server.url, server.id);
    while get(waveform.clone()).await.0 != StatusCode::OK {
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    let other = include_bytes!("fixtures/stem-16k.ogg");
    let (status, body) = send(
        &server.url,
        package(&server.hash, &[("bass", "a.ogg", other)], Value::Null),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["code"], "stems_rate_mismatch");
}

#[tokio::test]
async fn the_client_adapter_imports_lists_exports_and_removes() {
    let server = server("client").await;
    let link = HttpServerLink::new(&server.url);
    let id = RecordingId::new(server.id);

    let imported = link
        .send_stem_package(two_stems(&server.hash))
        .await
        .unwrap();
    assert_eq!((imported.stems, imported.regions_added), (2, 2));
    assert_eq!(link.fetch_stems(id).await.unwrap()[1].name, "other");
    let waveform = link.fetch_stem_waveform(id, 0).await.unwrap();
    assert!(!waveform.waveform.levels.is_empty());
    let package = link.fetch_stem_package(id).await.unwrap();
    link.remove_stems(id).await.unwrap();
    assert!(link.fetch_stems(id).await.unwrap().is_empty());
    assert!(link.send_stem_package(package).await.is_ok());

    let refused = link.send_stem_package(two_stems("unknown")).await;
    assert!(
        matches!(refused, Err(LinkError::Refused(reason)) if reason.code == "stems_unknown_recording")
    );
}

#[tokio::test]
async fn deleting_the_recording_removes_its_stem_files() {
    let server = server("delete").await;
    send(&server.url, two_stems(&server.hash)).await;
    let deleted = Client::new()
        .delete(format!("{}/api/v1/recordings/{}", server.url, server.id))
        .send()
        .await
        .unwrap();
    assert_eq!(deleted.status(), StatusCode::NO_CONTENT);
    assert_eq!(count_files(&server.stems), 0);
}

#[tokio::test]
async fn unusable_packages_are_refused_and_leave_nothing_behind() {
    let server = server("refused").await;
    let (status, body) = send(&server.url, b"not a zip".to_vec()).await;
    assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert_eq!(body["code"], "stems_package_invalid");

    let (status, body) = send(&server.url, two_stems("unknown")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], "stems_unknown_recording");

    let broken = package(
        &server.hash,
        &[("bass", "stems/a.ogg", b"noise")],
        Value::Null,
    );
    let (status, _) = send(&server.url, broken).await;
    assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert_eq!(count_files(&server.stems), 0);
}
