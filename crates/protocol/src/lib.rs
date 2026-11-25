//! Versioned wire messages for HTTP and the real-time channel. No system access.

mod accounts;
mod convert;

pub use accounts::{
    ApiTokenDto, ApiTokenListDto, CapabilityDto, CreatedApiTokenDto, CredentialsDto,
    GUEST_ACCESS_PATH, GuestAccessDto, IdentityDto, LOGIN_PATH, LOGOUT_PATH, ME_PATH,
    NewApiTokenDto, NewUserDto, PASSWORD_POLICY_PATH, PasswordPolicyDto, RoleDto, SETUP_PATH,
    STATUS_PATH, SessionDto, SetPasswordPolicyDto, StatusDto, TOKEN_PATH, TOKENS_PATH,
    TokenScopeDto, USER_PATH, USERS_PATH, UpdateUserDto, UserDto, capability_from_dto,
    capability_to_dto, password_policy_from_dto, password_policy_to_dto, role_from_dto,
    role_to_dto, token_scope_from_dto, token_scope_to_dto,
};

pub use convert::{
    EntryIssue, InvalidEntry, analysis_from_dto, analysis_to_dto, bookmarks_from_dto,
    bookmarks_to_dto, entry_path, imported_bookmarks_from_dto, intent_from_message,
    intent_to_message, label_from_dto, label_to_dto, library_entry_from_dto, library_entry_to_dto,
    new_bookmarks_from_dto, peaks_from_bytes, peaks_to_bytes, play_mode_from_dto, play_mode_to_dto,
    range_from_dto, range_to_dto, region_from_dto, region_to_dto, state_from_dto, state_to_dto,
    waveform_info_to_dto,
};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Why a request or an intent was refused. The `code` is stable and meant for
/// the interface to translate; `args` are the numbers its text needs, in a
/// fixed order per code; `message` is English, for logs and for codes the
/// interface does not know.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReasonDto {
    pub code: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<u64>,
    pub message: String,
}

impl From<&veltrunox_core::Reason> for ReasonDto {
    fn from(reason: &veltrunox_core::Reason) -> Self {
        Self {
            code: reason.code.clone(),
            args: reason.args.clone(),
            message: reason.message.clone(),
        }
    }
}

impl From<ReasonDto> for veltrunox_core::Reason {
    fn from(dto: ReasonDto) -> Self {
        Self {
            code: dto.code,
            args: dto.args,
            message: dto.message,
        }
    }
}

/// Path of the recording list endpoint.
pub const RECORDINGS_PATH: &str = "/api/v1/recordings";

/// Path of one recording (`PUT` renames it, `DELETE` removes it); `{id}` is the recording id.
pub const RECORDING_PATH: &str = "/api/v1/recordings/{id}";

/// Header of an upload (`POST` to `RECORDINGS_PATH`, the audio file as the raw body) with the
/// original file name, percent-encoded UTF-8.
pub const FILE_NAME_HEADER: &str = "x-file-name";

/// Optional header of an upload: an id the client made up (1 to 64 letters, digits, `-` or `_`).
/// The server reports the bytes it has received under this id as `upload_progress` events.
pub const UPLOAD_ID_HEADER: &str = "x-upload-id";

/// Longest accepted upload id.
pub const UPLOAD_ID_MAX_LENGTH: usize = 64;

/// Path to start the analysis of one recording again; `{id}` is the recording id.
pub const ANALYSIS_PATH: &str = "/api/v1/recordings/{id}/analysis";

/// Path of the waveform description of one recording (`WaveformInfoDto`); `{id}` is the recording id.
pub const WAVEFORM_PATH: &str = "/api/v1/recordings/{id}/waveform";

/// Path of the peaks of one waveform level as bytes (`peaks_from_bytes`); `{level}` counts from
/// the coarsest level (0).
pub const WAVEFORM_LEVEL_PATH: &str = "/api/v1/recordings/{id}/waveform/{level}";

/// Path of the real-time channel (WebSocket).
pub const EVENTS_PATH: &str = "/api/v1/events";

