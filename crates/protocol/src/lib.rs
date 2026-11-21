//! Versioned wire messages for HTTP and the real-time channel. No system access.

mod convert;

pub use convert::{
    InvalidEntry, analysis_from_dto, analysis_to_dto, bookmarks_from_dto, bookmarks_to_dto,
    intent_from_message, intent_to_message, label_to_dto, library_entry_from_dto,
    library_entry_to_dto, range_from_dto, range_to_dto, region_to_dto, state_from_dto,
    state_to_dto,
};

use serde::{Deserialize, Serialize};

/// Path of the recording list endpoint.
pub const RECORDINGS_PATH: &str = "/api/v1/recordings";

/// Path to start the analysis of one recording again; `{id}` is the recording id.
pub const ANALYSIS_PATH: &str = "/api/v1/recordings/{id}/analysis";

/// Path of the real-time channel (WebSocket).
pub const EVENTS_PATH: &str = "/api/v1/events";

/// Version of the message format of the real-time channel.
pub const PROTOCOL_VERSION: u32 = 1;

/// Path of the bookmark collection of one recording; `{id}` is the recording id.
pub const BOOKMARKS_PATH: &str = "/api/v1/recordings/{id}/bookmarks";
pub const LABELS_PATH: &str = "/api/v1/recordings/{id}/labels";
pub const REGIONS_PATH: &str = "/api/v1/recordings/{id}/regions";
/// Path of one label or region; `{id}` is the bookmark id.
pub const LABEL_PATH: &str = "/api/v1/labels/{id}";
pub const REGION_PATH: &str = "/api/v1/regions/{id}";

/// The shared playback state, complete, as published to every client.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlaybackStateDto {
    pub revision: u64,
    pub recording_id: Option<u64>,
    pub position_ms: u64,
    /// Server time at which `position_ms` is valid.
    pub position_server_time_ms: u64,
    pub running: bool,
    pub speed: f64,
    pub tuning_hz: f64,
    pub key_shift_semitones: i8,
    pub loop_range: Option<RangeDto>,
    pub scrubbing: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RangeDto {
    pub start_ms: u64,
    pub end_ms: u64,
}

/// Messages the server pushes over the real-time channel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerEventDto {
    /// First message after the connection is established.
    Hello { protocol_version: u32 },
    /// The current state; sent first to a joining client and after every change.
    State(PlaybackStateDto),
    /// Answer to `ping`, for the client's clock estimate.
    Pong {
        client_time_ms: u64,
        server_time_ms: u64,
    },
    /// The intent of this client was refused; sent to the sender only.
    Rejected { reason: String },
    /// Bookmarks of this recording changed; reload them.
    BookmarksChanged { recording_id: u64 },
    /// The analysis status or result of this recording changed; reload the library.
    AnalysisChanged { recording_id: u64 },
    /// Whether the server can play sound; sent to a joining client and on change.
    /// Without an output device the shared state still runs, silently.
    AudioOutput { available: bool },
}

