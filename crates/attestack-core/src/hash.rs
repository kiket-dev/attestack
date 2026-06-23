use sha2::{Digest, Sha256};

use crate::error::{AttestackError, Result};
use crate::types::Event;

pub const SHA256_PREFIX: &str = "sha256:";

pub fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

pub fn sha256_digest(bytes: &[u8]) -> String {
    format!("{SHA256_PREFIX}{}", sha256_hex(bytes))
}

pub fn hash_event_for_chain(event: &Event) -> Result<String> {
    let mut value = serde_json::to_value(event)?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| AttestackError::Canonicalization("event is not a JSON object".into()))?;
    object.remove("event_hash");
    object.remove("signature");

    let canonical = serde_jcs::to_string(&value)
        .map_err(|err| AttestackError::Canonicalization(err.to_string()))?;
    Ok(sha256_digest(canonical.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Event, EventKind, EventPayload, SessionStartedPayload, Signature};

    fn sample_event(signature: Option<Signature>) -> Event {
        Event {
            schema_version: crate::types::EVENT_SCHEMA_V1.into(),
            event_id: "evt_ses_test_000001".into(),
            session_id: "ses_test".into(),
            sequence: 1,
            recorded_at: time::macros::datetime!(2026-06-22 13:00:01 UTC),
            kind: EventKind::SessionStarted,
            payload: EventPayload::SessionStarted(SessionStartedPayload { title: "demo".into() }),
            prev_event_hash: None,
            event_hash: String::new(),
            signature,
        }
    }

    #[test]
    fn same_event_hashes_identically() {
        let a = sample_event(None);
        let b = sample_event(None);
        let ha = hash_event_for_chain(&a).unwrap();
        let hb = hash_event_for_chain(&b).unwrap();
        assert_eq!(ha, hb);
    }

    #[test]
    fn changing_payload_changes_hash() {
        let mut a = sample_event(None);
        let hash_a = hash_event_for_chain(&a).unwrap();
        if let EventPayload::SessionStarted(ref mut payload) = a.payload {
            payload.title = "other".into();
        }
        let hash_b = hash_event_for_chain(&a).unwrap();
        assert_ne!(hash_a, hash_b);
    }

    #[test]
    fn changing_signature_does_not_change_hash() {
        let unsigned = hash_event_for_chain(&sample_event(None)).unwrap();
        let signed = hash_event_for_chain(&sample_event(Some(Signature {
            alg: "Ed25519".into(),
            key_id: "id_test".into(),
            value: "c2lnbmF0dXJl".into(),
        })))
        .unwrap();
        assert_eq!(unsigned, signed);
    }

    #[test]
    fn canonical_json_ignores_key_order() {
        let first = serde_json::json!({"sequence": 1, "kind": "session.started", "title": "demo"});
        let second = serde_json::json!({"kind": "session.started", "title": "demo", "sequence": 1});
        assert_eq!(serde_jcs::to_string(&first).unwrap(), serde_jcs::to_string(&second).unwrap());
    }
}
