//! The MCP side: JSON-RPC messages, one per line, on a reader and a writer.

use crate::backend::Backend;
use crate::tools;
use serde_json::{Value, json};
use std::io::{BufRead, Write};

const DEFAULT_PROTOCOL_VERSION: &str = "2025-06-18";
const SERVER_NAME: &str = "veltrunox";

const PARSE_ERROR: i64 = -32700;
const METHOD_NOT_FOUND: i64 = -32601;
const INVALID_PARAMS: i64 = -32602;

/// Reads requests line by line until the input ends and answers each on the writer.
pub fn serve(
    backend: &dyn Backend,
    input: impl BufRead,
    mut output: impl Write,
) -> std::io::Result<()> {
    for line in input.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        if let Some(reply) = handle_message(backend, &line) {
            writeln!(output, "{reply}")?;
            output.flush()?;
        }
    }
    Ok(())
}

/// The answer to one message; `None` for a notification, which gets none.
pub fn handle_message(backend: &dyn Backend, line: &str) -> Option<String> {
    let message: Value = match serde_json::from_str(line) {
        Ok(message) => message,
        Err(_) => return Some(failure(&Value::Null, PARSE_ERROR, "not valid JSON").to_string()),
    };
    let id = message.get("id").cloned()?;
    let method = message.get("method").and_then(Value::as_str).unwrap_or("");
    let params = message.get("params").cloned().unwrap_or(Value::Null);
    let reply = match method {
        "initialize" => success(&id, initialize(&params)),
        "ping" => success(&id, json!({})),
        "tools/list" => success(&id, json!({"tools": tools::definitions()})),
        "tools/call" => call_tool(backend, &id, &params),
        other => failure(&id, METHOD_NOT_FOUND, &format!("unknown method: {other}")),
    };
    Some(reply.to_string())
}

fn initialize(params: &Value) -> Value {
    let version = params
        .get("protocolVersion")
        .and_then(Value::as_str)
        .unwrap_or(DEFAULT_PROTOCOL_VERSION);
    json!({
        "protocolVersion": version,
        "capabilities": {"tools": {}},
        "serverInfo": {"name": SERVER_NAME, "version": env!("CARGO_PKG_VERSION")}
    })
}

fn call_tool(backend: &dyn Backend, id: &Value, params: &Value) -> Value {
    let Some(name) = params.get("name").and_then(Value::as_str) else {
        return failure(id, INVALID_PARAMS, "a tool name is required");
    };
    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let output = tools::call(backend, name, &arguments);
    success(
        id,
        json!({
            "content": [{"type": "text", "text": output.text}],
            "isError": output.is_error
        }),
    )
}

fn success(id: &Value, result: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "result": result})
}

fn failure(id: &Value, code: i64, message: &str) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::FakeBackend;

    fn ask(backend: &FakeBackend, message: Value) -> Option<Value> {
        handle_message(backend, &message.to_string())
            .map(|reply| serde_json::from_str(&reply).unwrap())
    }

    #[test]
    fn initialize_names_the_server_and_its_tools_capability() {
        let backend = FakeBackend::answering(200, "{}");
        let reply = ask(
            &backend,
            json!({"jsonrpc": "2.0", "id": 1, "method": "initialize",
                   "params": {"protocolVersion": "2025-03-26", "capabilities": {}}}),
        )
        .unwrap();
        assert_eq!(reply["id"], 1);
        assert_eq!(reply["result"]["protocolVersion"], "2025-03-26");
        assert_eq!(reply["result"]["serverInfo"]["name"], "veltrunox");
        assert!(reply["result"]["capabilities"]["tools"].is_object());
    }

    #[test]
    fn notifications_get_no_answer() {
        let backend = FakeBackend::answering(200, "{}");
        let reply = ask(
            &backend,
            json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
        );
        assert_eq!(reply, None);
    }

    #[test]
    fn the_four_tools_are_listed() {
        let backend = FakeBackend::answering(200, "{}");
        let reply = ask(
            &backend,
            json!({"jsonrpc": "2.0", "id": "a", "method": "tools/list"}),
        )
        .unwrap();
        let names: Vec<_> = reply["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|tool| tool["name"].as_str().unwrap())
            .collect();
        assert_eq!(
            names,
            [
                "list_recordings",
                "get_recording",
                "get_bookmarks",
                "set_bookmarks"
            ]
        );
        assert_eq!(reply["id"], "a");
    }

    #[test]
    fn a_tool_call_answers_with_text_content() {
        let backend = FakeBackend::answering(200, "[1,2]");
        let reply = ask(
            &backend,
            json!({"jsonrpc": "2.0", "id": 7, "method": "tools/call",
                   "params": {"name": "list_recordings", "arguments": {}}}),
        )
        .unwrap();
        assert_eq!(reply["result"]["content"][0]["text"], "[1,2]");
        assert_eq!(reply["result"]["isError"], false);
    }

    #[test]
    fn bad_requests_get_protocol_errors() {
        let backend = FakeBackend::answering(200, "{}");
        let unknown = ask(
            &backend,
            json!({"jsonrpc": "2.0", "id": 1, "method": "resources/list"}),
        )
        .unwrap();
        assert_eq!(unknown["error"]["code"], METHOD_NOT_FOUND);
        let nameless = ask(
            &backend,
            json!({"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {}}),
        )
        .unwrap();
        assert_eq!(nameless["error"]["code"], INVALID_PARAMS);
        let garbage: Value =
            serde_json::from_str(&handle_message(&backend, "{oops").unwrap()).unwrap();
        assert_eq!(garbage["error"]["code"], PARSE_ERROR);
    }

    #[test]
    fn serve_answers_line_by_line_until_the_input_ends() {
        let backend = FakeBackend::answering(200, "{}");
        let input = "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\"}\n\n\
                     {\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n\
                     {\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"ping\"}\n";
        let mut output = Vec::new();
        serve(&backend, input.as_bytes(), &mut output).unwrap();
        let lines: Vec<_> = String::from_utf8(output)
            .unwrap()
            .lines()
            .map(str::to_owned)
            .collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[1].contains("\"id\":2"));
    }
}
