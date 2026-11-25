//! Sign-in, roles and guest access, checked through the real endpoints.

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

/// Creates a user as the admin and signs them in.
async fn user_token(url: &str, admin: &str, name: &str, role: &str) -> String {
    let (status, _) = call(
        url,
        Method::POST,
        "/api/v1/users",
        Some(admin),
        Some(json!({"user_name": name, "password": PASSWORD, "role": role})),
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
    session["token"].as_str().unwrap().to_owned()
}

#[tokio::test]
async fn first_start_sets_up_one_admin_without_a_default_password() {
    let url = secured_server().await;
    let (_, status) = call(&url, Method::GET, "/api/v1/status", None, None).await;
    assert_eq!(status["setup_needed"], true);
    assert_eq!(status["guest_enabled"], false);

    let (weak, _) = call(
        &url,
        Method::POST,
        "/api/v1/setup",
        None,
        Some(json!({"user_name": "root", "password": "short"})),
    )
    .await;
    assert_eq!(weak, StatusCode::BAD_REQUEST);

    set_up_admin(&url).await;
    let (_, status) = call(&url, Method::GET, "/api/v1/status", None, None).await;
    assert_eq!(status["setup_needed"], false);
    let (again, _) = call(
        &url,
        Method::POST,
        "/api/v1/setup",
        None,
        Some(json!({"user_name": "second", "password": PASSWORD})),
    )
    .await;
    assert_eq!(again, StatusCode::CONFLICT);
}

#[tokio::test]
async fn nothing_works_without_signing_in() {
    let url = secured_server().await;
    set_up_admin(&url).await;
    for (method, path) in [
        (Method::GET, "/api/v1/recordings"),
        (Method::GET, "/api/v1/recordings/1/bookmarks"),
        (Method::GET, "/api/v1/recordings/1/waveform"),
        (Method::POST, "/api/v1/recordings/1/analysis"),
        (Method::GET, "/api/v1/me"),
        (Method::GET, "/api/v1/users"),
        (Method::PUT, "/api/v1/guest-access"),
    ] {
        let (status, _) = call(&url, method.clone(), path, None, Some(json!({}))).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{method} {path}");
    }
    let (status, _) = call(&url, Method::GET, "/api/v1/recordings", Some("bogus"), None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_rejects_wrong_passwords_and_logout_ends_the_session() {
    let url = secured_server().await;
    let admin = set_up_admin(&url).await;
    let (wrong, _) = call(
        &url,
        Method::POST,
        "/api/v1/login",
        None,
        Some(json!({"user_name": "root", "password": "not the password"})),
    )
    .await;
    assert_eq!(wrong, StatusCode::UNAUTHORIZED);
    let (unknown, _) = call(
        &url,
        Method::POST,
        "/api/v1/login",
        None,
        Some(json!({"user_name": "nobody", "password": PASSWORD})),
    )
    .await;
    assert_eq!(unknown, StatusCode::UNAUTHORIZED);

    let (_, me) = call(&url, Method::GET, "/api/v1/me", Some(&admin), None).await;
    assert_eq!(me["role"], "admin");
    assert!(
        me["capabilities"]
            .as_array()
            .unwrap()
            .contains(&json!("manage_users"))
    );
    let (out, _) = call(&url, Method::POST, "/api/v1/logout", Some(&admin), None).await;
    assert_eq!(out, StatusCode::NO_CONTENT);
    let (after, _) = call(&url, Method::GET, "/api/v1/me", Some(&admin), None).await;
    assert_eq!(after, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn repeated_failures_delay_even_the_right_password() {
    let url = secured_server().await;
    set_up_admin(&url).await;
    let attempt = |password: &'static str| {
        let url = url.clone();
        async move {
            call(
                &url,
                Method::POST,
                "/api/v1/login",
                None,
                Some(json!({"user_name": "root", "password": password})),
            )
            .await
            .0
        }
    };
    for _ in 0..3 {
        assert_eq!(attempt("wrong wrong wrong").await, StatusCode::UNAUTHORIZED);
    }
    assert_eq!(attempt("wrong wrong wrong").await, StatusCode::UNAUTHORIZED);
    assert_eq!(attempt(PASSWORD).await, StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn a_user_cannot_manage_users_or_recordings_even_by_direct_request() {
    let url = secured_server().await;
    let admin = set_up_admin(&url).await;
    let user = user_token(&url, &admin, "ann", "user").await;

    let (status, _) = call(&url, Method::GET, "/api/v1/recordings", Some(&user), None).await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = call(
        &url,
        Method::POST,
        "/api/v1/recordings/1/labels",
        Some(&user),
        Some(json!({"name": "Chorus", "position_ms": 1000})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    for (method, path, body) in [
        (Method::GET, "/api/v1/users", json!({})),
        (
            Method::POST,
            "/api/v1/users",
            json!({"user_name": "mallory", "password": PASSWORD, "role": "admin"}),
        ),
        (Method::PUT, "/api/v1/users/1", json!({"role": "user"})),
        (Method::DELETE, "/api/v1/users/1", json!({})),
        (
            Method::PUT,
            "/api/v1/guest-access",
            json!({"enabled": true}),
        ),
        (Method::POST, "/api/v1/recordings/1/analysis", json!({})),
    ] {
        let (status, _) = call(&url, method.clone(), path, Some(&user), Some(body)).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{method} {path}");
    }
    // The admin account is untouched.
    let (_, users) = call(&url, Method::GET, "/api/v1/users", Some(&admin), None).await;
    assert_eq!(users.as_array().unwrap().len(), 2);
    assert_eq!(users[0]["role"], "admin");
}

#[tokio::test]
async fn the_admin_manages_users_and_the_last_admin_stays() {
    let url = secured_server().await;
    let admin = set_up_admin(&url).await;
    let user = user_token(&url, &admin, "ann", "user").await;

    let (status, _) = call(
        &url,
        Method::PUT,
        "/api/v1/users/2",
        Some(&admin),
        Some(json!({"active": false})),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (gone, _) = call(&url, Method::GET, "/api/v1/me", Some(&user), None).await;
    assert_eq!(gone, StatusCode::UNAUTHORIZED);

    for body in [json!({"role": "user"}), json!({"active": false})] {
        let (status, _) = call(
            &url,
            Method::PUT,
            "/api/v1/users/1",
            Some(&admin),
            Some(body),
        )
        .await;
        assert_eq!(status, StatusCode::CONFLICT);
    }
    let (status, _) = call(&url, Method::DELETE, "/api/v1/users/1", Some(&admin), None).await;
    assert_eq!(status, StatusCode::CONFLICT);
    let (status, _) = call(&url, Method::DELETE, "/api/v1/users/2", Some(&admin), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn guest_access_is_off_until_the_admin_switches_it_on() {
    let url = secured_server().await;
    let admin = set_up_admin(&url).await;
    let (status, _) = call(&url, Method::GET, "/api/v1/recordings", None, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let (status, _) = call(
        &url,
        Method::PUT,
        "/api/v1/guest-access",
        Some(&admin),
        Some(json!({"enabled": true})),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, public) = call(&url, Method::GET, "/api/v1/status", None, None).await;
    assert_eq!(public["guest_enabled"], true);

    let (status, _) = call(&url, Method::GET, "/api/v1/recordings", None, None).await;
    assert_eq!(status, StatusCode::OK);
    let (_, me) = call(&url, Method::GET, "/api/v1/me", None, None).await;
    assert_eq!(me["role"], "guest");
    assert_eq!(me["user_id"], Value::Null);
    // A guest holds the rights of a user, not of an admin.
    let (status, _) = call(&url, Method::GET, "/api/v1/users", None, None).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = call(&url, Method::GET, "/api/v1/recordings", Some("bogus"), None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    call(
        &url,
        Method::PUT,
        "/api/v1/guest-access",
        Some(&admin),
        Some(json!({"enabled": false})),
    )
    .await;
    let (status, _) = call(&url, Method::GET, "/api/v1/recordings", None, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

/// Connects to the channel, sends `first` if given and returns the `type` of the
/// messages until the connection closes or `until` has been seen.
fn channel_types(url: &str, first: Option<Value>, until: &str) -> Vec<String> {
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
    if let Some(first) = first {
        connection.send(WsMessage::Text(first.to_string()));
    }
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
async fn the_channel_needs_a_valid_token_or_guest_access() {
    let url = secured_server().await;
    let admin = set_up_admin(&url).await;
    let probe = |first: Option<Value>, until: &'static str| {
        let url = url.clone();
        tokio::task::spawn_blocking(move || channel_types(&url, first, until))
    };

    let accepted = probe(
        Some(json!({"type": "auth", "token": admin, "guest": false})),
        "state",
    )
    .await
    .unwrap();
    assert_eq!(accepted, ["hello", "authenticated", "state"]);

    let wrong = probe(
        Some(json!({"type": "auth", "token": "bogus", "guest": false})),
        "state",
    )
    .await
    .unwrap();
    assert_eq!(wrong, ["hello", "rejected"]);

    let not_auth = probe(Some(json!({"type": "play"})), "state").await.unwrap();
    assert_eq!(not_auth, ["hello", "rejected"]);

    let guest_off = probe(
        Some(json!({"type": "auth", "token": null, "guest": true})),
        "state",
    )
    .await
    .unwrap();
    assert_eq!(guest_off, ["hello", "rejected"]);

    call(
        &url,
        Method::PUT,
        "/api/v1/guest-access",
        Some(&admin),
        Some(json!({"enabled": true})),
    )
    .await;
    let guest = probe(
        Some(json!({"type": "auth", "token": null, "guest": true})),
        "state",
    )
    .await
    .unwrap();
    assert_eq!(guest, ["hello", "authenticated", "state"]);
}

#[tokio::test]
async fn the_admin_chooses_the_password_rules_and_nobody_else_does() {
    let url = secured_server().await;
    let admin = set_up_admin(&url).await;
    let member = user_token(&url, &admin, "member", "user").await;
    let (_, status) = call(&url, Method::GET, "/api/v1/status", None, None).await;
    assert_eq!(status["password_policy"], "standard");

    let strict = json!({"policy": "strict"});
    for token in [None, Some(member.as_str())] {
        let (status, _) = call(
            &url,
            Method::PUT,
            "/api/v1/password-policy",
            token,
            Some(strict.clone()),
        )
        .await;
        assert!(matches!(
            status,
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
        ));
    }
    let (status, _) = call(
        &url,
        Method::PUT,
        "/api/v1/password-policy",
        Some(&admin),
        Some(strict),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, status) = call(&url, Method::GET, "/api/v1/status", None, None).await;
    assert_eq!(status["password_policy"], "strict");

    let weak = json!({"user_name": "weak", "password": "simple", "role": "user"});
    let (status, _) = call(
        &url,
        Method::POST,
        "/api/v1/users",
        Some(&admin),
        Some(weak),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let strong = json!({"user_name": "strong", "password": "Str0ng-Passw", "role": "user"});
    let (status, _) = call(
        &url,
        Method::POST,
        "/api/v1/users",
        Some(&admin),
        Some(strong),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
}

#[tokio::test]
async fn an_account_changes_its_own_password_and_is_signed_out() {
    let url = secured_server().await;
    let admin = set_up_admin(&url).await;
    let ann = user_token(&url, &admin, "ann", "user").await;
    let path = "/api/v1/me/password";

    let (wrong, body) = call(
        &url,
        Method::PUT,
        path,
        Some(&ann),
        Some(json!({"current_password": "not it", "new_password": "another secret"})),
    )
    .await;
    assert_eq!(wrong, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "wrong_current_password");

    let (weak, _) = call(
        &url,
        Method::PUT,
        path,
        Some(&ann),
        Some(json!({"current_password": PASSWORD, "new_password": "x"})),
    )
    .await;
    assert_eq!(weak, StatusCode::BAD_REQUEST);

    let (guest, _) = call(
        &url,
        Method::PUT,
        path,
        None,
        Some(json!({"current_password": PASSWORD, "new_password": "another secret"})),
    )
    .await;
    assert!(guest == StatusCode::FORBIDDEN || guest == StatusCode::UNAUTHORIZED);

    let (done, _) = call(
        &url,
        Method::PUT,
        path,
        Some(&ann),
        Some(json!({"current_password": PASSWORD, "new_password": "another secret"})),
    )
    .await;
    assert_eq!(done, StatusCode::NO_CONTENT);
    let (after, _) = call(&url, Method::GET, "/api/v1/me", Some(&ann), None).await;
    assert_eq!(after, StatusCode::UNAUTHORIZED);
    let (old, _) = call(
        &url,
        Method::POST,
        "/api/v1/login",
        None,
        Some(json!({"user_name": "ann", "password": PASSWORD})),
    )
    .await;
    assert_eq!(old, StatusCode::UNAUTHORIZED);
    let (new, _) = call(
        &url,
        Method::POST,
        "/api/v1/login",
        None,
        Some(json!({"user_name": "ann", "password": "another secret"})),
    )
    .await;
    assert_eq!(new, StatusCode::OK);
}
