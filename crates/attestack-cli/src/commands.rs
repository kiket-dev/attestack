use std::path::{Path, PathBuf};

use attestack_core::{
    EventKind, EventPayload, GitSnapshotPayload, Session, SessionNoteAddedPayload,
    SessionStoppedPayload,
};
use attestack_store::{
    git_skip_message, is_git_repo, load_public_key_from_file, load_public_key_from_store,
    new_session, run_doctor, verify_bundle_file, verify_local_session, BundleCreateOptions,
    DoctorReport, PrSummaryOptions, ReportOptions, Store, StoreError, STORE_DIR,
};

pub fn init(repo_root: &Path, force: bool, update_gitignore: bool) -> Result<PathBuf, String> {
    let store_root = repo_root.join(STORE_DIR);
    if store_root.join(attestack_store::CONFIG_FILE).is_file() && !force {
        return Err(format!(
            "Attestack already initialized at {}; use --force to overwrite config",
            store_root.display()
        ));
    }

    Store::init(repo_root).map_err(format_store_error)?;
    if update_gitignore {
        update_gitignore_file(repo_root)?;
    }

    Ok(store_root)
}

pub fn start(
    repo_root: &Path,
    title: String,
    allow_parallel: bool,
    capture_git: bool,
) -> Result<Session, String> {
    let store = Store::open(repo_root).map_err(format_store_error)?;
    if !allow_parallel && store.find_open_session().map_err(format_store_error)?.is_some() {
        return Err("a session is already open; use --allow-parallel to override".into());
    }

    let identity_id = store.config().map_err(format_store_error)?.default_identity_id;
    let mut session = new_session(title, identity_id);
    store.enrich_session_repo_metadata(&mut session, repo_root).map_err(format_store_error)?;
    store.create_session(session.clone()).map_err(format_store_error)?;

    if capture_git {
        match store.record_git_snapshot_for_session(&session.session_id, repo_root, false) {
            Ok(_) => {}
            Err(err) if is_git_skip_error(&err) => {}
            Err(err) => return Err(format_store_error(err)),
        }
    }

    Ok(session)
}

pub fn status(repo_root: &Path) -> Result<Option<Session>, String> {
    let store = Store::open(repo_root).map_err(format_store_error)?;
    store.find_open_session().map_err(format_store_error)
}

pub fn note(repo_root: &Path, text: String, session_id: Option<String>) -> Result<(), String> {
    let store = Store::open(repo_root).map_err(format_store_error)?;
    let session = resolve_session(&store, session_id)?;
    store.verify_session_chain(&session.session_id).map_err(format_store_error)?;

    store
        .append_typed_event(
            &session.session_id,
            EventKind::SessionNoteAdded,
            EventPayload::SessionNoteAdded(SessionNoteAddedPayload { text }),
        )
        .map_err(format_store_error)?;
    Ok(())
}

pub fn snapshot(
    repo_root: &Path,
    session_id: Option<String>,
    include_diff: bool,
) -> Result<GitSnapshotPayload, String> {
    if !is_git_repo(repo_root) {
        return Err("not inside a git repository; snapshot requires a git work tree".into());
    }

    let store = Store::open(repo_root).map_err(format_store_error)?;
    let session = resolve_session(&store, session_id)?;
    store.verify_session_chain(&session.session_id).map_err(format_store_error)?;

    store
        .record_git_snapshot_for_session(&session.session_id, repo_root, include_diff)
        .map_err(format_store_error)
}

