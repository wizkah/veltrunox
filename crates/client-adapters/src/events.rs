//! Real-time channel client (WebSocket); works natively and on `wasm32`.

use crate::access::rights_from_dto;
use ewebsock::{Options, WsEvent, WsMessage, WsSender};
use std::ops::ControlFlow;
use veltrunox_client_app::{ChannelAccess, ChannelEvent, LinkError, LinkEvent, ServerEvent};
use veltrunox_core::{Intent, Millis, RecordingId};
use veltrunox_protocol::{
    ClientMessageDto, EVENTS_PATH, PROTOCOL_VERSION, ServerEventDto, intent_to_message,
    state_from_dto,
};

/// Keeps the real-time channel open; dropping it closes the connection.
pub struct EventConnection {
    sender: WsSender,
}

impl EventConnection {
    /// Sends an intent; the server answers with the new state or a rejection.
    pub fn send_intent(&mut self, intent: Intent) {
        self.send(&intent_to_message(intent));
    }

    /// Presents the client to a server that asks for sign-in; must be the first message.
    pub fn send_access(&mut self, access: &ChannelAccess) {
        match access {
            ChannelAccess::Open => {}
            ChannelAccess::Token(token) => self.send(&ClientMessageDto::Auth {
                token: Some(token.clone()),
                guest: false,
            }),
            ChannelAccess::Guest => self.send(&ClientMessageDto::Auth {
                token: None,
                guest: true,
            }),
        }
    }

    /// Asks the server for its time; the answer feeds the clock estimate.
    pub fn send_ping(&mut self, client_time: Millis) {
        self.send(&ClientMessageDto::Ping {
            client_time_ms: client_time.value(),
        });
    }

    fn send(&mut self, message: &ClientMessageDto) {
        // Serialising these plain messages cannot fail; a closed connection
        // is reported through the event callback, not here.
        if let Ok(text) = serde_json::to_string(message) {
            self.sender.send(WsMessage::Text(text));
        }
    }
}

/// Opens the channel and reports what happens on it to `on_event`.
pub fn connect_events(
    base_url: &str,
    on_event: impl Fn(ChannelEvent) + Send + 'static,
) -> Result<EventConnection, LinkError> {
    let url = websocket_url(base_url);
    let sender = ewebsock::ws_connect(
        url,
        Options::default(),
        Box::new(move |event| {
            if let Some(channel_event) = into_channel_event(&event) {
                on_event(channel_event);
            }
            ControlFlow::Continue(())
        }),
    )
    .map_err(LinkError::Unreachable)?;
    Ok(EventConnection { sender })
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

fn into_channel_event(event: &WsEvent) -> Option<ChannelEvent> {
    match event {
        WsEvent::Opened | WsEvent::Message(WsMessage::Binary(_)) => None,
        WsEvent::Message(WsMessage::Text(text)) => channel_event_from(text),
        WsEvent::Message(_) => None,
        WsEvent::Error(_) => Some(ChannelEvent::Link(LinkEvent::Failed)),
        WsEvent::Closed => Some(ChannelEvent::Link(LinkEvent::Closed)),
    }
}

/// A greeting counts only if it speaks the protocol version this client knows;
/// a message that cannot be understood means the channel cannot be trusted.
fn channel_event_from(text: &str) -> Option<ChannelEvent> {
    let Ok(event) = serde_json::from_str::<ServerEventDto>(text) else {
        return Some(ChannelEvent::Link(LinkEvent::Failed));
    };
    match event {
        ServerEventDto::Hello { protocol_version } if protocol_version == PROTOCOL_VERSION => {
            Some(ChannelEvent::Link(LinkEvent::Greeted))
        }
        ServerEventDto::Hello { .. } => Some(ChannelEvent::Link(LinkEvent::Failed)),
        ServerEventDto::State(dto) => Some(match state_from_dto(&dto) {
            Ok(state) => ChannelEvent::Server(ServerEvent::State(state)),
            Err(_) => ChannelEvent::Link(LinkEvent::Failed),
        }),
        ServerEventDto::Pong {
            client_time_ms,
            server_time_ms,
        } => Some(ChannelEvent::Server(ServerEvent::Pong {
            client_time: Millis::new(client_time_ms),
            server_time: Millis::new(server_time_ms),
        })),
        ServerEventDto::Authenticated(identity) => Some(ChannelEvent::Server(
            ServerEvent::Authenticated(rights_from_dto(&identity)),
        )),
        ServerEventDto::Rejected { reason } => {
            Some(ChannelEvent::Server(ServerEvent::Rejected(reason.into())))
        }
        ServerEventDto::BookmarksChanged { recording_id } => Some(ChannelEvent::Server(
            ServerEvent::BookmarksChanged(RecordingId::new(recording_id)),
        )),
        ServerEventDto::AnalysisChanged { recording_id } => Some(ChannelEvent::Server(
            ServerEvent::AnalysisChanged(RecordingId::new(recording_id)),
        )),
        ServerEventDto::LibraryChanged => Some(ChannelEvent::Server(ServerEvent::LibraryChanged)),
        ServerEventDto::AudioOutput { available } => {
            Some(ChannelEvent::Server(ServerEvent::AudioOutput(available)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use veltrunox_core::Reason;

    #[test]
    fn analysis_notices_become_server_events() {
        assert_eq!(
            channel_event_from(r#"{"type":"analysis_changed","recording_id":6}"#),
            Some(ChannelEvent::Server(ServerEvent::AnalysisChanged(
                RecordingId::new(6)
            )))
        );
    }

    #[test]
    fn library_notices_become_server_events() {
        assert_eq!(
            channel_event_from(r#"{"type":"library_changed"}"#),
            Some(ChannelEvent::Server(ServerEvent::LibraryChanged))
        );
    }

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
        assert_eq!(
            channel_event_from(hello),
            Some(ChannelEvent::Link(LinkEvent::Greeted))
        );
        assert_eq!(
            channel_event_from(other_version),
            Some(ChannelEvent::Link(LinkEvent::Failed))
        );
        assert_eq!(
            channel_event_from("garbage"),
            Some(ChannelEvent::Link(LinkEvent::Failed))
        );
    }

    #[test]
    fn notices_and_rejections_become_server_events() {
        assert_eq!(
            channel_event_from(r#"{"type":"bookmarks_changed","recording_id":4}"#),
            Some(ChannelEvent::Server(ServerEvent::BookmarksChanged(
                RecordingId::new(4)
            )))
        );
        assert_eq!(
            channel_event_from(r#"{"type":"rejected","reason":{"code":"nope","message":"nope"}}"#),
            Some(ChannelEvent::Server(ServerEvent::Rejected(Reason::coded(
                "nope", "nope"
            ))))
        );
    }

    #[test]
    fn a_state_with_invalid_values_fails_the_channel() {
        let bad = r#"{"type":"state","revision":1,"recording_id":null,"position_ms":0,"position_server_time_ms":0,"running":false,"speed":9.0,"tuning_hz":440.0,"key_shift_semitones":0,"play_range":null,"play_mode":"to_end","scrubbing":false}"#;
        assert_eq!(
            channel_event_from(bad),
            Some(ChannelEvent::Link(LinkEvent::Failed))
        );
    }
}
