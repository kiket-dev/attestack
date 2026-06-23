mod artifacts;
mod bundle;
mod doctor;
mod git;
mod identity;
mod io;
mod report;
mod session_ops;

use std::fs;
use std::path::{Path, PathBuf};

use attestack_core::{
    hash_event_for_chain, new_session_id, AttestackError, Event, EventKind, EventPayload, Session,
    SessionStartedPayload, SessionStatus, EVENT_SCHEMA_V1,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const STORE_DIR: &str = ".attestack";
pub const CONFIG_FILE: &str = "config.toml";
pub const SESSIONS_DIR: &str = "sessions";
pub const BUNDLES_DIR: &str = "bundles";
pub const IDENTITIES_DIR: &str = "identities";

#[derive(Debug, Error)]
pub enum StoreError {
    #[error(transparent)]
    Core(#[from] AttestackError),

    #[error("store error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error(transparent)]
    Identity(#[from] identity::IdentityError),

    #[error("store not initialized at {0}")]
    NotInitialized(PathBuf),

    #[error("no open session")]
    NoOpenSession,

    #[error("session already open: {0}")]
    SessionAlreadyOpen(String),
}

pub type Result<T> = std::result::Result<T, StoreError>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoreConfig {
    pub schema_version: String,
    pub default_identity_id: String,
}

impl StoreConfig {
    pub fn new(default_identity_id: String) -> Self {
        Self { schema_version: "attestack.config.v1".into(), default_identity_id }
    }
}

pub struct Store {
    root: PathBuf,
}

impl Store {
    pub fn open(repo_root: impl AsRef<Path>) -> Result<Self> {
        let root = repo_root.as_ref().join(STORE_DIR);
        if !root.join(CONFIG_FILE).is_file() {
            return Err(StoreError::NotInitialized(root));
        }
        Ok(Self { root })
    }

    pub fn init(repo_root: impl AsRef<Path>) -> Result<Self> {
        let root = repo_root.as_ref().join(STORE_DIR);
        fs::create_dir_all(root.join(SESSIONS_DIR))?;
        fs::create_dir_all(root.join(BUNDLES_DIR))?;
        fs::create_dir_all(root.join(IDENTITIES_DIR))?;

        let identity = identity::Identity::ensure_default(&root)?;
        let config = StoreConfig::new(identity.identity_id.clone());
        let config_path = root.join(CONFIG_FILE);
        io::atomic_write(
            &config_path,
            toml::to_string_pretty(&config)
                .map_err(|err| {
                    StoreError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, err))
                })?
                .as_bytes(),
        )?;

        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn config(&self) -> Result<StoreConfig> {
        let raw = fs::read_to_string(self.root.join(CONFIG_FILE))?;
        toml::from_str(&raw).map_err(|err| {
            StoreError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, err))
        })
    }

    pub fn find_open_session(&self) -> Result<Option<Session>> {
        let sessions_dir = self.root.join(SESSIONS_DIR);
        if !sessions_dir.is_dir() {
            return Ok(None);
        }

        for entry in fs::read_dir(sessions_dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let session = self.read_session(entry.path())?;
            if session.status == SessionStatus::Open {
                return Ok(Some(session));
            }
        }
        Ok(None)
    }

    pub fn create_session(&self, session: Session) -> Result<()> {
        if let Some(open) = self.find_open_session()? {
            return Err(StoreError::SessionAlreadyOpen(open.session_id));
        }

        let session_dir = self.session_dir(&session.session_id);
        fs::create_dir_all(&session_dir)?;

        let started = self.build_event(
            &session.session_id,
            1,
            None,
            EventKind::SessionStarted,
            EventPayload::SessionStarted(SessionStartedPayload { title: session.title.clone() }),
        )?;

        self.write_session(&session)?;
        self.append_event(&session.session_id, started)?;
        Ok(())
    }

    pub fn read_session(&self, session_dir: impl AsRef<Path>) -> Result<Session> {
        let path = session_dir.as_ref().join("session.json");
        let raw = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&raw)?)
    }

    pub fn read_events(&self, session_id: &str) -> Result<Vec<Event>> {
        let path = self.session_dir(session_id).join("events.jsonl");
        if !path.is_file() {
            return Ok(Vec::new());
        }

        let raw = fs::read_to_string(path)?;
        raw.lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| Ok(serde_json::from_str(line)?))
            .collect()
    }

    pub fn append_event(&self, session_id: &str, mut event: Event) -> Result<Event> {
        if event.schema_version != EVENT_SCHEMA_V1 {
            return Err(StoreError::Core(AttestackError::InvalidSchemaVersion(
                event.schema_version.clone(),
            )));
        }

        event.event_hash = hash_event_for_chain(&event)?;
        let path = self.session_dir(session_id).join("events.jsonl");
        let line = serde_json::to_string(&event)?;
        io::locked_append_line(&path, &line)?;
        Ok(event)
    }

    pub fn write_session(&self, session: &Session) -> Result<()> {
        let path = self.session_dir(&session.session_id).join("session.json");
        io::atomic_write(&path, serde_json::to_string_pretty(session)?.as_bytes())?;
        Ok(())
    }

    pub fn build_event(
        &self,
        session_id: &str,
        sequence: u64,
        prev_event_hash: Option<String>,
        kind: EventKind,
        payload: EventPayload,
    ) -> Result<Event> {
        let mut event =
            Event::new(session_id.to_string(), sequence, kind, payload, prev_event_hash);
        event.event_hash = hash_event_for_chain(&event)?;
        Ok(event)
    }

    pub fn next_sequence(&self, session_id: &str) -> Result<u64> {
        let events = self.read_events(session_id)?;
        Ok(events.len() as u64 + 1)
    }

    pub fn session_dir(&self, session_id: &str) -> PathBuf {
        self.root.join(SESSIONS_DIR).join(session_id)
    }

    pub fn find_latest_closed_session(&self) -> Result<Option<Session>> {
        let sessions_dir = self.root.join(SESSIONS_DIR);
        if !sessions_dir.is_dir() {
            return Ok(None);
        }

        let mut best: Option<Session> = None;
        for entry in fs::read_dir(sessions_dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let session = self.read_session(entry.path())?;
            if session.status != attestack_core::SessionStatus::Closed {
                continue;
            }
            let replace = match &best {
                None => true,
                Some(current) => session.closed_at > current.closed_at,
            };
            if replace {
                best = Some(session);
            }
        }
        Ok(best)
    }
}

