use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerificationReport {
    pub verified: bool,
    pub target: String,
    pub kind: VerificationTargetKind,
    pub event_count: Option<u64>,
    pub file_count: Option<u64>,
    pub signature_verified: Option<bool>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VerificationTargetKind {
    Session,
    Bundle,
}

impl VerificationReport {
    pub fn success(target: String, kind: VerificationTargetKind) -> Self {
        Self {
            verified: true,
            target,
            kind,
            event_count: None,
            file_count: None,
            signature_verified: None,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn failure(target: String, kind: VerificationTargetKind, errors: Vec<String>) -> Self {
        Self {
            verified: false,
            target,
            kind,
            event_count: None,
            file_count: None,
            signature_verified: None,
            errors,
            warnings: Vec::new(),
        }
    }
}
