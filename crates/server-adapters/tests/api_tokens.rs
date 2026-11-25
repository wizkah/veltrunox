//! API tokens: made by signed-in users, limited in reach, accepted like a login token.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use ewebsock::{Options, WsEvent, WsMessage};
use reqwest::{Client, Method, StatusCode};
use serde_json::{Value, json};
use std::sync::mpsc;
use std::time::Duration;
use support::secured_server;

const PASSWORD: &str = "correct horse battery";

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

async fn set_up_admin(url: &str) -> String {
    let (status, session) = call(
        url,
        Method::POST,
        "/api/v1/setup",
        None,
        Some(json!({"user_name": "root", "password": PASSWORD})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    session["token"].as_str().unwrap().to_owned()
}

/// Creates a user as the admin; returns the user id and a login token.
async fn add_user(url: &str, admin: &str, name: &str) -> (u64, String) {
    let (status, user) = call(
        url,
        Method::POST,
        "/api/v1/users",
        Some(admin),
        Some(json!({"user_name": name, "password": PASSWORD, "role": "user"})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, session) = call(
        url,
        Method::POST,
        "/api/v1/login",
        None,
        Some(json!({"user_name": name, "password": PASSWORD})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    (
        user["id"].as_u64().unwrap(),
        session["token"].as_str().unwrap().to_owned(),
    )
}

/// Makes an API token for the holder of `login`; returns its id and text.
async fn make_token(url: &str, login: &str, name: &str, scope: &str) -> (u64, String) {
    let (status, created) = call(
        url,
        Method::POST,
        "/api/v1/tokens",
        Some(login),
        Some(json!({"name": name, "scope": scope})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    (
        created["api_token"]["id"].as_u64().unwrap(),
        created["token"].as_str().unwrap().to_owned(),
    )
}

#[tokio::test]
async fn a_token_is_shown_once_and_listed_without_its_text() {
    let url = secured_server().await;
    let admin = set_up_admin(&url).await;
    let (_, ann) = add_user(&url, &admin, "ann").await;
    let (id, token) = make_token(&url, &ann, " LLM ", "edit_bookmarks").await;
    assert!(token.starts_with("vxt_"));

    let (status, listed) = call(&url, Method::GET, "/api/v1/tokens", Some(&ann), None).await;
    assert_eq!(status, StatusCode::OK);
    let entry = &listed["tokens"][0];
    assert_eq!(entry["id"], id);
    assert_eq!(entry["name"], "LLM");
    assert_eq!(entry["scope"], "edit_bookmarks");
    assert!(entry["last_used"].is_null());
    assert!(!listed.to_string().contains(&token));

    // Using the token is noted.
    call(&url, Method::GET, "/api/v1/export", Some(&token), None).await;
    let (_, listed) = call(&url, Method::GET, "/api/v1/tokens", Some(&ann), None).await;
    assert!(listed["tokens"][0]["last_used"].is_u64());

    let (status, reason) = call(
        &url,
        Method::POST,
        "/api/v1/tokens",
        Some(&ann),
        Some(json!({"name": " ", "scope": "read_only"})),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(reason["code"], "empty_name");
}

#[tokio::test]
async fn a_read_token_reads_and_nothing_else() {
    let url = secured_server().await;
    let admin = set_up_admin(&url).await;
    let (_, token) = make_token(&url, &admin, "reader", "read_only").await;

    let (status, _) = call(&url, Method::GET, "/api/v1/export", Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);
    let (status, me) = call(&url, Method::GET, "/api/v1/me", Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(me["capabilities"], json!([]));

    let (status, _) = call(
        &url,
        Method::PUT,
        "/api/v1/recordings/1/bookmarks",
        Some(&token),
        Some(json!({})),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn an_edit_token_writes_bookmarks_even_for_an_admin_but_does_no_more() {
    let url = secured_server().await;
    let admin = set_up_admin(&url).await;
    let (_, token) = make_token(&url, &admin, "writer", "edit_bookmarks").await;

    let (status, me) = call(&url, Method::GET, "/api/v1/me", Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(me["role"], "admin");
    assert_eq!(me["capabilities"], json!(["edit_bookmarks"]));

    let (status, _) = call(
        &url,
        Method::PUT,
        "/api/v1/recordings/1/bookmarks",
        Some(&token),
        Some(json!({"labels": [{"name": "Verse", "position_ms": 5}]})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = call(
        &url,
        Method::POST,
        "/api/v1/import?dry_run=true",
        Some(&token),
        Some(json!({"recordings": []})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // The admin's other rights stay out of reach.
    for (method, path, body) in [
        (
            Method::PUT,
            "/api/v1/recordings/1",
            json!({"title": "Renamed"}),
        ),
        (Method::DELETE, "/api/v1/recordings/1", Value::Null),
        (Method::GET, "/api/v1/users", Value::Null),
        (
            Method::POST,
            "/api/v1/users",
            json!({"user_name": "eve", "password": PASSWORD, "role": "admin"}),
        ),
    ] {
        let body = (!body.is_null()).then_some(body);
        let (status, _) = call(&url, method, path, Some(&token), body).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{path}");
    }
}

#[tokio::test]
async fn a_token_cannot_make_or_list_tokens_and_a_guest_has_none() {
    let url = secured_server().await;
    let admin = set_up_admin(&url).await;
    let (_, token) = make_token(&url, &admin, "writer", "edit_bookmarks").await;
    let (status, _) = call(
        &url,
        Method::POST,
        "/api/v1/tokens",
        Some(&token),
        Some(json!({"name": "wider", "scope": "read_only"})),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = call(&url, Method::GET, "/api/v1/tokens", Some(&token), None).await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    call(
        &url,
        Method::PUT,
        "/api/v1/guest-access",
        Some(&admin),
        Some(json!({"enabled": true})),
    )
    .await;
    let (status, _) = call(
        &url,
        Method::POST,
        "/api/v1/tokens",
        None,
        Some(json!({"name": "guest", "scope": "read_only"})),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn revoking_is_for_the_owner_only_and_ends_the_token() {
    let url = secured_server().await;
    let admin = set_up_admin(&url).await;
    let (_, ann) = add_user(&url, &admin, "ann").await;
    let (_, bob) = add_user(&url, &admin, "bob").await;
    let (id, token) = make_token(&url, &ann, "mine", "read_only").await;

    let path = format!("/api/v1/tokens/{id}");
    let (status, _) = call(&url, Method::DELETE, &path, Some(&bob), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = call(&url, Method::GET, "/api/v1/export", Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = call(&url, Method::DELETE, &path, Some(&ann), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = call(&url, Method::GET, "/api/v1/export", Some(&token), None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn a_token_stops_working_with_its_user() {
    let url = secured_server().await;
    let admin = set_up_admin(&url).await;
    let (ann_id, ann) = add_user(&url, &admin, "ann").await;
    let (_, token) = make_token(&url, &ann, "mine", "read_only").await;

    let path = format!("/api/v1/users/{ann_id}");
    call(
        &url,
        Method::PUT,
        &path,
        Some(&admin),
        Some(json!({"active": false})),
    )
    .await;
    let (status, _) = call(&url, Method::GET, "/api/v1/export", Some(&token), None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    call(
        &url,
        Method::PUT,
        &path,
        Some(&admin),
        Some(json!({"active": true})),
    )
    .await;
    let (status, _) = call(&url, Method::GET, "/api/v1/export", Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);

    call(&url, Method::DELETE, &path, Some(&admin), None).await;
    let (status, _) = call(&url, Method::GET, "/api/v1/export", Some(&token), None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn a_made_up_token_is_refused() {
    let url = secured_server().await;
    set_up_admin(&url).await;
    let (status, _) = call(&url, Method::GET, "/api/v1/export", Some("vxt_0000"), None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

/// Connects to the channel, sends `first` and returns the `type` of the messages until the
/// connection closes or `until` has been seen.
fn channel_types(url: &str, first: Value, until: &str) -> Vec<String> {
    let (sender, receiver) = mpsc::channel();
    let mut connection = ewebsock::ws_connect(
        format!("{}/api/v1/events", url.replace("http://", "ws://")),
        Options::default(),
        Box::new(move |event| {
            let _ = sender.send(event);
            std::ops::ControlFlow::Continue(())
        }),
    )
    .unwrap();
    connection.send(WsMessage::Text(first.to_string()));
    let mut types = Vec::new();
    while let Ok(event) = receiver.recv_timeout(Duration::from_secs(8)) {
        match event {
            WsEvent::Message(WsMessage::Text(text)) => {
                let message: Value = serde_json::from_str(&text).unwrap();
                let kind = message["type"].as_str().unwrap().to_owned();
                let done = kind == until;
                types.push(kind);
                if done {
                    break;
                }
            }
            WsEvent::Closed | WsEvent::Error(_) => break,
            _ => {}
        }
    }
    types
}

#[tokio::test]
async fn the_channel_accepts_a_token_as_it_accepts_a_login() {
    let url = secured_server().await;
    let admin = set_up_admin(&url).await;
    let (_, token) = make_token(&url, &admin, "watcher", "read_only").await;
    let accepted = {
        let url = url.clone();
        tokio::task::spawn_blocking(move || {
            channel_types(
                &url,
                json!({"type": "auth", "token": token, "guest": false}),
                "state",
            )
        })
        .await
        .unwrap()
    };
    assert_eq!(accepted, ["hello", "authenticated", "state"]);
}
