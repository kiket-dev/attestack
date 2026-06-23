use thiserror::Error;

#[derive(Debug, Error)]
pub enum AttestackError {
    #[error("invalid schema version: {0}")]
    InvalidSchemaVersion(String),

    #[error("hash chain broken at sequence {sequence}: {reason}")]
    HashChainBroken { sequence: u64, reason: String },

    #[error("canonicalization failed: {0}")]
    Canonicalization(String),

    #[error("serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("signature verification failed")]
    SignatureInvalid,

    #[error("unsupported signature algorithm: {0}")]
    UnsupportedSignatureAlgorithm(String),
}

pub type Result<T> = std::result::Result<T, AttestackError>;