pub fn stop(
    repo_root: &Path,
    session_id: Option<String>,
    write_report: bool,
    capture_git: bool,
) -> Result<Session, String> {
    let store = Store::open(repo_root).map_err(format_store_error)?;
    let mut session = resolve_session(&store, session_id)?;
    store.verify_session_chain(&session.session_id).map_err(format_store_error)?;

    if capture_git {
        match store.record_git_snapshot_for_session(&session.session_id, repo_root, false) {
            Ok(_) => {}
            Err(err) if is_git_skip_error(&err) => {}
            Err(err) => return Err(format_store_error(err)),
        }
    }

    let events = store.read_events(&session.session_id).map_err(format_store_error)?;
    let report_path = if write_report {
        Some(
            store
                .write_session_report(&session, &events, &ReportOptions::default())
                .map_err(|err| err.to_string())?
                .display()
                .to_string(),
        )
    } else {
        None
    };

    store
        .append_typed_event(
            &session.session_id,
            EventKind::SessionStopped,
            EventPayload::SessionStopped(SessionStoppedPayload {
                report_path: report_path.clone(),
            }),
        )
        .map_err(format_store_error)?;

    store.close_session(&mut session).map_err(format_store_error)?;
    Ok(session)
}

fn resolve_session(store: &Store, session_id: Option<String>) -> Result<Session, String> {
    if let Some(session_id) = session_id {
        let session_dir = store.session_dir(&session_id);
        return store.read_session(session_dir).map_err(format_store_error);
    }

    store.find_open_session().map_err(format_store_error)?.ok_or_else(|| "no open session".into())
}

pub(crate) fn resolve_session_for_agent(
    store: &Store,
    session_id: Option<String>,
) -> Result<Session, String> {
    resolve_session(store, session_id)
}

pub(crate) fn format_store_error(err: StoreError) -> String {
    err.to_string()
}

fn is_git_skip_error(err: &StoreError) -> bool {
    matches!(err, StoreError::Io(io_err) if io_err.to_string().contains("not inside a git repository"))
}

pub fn report(
    repo_root: &Path,
    session_id: Option<String>,
    output: Option<PathBuf>,
    include_command_output: bool,
) -> Result<(Session, String), String> {
    let store = Store::open(repo_root).map_err(format_store_error)?;
    let session = store.resolve_session_for_report(session_id).map_err(format_store_error)?;
    let report = store
        .generate_session_report(&session, &ReportOptions { include_command_output })
        .map_err(format_store_error)?;

    if let Some(output_path) = output {
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).map_err(|err| err.to_string())?;
        }
        std::fs::write(&output_path, &report.markdown).map_err(|err| err.to_string())?;
        Ok((session, output_path.display().to_string()))
    } else {
        Ok((session, report.markdown))
    }
}

pub fn pr_summary(
    repo_root: &Path,
    session_id: Option<String>,
    bundle: Option<PathBuf>,
) -> Result<(Session, String), String> {
    let store = Store::open(repo_root).map_err(format_store_error)?;
    let session = store.resolve_session_for_report(session_id).map_err(format_store_error)?;

    let bundle_sha256 = bundle.as_ref().map(|path| sha256_file(path)).transpose()?;
    let summary = store
        .generate_pr_summary(&session, &PrSummaryOptions { bundle_path: bundle, bundle_sha256 })
        .map_err(format_store_error)?;
    Ok((session, summary))
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|err| err.to_string())?;
    Ok(attestack_core::sha256_hex(&bytes))
}

pub fn doctor(repo_root: &Path) -> DoctorReport {
    run_doctor(repo_root)
}

fn update_gitignore_file(repo_root: &Path) -> Result<(), String> {
    let gitignore_path = repo_root.join(".gitignore");
    let entries = [".attestack/runs/", ".attestack/tmp/", ".attestack/keys/"];
    let mut existing = if gitignore_path.is_file() {
        std::fs::read_to_string(&gitignore_path).map_err(|err| err.to_string())?
    } else {
        String::new()
    };

    for entry in entries {
        if !existing.lines().any(|line| line.trim() == entry) {
            if !existing.is_empty() && !existing.ends_with('\n') {
                existing.push('\n');
            }
            existing.push_str(entry);
            existing.push('\n');
        }
    }

    std::fs::write(gitignore_path, existing).map_err(|err| err.to_string())?;
    Ok(())
}

pub fn git_status_message(repo_root: &Path) -> Option<String> {
    if is_git_repo(repo_root) {
        None
    } else {
        Some(git_skip_message())
    }
}

