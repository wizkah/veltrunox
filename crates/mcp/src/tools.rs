//! The tools the LLM can call and how each one is carried out against the server.

use crate::backend::{Backend, Reply};
use serde::Deserialize;
use serde_json::{Value, json};
use veltrunox_protocol::{
    EXPORT_FORMAT_VERSION, IMPORT_PATH, ImportBookmarksDto, ImportDto, ImportLabelDto,
    ImportRecordingDto, ImportRegionDto, ImportResultDto, ImportStatusDto, RECORDINGS_PATH,
    RecordingRefDto,
};

/// What a tool call came to; an error is reported to the LLM as text, not as a protocol error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolOutput {
    pub text: String,
    pub is_error: bool,
}

impl ToolOutput {
    fn ok(text: String) -> Self {
        Self {
            text,
            is_error: false,
        }
    }

    fn error(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            is_error: true,
        }
    }
}

const LIST_RECORDINGS: &str = "list_recordings";
const GET_RECORDING: &str = "get_recording";
const GET_BOOKMARKS: &str = "get_bookmarks";
const SET_BOOKMARKS: &str = "set_bookmarks";

fn recording_id_property() -> Value {
    json!({"type": "integer", "minimum": 0, "description": "Id of the recording, from list_recordings."})
}

/// The tool definitions for `tools/list`.
pub fn definitions() -> Value {
    json!([
        {
            "name": LIST_RECORDINGS,
            "description": "Lists the recordings of the library with id, title, length in milliseconds and, where analysed, tuning, key and tempo.",
            "inputSchema": {"type": "object", "properties": {}}
        },
        {
            "name": GET_RECORDING,
            "description": "One recording with its details and its bookmarks (labels and regions). Times are in milliseconds from the start.",
            "inputSchema": {
                "type": "object",
                "properties": {"recording_id": recording_id_property()},
                "required": ["recording_id"]
            }
        },
        {
            "name": GET_BOOKMARKS,
            "description": "The labels (a point in time) and regions (a span) of one recording, each with id and version.",
            "inputSchema": {
                "type": "object",
                "properties": {"recording_id": recording_id_property()},
                "required": ["recording_id"]
            }
        },
        {
            "name": SET_BOOKMARKS,
            "description": "Gives a recording exactly the labels and regions passed: everything stored and not passed is removed. Pass id and version of entries taken over from get_bookmarks, so that a change made by someone else in the meantime is reported as a conflict instead of being overwritten; leave them out for new entries. Names must not be empty, a region must end after it starts, and nothing may lie beyond the end of the recording. Use dry_run first to see what would change.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "recording_id": recording_id_property(),
                    "labels": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "id": {"type": "integer"},
                                "version": {"type": "integer"},
                                "name": {"type": "string"},
                                "position_ms": {"type": "integer", "minimum": 0}
                            },
                            "required": ["name", "position_ms"]
                        }
                    },
                    "regions": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "id": {"type": "integer"},
                                "version": {"type": "integer"},
                                "name": {"type": "string"},
                                "range": {
                                    "type": "object",
                                    "properties": {
                                        "start_ms": {"type": "integer", "minimum": 0},
                                        "end_ms": {"type": "integer", "minimum": 0}
                                    },
                                    "required": ["start_ms", "end_ms"]
                                }
                            },
                            "required": ["name", "range"]
                        }
                    },
                    "dry_run": {"type": "boolean", "description": "Only report what would change; write nothing."}
                },
                "required": ["recording_id"]
            }
        }
    ])
}

