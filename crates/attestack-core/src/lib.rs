pub mod error;
pub mod hash;
pub mod ids;
pub mod sign;
pub mod types;
pub mod verify;
pub mod verify_report;

pub use error::{AttestackError, Result};
pub use hash::{hash_event_for_chain, sha256_digest, sha256_hex, SHA256_PREFIX};
pub use ids::{
    artifact_id_from_digest, identity_id_from_public_key, new_bundle_id, new_command_id,
    new_event_id, new_session_id,
};
pub use sign::{
    generate_signing_key, sign_bundle_manifest, sign_json_value, signing_key_from_bytes,
    verify_bundle_manifest, verify_json_signature, verifying_key_from_bytes, SIGNATURE_ALG_ED25519,
};
pub use types::*;
pub use verify::verify_event_chain;
pub use verify_report::{VerificationReport, VerificationTargetKind};
