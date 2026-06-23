use std::path::Path;

use attestack_core::{
    AiApprovalPayload, AiDecisionPayload, AiPromptPayload, AiResponsePayload, AiToolCallPayload,
    EventKind, EventPayload,
};
use attestack_store::Store;

use crate::commands::{format_store_error, resolve_session_for_agent};

pub fn agent_tool_call(
    repo_root: &Path,
    session_id: Option<String>,
    tool: String,
    input_hash: Option<String>,
    output_hash: Option<String>,
    summary: Option<String>,
) -> Result<(), String> {
    let store = Store::open(repo_root).map_err(format_store_error)?;
    let session = resolve_session_for_agent(&store, session_id)?;
    store.verify_session_chain(&session.session_id).map_err(format_store_error)?;
    store
        .append_typed_event(
            &session.session_id,
            EventKind::AiToolCall,
            EventPayload::AiToolCall(AiToolCallPayload { tool, input_hash, output_hash, summary }),
        )
        .map_err(format_store_error)
}

pub fn agent_decision(
    repo_root: &Path,
    session_id: Option<String>,
    summary: String,
    rationale: Option<String>,
) -> Result<(), String> {
    let store = Store::open(repo_root).map_err(format_store_error)?;
    let session = resolve_session_for_agent(&store, session_id)?;
    store.verify_session_chain(&session.session_id).map_err(format_store_error)?;
    store
        .append_typed_event(
            &session.session_id,
            EventKind::AiDecision,
            EventPayload::AiDecision(AiDecisionPayload { summary, rationale }),
        )
        .map_err(format_store_error)
}

pub fn agent_approval(
    repo_root: &Path,
    session_id: Option<String>,
    subject: String,
    approved: bool,
) -> Result<(), String> {
    let store = Store::open(repo_root).map_err(format_store_error)?;
    let session = resolve_session_for_agent(&store, session_id)?;
    store.verify_session_chain(&session.session_id).map_err(format_store_error)?;
    store
        .append_typed_event(
            &session.session_id,
            EventKind::AiApproval,
            EventPayload::AiApproval(AiApprovalPayload { subject, approved }),
        )
        .map_err(format_store_error)
}

pub fn agent_prompt(
    repo_root: &Path,
    session_id: Option<String>,
    content_hash: String,
    model: Option<String>,
) -> Result<(), String> {
    let store = Store::open(repo_root).map_err(format_store_error)?;
    let session = resolve_session_for_agent(&store, session_id)?;
    store.verify_session_chain(&session.session_id).map_err(format_store_error)?;
    store
        .append_typed_event(
            &session.session_id,
            EventKind::AiPrompt,
            EventPayload::AiPrompt(AiPromptPayload { content_hash, model }),
        )
        .map_err(format_store_error)
}

pub fn agent_response(
    repo_root: &Path,
    session_id: Option<String>,
    content_hash: String,
    model: Option<String>,
) -> Result<(), String> {
    let store = Store::open(repo_root).map_err(format_store_error)?;
    let session = resolve_session_for_agent(&store, session_id)?;
    store.verify_session_chain(&session.session_id).map_err(format_store_error)?;
    store
        .append_typed_event(
            &session.session_id,
            EventKind::AiResponse,
            EventPayload::AiResponse(AiResponsePayload { content_hash, model }),
        )
        .map_err(format_store_error)
}