/// Carries out one tool call. Unknown tools and bad arguments come back as error output.
pub fn call(backend: &dyn Backend, name: &str, arguments: &Value) -> ToolOutput {
    match name {
        LIST_RECORDINGS => read(backend, RECORDINGS_PATH),
        GET_RECORDING => with_recording(arguments, |id| {
            read(backend, &format!("{RECORDINGS_PATH}/{id}/export"))
        }),
        GET_BOOKMARKS => with_recording(arguments, |id| {
            read(backend, &format!("{RECORDINGS_PATH}/{id}/bookmarks"))
        }),
        SET_BOOKMARKS => set_bookmarks(backend, arguments),
        other => ToolOutput::error(format!("unknown tool: {other}")),
    }
}

fn with_recording(arguments: &Value, run: impl FnOnce(u64) -> ToolOutput) -> ToolOutput {
    match arguments.get("recording_id").and_then(Value::as_u64) {
        Some(id) => run(id),
        None => ToolOutput::error("recording_id is required and must be a whole number"),
    }
}

fn read(backend: &dyn Backend, path: &str) -> ToolOutput {
    match backend.get(path) {
        Ok(reply) => reply_output(&reply),
        Err(error) => ToolOutput::error(error.to_string()),
    }
}

/// The server's answer as text; a refusal carries the server's reason.
fn reply_output(reply: &Reply) -> ToolOutput {
    if reply.is_success() {
        ToolOutput::ok(reply.body.clone())
    } else {
        ToolOutput::error(format!(
            "the server answered {}: {}",
            reply.status, reply.body
        ))
    }
}

#[derive(Deserialize)]
struct SetBookmarksArguments {
    recording_id: u64,
    #[serde(default)]
    labels: Vec<ImportLabelDto>,
    #[serde(default)]
    regions: Vec<ImportRegionDto>,
    #[serde(default)]
    dry_run: bool,
}

