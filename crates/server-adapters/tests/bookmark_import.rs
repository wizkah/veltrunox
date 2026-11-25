//! Importing bookmarks as JSON and CSV with preview, conflicts and per-recording outcomes.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use reqwest::{Client, StatusCode};
use serde_json::{Value, json};
use std::sync::Arc;
use support::{TestClient, secured_server, start_server};
use veltrunox_client_app::{ChannelEvent, ServerEvent};
use veltrunox_core::{Millis, RecordingId, RecordingTitle, Waveform};
use veltrunox_protocol::{BookmarksDto, ExportDto, ImportResultDto, ImportStatusDto};
use veltrunox_server_adapters::SqliteStore;
use veltrunox_server_app::{RecordingRepository, WaveformRepository};

/// Two recordings: id 1 is 60 s long, id 2 has no known length.
fn store() -> Arc<SqliteStore> {
    let store = Arc::new(SqliteStore::in_memory().unwrap());
    for title in ["First", "Second"] {
        store.add(RecordingTitle::new(title).unwrap()).unwrap();
    }
    store
        .save_waveform(
            RecordingId::new(1),
            Millis::new(60_000),
            &Waveform::from_mono(&[0.5; 10], 8_000),
        )
        .unwrap();
    store
}

fn import_url(base: &str, query: &str) -> String {
    format!("{base}/api/v1/import{query}")
}

async fn import(base: &str, query: &str, body: &Value) -> ImportResultDto {
    let response = Client::new()
        .post(import_url(base, query))
        .json(body)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    response.json().await.unwrap()
}

