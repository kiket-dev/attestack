use rand::RngCore;
use sha2::{Digest, Sha256};

const ID_ALPHABET: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";

fn random_suffix(len: usize) -> String {
    let mut rng = rand::thread_rng();
    let mut out = String::with_capacity(len);
    for _ in 0..len {
        let idx = (rng.next_u32() as usize) % ID_ALPHABET.len();
        out.push(ID_ALPHABET[idx] as char);
    }
    out
}

pub fn new_session_id(now: time::OffsetDateTime) -> String {
    let stamp = format!("{:04}{:02}{:02}", now.year(), u8::from(now.month()), now.day());
    format!("ses_{stamp}_{}", random_suffix(6))
}

pub fn new_event_id(session_id: &str, sequence: u64) -> String {
    format!("evt_{session_id}_{sequence:06}")
}

pub fn new_command_id() -> String {
    format!("cmd_{}", random_suffix(8))
}

pub fn new_bundle_id() -> String {
    format!("bun_{}", random_suffix(8))
}

pub fn artifact_id_from_digest(digest_hex: &str) -> String {
    let prefix = digest_hex.chars().take(12).collect::<String>();
    format!("art_{prefix}")
}

pub fn identity_id_from_public_key(public_key_bytes: &[u8]) -> String {
    let digest = Sha256::digest(public_key_bytes);
    let prefix = hex::encode(digest).chars().take(12).collect::<String>();
    format!("id_{prefix}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_id_has_expected_prefix() {
        let id = new_session_id(time::OffsetDateTime::now_utc());
        assert!(id.starts_with("ses_"));
    }

    #[test]
    fn event_id_includes_sequence() {
        let id = new_event_id("ses_20260622_abc123", 1);
        assert_eq!(id, "evt_ses_20260622_abc123_000001");
    }
}
