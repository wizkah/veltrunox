//! The real-time channel: greets, publishes state, takes intents.

use crate::state::AppState;
use axum::{
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::Response,
};
use std::time::Duration;
use tokio::{sync::broadcast::error::RecvError, time::Instant};
use veltrunox_protocol::{
    ClientMessageDto, PROTOCOL_VERSION, ServerEventDto, intent_from_message, state_to_dto,
};

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
    let mut pings = tokio::time::interval(PING_INTERVAL);
    let mut last_heard = Instant::now();

    let greeting = ServerEventDto::Hello {
        protocol_version: PROTOCOL_VERSION,
    };
    if send(&mut socket, &greeting).await.is_err() {
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
                    handle_incoming(&app, &mut socket, message).await
                }
                _ => Outcome::Stop,
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
            _ = pings.tick() => {
                let silent = last_heard.elapsed() > PING_INTERVAL * 2;
                if silent || socket.send(Message::Ping(Vec::new().into())).await.is_err() {
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

async fn handle_incoming(app: &AppState, socket: &mut WebSocket, message: Message) -> Outcome {
    match message {
        Message::Text(text) => match serde_json::from_str::<ClientMessageDto>(&text) {
            Ok(client_message) => handle_message(app, socket, client_message).await,
            Err(_) => reject(socket, "unrecognised message").await,
        },
        Message::Close(_) => Outcome::Stop,
        // Pongs and pings only prove the client is alive.
        Message::Ping(_) | Message::Pong(_) => Outcome::Continue,
        Message::Binary(_) => reject(socket, "binary messages are not supported").await,
    }
}

async fn handle_message(
    app: &AppState,
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
        Err(error) => return reject(socket, &error.to_string()).await,
    };
    let playback = app.playback.clone();
    // Applying an intent may write to the database, which blocks.
    match tokio::task::spawn_blocking(move || playback.apply(intent)).await {
        Ok(Ok(_)) => Outcome::Continue,
        Ok(Err(error)) => reject(socket, &error.to_string()).await,
        Err(_) => Outcome::Stop,
    }
}

async fn reject(socket: &mut WebSocket, reason: &str) -> Outcome {
    forward(
        socket,
        ServerEventDto::Rejected {
            reason: reason.to_owned(),
        },
    )
    .await
}