fn set_bookmarks(backend: &dyn Backend, arguments: &Value) -> ToolOutput {
    let arguments: SetBookmarksArguments = match serde_json::from_value(arguments.clone()) {
        Ok(parsed) => parsed,
        Err(error) => return ToolOutput::error(format!("invalid arguments: {error}")),
    };
    let import = ImportDto {
        format_version: EXPORT_FORMAT_VERSION,
        recordings: vec![ImportRecordingDto {
            recording: RecordingRefDto {
                id: arguments.recording_id,
            },
            bookmarks: ImportBookmarksDto {
                labels: arguments.labels,
                regions: arguments.regions,
            },
        }],
    };
    let path = if arguments.dry_run {
        format!("{IMPORT_PATH}?dry_run=true")
    } else {
        IMPORT_PATH.to_owned()
    };
    let body = match serde_json::to_value(&import) {
        Ok(body) => body,
        Err(error) => return ToolOutput::error(error.to_string()),
    };
    let reply = match backend.post_json(&path, &body) {
        Ok(reply) => reply,
        Err(error) => return ToolOutput::error(error.to_string()),
    };
    let output = reply_output(&reply);
    if output.is_error {
        return output;
    }
    // The server answers 200 even when it left the recording as it was; say so as an error.
    match serde_json::from_str::<ImportResultDto>(&reply.body) {
        Ok(result)
            if result
                .recordings
                .iter()
                .all(|r| r.status == ImportStatusDto::Ok) =>
        {
            output
        }
        Ok(_) => ToolOutput::error(format!("nothing was changed: {}", reply.body)),
        Err(_) => output,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{FakeBackend, Request};

    fn call_with(backend: &FakeBackend, name: &str, arguments: Value) -> ToolOutput {
        call(backend, name, &arguments)
    }

    #[test]
    fn reading_tools_ask_for_the_right_paths() {
        let backend = FakeBackend::answering(200, "{}");
        call_with(&backend, "list_recordings", json!({}));
        call_with(&backend, "get_recording", json!({"recording_id": 4}));
        call_with(&backend, "get_bookmarks", json!({"recording_id": 4}));
        assert_eq!(
            *backend.requests.borrow(),
            [
                Request::Get("/api/v1/recordings".into()),
                Request::Get("/api/v1/recordings/4/export".into()),
                Request::Get("/api/v1/recordings/4/bookmarks".into()),
            ]
        );
    }

    #[test]
    fn a_recording_id_is_required_and_must_be_a_number() {
        let backend = FakeBackend::answering(200, "{}");
        for arguments in [
            json!({}),
            json!({"recording_id": "four"}),
            json!({"recording_id": -1}),
        ] {
            let output = call_with(&backend, "get_bookmarks", arguments);
            assert!(output.is_error);
        }
        assert!(backend.requests.borrow().is_empty());
    }

    #[test]
    fn setting_bookmarks_posts_one_import_for_the_recording() {
        let backend = FakeBackend::answering(
            200,
            r#"{"dry_run":false,"recordings":[{"recording_id":4,"status":"ok","added":1,"changed":0,"unchanged":0,"removed":0,"issues":[]}]}"#,
        );
        let output = call_with(
            &backend,
            "set_bookmarks",
            json!({
                "recording_id": 4,
                "labels": [{"name": "Verse", "position_ms": 1000}],
                "regions": [{"id": 2, "version": 3, "name": "Chorus", "range": {"start_ms": 5, "end_ms": 9}}]
            }),
        );
        assert!(!output.is_error);
        let requests = backend.requests.borrow();
        let Request::Post(path, body) = &requests[0] else {
            panic!("a post was expected");
        };
        assert_eq!(path, "/api/v1/import");
        assert_eq!(body["format_version"], 1);
        assert_eq!(body["recordings"][0]["recording"]["id"], 4);
        assert_eq!(
            body["recordings"][0]["bookmarks"]["labels"][0]["name"],
            "Verse"
        );
        assert_eq!(
            body["recordings"][0]["bookmarks"]["regions"][0]["version"],
            3
        );
    }

    #[test]
    fn a_dry_run_asks_for_the_preview() {
        let backend = FakeBackend::answering(200, r#"{"dry_run":true,"recordings":[]}"#);
        call_with(
            &backend,
            "set_bookmarks",
            json!({"recording_id": 1, "dry_run": true}),
        );
        let requests = backend.requests.borrow();
        assert!(
            matches!(&requests[0], Request::Post(path, _) if path == "/api/v1/import?dry_run=true")
        );
    }

    #[test]
    fn a_recording_the_server_left_alone_is_reported_as_an_error() {
        let backend = FakeBackend::answering(
            200,
            r#"{"dry_run":false,"recordings":[{"recording_id":4,"status":"conflict","added":0,"changed":0,"unchanged":0,"removed":0,"issues":[{"entry":"labels[0]","reason":{"code":"bookmark_conflict","args":[3],"message":"changed"}}]}]}"#,
        );
        let output = call_with(&backend, "set_bookmarks", json!({"recording_id": 4}));
        assert!(output.is_error);
        assert!(output.text.contains("bookmark_conflict"));
    }

    #[test]
    fn malformed_bookmarks_are_refused_before_the_server_is_asked() {
        let backend = FakeBackend::answering(200, "{}");
        let output = call_with(
            &backend,
            "set_bookmarks",
            json!({"recording_id": 1, "labels": [{"name": "No time"}]}),
        );
        assert!(output.is_error);
        assert!(backend.requests.borrow().is_empty());
    }

    #[test]
    fn a_refusal_of_the_server_is_shown_with_its_reason() {
        let backend = FakeBackend::answering(403, r#"{"code":"not_allowed"}"#);
        let output = call_with(&backend, "set_bookmarks", json!({"recording_id": 1}));
        assert!(output.is_error);
        assert!(output.text.contains("403") && output.text.contains("not_allowed"));
    }

    #[test]
    fn an_unreachable_server_and_an_unknown_tool_are_errors() {
        let down = FakeBackend::unreachable();
        assert!(call_with(&down, "list_recordings", json!({})).is_error);
        assert!(call_with(&down, "play", json!({})).is_error);
    }
}