/// Version of the message format of the real-time channel.
pub const PROTOCOL_VERSION: u32 = 1;

/// Path of the bookmark collection of one recording; `{id}` is the recording id.
pub const BOOKMARKS_PATH: &str = "/api/v1/recordings/{id}/bookmarks";
pub const LABELS_PATH: &str = "/api/v1/recordings/{id}/labels";
pub const REGIONS_PATH: &str = "/api/v1/recordings/{id}/regions";
/// Path of the export of the whole library: every recording with details and bookmarks.
pub const EXPORT_PATH: &str = "/api/v1/export";
/// Path of the export of one recording; `{id}` is the recording id.
pub const RECORDING_EXPORT_PATH: &str = "/api/v1/recordings/{id}/export";
/// Path of one label or region; `{id}` is the bookmark id.
pub const LABEL_PATH: &str = "/api/v1/labels/{id}";
pub const REGION_PATH: &str = "/api/v1/regions/{id}";
/// Replaces the label `{id}` by a region (body: `UpdateRegionDto`, answer: `RegionDto`).
pub const LABEL_TO_REGION_PATH: &str = "/api/v1/labels/{id}/to-region";
/// Replaces the region `{id}` by a label (body: `UpdateLabelDto`, answer: `LabelDto`).
pub const REGION_TO_LABEL_PATH: &str = "/api/v1/regions/{id}/to-label";

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
    /// The region being played, if any.
    pub play_range: Option<RangeDto>,
    pub play_mode: PlayModeDto,
    pub scrubbing: bool,
    /// Output volume of the server in percent.
    pub volume_percent: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlayModeDto {
    ToEnd,
    ToRegionEnd,
    Loop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
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
    /// Answer to a valid `auth`: who the client is and what it may do.
    Authenticated(IdentityDto),
    /// The current state; sent first to a joining client and after every change.
    State(PlaybackStateDto),
    /// Answer to `ping`, for the client's clock estimate.
    Pong {
        client_time_ms: u64,
        server_time_ms: u64,
    },
    /// The intent of this client was refused; sent to the sender only.
    Rejected { reason: ReasonDto },
    /// Bookmarks of this recording changed; reload them.
    BookmarksChanged { recording_id: u64 },
    /// The analysis status or result of this recording changed; reload the library.
    AnalysisChanged { recording_id: u64 },
    /// Recordings were added, renamed or removed; reload the library.
    LibraryChanged,
    /// How many bytes of the upload with this id the server has received. Sent to every
    /// client (the id is the sender's secret), at most a few times per second.
    UploadProgress {
        upload_id: String,
        received_bytes: u64,
    },
    /// Whether the server can play sound; sent to a joining client and on change.
    /// Without an output device the shared state still runs, silently.
    AudioOutput { available: bool },
}

/// Messages a client sends over the real-time channel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessageDto {
    Ping {
        client_time_ms: u64,
    },
    /// Must be the first message once the server asks for sign-in: a token from
    /// `login`, or `guest: true` while guest access is on.
    Auth {
        token: Option<String>,
        guest: bool,
    },
    SelectRecording {
        recording_id: u64,
    },
    Play,
    Pause,
    Seek {
        position_ms: u64,
        scrubbing: bool,
    },
    SetSpeed {
        speed: f64,
    },
    SetTuning {
        tuning_hz: f64,
    },
    SetKey {
        key_shift_semitones: i8,
    },
    PlayRegion {
        range: RangeDto,
    },
    SetPlayMode {
        play_mode: PlayModeDto,
    },
    SetVolume {
        volume_percent: u8,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct LabelDto {
    pub id: u64,
    pub version: u64,
    pub name: String,
    pub position_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RegionDto {
    pub id: u64,
    pub version: u64,
    pub name: String,
    pub range: RangeDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BookmarksDto {
    pub labels: Vec<LabelDto>,
    pub regions: Vec<RegionDto>,
}

/// Body that replaces all bookmarks of a recording (`PUT` to `BOOKMARKS_PATH`; answer:
/// `BookmarksDto` with the new ids).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplaceBookmarksDto {
    #[serde(default)]
    pub labels: Vec<NewLabelDto>,
    #[serde(default)]
    pub regions: Vec<NewRegionDto>,
}

/// One recording with its details and bookmarks, as exchanged by export.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ExportedRecordingDto {
    pub recording: RecordingSummaryDto,
    pub bookmarks: BookmarksDto,
}

/// Version of the export format; it rises when the format changes incompatibly.
pub const EXPORT_FORMAT_VERSION: u32 = 1;

/// Recordings with details and bookmarks (`GET` of `EXPORT_PATH` or `RECORDING_EXPORT_PATH`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ExportDto {
    pub format_version: u32,
    pub recordings: Vec<ExportedRecordingDto>,
}