pub fn bundle_create(
    repo_root: &Path,
    session_id: Option<String>,
    options: BundleCreateOptions,
) -> Result<attestack_store::BundleCreateResult, String> {
    let store = Store::open(repo_root).map_err(format_store_error)?;
    store.create_bundle(repo_root, session_id, options).map_err(|err| err.to_string())
}

pub fn verify_path(
    repo_root: &Path,
    path: &Path,
    public_key_path: Option<&Path>,
) -> Result<attestack_core::VerificationReport, String> {
    if !path.exists() {
        return Err(format!(
            "path not found: {}; expected a session directory or .attestack.zip bundle",
            path.display()
        ));
    }

    if path.extension().and_then(|ext| ext.to_str()) == Some("zip")
        || path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".attestack.zip"))
    {
        let public_key = if let Some(key_path) = public_key_path {
            Some(load_public_key_from_file(key_path).map_err(|err| err.to_string())?)
        } else {
            Store::open(repo_root)
                .ok()
                .and_then(|store| load_public_key_from_store(store.root()).ok())
                .flatten()
        };
        return verify_bundle_file(path, public_key).map_err(|err| err.to_string());
    }

    let session_dir = if path.join("session.json").is_file() || path.join("events.jsonl").is_file()
    {
        path.to_path_buf()
    } else if path.file_name().and_then(|name| name.to_str()) == Some("events.jsonl") {
        path.parent().ok_or_else(|| "invalid session path".to_string())?.to_path_buf()
    } else {
        return Err(format!(
            "unsupported verify path: {}; expected a session directory or .attestack.zip bundle",
            path.display()
        ));
    };

    verify_local_session(&session_dir).map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::tempdir;

    fn init_git_repo(path: &Path) {
        Command::new("git").args(["init", "-b", "main"]).current_dir(path).output().unwrap();
        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(path)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(path)
            .output()
            .unwrap();
        std::fs::write(path.join("README.md"), "hello\n").unwrap();
        Command::new("git").args(["add", "README.md"]).current_dir(path).output().unwrap();
        Command::new("git").args(["commit", "-m", "init"]).current_dir(path).output().unwrap();
    }

    #[test]
    fn init_start_note_stop_flow() {
        let dir = tempdir().unwrap();
        init(dir.path(), false, false).unwrap();
        let session = start(dir.path(), "demo".into(), false, false).unwrap();
        note(dir.path(), "manual review".into(), None).unwrap();
        let stopped = stop(dir.path(), None, true, false).unwrap();
        assert_eq!(stopped.session_id, session.session_id);
        assert_eq!(stopped.status, attestack_core::SessionStatus::Closed);
        assert!(status(dir.path()).unwrap().is_none());
    }

    #[test]
    fn snapshot_in_git_repo() {
        let dir = tempdir().unwrap();
        init_git_repo(dir.path());
        init(dir.path(), false, false).unwrap();
        start(dir.path(), "demo".into(), false, true).unwrap();
        let payload = snapshot(dir.path(), None, false).unwrap();
        assert!(payload.head.is_some());
    }

    #[test]
    fn report_generates_markdown() {
        let dir = tempdir().unwrap();
        init(dir.path(), false, false).unwrap();
        start(dir.path(), "report test".into(), false, false).unwrap();
        note(dir.path(), "important note".into(), None).unwrap();
        stop(dir.path(), None, true, false).unwrap();
        let (session, content) = report(dir.path(), None, None, false).unwrap();
        assert!(content.contains(&session.session_id));
        assert!(content.contains("important note"));
        assert!(content.contains("## Verification"));
    }

    #[test]
    fn doctor_passes_after_init() {
        let dir = tempdir().unwrap();
        init(dir.path(), false, false).unwrap();
        let report = doctor(dir.path());
        assert!(report.passed(), "{:?}", report.checks);
    }

    #[test]
    fn doctor_fails_before_init() {
        let dir = tempdir().unwrap();
        let report = doctor(dir.path());
        assert!(!report.passed());
    }
}
