//! The real `veltrunox-mcp` program against a real server, with real API tokens.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use serde_json::{Value, json};
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::Arc;
use veltrunox_core::RecordingTitle;
use veltrunox_server_adapters::{
    AppState, Argon2Hasher, RandomTokens, SqliteStore, SystemClock, SystemWallClock, router,
};
use veltrunox_server_app::{AccountService, ApiTokenService, RecordingRepository};

const PASSWORD: &str = "correct horse battery";

/// A secured server with one recording; returns its address.
async fn start_server() -> String {
    let store = Arc::new(SqliteStore::in_memory().unwrap());
    store
        .add(RecordingTitle::new("Soundcheck").unwrap())
        .unwrap();
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
    let api_tokens = Arc::new(ApiTokenService::new(
        store.clone(),
        Arc::new(RandomTokens),
        Arc::new(SystemWallClock),
    ));
    let app = AppState::new(store, Arc::new(SystemClock::new()))
        .with_accounts(accounts)
        .with_api_tokens(api_tokens);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router(app)).await.unwrap() });
    format!("http://{address}")
}

/// Sets up the admin and returns an API token of the given scope.
async fn api_token(url: &str, scope: &str) -> String {
    let client = reqwest::Client::new();
    let session: Value = client
        .post(format!("{url}/api/v1/setup"))
        .json(&json!({"user_name": "root", "password": PASSWORD}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let login = session["token"].as_str().unwrap();
    let created: Value = client
        .post(format!("{url}/api/v1/tokens"))
        .bearer_auth(login)
        .json(&json!({"name": "mcp", "scope": scope}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    created["token"].as_str().unwrap().to_owned()
}

/// Starts the program, sends the requests (one per line) and returns its answers.
fn run_program(url: &str, token: &str, requests: &[Value]) -> Vec<Value> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_veltrunox-mcp"))
        .env("VELTRUNOX_URL", url)
        .env("VELTRUNOX_TOKEN", token)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    for request in requests {
        writeln!(input, "{request}").unwrap();
    }
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

fn tool_call(id: u64, name: &str, arguments: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "method": "tools/call",
           "params": {"name": name, "arguments": arguments}})
}

fn text(reply: &Value) -> &str {
    reply["result"]["content"][0]["text"].as_str().unwrap()
}

#[tokio::test(flavor = "multi_thread")]
async fn an_llm_reads_the_library_and_writes_bookmarks_through_the_program() {
    let url = start_server().await;
    let token = api_token(&url, "edit_bookmarks").await;
    let requests = [
        json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}),
        json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
        tool_call(2, "list_recordings", json!({})),
        tool_call(
            3,
            "set_bookmarks",
            json!({"recording_id": 1, "dry_run": true,
                   "labels": [{"name": "Verse", "position_ms": 12000}]}),
        ),
        tool_call(4, "get_bookmarks", json!({"recording_id": 1})),
        tool_call(
            5,
            "set_bookmarks",
            json!({"recording_id": 1,
                   "labels": [{"name": "Verse", "position_ms": 12000}],
                   "regions": [{"name": "Chorus", "range": {"start_ms": 30000, "end_ms": 60000}}]}),
        ),
        tool_call(6, "get_recording", json!({"recording_id": 1})),
    ];
    let replies = tokio::task::spawn_blocking(move || run_program(&url, &token, &requests))
        .await
        .unwrap();

    // The notification gets no answer.
    assert_eq!(replies.len(), 6);
    assert!(text(&replies[1]).contains("Soundcheck"));
    assert_eq!(replies[2]["result"]["isError"], false);
    assert!(text(&replies[2]).contains("\"dry_run\":true"));
    // The preview wrote nothing.
    assert_eq!(text(&replies[3]), r#"{"labels":[],"regions":[]}"#);
    assert_eq!(replies[4]["result"]["isError"], false);
    let recording: Value = serde_json::from_str(text(&replies[5])).unwrap();
    let bookmarks = &recording["recordings"][0]["bookmarks"];
    assert_eq!(bookmarks["labels"][0]["name"], "Verse");
    assert_eq!(bookmarks["regions"][0]["range"]["end_ms"], 60000);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_read_only_token_cannot_write_through_the_program() {
    let url = start_server().await;
    let token = api_token(&url, "read_only").await;
    let requests = [
        tool_call(1, "list_recordings", json!({})),
        tool_call(
            2,
            "set_bookmarks",
            json!({"recording_id": 1, "labels": [{"name": "Verse", "position_ms": 1}]}),
        ),
    ];
    let replies = tokio::task::spawn_blocking(move || run_program(&url, &token, &requests))
        .await
        .unwrap();
    assert_eq!(replies[0]["result"]["isError"], false);
    assert_eq!(replies[1]["result"]["isError"], true);
    assert!(text(&replies[1]).contains("403"));
}

#[tokio::test(flavor = "multi_thread")]
async fn a_wrong_token_is_reported_as_a_refusal() {
    let url = start_server().await;
    api_token(&url, "read_only").await;
    let requests = [tool_call(1, "list_recordings", json!({}))];
    let replies = tokio::task::spawn_blocking(move || run_program(&url, "vxt_wrong", &requests))
        .await
        .unwrap();
    assert_eq!(replies[0]["result"]["isError"], true);
    assert!(text(&replies[0]).contains("401"));
}

#[test]
fn the_program_needs_a_token() {
    let status = Command::new(env!("CARGO_BIN_EXE_veltrunox-mcp"))
        .env_remove("VELTRUNOX_TOKEN")
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(2));
}
