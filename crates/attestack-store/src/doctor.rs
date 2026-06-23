use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::identity::{self, user_keys_dir, DEFAULT_IDENTITY_FILE};
use crate::{is_git_repo, Store, CONFIG_FILE, STORE_DIR};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DoctorCheck {
    pub name: String,
    pub status: CheckStatus,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DoctorReport {
    pub checks: Vec<DoctorCheck>,
}

impl DoctorReport {
    pub fn passed(&self) -> bool {
        !self.checks.iter().any(|check| check.status == CheckStatus::Fail)
    }

    pub fn has_warnings(&self) -> bool {
        self.checks.iter().any(|check| check.status == CheckStatus::Warn)
    }
}

pub fn run_doctor(repo_root: &Path) -> DoctorReport {
    let mut checks = Vec::new();
    let store_root = repo_root.join(STORE_DIR);

    if store_root.join(CONFIG_FILE).is_file() {
        checks.push(DoctorCheck {
            name: "store".into(),
            status: CheckStatus::Pass,
            message: format!("store initialized at {}", store_root.display()),
        });
    } else {
        checks.push(DoctorCheck {
            name: "store".into(),
            status: CheckStatus::Fail,
            message: "Attestack is not initialized; run `attestack init`".into(),
        });
        return DoctorReport { checks };
    }

    match Store::open(repo_root) {
        Ok(store) => match store.config() {
            Ok(config) => checks.push(DoctorCheck {
                name: "config".into(),
                status: CheckStatus::Pass,
                message: format!("config readable (identity_id={})", config.default_identity_id),
            }),
            Err(err) => checks.push(DoctorCheck {
                name: "config".into(),
                status: CheckStatus::Fail,
                message: err.to_string(),
            }),
        },
        Err(err) => checks.push(DoctorCheck {
            name: "config".into(),
            status: CheckStatus::Fail,
            message: err.to_string(),
        }),
    }

    let public_path = store_root.join(crate::IDENTITIES_DIR).join(DEFAULT_IDENTITY_FILE);
    if public_path.is_file() {
        match identity::Identity::load(&store_root) {
            Ok(identity) => {
                let private_key = user_keys_dir().join(format!("{}.key", identity.identity_id));
                if private_key.is_file() {
                    checks.push(DoctorCheck {
                        name: "identity".into(),
                        status: CheckStatus::Pass,
                        message: format!(
                            "identity {} with private key at {}",
                            identity.identity_id,
                            private_key.display()
                        ),
                    });
                } else {
                    checks.push(DoctorCheck {
                        name: "identity".into(),
                        status: CheckStatus::Fail,
                        message: format!(
                            "public identity found but private key missing at {}",
                            private_key.display()
                        ),
                    });
                }
            }
            Err(err) => checks.push(DoctorCheck {
                name: "identity".into(),
                status: CheckStatus::Fail,
                message: err.to_string(),
            }),
        }
    } else {
        checks.push(DoctorCheck {
            name: "identity".into(),
            status: CheckStatus::Fail,
            message: "missing default public identity; run `attestack init --force`".into(),
        });
    }

    if Command::new("git").arg("--version").output().is_ok() {
        let git_message = if is_git_repo(repo_root) {
            "git available and repository detected".into()
        } else {
            "git available; current directory is not a git repository".into()
        };
        checks.push(DoctorCheck {
            name: "git".into(),
            status: CheckStatus::Pass,
            message: git_message,
        });
    } else {
        checks.push(DoctorCheck {
            name: "git".into(),
            status: CheckStatus::Warn,
            message: "git not found in PATH; snapshot commands will not work".into(),
        });
    }

    if let Ok(store) = Store::open(repo_root) {
        match store.find_open_session() {
            Ok(Some(session)) => match store.verify_session_chain(&session.session_id) {
                Ok(()) => checks.push(DoctorCheck {
                    name: "active_session".into(),
                    status: CheckStatus::Pass,
                    message: format!("open session {} with valid event chain", session.session_id),
                }),
                Err(err) => checks.push(DoctorCheck {
                    name: "active_session".into(),
                    status: CheckStatus::Fail,
                    message: format!(
                        "open session {} has invalid event chain: {err}",
                        session.session_id
                    ),
                }),
            },
            Ok(None) => checks.push(DoctorCheck {
                name: "active_session".into(),
                status: CheckStatus::Pass,
                message: "no open session".into(),
            }),
            Err(err) => checks.push(DoctorCheck {
                name: "active_session".into(),
                status: CheckStatus::Fail,
                message: err.to_string(),
            }),
        }
    }

    DoctorReport { checks }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn doctor_fails_when_not_initialized() {
        let dir = tempdir().unwrap();
        let report = run_doctor(dir.path());
        assert!(!report.passed());
        assert!(report.checks.iter().any(|check| check.name == "store"));
    }

    #[test]
    fn doctor_passes_after_init() {
        let dir = tempdir().unwrap();
        Store::init(dir.path()).unwrap();
        let report = run_doctor(dir.path());
        assert!(report.passed(), "{:?}", report.checks);
    }
}
