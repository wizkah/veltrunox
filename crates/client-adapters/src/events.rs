//! Real-time channel client (WebSocket); works natively and on `wasm32`.

use ewebsock::{Options, WsEvent, WsMessage, WsSender};
use std::ops::ControlFlow;
use veltrunox_client_app::{LinkError, LinkEvent};
use veltrunox_protocol::{EVENTS_PATH, PROTOCOL_VERSION, ServerEventDto};

/// Keeps the real-time channel open; dropping it closes the connection.
pub struct EventConnection {
    _sender: WsSender,
}

/// Opens the channel and reports what happens on it to `on_event`.
pub fn connect_events(
    base_url: &str,
    on_event: impl Fn(LinkEvent) + Send + 'static,
) -> Result<EventConnection, LinkError> {
    let url = websocket_url(base_url);
    let sender = ewebsock::ws_connect(
        url,
        Options::default(),
        Box::new(move |event| {
            if let Some(link_event) = into_link_event(&event) {
                on_event(link_event);
            }
            ControlFlow::Continue(())
        }),
    )
    .map_err(LinkError::Unreachable)?;
    Ok(EventConnection { _sender: sender })
}

fn websocket_url(base_url: &str) -> String {
    let base = base_url.trim_end_matches('/');
    let base = base
        .strip_prefix("https://")
        .map(|rest| format!("wss://{rest}"))
        .or_else(|| {
            base.strip_prefix("http://")
                .map(|rest| format!("ws://{rest}"))
        })
        .unwrap_or_else(|| base.to_owned());
    format!("{base}{EVENTS_PATH}")
}

fn into_link_event(event: &WsEvent) -> Option<LinkEvent> {
    match event {
        WsEvent::Opened => None,
        WsEvent::Message(WsMessage::Text(text)) => greeting_from(text),
        WsEvent::Message(_) => None,
        WsEvent::Error(_) => Some(LinkEvent::Failed),
        WsEvent::Closed => Some(LinkEvent::Closed),
    }
}

/// A greeting counts only if it speaks the protocol version this client knows.
fn greeting_from(text: &str) -> Option<LinkEvent> {
    match serde_json::from_str(text) {
        Ok(ServerEventDto::Hello { protocol_version }) if protocol_version == PROTOCOL_VERSION => {
            Some(LinkEvent::Greeted)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_urls_become_websocket_urls() {
        assert_eq!(
            websocket_url("http://host:8080/"),
            "ws://host:8080/api/v1/events"
        );
        assert_eq!(websocket_url("https://host"), "wss://host/api/v1/events");
    }

    #[test]
    fn only_a_matching_greeting_counts() {
        let hello = r#"{"type":"hello","protocol_version":1}"#;
        let other_version = r#"{"type":"hello","protocol_version":99}"#;
        assert_eq!(greeting_from(hello), Some(LinkEvent::Greeted));
        assert_eq!(greeting_from(other_version), None);
        assert_eq!(greeting_from("garbage"), None);
    }
}
