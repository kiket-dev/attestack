use serde::{Deserialize, Serialize};

use super::Signature;
use super::BUNDLE_SCHEMA_V1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BundleFileEntry {
    pub path: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub media_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BundleManifest {
    pub schema_version: String,
    pub bundle_id: String,
    pub created_at: time::OffsetDateTime,
    pub sessions: Vec<String>,
    pub files: Vec<BundleFileEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<Signature>,
}

impl BundleManifest {
    pub fn new(bundle_id: String, sessions: Vec<String>, files: Vec<BundleFileEntry>) -> Self {
        Self {
            schema_version: BUNDLE_SCHEMA_V1.into(),
            bundle_id,
            created_at: time::OffsetDateTime::now_utc(),
            sessions,
            files,
            signature: None,
        }
    }
}
