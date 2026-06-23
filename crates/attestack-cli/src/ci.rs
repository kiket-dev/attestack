use std::env;
use std::path::{Path, PathBuf};

use attestack_store::{BundleCreateOptions, Store, StoreError, STORE_DIR};

use crate::commands::{bundle_create, format_store_error, init, start, stop};

pub struct CiStartResult {
    pub session_id: String,
    pub title: String,
}

pub struct CiFinishResult {
    pub session_id: String,
    pub bundle_id: String,
    pub bundle_path: PathBuf,
}

pub fn ci_title(explicit: Option<String>) -> String {
    if let Some(title) = explicit {
        return title;
    }

    if let Ok(workflow) = env::var("GITHUB_WORKFLOW") {
        let run_id = env::var("GITHUB_RUN_ID").unwrap_or_else(|_| "local".into());
        return format!("CI {workflow} #{run_id}");
    }

    if let Ok(pipeline) = env::var("CI_PIPELINE_ID") {
        return format!("CI pipeline {pipeline}");
    }

    if env::var("CI").is_ok() {
        return "CI run".into();
    }

    "CI local".into()
}

pub fn ci_start(
    repo_root: &Path,
    title: Option<String>,
    force_init: bool,
) -> Result<CiStartResult, String> {
    if !repo_root.join(STORE_DIR).join(attestack_store::CONFIG_FILE).is_file() {
        init(repo_root, force_init, false)?;
    }

    let title = ci_title(title);
    let session = start(repo_root, title.clone(), true, true)?;
    Ok(CiStartResult { session_id: session.session_id, title })
}

pub fn ci_finish(
    repo_root: &Path,
    session_id: Option<String>,
    output: Option<PathBuf>,
) -> Result<CiFinishResult, String> {
    let store = Store::open(repo_root).map_err(format_store_error)?;
    let session = match session_id {
        Some(id) => store.read_session(store.session_dir(&id)).map_err(format_store_error)?,
        None => store
            .find_open_session()
            .map_err(format_store_error)?
            .ok_or_else(|| "no open CI session; run `attestack ci start` first".to_string())?,
    };

    stop(repo_root, Some(session.session_id.clone()), true, true)?;
    let bundle = bundle_create(
        repo_root,
        Some(session.session_id.clone()),
        BundleCreateOptions {
            output,
            include_diff: false,
            include_command_output: true,
            redact_paths: true,
        },
    )?;

    Ok(CiFinishResult {
        session_id: session.session_id,
        bundle_id: bundle.bundle_id,
        bundle_path: bundle.bundle_path,
    })
}

pub fn is_not_initialized(err: &str) -> bool {
    err.contains("not initialized")
}

pub fn map_ci_start_error(err: String) -> String {
    if is_not_initialized(&err) {
        format!("{err}; run `attestack ci start` or `attestack init` first")
    } else {
        err
    }
}

#[allow(dead_code)]
pub fn map_store_error(err: StoreError) -> String {
    format_store_error(err)
}
