use serde::{Deserialize, Serialize};

use super::SESSION_SCHEMA_V1;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Open,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepoMetadata {
    pub root: String,
    pub vcs: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_head: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Session {
    pub schema_version: String,
    pub session_id: String,
    pub title: String,
    pub created_at: time::OffsetDateTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<time::OffsetDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<RepoMetadata>,
    pub identity_id: String,
    pub status: SessionStatus,
}

impl Session {
    pub fn new(
        session_id: String,
        title: String,
        identity_id: String,
        repo: Option<RepoMetadata>,
    ) -> Self {
        Self {
            schema_version: SESSION_SCHEMA_V1.into(),
            session_id,
            title,
            created_at: time::OffsetDateTime::now_utc(),
            closed_at: None,
            repo,
            identity_id,
            status: SessionStatus::Open,
        }
    }
}
