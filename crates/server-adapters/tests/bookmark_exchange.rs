//! Replacing all bookmarks of a recording and exporting the library, over the real endpoints.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use reqwest::{Client, StatusCode};
use serde_json::{Value, json};
use std::sync::Arc;
use support::{TestClient, secured_server, start_server, store_with_recording};
use veltrunox_client_app::{ChannelEvent, ServerEvent};
use veltrunox_core::{Millis, RecordingId, Waveform};
use veltrunox_protocol::{BookmarksDto, ExportDto};
use veltrunox_server_adapters::SqliteStore;
use veltrunox_server_app::WaveformRepository;

const RECORDING: RecordingId = RecordingId::new(1);

fn bookmarks_url(base: &str) -> String {
    format!("{base}/api/v1/recordings/1/bookmarks")
}

fn replacement() -> Value {
    json!({
        "labels": [{"name": "Verse", "position_ms": 12_000}],
        "regions": [{"name": "Chorus", "range": {"start_ms": 30_000, "end_ms": 60_000}}]
    })
}

async fn put(url: &str, body: &Value) -> reqwest::Response {
    Client::new().put(url).json(body).send().await.unwrap()
}

async fn stored(base: &str) -> BookmarksDto {
    Client::new()
        .get(bookmarks_url(base))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

fn store_with_length(length_ms: u64) -> Arc<SqliteStore> {
    let store = store_with_recording();
    let waveform = Waveform::from_mono(&[0.5; 10], 8_000);
    store
        .save_waveform(RECORDING, Millis::new(length_ms), &waveform)
        .unwrap();
    store
}

#[tokio::test(flavor = "multi_thread")]
async fn replacing_swaps_the_whole_set_and_tells_the_clients() {
    let url = start_server(store_with_recording()).await;
    let mut watcher = TestClient::connect(&url);
    let first = put(&bookmarks_url(&url), &replacement()).await;
    assert_eq!(first.status(), StatusCode::OK);
    let announced = watcher.next_matching(|event| match event {
        ChannelEvent::Server(ServerEvent::BookmarksChanged(id)) => Some(*id),
        _ => None,
    });
    assert_eq!(announced, RECORDING);

    let second = put(
        &bookmarks_url(&url),
        &json!({"labels": [{"name": "Outro", "position_ms": 90_000}]}),
    )
    .await;
    let answer: BookmarksDto = second.json().await.unwrap();
    assert_eq!(answer.labels.len(), 1);
    assert_eq!(answer.labels[0].name, "Outro");
    assert!(answer.regions.is_empty());
    assert_eq!(stored(&url).await, answer);
}

#[tokio::test]
async fn an_invalid_entry_changes_nothing() {
    let url = start_server(store_with_recording()).await;
    put(&bookmarks_url(&url), &replacement()).await;
    let before = stored(&url).await;

    let unnamed = put(
        &bookmarks_url(&url),
        &json!({
            "labels": [{"name": "Fine", "position_ms": 1}, {"name": " ", "position_ms": 2}]
        }),
    )
    .await;
    assert_eq!(unnamed.status(), StatusCode::BAD_REQUEST);
    let reason: Value = unnamed.json().await.unwrap();
    assert_eq!(reason["code"], "empty_name");

    let backwards = put(
        &bookmarks_url(&url),
        &json!({"regions": [{"name": "X", "range": {"start_ms": 5, "end_ms": 5}}]}),
    )
    .await;
    assert_eq!(backwards.status(), StatusCode::BAD_REQUEST);
    assert_eq!(stored(&url).await, before);
}

#[tokio::test]
async fn bookmarks_past_the_end_of_a_known_length_are_refused() {
    let url = start_server(store_with_length(60_000)).await;
    let past_end = put(
        &bookmarks_url(&url),
        &json!({"labels": [{"name": "Late", "position_ms": 60_001}]}),
    )
    .await;
    assert_eq!(past_end.status(), StatusCode::BAD_REQUEST);
    let reason: Value = past_end.json().await.unwrap();
    assert_eq!(reason["code"], "outside_recording");
    assert!(stored(&url).await.labels.is_empty());

    let at_end = put(&bookmarks_url(&url), &replacement()).await;
    assert_eq!(at_end.status(), StatusCode::OK);
}

#[tokio::test]
async fn an_unknown_recording_is_not_found() {
    let url = start_server(store_with_recording()).await;
    let response = put(
        &format!("{url}/api/v1/recordings/99/bookmarks"),
        &replacement(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn the_export_carries_details_and_bookmarks() {
    let url = start_server(store_with_length(90_000)).await;
    put(&bookmarks_url(&url), &replacement()).await;

    let all: ExportDto = Client::new()
        .get(format!("{url}/api/v1/export"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(all.format_version, 1);
    assert_eq!(all.recordings.len(), 1);
    let exported = &all.recordings[0];
    assert_eq!(exported.recording.id, 1);
    assert_eq!(exported.recording.title, "Soundcheck");
    assert_eq!(exported.recording.length_ms, Some(90_000));
    assert_eq!(exported.bookmarks, stored(&url).await);

    let one = Client::new()
        .get(format!("{url}/api/v1/recordings/1/export"))
        .send()
        .await
        .unwrap();
    assert_eq!(one.status(), StatusCode::OK);
    let one: ExportDto = one.json().await.unwrap();
    assert_eq!(one, all);

    let missing = Client::new()
        .get(format!("{url}/api/v1/recordings/99/export"))
        .send()
        .await
        .unwrap();
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn export_and_replace_need_a_sign_in() {
    let url = secured_server().await;
    let export = Client::new()
        .get(format!("{url}/api/v1/export"))
        .send()
        .await
        .unwrap();
    assert_eq!(export.status(), StatusCode::UNAUTHORIZED);
    let replace = put(&bookmarks_url(&url), &replacement()).await;
    assert_eq!(replace.status(), StatusCode::UNAUTHORIZED);
}
