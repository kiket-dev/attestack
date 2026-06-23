use std::path::Path;

use attestack_core::{
    verify_event_chain, GitSnapshotPayload, RepoMetadata, Session, SessionStatus,
};

use crate::git::{self, GitError};
use crate::{Store, StoreError};

impl Store {
    pub fn record_git_snapshot_for_session(
        &self,
        session_id: &str,
        repo_root: &Path,
        include_diff: bool,
    ) -> Result<GitSnapshotPayload, StoreError> {
        let payload = git::capture_snapshot(repo_root, false).map_err(map_git_error)?;
        let diff_bytes = if include_diff {
            git::snapshot_diff_bytes(repo_root).map_err(map_git_error)?
        } else {
            None
        };
        self.append_git_snapshot(session_id, payload.clone(), diff_bytes)?;
        Ok(payload)
    }

    pub fn enrich_session_repo_metadata(
        &self,
        session: &mut Session,
        repo_root: &Path,
    ) -> Result<(), StoreError> {
        if !git::is_git_repo(repo_root) {
            return Ok(());
        }

        let head = git::capture_snapshot(repo_root, false).ok().and_then(|snapshot| snapshot.head);

        session.repo = Some(RepoMetadata {
            root: repo_root
                .canonicalize()
                .unwrap_or_else(|_| repo_root.to_path_buf())
                .display()
                .to_string(),
            vcs: "git".into(),
            initial_head: head,
        });
        Ok(())
    }

    pub fn close_session(&self, session: &mut Session) -> Result<(), StoreError> {
        session.status = SessionStatus::Closed;
        session.closed_at = Some(time::OffsetDateTime::now_utc());
        self.write_session(session)
    }

    pub fn verify_session_chain(&self, session_id: &str) -> Result<(), StoreError> {
        let events = self.read_events(session_id)?;
        verify_event_chain(&events).map_err(StoreError::Core)
    }
}

pub fn map_git_error(err: GitError) -> StoreError {
    StoreError::Io(std::io::Error::other(err.to_string()))
}

pub fn git_skip_message() -> String {
    "skipped git snapshot: not inside a git repository".into()
}
