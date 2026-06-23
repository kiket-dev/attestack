use std::path::Path;

use attestack_core::{
    AiDecisionPayload, AiToolCallPayload, EventKind, EventPayload, SessionNoteAddedPayload,
};
use attestack_store::{is_git_repo, Store};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Serialize)]
pub struct McpResponse {
    pub jsonrpc: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,
}

#[derive(Debug, Serialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
}

pub fn error_response(id: Option<Value>, code: i32, message: String) -> McpResponse {
    McpResponse { jsonrpc: "2.0", id, result: None, error: Some(McpError { code, message }) }
}

pub fn ok_response(id: Option<Value>, result: Value) -> McpResponse {
    McpResponse { jsonrpc: "2.0", id, result: Some(result), error: None }
}

pub fn handle_request(repo_root: &Path, request: McpRequest) -> McpResponse {
    if request.jsonrpc != "2.0" {
        return error_response(request.id, -32600, "invalid jsonrpc version".into());
    }

    match request.method.as_str() {
        "initialize" => ok_response(
            request.id,
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {} },
                "serverInfo": { "name": "attestack-mcp", "version": env!("CARGO_PKG_VERSION") }
            }),
        ),
        "notifications/initialized" | "initialized" => ok_response(request.id, json!({})),
        "tools/list" => ok_response(request.id, json!({ "tools": tool_definitions() })),
        "tools/call" => match call_tool(repo_root, &request.params) {
            Ok(content) => ok_response(
                request.id,
                json!({ "content": [{ "type": "text", "text": content }], "isError": false }),
            ),
            Err(message) => ok_response(
                request.id,
                json!({ "content": [{ "type": "text", "text": message }], "isError": true }),
            ),
        },
        "ping" => ok_response(request.id, json!({})),
        _ => error_response(request.id, -32601, format!("method not found: {}", request.method)),
    }
}

fn tool_definitions() -> Value {
    json!([
        {
            "name": "attestack_status",
            "description": "Return the active Attestack session status for the current repository",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "attestack_note",
            "description": "Append a note to the active session",
            "inputSchema": {
                "type": "object",
                "properties": { "text": { "type": "string" } },
                "required": ["text"]
            }
        },
        {
            "name": "attestack_agent_tool_call",
            "description": "Record an AI tool call with optional input/output hashes",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "tool": { "type": "string" },
                    "input_hash": { "type": "string" },
                    "output_hash": { "type": "string" },
                    "summary": { "type": "string" }
                },
                "required": ["tool"]
            }
        },
        {
            "name": "attestack_agent_decision",
            "description": "Record an agent or human decision summary",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "summary": { "type": "string" },
                    "rationale": { "type": "string" }
                },
                "required": ["summary"]
            }
        },
        {
            "name": "attestack_snapshot",
            "description": "Capture a Git snapshot for the active session",
            "inputSchema": {
                "type": "object",
                "properties": { "include_diff": { "type": "boolean" } }
            }
        }
    ])
}

fn call_tool(repo_root: &Path, params: &Value) -> Result<String, String> {
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| "missing tool name".to_string())?;
    let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

    let store = Store::open(repo_root).map_err(|err| err.to_string())?;
    let session = store
        .find_open_session()
        .map_err(|err| err.to_string())?
        .ok_or_else(|| "no open session; run `attestack start` first".to_string())?;
    store.verify_session_chain(&session.session_id).map_err(|err| err.to_string())?;

    match name {
        "attestack_status" => {
            Ok(format!("open session {} — {}", session.session_id, session.title))
        }
        "attestack_note" => {
            let text = arguments
                .get("text")
                .and_then(Value::as_str)
                .ok_or_else(|| "text is required".to_string())?;
            store
                .append_typed_event(
                    &session.session_id,
                    EventKind::SessionNoteAdded,
                    EventPayload::SessionNoteAdded(SessionNoteAddedPayload { text: text.into() }),
                )
                .map_err(|err| err.to_string())?;
            Ok("note recorded".into())
        }
        "attestack_agent_tool_call" => {
            let tool = arguments
                .get("tool")
                .and_then(Value::as_str)
                .ok_or_else(|| "tool is required".to_string())?;
            store
                .append_typed_event(
                    &session.session_id,
                    EventKind::AiToolCall,
                    EventPayload::AiToolCall(AiToolCallPayload {
                        tool: tool.into(),
                        input_hash: string_arg(&arguments, "input_hash"),
                        output_hash: string_arg(&arguments, "output_hash"),
                        summary: string_arg(&arguments, "summary"),
                    }),
                )
                .map_err(|err| err.to_string())?;
            Ok(format!("recorded ai.tool_call for {tool}"))
        }
        "attestack_agent_decision" => {
            let summary = arguments
                .get("summary")
                .and_then(Value::as_str)
                .ok_or_else(|| "summary is required".to_string())?;
            store
                .append_typed_event(
                    &session.session_id,
                    EventKind::AiDecision,
                    EventPayload::AiDecision(AiDecisionPayload {
                        summary: summary.into(),
                        rationale: string_arg(&arguments, "rationale"),
                    }),
                )
                .map_err(|err| err.to_string())?;
            Ok("recorded ai.decision".into())
        }
        "attestack_snapshot" => {
            if !is_git_repo(repo_root) {
                return Err("not inside a git repository".into());
            }
            let include_diff =
                arguments.get("include_diff").and_then(Value::as_bool).unwrap_or(false);
            store
                .record_git_snapshot_for_session(&session.session_id, repo_root, include_diff)
                .map_err(|err| err.to_string())?;
            Ok("git snapshot recorded".into())
        }
        _ => Err(format!("unknown tool: {name}")),
    }
}

fn string_arg(arguments: &Value, key: &str) -> Option<String> {
    arguments.get(key).and_then(Value::as_str).map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn initialize_returns_capabilities() {
        let dir = tempdir().unwrap();
        let response = handle_request(
            dir.path(),
            McpRequest {
                jsonrpc: "2.0".into(),
                id: Some(json!(1)),
                method: "initialize".into(),
                params: json!({}),
            },
        );
        assert!(response.error.is_none());
        assert!(response.result.unwrap().get("capabilities").is_some());
    }

    #[test]
    fn tools_list_includes_status() {
        let dir = tempdir().unwrap();
        let response = handle_request(
            dir.path(),
            McpRequest {
                jsonrpc: "2.0".into(),
                id: Some(json!(2)),
                method: "tools/list".into(),
                params: json!({}),
            },
        );
        let result = response.result.unwrap();
        let tools = result.get("tools").unwrap().as_array().unwrap();
        assert!(tools.iter().any(|tool| tool.get("name").unwrap() == "attestack_status"));
    }
}