pub use bundle::{
    load_public_key_from_file, load_public_key_from_store, verify_bundle_file,
    verify_local_session, BundleCreateOptions, BundleCreateResult, BundleError,
    MAX_BUNDLE_FILE_BYTES,
};
pub use doctor::{run_doctor, CheckStatus, DoctorCheck, DoctorReport};
pub use identity::{user_keys_dir, Identity, IdentityError, DEFAULT_IDENTITY_FILE};
pub use report::{render_session_report, PrSummaryOptions, ReportOptions, SessionReport};

pub use artifacts::{default_output_limit, LimitedBuffer};
pub use git::{capture_snapshot, is_git_repo, GitError};
pub use session_ops::{git_skip_message, map_git_error};

pub fn default_identity_id() -> String {
    "id_local_dev".into()
}

pub fn new_session(title: String, identity_id: String) -> Session {
    Session::new(new_session_id(time::OffsetDateTime::now_utc()), title, identity_id, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use attestack_core::verify_event_chain;
    use tempfile::tempdir;

    #[test]
    fn init_and_create_session() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path()).unwrap();
        let session = new_session("demo".into(), store.config().unwrap().default_identity_id);
        store.create_session(session.clone()).unwrap();

        let open = store.find_open_session().unwrap().unwrap();
        assert_eq!(open.session_id, session.session_id);

        let events = store.read_events(&session.session_id).unwrap();
        assert_eq!(events.len(), 1);
        verify_event_chain(&events).unwrap();
    }
}