/// Path of the import of bookmarks (`POST`; JSON as `ImportDto` or CSV with the content type
/// `text/csv`; query `dry_run` and `overwrite`; answer `ImportResultDto`).
pub const IMPORT_PATH: &str = "/api/v1/import";
/// Path of the JSON schema of `ImportDto`.
pub const IMPORT_SCHEMA_PATH: &str = "/api/v1/import/schema";

/// Reference to a recording in an import; an export file carries more fields, which are ignored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RecordingRefDto {
    pub id: u64,
}

/// A label of an import. `id` and `version` as exported tie it to a stored label; without
/// them the label is new.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ImportLabelDto {
    #[serde(default)]
    pub id: Option<u64>,
    #[serde(default)]
    pub version: Option<u64>,
    pub name: String,
    pub position_ms: u64,
}

/// A region of an import; see `ImportLabelDto`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ImportRegionDto {
    #[serde(default)]
    pub id: Option<u64>,
    #[serde(default)]
    pub version: Option<u64>,
    pub name: String,
    pub range: RangeDto,
}

/// The bookmarks a recording is to have after the import.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ImportBookmarksDto {
    #[serde(default)]
    pub labels: Vec<ImportLabelDto>,
    #[serde(default)]
    pub regions: Vec<ImportRegionDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ImportRecordingDto {
    pub recording: RecordingRefDto,
    pub bookmarks: ImportBookmarksDto,
}

/// What an import contains: per recording the complete set of bookmarks it is to have. An
/// export file is a valid import.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ImportDto {
    #[serde(default = "export_format_version")]
    pub format_version: u32,
    pub recordings: Vec<ImportRecordingDto>,
}

/// The JSON schema of `ImportDto`, for people and programs that prepare an import.
pub fn import_schema() -> schemars::Schema {
    schemars::schema_for!(ImportDto)
}

fn export_format_version() -> u32 {
    EXPORT_FORMAT_VERSION
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportStatusDto {
    /// Applied, or, for a dry run, would be applied.
    Ok,
    /// An entry's stored original changed or is gone.
    Conflict,
    /// Entries are invalid; see `issues`.
    Invalid,
    UnknownRecording,
}

/// A problem with one entry of an import; `entry` reads like `labels[2]`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportIssueDto {
    pub entry: String,
    pub reason: ReasonDto,
}

/// The outcome for one recording; the counts are as in `ImportDiff` and zero unless the status is `ok`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportOutcomeDto {
    pub recording_id: u64,
    pub status: ImportStatusDto,
    pub added: u32,
    pub changed: u32,
    pub unchanged: u32,
    pub removed: u32,
    pub issues: Vec<ImportIssueDto>,
}

/// Answer to an import: one outcome per recording, in the order of the import. A recording that
/// is not `ok` is left as it was; the others are applied unless `dry_run`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportResultDto {
    pub dry_run: bool,
    pub recordings: Vec<ImportOutcomeDto>,
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum AnalysisProgressDto {
    None,
    Queued,
    Running {
        /// How far the analysis has come, 0 to 100.
        #[serde(default)]
        percent: u8,
    },
    Failed {
        /// Code of the reason (see `ReasonDto`); the interface words it.
        code: String,
    },
    Done,
}

/// One resolution of a waveform: block `n` covers source frames starting at
/// `n * frames_per_block`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WaveformLevelInfoDto {
    pub frames_per_block: u32,
    pub blocks: u32,
}