/// Messages a client sends over the real-time channel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessageDto {
    Ping { client_time_ms: u64 },
    SelectRecording { recording_id: u64 },
    Play,
    Pause,
    Seek { position_ms: u64, scrubbing: bool },
    SetSpeed { speed: f64 },
    SetTuning { tuning_hz: f64 },
    SetKey { key_shift_semitones: i8 },
    SetLoop { loop_range: Option<RangeDto> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabelDto {
    pub id: u64,
    pub version: u64,
    pub name: String,
    pub position_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegionDto {
    pub id: u64,
    pub version: u64,
    pub name: String,
    pub range: RangeDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BookmarksDto {
    pub labels: Vec<LabelDto>,
    pub regions: Vec<RegionDto>,
}

/// Body to create a label.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewLabelDto {
    pub name: String,
    pub position_ms: u64,
}

/// Body to change a label; `expected_version` is the version the client saw.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateLabelDto {
    pub expected_version: u64,
    pub name: String,
    pub position_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewRegionDto {
    pub name: String,
    pub range: RangeDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateRegionDto {
    pub expected_version: u64,
    pub name: String,
    pub range: RangeDto,
}

/// Query of a delete: the version the client saw.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExpectedVersionQuery {
    pub expected_version: u64,
}

/// Body of an HTTP conflict response (status 409).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConflictDto {
    pub current_version: u64,
}

/// Estimated values of a recording; the confidences are ratios of the best to the second-best score.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisResultDto {
    pub tuning_hz: f64,
    pub tuning_confidence: f64,
    /// Pitch class of the key's tonic, 0 = C … 11 = B.
    pub key_tonic: u8,
    pub key_minor: bool,
    pub key_confidence: f64,
    pub tempo_bpm: f64,
    pub tempo_confidence: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum AnalysisProgressDto {
    None,
    Queued,
    Running,
    Failed { reason: String },
    Done,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordingSummaryDto {
    pub id: u64,
    pub title: String,
    pub analysis_progress: AnalysisProgressDto,
    pub analysis: Option<AnalysisResultDto>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
                analysis_progress: AnalysisProgressDto::Failed {
                    reason: "too short".into(),
                },
                analysis: None,
            }],
        };
        let json = serde_json::to_string(&list).unwrap();
        assert_eq!(
            json,
            concat!(
                r#"{"recordings":[{"id":1,"title":"Intro","#,
                r#""analysis_progress":{"state":"failed","reason":"too short"},"analysis":null}]}"#
            )
        );
        assert_eq!(
            serde_json::from_str::<RecordingListDto>(&json).unwrap(),
            list
        );
    }

    #[test]
    fn state_event_has_a_stable_wire_format() {
        let event = ServerEventDto::State(PlaybackStateDto {
            revision: 3,
            recording_id: Some(2),
            position_ms: 1500,
            position_server_time_ms: 90_000,
            running: true,
            speed: 0.5,
            tuning_hz: 440.0,
            key_shift_semitones: -2,
            loop_range: Some(RangeDto {
                start_ms: 1000,
                end_ms: 2000,
            }),
            scrubbing: false,
        });
        let json = serde_json::to_string(&event).unwrap();
        assert_eq!(
            json,
            concat!(
                r#"{"type":"state","revision":3,"recording_id":2,"position_ms":1500,"#,
                r#""position_server_time_ms":90000,"running":true,"speed":0.5,"#,
                r#""tuning_hz":440.0,"key_shift_semitones":-2,"#,
                r#""loop_range":{"start_ms":1000,"end_ms":2000},"scrubbing":false}"#
            )
        );
        assert_eq!(
            serde_json::from_str::<ServerEventDto>(&json).unwrap(),
            event
        );
    }

    #[test]
    fn client_messages_are_tagged_by_type() {
        let seek = ClientMessageDto::Seek {
            position_ms: 42,
            scrubbing: true,
        };
        let json = serde_json::to_string(&seek).unwrap();
        assert_eq!(json, r#"{"type":"seek","position_ms":42,"scrubbing":true}"#);
        assert_eq!(
            serde_json::to_string(&ClientMessageDto::Play).unwrap(),
            r#"{"type":"play"}"#
        );
        assert_eq!(
            serde_json::from_str::<ClientMessageDto>(&json).unwrap(),
            seek
        );
    }

    #[test]
    fn audio_output_event_has_a_stable_wire_format() {
        let event = ServerEventDto::AudioOutput { available: false };
        let json = serde_json::to_string(&event).unwrap();
        assert_eq!(json, r#"{"type":"audio_output","available":false}"#);
        assert_eq!(
            serde_json::from_str::<ServerEventDto>(&json).unwrap(),
            event
        );
    }

    #[test]
    fn analysis_changed_event_has_a_stable_wire_format() {
        let event = ServerEventDto::AnalysisChanged { recording_id: 4 };
        let json = serde_json::to_string(&event).unwrap();
        assert_eq!(json, r#"{"type":"analysis_changed","recording_id":4}"#);
        assert_eq!(
            serde_json::from_str::<ServerEventDto>(&json).unwrap(),
            event
        );
    }

    #[test]
    fn unknown_message_types_are_rejected() {
        assert!(serde_json::from_str::<ClientMessageDto>(r#"{"type":"explode"}"#).is_err());
    }
}
