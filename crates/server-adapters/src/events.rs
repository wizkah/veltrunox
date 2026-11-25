//! The real-time channel: greets, publishes state, takes intents.

use crate::{
    access::{identify, open_identity},
    accounts_http::identity_to_dto,
    state::AppState,
};
use axum::{
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::Response,
};
use std::time::Duration;
use tokio::{sync::broadcast::error::RecvError, time::Instant};
use veltrunox_core::{Capability, Reason, Refusal};
use veltrunox_protocol::{
    ClientMessageDto, PROTOCOL_VERSION, ReasonDto, ServerEventDto, intent_from_message,
    state_to_dto,
};
use veltrunox_server_app::Identity;

/// A client must sign in within this time after the greeting.
const AUTH_TIMEOUT: Duration = Duration::from_secs(5);

/// The server pings this often; a client silent for two intervals is gone.
const PING_INTERVAL: Duration = Duration::from_secs(15);

/// Upgrades the request to the real-time channel.
pub async fn events(State(app): State<AppState>, upgrade: WebSocketUpgrade) -> Response {
    upgrade.on_upgrade(move |socket| serve_connection(app, socket))
}

enum Outcome {
    Continue,
    Stop,
}

async fn serve_connection(app: AppState, mut socket: WebSocket) {
    let mut states = app.hub.subscribe_state();
    let mut audio_output = app.hub.subscribe_audio_output();
    let mut bookmark_notices = app.hub.subscribe_bookmarks();
    let mut analysis_notices = app.hub.subscribe_analysis();
    let mut library_notices = app.hub.subscribe_library();
    let mut upload_notices = app.hub.subscribe_uploads();
    let mut pings = tokio::time::interval(PING_INTERVAL);
    let mut last_heard = Instant::now();

    let greeting = ServerEventDto::Hello {
        protocol_version: PROTOCOL_VERSION,
    };
    if send(&mut socket, &greeting).await.is_err() {
        return;
    }
    let Some(identity) = authenticate(&app, &mut socket).await else {
        return;
    };
    if send(
        &mut socket,
        &ServerEventDto::Authenticated(identity_to_dto(&identity)),
    )
    .await
    .is_err()
    {
        return;
    }
    // A joining client gets the current state first; no separate catch-up exists.
    let current = *states.borrow_and_update();
    if send(&mut socket, &ServerEventDto::State(state_to_dto(&current)))
        .await
        .is_err()
    {
        return;
    }

    let available = *audio_output.borrow_and_update();
    if send(&mut socket, &ServerEventDto::AudioOutput { available })
        .await
        .is_err()
    {
        return;
    }

    loop {
        let outcome = tokio::select! {
            incoming = socket.recv() => match incoming {
                Some(Ok(message)) => {
                    last_heard = Instant::now();
                    handle_incoming(&app, &identity, &mut socket, message).await
                }
                Some(Err(error)) => {
                    eprintln!("event channel closed: receive failed: {error}");
                    Outcome::Stop
                }
                None => {
                    eprintln!("event channel closed: client disconnected");
                    Outcome::Stop
                }
            },
            changed = states.changed() => match changed {
                Ok(()) => {
                    let state = *states.borrow_and_update();
                    forward(&mut socket, ServerEventDto::State(state_to_dto(&state))).await
                }
                Err(_) => Outcome::Stop,
            },
            changed = audio_output.changed() => match changed {
                Ok(()) => {
                    let available = *audio_output.borrow_and_update();
                    forward(&mut socket, ServerEventDto::AudioOutput { available }).await
                }
                Err(_) => Outcome::Stop,
            },
            notice = bookmark_notices.recv() => match notice {
                Ok(recording) => {
                    forward(&mut socket, ServerEventDto::BookmarksChanged {
                        recording_id: recording.value(),
                    }).await
                }
                // Missed notices only mean a reload is due; the next one triggers it.
                Err(RecvError::Lagged(_)) => Outcome::Continue,
                Err(RecvError::Closed) => Outcome::Stop,
            },
            notice = analysis_notices.recv() => match notice {
                Ok(recording) => {
                    forward(&mut socket, ServerEventDto::AnalysisChanged {
                        recording_id: recording.value(),
                    }).await
                }
                Err(RecvError::Lagged(_)) => Outcome::Continue,
                Err(RecvError::Closed) => Outcome::Stop,
            },
            notice = library_notices.recv() => match notice {
                Ok(()) | Err(RecvError::Lagged(_)) => {
                    forward(&mut socket, ServerEventDto::LibraryChanged).await
                }
                Err(RecvError::Closed) => Outcome::Stop,
            },
            notice = upload_notices.recv() => match notice {
                Ok(progress) => {
                    forward(&mut socket, ServerEventDto::UploadProgress {
                        upload_id: progress.upload_id,
                        received_bytes: progress.received_bytes,
                    }).await
                }
                // A later notice carries a larger count anyway.
                Err(RecvError::Lagged(_)) => Outcome::Continue,
                Err(RecvError::Closed) => Outcome::Stop,
            },
            _ = pings.tick() => {
                let silent = last_heard.elapsed() > PING_INTERVAL * 2;
                if silent {
                    eprintln!(
                        "event channel closed: client silent for {} s",
                        last_heard.elapsed().as_secs()
                    );
                    Outcome::Stop
                } else if socket.send(Message::Ping(Vec::new().into())).await.is_err() {
                    eprintln!("event channel closed: ping could not be sent");
                    Outcome::Stop
                } else {
                    Outcome::Continue
                }
            }
        };
        if matches!(outcome, Outcome::Stop) {
            break;
        }
    }
}

