use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Artifact {
    pub artifact_id: String,
    pub path: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub media_type: String,
    pub purpose: String,
    pub redacted: bool,
}
