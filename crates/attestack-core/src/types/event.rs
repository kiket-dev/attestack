use serde::{Deserialize, Serialize};

use super::Signature;
use super::EVENT_SCHEMA_V1;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EventKind {
    #[serde(rename = "session.started")]
    SessionStarted,
    #[serde(rename = "session.note_added")]
    SessionNoteAdded,
    #[serde(rename = "command.started")]
    CommandStarted,
    #[serde(rename = "command.finished")]
    CommandFinished,
    #[serde(rename = "git.snapshot")]
    GitSnapshot,
    #[serde(rename = "artifact.attached")]
    ArtifactAttached,
    #[serde(rename = "bundle.created")]
    BundleCreated,
    #[serde(rename = "session.stopped")]
    SessionStopped,
    #[serde(rename = "ai.tool_call")]
    AiToolCall,
    #[serde(rename = "ai.decision")]
    AiDecision,
    #[serde(rename = "ai.approval")]
    AiApproval,
    #[serde(rename = "ai.prompt")]
    AiPrompt,
    #[serde(rename = "ai.response")]
    AiResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum EventPayload {
    SessionStarted(SessionStartedPayload),
    SessionNoteAdded(SessionNoteAddedPayload),
    CommandStarted(CommandStartedPayload),
    CommandFinished(CommandFinishedPayload),
    AiToolCall(AiToolCallPayload),
    AiDecision(AiDecisionPayload),
    AiApproval(AiApprovalPayload),
    AiPrompt(AiPromptPayload),
    AiResponse(AiResponsePayload),
    GitSnapshot(GitSnapshotPayload),
    ArtifactAttached(ArtifactAttachedPayload),
    BundleCreated(BundleCreatedPayload),
    SessionStopped(SessionStoppedPayload),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SessionStartedPayload {
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SessionNoteAddedPayload {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandStartedPayload {
    pub command_id: String,
    pub argv: Vec<String>,
    pub cwd: String,
    pub started_at: time::OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandFinishedPayload {
    pub command_id: String,
    pub exit_code: i32,
    pub duration_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdout_artifact: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr_artifact: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GitSnapshotPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_root_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    pub dirty: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub staged_diff_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unstaged_diff_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub untracked_files_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff_artifact: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactAttachedPayload {
    pub artifact_id: String,
    pub purpose: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BundleCreatedPayload {
    pub bundle_id: String,
    pub bundle_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SessionStoppedPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AiToolCallPayload {
    pub tool: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AiDecisionPayload {
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AiApprovalPayload {
    pub subject: String,
    pub approved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AiPromptPayload {
    pub content_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AiResponsePayload {
    pub content_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Event {
    pub schema_version: String,
    pub event_id: String,
    pub session_id: String,
    pub sequence: u64,
    pub recorded_at: time::OffsetDateTime,
    pub kind: EventKind,
    pub payload: EventPayload,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_event_hash: Option<String>,
    pub event_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<Signature>,
}

impl Event {
    pub fn new(
        session_id: String,
        sequence: u64,
        kind: EventKind,
        payload: EventPayload,
        prev_event_hash: Option<String>,
    ) -> Self {
        Self {
            schema_version: EVENT_SCHEMA_V1.into(),
            event_id: crate::ids::new_event_id(&session_id, sequence),
            session_id,
            sequence,
            recorded_at: time::OffsetDateTime::now_utc(),
            kind,
            payload,
            prev_event_hash,
            event_hash: String::new(),
            signature: None,
        }
    }
}