/// What a client needs to fetch and place the peaks; levels run from coarse to fine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WaveformInfoDto {
    pub length_ms: u64,
    pub sample_rate: u32,
    pub levels: Vec<WaveformLevelInfoDto>,
}

/// Answer to an upload; `existing` is true when the same audio was in the library already.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UploadedRecordingDto {
    pub id: u64,
    pub title: String,
    pub existing: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenameRecordingDto {
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RecordingSummaryDto {
    pub id: u64,
    pub title: String,
    /// Length of the audio; absent until the recording has been scanned.
    #[serde(default)]
    pub length_ms: Option<u64>,
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
                length_ms: Some(61_000),
                analysis_progress: AnalysisProgressDto::Failed {
                    code: "too_short".into(),
                },
                analysis: None,
            }],
        };
        let json = serde_json::to_string(&list).unwrap();
        assert_eq!(
            json,
            concat!(
                r#"{"recordings":[{"id":1,"title":"Intro","length_ms":61000,"#,
                r#""analysis_progress":{"state":"failed","code":"too_short"},"analysis":null}]}"#
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
            play_range: Some(RangeDto {
                start_ms: 1000,
                end_ms: 2000,
            }),
            play_mode: PlayModeDto::ToRegionEnd,
            scrubbing: false,
            volume_percent: 70,
        });
        let json = serde_json::to_string(&event).unwrap();
        assert_eq!(
            json,
            concat!(
                r#"{"type":"state","revision":3,"recording_id":2,"position_ms":1500,"#,
                r#""position_server_time_ms":90000,"running":true,"speed":0.5,"#,
                r#""tuning_hz":440.0,"key_shift_semitones":-2,"#,
                r#""play_range":{"start_ms":1000,"end_ms":2000},"play_mode":"to_region_end","#,
                r#""scrubbing":false,"volume_percent":70}"#
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
    fn a_running_analysis_carries_its_percentage() {
        let json = serde_json::to_string(&AnalysisProgressDto::Running { percent: 40 }).unwrap();
        assert_eq!(json, r#"{"state":"running","percent":40}"#);
        // An older server sends no percentage.
        assert_eq!(
            serde_json::from_str::<AnalysisProgressDto>(r#"{"state":"running"}"#).unwrap(),
            AnalysisProgressDto::Running { percent: 0 }
        );
    }

    #[test]
    fn upload_progress_event_has_a_stable_wire_format() {
        let event = ServerEventDto::UploadProgress {
            upload_id: "a-1".into(),
            received_bytes: 4096,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert_eq!(
            json,
            r#"{"type":"upload_progress","upload_id":"a-1","received_bytes":4096}"#
        );
        assert_eq!(
            serde_json::from_str::<ServerEventDto>(&json).unwrap(),
            event
        );
    }

    #[test]
    fn library_changed_event_has_a_stable_wire_format() {
        let json = serde_json::to_string(&ServerEventDto::LibraryChanged).unwrap();
        assert_eq!(json, r#"{"type":"library_changed"}"#);
        assert_eq!(
            serde_json::from_str::<ServerEventDto>(&json).unwrap(),
            ServerEventDto::LibraryChanged
        );
    }

    #[test]
    fn unknown_message_types_are_rejected() {
        assert!(serde_json::from_str::<ClientMessageDto>(r#"{"type":"explode"}"#).is_err());
    }

    #[test]
    fn a_reason_carries_code_numbers_and_message() {
        let reason = veltrunox_core::Reason::with_args(&"too long", "file_too_large", vec![200]);
        let dto = ReasonDto::from(&reason);
        assert_eq!(
            serde_json::to_string(&dto).unwrap(),
            r#"{"code":"file_too_large","args":[200],"message":"too long"}"#
        );
        let without_numbers: ReasonDto =
            serde_json::from_str(r#"{"code":"not_allowed","message":"not allowed"}"#).unwrap();
        assert!(without_numbers.args.is_empty());
        assert_eq!(veltrunox_core::Reason::from(dto), reason);
    }
}