async fn forward(socket: &mut WebSocket, event: ServerEventDto) -> Outcome {
    match send(socket, &event).await {
        Ok(()) => Outcome::Continue,
        Err(()) => Outcome::Stop,
    }
}

async fn send(socket: &mut WebSocket, event: &ServerEventDto) -> Result<(), ()> {
    let text = serde_json::to_string(event).map_err(|_| ())?;
    socket
        .send(Message::Text(text.into()))
        .await
        .map_err(|_| ())
}

async fn handle_incoming(
    app: &AppState,
    identity: &Identity,
    socket: &mut WebSocket,
    message: Message,
) -> Outcome {
    match message {
        Message::Text(text) => match serde_json::from_str::<ClientMessageDto>(&text) {
            Ok(client_message) => handle_message(app, identity, socket, client_message).await,
            Err(_) => {
                reject(
                    socket,
                    Reason::coded("message_unrecognised", "unrecognised message"),
                )
                .await
            }
        },
        Message::Close(_) => {
            eprintln!("event channel closed: client sent close");
            Outcome::Stop
        }
        // Pongs and pings only prove the client is alive.
        Message::Ping(_) | Message::Pong(_) => Outcome::Continue,
        Message::Binary(_) => {
            reject(
                socket,
                Reason::coded("binary_unsupported", "binary messages are not supported"),
            )
            .await
        }
    }
}

async fn handle_message(
    app: &AppState,
    identity: &Identity,
    socket: &mut WebSocket,
    message: ClientMessageDto,
) -> Outcome {
    if let ClientMessageDto::Ping { client_time_ms } = message {
        let pong = ServerEventDto::Pong {
            client_time_ms,
            server_time_ms: app.clock.now().value(),
        };
        return forward(socket, pong).await;
    }
    let intent = match intent_from_message(&message) {
        Ok(Some(intent)) => intent,
        Ok(None) => return Outcome::Continue,
        Err(error) => return reject(socket, error.reason()).await,
    };
    if !identity.allows(Capability::PlayControl) {
        return reject(socket, Reason::coded("not_allowed", "not allowed")).await;
    }
    let playback = app.playback.clone();
    // Applying an intent may write to the database, which blocks.
    match tokio::task::spawn_blocking(move || playback.apply(intent)).await {
        Ok(Ok(_)) => Outcome::Continue,
        Ok(Err(error)) => reject(socket, error.reason()).await,
        Err(_) => Outcome::Stop,
    }
}

async fn reject(socket: &mut WebSocket, reason: Reason) -> Outcome {
    forward(
        socket,
        ServerEventDto::Rejected {
            reason: ReasonDto::from(&reason),
        },
    )
    .await
}

/// Without accounts everybody is let in. Otherwise the first message must be `auth` with a
/// valid token or guest access; anything else, or silence, ends the connection.
async fn authenticate(app: &AppState, socket: &mut WebSocket) -> Option<Identity> {
    let Some(accounts) = app.accounts.clone() else {
        return Some(open_identity());
    };
    let api_tokens = app.api_tokens.clone();
    let first = tokio::time::timeout(AUTH_TIMEOUT, async {
        loop {
            match socket.recv().await {
                Some(Ok(Message::Text(text))) => return Some(text),
                Some(Ok(Message::Ping(_) | Message::Pong(_))) => {}
                _ => return None,
            }
        }
    })
    .await
    .ok()
    .flatten();
    let found = match first.and_then(|text| serde_json::from_str::<ClientMessageDto>(&text).ok()) {
        Some(ClientMessageDto::Auth { token, guest }) => {
            let resolved = tokio::task::spawn_blocking(move || match (token, guest) {
                (Some(token), _) => identify(&accounts, api_tokens.as_deref(), &token)
                    .ok()
                    .flatten(),
                (None, true) => accounts.guest().ok(),
                (None, false) => None,
            })
            .await
            .ok()
            .flatten();
            resolved.ok_or(Reason::coded("sign_in_required", "sign-in required"))
        }
        _ => Err(Reason::coded(
            "auth_expected",
            "the first message must be auth",
        )),
    };
    match found {
        Ok(identity) => Some(identity),
        Err(reason) => {
            let _ = send(
                socket,
                &ServerEventDto::Rejected {
                    reason: ReasonDto::from(&reason),
                },
            )
            .await;
            None
        }
    }
}
