//! Upload and administration of recordings, checked through the real endpoints.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use reqwest::{Client, Method, StatusCode};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use support::store_with_recording;
use veltrunox_core::{Intent, RecordingId};
use veltrunox_server_adapters::{
    AppState, Argon2Hasher, RandomTokens, SqliteStore, SystemClock, SystemWallClock, UploadStore,
    import_directory, router, watch_end,
};
use veltrunox_server_app::{AccountService, MediaCatalog};

const RATE: u32 = 8_000;
const PASSWORD: &str = "correct horse battery";

/// Mono 16-bit WAV of a tone; `hertz` makes the content differ.
fn wav(seconds: u32, hertz: f64) -> Vec<u8> {
    let samples: Vec<i16> = (0..seconds * RATE)
        .map(|frame| {
            let time = f64::from(frame) / f64::from(RATE);
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
    bytes.extend(RATE.to_le_bytes());
    bytes.extend((RATE * 2).to_le_bytes());
    bytes.extend(2_u16.to_le_bytes());
    bytes.extend(16_u16.to_le_bytes());
    bytes.extend(b"data");
    bytes.extend(data_bytes.to_le_bytes());
    for sample in samples {
        bytes.extend(sample.to_le_bytes());
    }
    bytes
}

fn fresh_directory(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("veltrunox-upload-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    path
}

fn files_in(directory: &Path) -> Vec<String> {
    let mut names: Vec<_> = std::fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

struct Server {
    url: String,
    app: AppState,
    store: Arc<SqliteStore>,
    media: PathBuf,
}

async fn serve(app: AppState) -> String {
    watch_end(&app);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router(app)).await.unwrap() });
    format!("http://{address}")
}

fn accounts(store: &Arc<SqliteStore>) -> Arc<AccountService> {
    Arc::new(
        AccountService::new(
            store.clone(),
            store.clone(),
            Arc::new(Argon2Hasher::new()),
            Arc::new(RandomTokens),
            Arc::new(SystemWallClock),
        )
        .unwrap(),
    )
}

/// An open server (everybody is admin) with an empty library and uploads up to `max` bytes.
async fn open_server(name: &str, max: u64) -> Server {
    let store = Arc::new(SqliteStore::in_memory().unwrap());
    server_over(store, name, max, false).await
}

async fn server_over(store: Arc<SqliteStore>, name: &str, max: u64, secured: bool) -> Server {
    let media = fresh_directory(name);
    let uploads = UploadStore::open(&media, max, store.as_ref()).unwrap();
    let mut app = AppState::new(store.clone(), Arc::new(SystemClock::new())).with_uploads(uploads);
    if secured {
        app = app.with_accounts(accounts(&store));
    }
    let url = serve(app.clone()).await;
    Server {
        url,
        app,
        store,
        media,
    }
}

async fn upload(url: &str, token: Option<&str>, name: &str, body: Vec<u8>) -> (StatusCode, String) {
    let mut request = Client::new()
        .post(format!("{url}/api/v1/recordings"))
        .header("x-file-name", name)
        .body(body);
    if let Some(token) = token {
        request = request.bearer_auth(token);
    }
    let response = request.send().await.unwrap();
    (response.status(), response.text().await.unwrap())
}

async fn call(
    url: &str,
    method: Method,
    path: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut request = Client::new().request(method, format!("{url}{path}"));
    if let Some(token) = token {
        request = request.bearer_auth(token);
    }
    if let Some(body) = body {
        request = request.json(&body);
    }
    let response = request.send().await.unwrap();
    let status = response.status();
    let text = response.text().await.unwrap();
    (status, serde_json::from_str(&text).unwrap_or(Value::Null))
}

async fn library(url: &str) -> Vec<Value> {
    let (status, body) = call(url, Method::GET, "/api/v1/recordings", None, None).await;
    assert_eq!(status, StatusCode::OK);
    body["recordings"].as_array().unwrap().clone()
}

fn json_of(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

/// Waits until the analysis of the recording is over, whatever its outcome.
async fn wait_for_scan(url: &str, id: u64) -> Value {
    let started = Instant::now();
    loop {
        let entries = library(url).await;
        let entry = entries.iter().find(|entry| entry["id"] == id).unwrap();
        if matches!(
            entry["analysis_progress"]["state"].as_str(),
            Some("done" | "failed")
        ) {
            return entry.clone();
        }
        assert!(
            started.elapsed() < Duration::from_secs(20),
            "scan took too long"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

#[tokio::test]
async fn an_uploaded_file_becomes_a_recording_that_gets_scanned() {
    let server = open_server("create", 1 << 20).await;
    let mut notices = server.app.hub.subscribe_library();
    let (status, body) = upload(&server.url, None, "Blue%20Monday.wav", wav(3, 440.0)).await;
    assert_eq!(status, StatusCode::CREATED);
    let created = json_of(&body);
    assert_eq!(created["title"], "Blue Monday");
    assert_eq!(created["existing"], false);
    let id = created["id"].as_u64().unwrap();

    assert!(
        notices.try_recv().is_ok(),
        "the library change is announced"
    );
    let entry = wait_for_scan(&server.url, id).await;
    assert_eq!(entry["title"], "Blue Monday");
    assert_eq!(entry["length_ms"], 3000);
    let (status, _) = call(
        &server.url,
        Method::GET,
        &format!("/api/v1/recordings/{id}/waveform"),
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let files = files_in(&server.media);
    assert_eq!(files.len(), 1);
    assert!(files[0].ends_with(".wav") && !files[0].starts_with('.'));
}

/// Sends an upload by hand, a chunk at a time with a pause between the chunks, so the server
/// sees the body arrive slowly. Returns the status line of the answer.
async fn upload_slowly(url: &str, id: Option<&str>, body: &[u8], chunks: usize) -> String {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let address = url.trim_start_matches("http://");
    let mut stream = tokio::net::TcpStream::connect(address).await.unwrap();
    let id_header = id.map_or(String::new(), |id| format!("x-upload-id: {id}\r\n"));
    let head = format!(
        "POST /api/v1/recordings HTTP/1.1\r\nHost: {address}\r\nx-file-name: slow.wav\r\n{id_header}\
         Content-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(head.as_bytes()).await.unwrap();
    for chunk in body.chunks(body.len().div_ceil(chunks)) {
        // A refused upload may be answered and closed before the body is sent.
        if stream.write_all(chunk).await.is_err() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(130)).await;
    }
    let mut answer = String::new();
    let _ = stream.read_to_string(&mut answer).await;
    answer.lines().next().unwrap_or_default().to_owned()
}

#[tokio::test]
async fn the_server_reports_the_bytes_it_has_received_under_the_upload_id() {
    let server = open_server("progress", 1 << 20).await;
    let mut notices = server.app.hub.subscribe_uploads();
    let body = wav(3, 330.0);
    let status = upload_slowly(&server.url, Some("slow-1"), &body, 4).await;
    assert!(status.contains("201"), "{status}");

    let mut counts = Vec::new();
    while let Ok(notice) = notices.try_recv() {
        assert_eq!(notice.upload_id, "slow-1");
        counts.push(notice.received_bytes);
    }
    assert!(counts.len() >= 2, "several notices: {counts:?}");
    assert!(
        counts.windows(2).all(|pair| pair[0] < pair[1]),
        "{counts:?}"
    );
    assert!(counts.iter().all(|count| *count <= body.len() as u64));
}

#[tokio::test]
async fn an_upload_without_an_id_reports_nothing_and_a_bad_id_is_refused() {
    let server = open_server("progress-none", 1 << 20).await;
    let mut notices = server.app.hub.subscribe_uploads();
    let status = upload_slowly(&server.url, None, &wav(2, 330.0), 3).await;
    assert!(status.contains("201"), "{status}");
    assert!(notices.try_recv().is_err());

    let refused = Client::new()
        .post(format!("{}/api/v1/recordings", server.url))
        .header("x-file-name", "bad.wav")
        .header("x-upload-id", "not valid!")
        .body(wav(2, 350.0))
        .send()
        .await
        .unwrap();
    assert_eq!(refused.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn the_same_audio_is_not_stored_twice() {
    let server = open_server("duplicate", 1 << 20).await;
    let (_, first) = upload(&server.url, None, "one.wav", wav(1, 300.0)).await;
    let (status, second) = upload(&server.url, None, "renamed copy.wav", wav(1, 300.0)).await;
    assert_eq!(status, StatusCode::OK);
    let (first, second) = (json_of(&first), json_of(&second));
    assert_eq!(second["existing"], true);
    assert_eq!(second["id"], first["id"]);
    assert_eq!(second["title"], "one");
    assert_eq!(library(&server.url).await.len(), 1);
    assert_eq!(files_in(&server.media).len(), 1);
}

#[tokio::test]
async fn a_file_that_is_not_audio_is_refused_and_leaves_nothing_behind() {
    let server = open_server("not-audio", 1 << 20).await;
    let (status, message) =
        upload(&server.url, None, "notes.wav", b"just some text".to_vec()).await;
    assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert!(message.contains("not audio"), "{message}");
    assert!(library(&server.url).await.is_empty());
    assert!(files_in(&server.media).is_empty());
}

#[tokio::test]
async fn too_large_empty_and_nameless_uploads_are_refused() {
    let server = open_server("limits", 1_000).await;
    let (status, message) = upload(&server.url, None, "big.wav", wav(1, 440.0)).await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
    assert!(message.contains("limit"), "{message}");
    let (status, _) = upload(&server.url, None, "empty.wav", Vec::new()).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let response = Client::new()
        .post(format!("{}/api/v1/recordings", server.url))
        .body(wav(0, 1.0))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert!(library(&server.url).await.is_empty());
    assert!(files_in(&server.media).is_empty());
}

#[tokio::test]
async fn a_server_without_an_upload_store_refuses_uploads() {
    let store = store_with_recording();
    let app = AppState::new(store, Arc::new(SystemClock::new()));
    let url = serve(app).await;
    let (status, _) = upload(&url, None, "a.wav", wav(1, 440.0)).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn renaming_changes_the_title_and_rejects_bad_requests() {
    let server = open_server("rename", 1 << 20).await;
    let (_, created) = upload(&server.url, None, "old.wav", wav(1, 440.0)).await;
    let id = json_of(&created)["id"].as_u64().unwrap();
    let path = format!("/api/v1/recordings/{id}");
    let mut notices = server.app.hub.subscribe_library();

    let (status, _) = call(
        &server.url,
        Method::PUT,
        &path,
        None,
        Some(json!({"title": "  New name "})),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(notices.try_recv().is_ok(), "the rename is announced");
    assert_eq!(library(&server.url).await[0]["title"], "New name");
    let (status, _) = call(
        &server.url,
        Method::PUT,
        &path,
        None,
        Some(json!({"title": "  "})),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = call(
        &server.url,
        Method::PUT,
        "/api/v1/recordings/999",
        None,
        Some(json!({"title": "x"})),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn deleting_removes_the_recording_its_data_and_its_file_and_unloads_it() {
    let server = open_server("delete", 1 << 20).await;
    let (_, created) = upload(&server.url, None, "doomed.wav", wav(2, 440.0)).await;
    let id = json_of(&created)["id"].as_u64().unwrap();
    wait_for_scan(&server.url, id).await;
    let (status, _) = call(
        &server.url,
        Method::POST,
        &format!("/api/v1/recordings/{id}/labels"),
        None,
        Some(json!({"name": "verse", "position_ms": 500})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    server
        .app
        .playback
        .apply(Intent::SelectRecording(RecordingId::new(id)))
        .unwrap();
    assert_eq!(files_in(&server.media).len(), 1);

    let path = format!("/api/v1/recordings/{id}");
    let (status, _) = call(&server.url, Method::DELETE, &path, None, None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    assert!(library(&server.url).await.is_empty());
    assert!(files_in(&server.media).is_empty());
    assert_eq!(server.app.playback.current().recording, None);
    let (status, _) = call(&server.url, Method::DELETE, &path, None, None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    // The same audio can come back as a new recording without bookmarks.
    let (status, again) = upload(&server.url, None, "doomed.wav", wav(2, 440.0)).await;
    assert_eq!(status, StatusCode::CREATED);
    let again = json_of(&again)["id"].as_u64().unwrap();
    let (_, bookmarks) = call(
        &server.url,
        Method::GET,
        &format!("/api/v1/recordings/{again}/bookmarks"),
        None,
        None,
    )
    .await;
    assert!(bookmarks["labels"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn deleting_never_touches_files_outside_the_upload_store() {
    let imported = fresh_directory("imported");
    std::fs::create_dir_all(&imported).unwrap();
    std::fs::write(imported.join("mine.wav"), wav(1, 500.0)).unwrap();
    let store = Arc::new(SqliteStore::in_memory().unwrap());
    import_directory(&imported, store.as_ref()).unwrap();
    let server = server_over(store, "outside", 1 << 20, false).await;
    let id = library(&server.url).await[0]["id"].as_u64().unwrap();

    let (status, _) = call(
        &server.url,
        Method::DELETE,
        &format!("/api/v1/recordings/{id}"),
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(imported.join("mine.wav").exists());
    std::fs::remove_dir_all(imported).unwrap();
}

#[test]
fn opening_the_store_removes_leftovers_and_keeps_referenced_files() {
    let directory = fresh_directory("sweep");
    std::fs::create_dir_all(&directory).unwrap();
    let kept = directory.join("aaaa.wav");
    std::fs::write(&kept, b"kept").unwrap();
    std::fs::write(directory.join(".upload-1-1.wav"), b"half").unwrap();
    std::fs::write(directory.join("bbbb.mp3"), b"orphan").unwrap();
    let store = SqliteStore::in_memory().unwrap();
    store
        .add_media(
            veltrunox_core::RecordingTitle::new("kept").unwrap(),
            &kept,
            "aaaa",
        )
        .unwrap();

    UploadStore::open(&directory, 1000, &store).unwrap();
    assert_eq!(files_in(&directory), ["aaaa.wav"]);
    std::fs::remove_dir_all(directory).unwrap();
}

async fn sign_in(url: &str, name: &str, role: &str, admin: Option<&str>) -> String {
    if let Some(admin) = admin {
        let (status, _) = call(
            url,
            Method::POST,
            "/api/v1/users",
            Some(admin),
            Some(json!({"user_name": name, "password": PASSWORD, "role": role})),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
    }
    let (status, session) = call(
        url,
        Method::POST,
        if admin.is_some() {
            "/api/v1/login"
        } else {
            "/api/v1/setup"
        },
        None,
        Some(json!({"user_name": name, "password": PASSWORD})),
    )
    .await;
    assert!(status == StatusCode::OK || status == StatusCode::CREATED);
    session["token"].as_str().unwrap().to_owned()
}

#[tokio::test]
async fn rights_decide_who_uploads_and_who_manages() {
    let store = Arc::new(SqliteStore::in_memory().unwrap());
    let server = server_over(store, "rights", 1 << 20, true).await;
    let admin = sign_in(&server.url, "root", "admin", None).await;
    let user = sign_in(&server.url, "ann", "user", Some(&admin)).await;

    let (status, _) = upload(&server.url, None, "a.wav", wav(1, 440.0)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, body) = upload(&server.url, Some(&user), "a.wav", wav(1, 440.0)).await;
    assert_eq!(status, StatusCode::CREATED);
    let id = json_of(&body)["id"].as_u64().unwrap();
    let path = format!("/api/v1/recordings/{id}");

    let (status, _) = call(
        &server.url,
        Method::PUT,
        &path,
        Some(&user),
        Some(json!({"title": "x"})),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = call(&server.url, Method::DELETE, &path, Some(&user), None).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = call(&server.url, Method::DELETE, &path, None, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(files_in(&server.media).len(), 1);

    let (status, _) = call(
        &server.url,
        Method::PUT,
        &path,
        Some(&admin),
        Some(json!({"title": "x"})),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = call(&server.url, Method::DELETE, &path, Some(&admin), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(files_in(&server.media).is_empty());
    let _ = &server.store;
}
