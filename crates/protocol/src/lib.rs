//! Versioned wire messages for HTTP and the real-time channel. No system access.

use serde::{Deserialize, Serialize};

/// Path of the recording list endpoint.
pub const RECORDINGS_PATH: &str = "/api/v1/recordings";

/// Path of the real-time channel (WebSocket).
pub const EVENTS_PATH: &str = "/api/v1/events";

/// Version of the message format of the real-time channel.
pub const PROTOCOL_VERSION: u32 = 1;

/// Messages the server pushes over the real-time channel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerEventDto {
    /// First message after the connection is established.
    Hello { protocol_version: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordingSummaryDto {
    pub id: u64,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordingListDto {
    pub recordings: Vec<RecordingSummaryDto>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_event_has_a_stable_wire_format() {
        let json = serde_json::to_string(&ServerEventDto::Hello {
            protocol_version: 1,
        })
        .unwrap();
        assert_eq!(json, r#"{"type":"hello","protocol_version":1}"#);
    }

    #[test]
    fn list_round_trips_through_json() {
        let list = RecordingListDto {
            recordings: vec![RecordingSummaryDto {
                id: 1,
                title: "Intro".into(),
            }],
        };
        let json = serde_json::to_string(&list).unwrap();
        assert_eq!(json, r#"{"recordings":[{"id":1,"title":"Intro"}]}"#);
        assert_eq!(
            serde_json::from_str::<RecordingListDto>(&json).unwrap(),
            list
        );
    }
}
