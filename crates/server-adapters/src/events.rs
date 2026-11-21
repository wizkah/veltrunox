use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Response,
};
use veltrunox_protocol::{PROTOCOL_VERSION, ServerEventDto};

/// Upgrades the request to the real-time channel.
pub async fn events(upgrade: WebSocketUpgrade) -> Response {
    upgrade.on_upgrade(serve_connection)
}

/// Greets the client, then keeps the connection open until the client closes it.
async fn serve_connection(mut socket: WebSocket) {
    let hello = ServerEventDto::Hello {
        protocol_version: PROTOCOL_VERSION,
    };
    let Ok(text) = serde_json::to_string(&hello) else {
        return;
    };
    if socket.send(Message::Text(text.into())).await.is_err() {
        return;
    }
    while let Some(Ok(message)) = socket.recv().await {
        if matches!(message, Message::Close(_)) {
            break;
        }
    }
}