async fn bookmarks(base: &str, recording: u64) -> BookmarksDto {
    Client::new()
        .get(format!("{base}/api/v1/recordings/{recording}/bookmarks"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

async fn export(base: &str) -> ExportDto {
    Client::new()
        .get(format!("{base}/api/v1/export"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

fn two_labels() -> Value {
    json!({"recordings": [{
        "recording": {"id": 1},
        "bookmarks": {
            "labels": [
                {"name": "Verse", "position_ms": 10_000},
                {"name": "Bridge", "position_ms": 40_000}
            ],
            "regions": [{"name": "Chorus", "range": {"start_ms": 20_000, "end_ms": 30_000}}]
        }
    }]})
}

#[tokio::test]
async fn a_dry_run_reports_the_change_and_writes_nothing() {
    let url = start_server(store()).await;
    let result = import(&url, "?dry_run=true", &two_labels()).await;
    assert!(result.dry_run);
    let outcome = &result.recordings[0];
    assert_eq!(outcome.status, ImportStatusDto::Ok);
    assert_eq!((outcome.added, outcome.removed), (3, 0));
    assert!(bookmarks(&url, 1).await.labels.is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn an_import_replaces_the_bookmarks_and_tells_the_clients() {
    let url = start_server(store()).await;
    let mut watcher = TestClient::connect(&url);
    let result = import(&url, "", &two_labels()).await;
    assert_eq!(result.recordings[0].status, ImportStatusDto::Ok);
    watcher.next_matching(|event| match event {
        ChannelEvent::Server(ServerEvent::BookmarksChanged(id)) => Some(*id),
        _ => None,
    });
    let stored = bookmarks(&url, 1).await;
    assert_eq!(stored.labels.len(), 2);
    assert_eq!(stored.regions[0].name, "Chorus");
}

#[tokio::test]
async fn an_export_file_is_a_valid_import_that_changes_nothing() {
    let url = start_server(store()).await;
    import(&url, "", &two_labels()).await;
    let exported = serde_json::to_value(export(&url).await).unwrap();

    let result = import(&url, "?dry_run=true", &exported).await;
    assert_eq!(result.recordings.len(), 2);
    let first = &result.recordings[0];
    assert_eq!(first.status, ImportStatusDto::Ok);
    assert_eq!((first.unchanged, first.added, first.changed), (3, 0, 0));
}

#[tokio::test]
async fn an_entry_changed_since_the_export_is_a_conflict_unless_overwritten() {
    let url = start_server(store()).await;
    import(&url, "", &two_labels()).await;
    let mut exported = serde_json::to_value(export(&url).await).unwrap();

    // Someone edits the first label after the export.
    let label = &bookmarks(&url, 1).await.labels[0];
    let response = Client::new()
        .put(format!("{url}/api/v1/labels/{}", label.id))
        .json(&json!({
            "expected_version": label.version, "name": "Renamed", "position_ms": 10_000
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    exported["recordings"][0]["bookmarks"]["labels"][0]["name"] = json!("From the file");

    let refused = import(&url, "", &exported).await;
    let outcome = &refused.recordings[0];
    assert_eq!(outcome.status, ImportStatusDto::Conflict);
    assert_eq!(outcome.issues[0].entry, "labels[0]");
    assert_eq!(outcome.issues[0].reason.code, "bookmark_conflict");
    assert_eq!(bookmarks(&url, 1).await.labels[0].name, "Renamed");

    let forced = import(&url, "?overwrite=true", &exported).await;
    assert_eq!(forced.recordings[0].status, ImportStatusDto::Ok);
    assert_eq!(bookmarks(&url, 1).await.labels[0].name, "From the file");
}

#[tokio::test]
async fn an_entry_whose_original_was_deleted_is_a_conflict() {
    let url = start_server(store()).await;
    import(&url, "", &two_labels()).await;
    let exported = serde_json::to_value(export(&url).await).unwrap();
    import(
        &url,
        "",
        &json!({"recordings": [{"recording": {"id": 1}, "bookmarks": {}}]}),
    )
    .await;

    let result = import(&url, "", &exported).await;
    let outcome = &result.recordings[0];
    assert_eq!(outcome.status, ImportStatusDto::Conflict);
    assert_eq!(outcome.issues[0].reason.code, "bookmark_not_found");
    assert!(bookmarks(&url, 1).await.labels.is_empty());
}

#[tokio::test]
async fn each_recording_has_its_own_outcome() {
    let url = start_server(store()).await;
    let body = json!({"recordings": [
        {"recording": {"id": 1}, "bookmarks": {"labels": [
            {"name": "Fine", "position_ms": 1},
            {"name": " ", "position_ms": 2},
            {"name": "Late", "position_ms": 99}
        ]}},
        {"recording": {"id": 1}, "bookmarks": {"labels": [
            {"name": "Past the end", "position_ms": 60_001}
        ]}},
        {"recording": {"id": 77}, "bookmarks": {}},
        {"recording": {"id": 2}, "bookmarks": {"labels": [{"name": "Open end", "position_ms": 500_000}]}}
    ]});
    let result = import(&url, "", &body).await;
    let statuses: Vec<_> = result.recordings.iter().map(|r| r.status).collect();
    assert_eq!(
        statuses,
        [
            ImportStatusDto::Invalid,
            ImportStatusDto::Invalid,
            ImportStatusDto::UnknownRecording,
            ImportStatusDto::Ok
        ]
    );
    assert_eq!(result.recordings[0].issues[0].entry, "labels[1]");
    assert_eq!(result.recordings[0].issues[0].reason.code, "empty_name");
    assert_eq!(result.recordings[1].issues[0].entry, "labels[0]");
    assert_eq!(
        result.recordings[1].issues[0].reason.code,
        "outside_recording"
    );
    // The invalid recording stays as it was; the valid one is stored.
    assert!(bookmarks(&url, 1).await.labels.is_empty());
    assert_eq!(bookmarks(&url, 2).await.labels.len(), 1);
}

#[tokio::test]
async fn a_malformed_or_unsupported_body_is_refused_as_a_whole() {
    let url = start_server(store()).await;
    let malformed = Client::new()
        .post(import_url(&url, ""))
        .body("{not json")
        .send()
        .await
        .unwrap();
    assert_eq!(malformed.status(), StatusCode::BAD_REQUEST);
    let reason: Value = malformed.json().await.unwrap();
    assert_eq!(reason["code"], "import_malformed");

    let future = Client::new()
        .post(import_url(&url, ""))
        .json(&json!({"format_version": 2, "recordings": []}))
        .send()
        .await
        .unwrap();
    assert_eq!(future.status(), StatusCode::BAD_REQUEST);
    let reason: Value = future.json().await.unwrap();
    assert_eq!(reason["code"], "import_format_unsupported");
}

#[tokio::test]
async fn a_csv_table_goes_in_and_out() {
    let url = start_server(store()).await;
    let table = "recording_id,type,name,start_ms,end_ms\n\
                 1,label,\"Verse, soft\",10000,\n\
                 1,region,Chorus,20000,30000\n";
    let response = Client::new()
        .post(import_url(&url, ""))
        .header("content-type", "text/csv")
        .body(table)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let result: ImportResultDto = response.json().await.unwrap();
    assert_eq!(result.recordings[0].status, ImportStatusDto::Ok);
    assert_eq!(bookmarks(&url, 1).await.labels[0].name, "Verse, soft");

    let csv = Client::new()
        .get(format!("{url}/api/v1/export?format=csv"))
        .send()
        .await
        .unwrap();
    assert_eq!(csv.headers()["content-type"], "text/csv; charset=utf-8");
    let text = csv.text().await.unwrap();
    assert!(text.starts_with("recording_id,type,name,start_ms,end_ms,id,version\n"));
    assert!(text.contains("\"Verse, soft\""));

    // The exported table imports again without a change.
    let again = Client::new()
        .post(import_url(&url, "?dry_run=true"))
        .header("content-type", "text/csv")
        .body(text)
        .send()
        .await
        .unwrap();
    let again: ImportResultDto = again.json().await.unwrap();
    assert_eq!(again.recordings[0].unchanged, 2);

    let broken = Client::new()
        .post(import_url(&url, ""))
        .header("content-type", "text/csv")
        .body("recording_id,type,name,start_ms\n1,label,A,soon\n")
        .send()
        .await
        .unwrap();
    assert_eq!(broken.status(), StatusCode::BAD_REQUEST);
    let reason: Value = broken.json().await.unwrap();
    assert_eq!(reason["code"], "csv_invalid");
    assert_eq!(reason["args"], json!([2]));
}

#[tokio::test]
async fn the_schema_describes_what_an_import_needs() {
    let url = start_server(store()).await;
    let schema: Value = Client::new()
        .get(format!("{url}/api/v1/import/schema"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(schema["required"], json!(["recordings"]));
    assert!(schema["properties"]["format_version"].is_object());
    assert!(schema["$defs"]["ImportLabelDto"]["properties"]["position_ms"].is_object());
}

#[tokio::test]
async fn an_unknown_export_format_is_refused() {
    let url = start_server(store()).await;
    let response = Client::new()
        .get(format!("{url}/api/v1/export?format=xml"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn importing_needs_a_sign_in() {
    let url = secured_server().await;
    let response = Client::new()
        .post(import_url(&url, ""))
        .json(&two_labels())
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let schema = Client::new()
        .get(format!("{url}/api/v1/import/schema"))
        .send()
        .await
        .unwrap();
    assert_eq!(schema.status(), StatusCode::UNAUTHORIZED);
}
