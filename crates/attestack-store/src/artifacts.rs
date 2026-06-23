use std::fs;
use std::path::PathBuf;

use attestack_core::{
    artifact_id_from_digest, sha256_hex, Artifact, ArtifactAttachedPayload, EventKind,
    EventPayload, GitSnapshotPayload, COMMAND_OUTPUT_LIMIT_BYTES,
};

use crate::Store;

impl Store {
    pub fn artifacts_dir(&self, session_id: &str) -> PathBuf {
        self.session_dir(session_id).join("artifacts")
    }

    pub fn store_artifact(
        &self,
        session_id: &str,
        bytes: &[u8],
        media_type: &str,
        purpose: &str,
    ) -> crate::Result<Artifact> {
        let digest_hex = sha256_hex(bytes);
        let artifact_id = artifact_id_from_digest(&digest_hex);
        let artifacts_dir = self.artifacts_dir(session_id);
        fs::create_dir_all(&artifacts_dir)?;

        let relative_path = format!("artifacts/{artifact_id}");
        let absolute_path = self.session_dir(session_id).join(&relative_path);
        fs::write(&absolute_path, bytes)?;

        Ok(Artifact {
            artifact_id: artifact_id.clone(),
            path: relative_path,
            sha256: digest_hex,
            size_bytes: bytes.len() as u64,
            media_type: media_type.to_string(),
            purpose: purpose.to_string(),
            redacted: false,
        })
    }

    pub fn append_git_snapshot(
        &self,
        session_id: &str,
        mut payload: GitSnapshotPayload,
        diff_bytes: Option<Vec<u8>>,
    ) -> crate::Result<()> {
        if let Some(bytes) = diff_bytes {
            let artifact = self.store_artifact(session_id, &bytes, "text/x-diff", "git.diff")?;
            payload.diff_artifact = Some(artifact.artifact_id.clone());
            self.append_artifact_attached(session_id, &artifact)?;
        }

        self.append_typed_event(
            session_id,
            EventKind::GitSnapshot,
            EventPayload::GitSnapshot(payload),
        )
    }

    pub fn append_artifact_attached(
        &self,
        session_id: &str,
        artifact: &Artifact,
    ) -> crate::Result<()> {
        self.append_typed_event(
            session_id,
            EventKind::ArtifactAttached,
            EventPayload::ArtifactAttached(ArtifactAttachedPayload {
                artifact_id: artifact.artifact_id.clone(),
                purpose: artifact.purpose.clone(),
            }),
        )
    }

    pub fn append_typed_event(
        &self,
        session_id: &str,
        kind: EventKind,
        payload: EventPayload,
    ) -> crate::Result<()> {
        let events = self.read_events(session_id)?;
        let prev_hash = events.last().map(|event| event.event_hash.clone());
        let sequence = events.len() as u64 + 1;
        let event = self.build_event(session_id, sequence, prev_hash, kind, payload)?;
        self.append_event(session_id, event)?;
        Ok(())
    }
}

pub struct LimitedBuffer {
    bytes: Vec<u8>,
    limit: u64,
    truncated: bool,
}

impl LimitedBuffer {
    pub fn new(limit: u64) -> Self {
        Self { bytes: Vec::new(), limit, truncated: false }
    }

    pub fn push(&mut self, chunk: &[u8]) {
        if self.truncated {
            return;
        }
        let remaining = self.limit.saturating_sub(self.bytes.len() as u64);
        if chunk.len() as u64 > remaining {
            self.bytes.extend_from_slice(&chunk[..remaining as usize]);
            self.truncated = true;
        } else {
            self.bytes.extend_from_slice(chunk);
        }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    pub fn truncated(&self) -> bool {
        self.truncated
    }
}

pub fn default_output_limit() -> u64 {
    COMMAND_OUTPUT_LIMIT_BYTES
}
