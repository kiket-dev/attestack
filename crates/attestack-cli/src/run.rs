use std::io::{self, Read, Write};
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::Instant;

use attestack_core::{
    new_command_id, verify_event_chain, CommandFinishedPayload, CommandStartedPayload, EventKind,
    EventPayload,
};
use attestack_store::{default_output_limit, LimitedBuffer, Store};

pub struct RunOptions {
    pub session_id: Option<String>,
    pub capture_output: bool,
    pub use_shell: bool,
}

pub struct RunResult {
    pub exit_code: i32,
    pub command_id: String,
    pub duration_ms: u64,
}

pub fn run_command(
    repo_root: &Path,
    argv: Vec<String>,
    options: RunOptions,
) -> Result<RunResult, String> {
    if argv.is_empty() {
        return Err("run requires a command after --".into());
    }

    let store = Store::open(repo_root).map_err(|err| err.to_string())?;
    let session = resolve_session(&store, options.session_id)?;
    let events = store.read_events(&session.session_id).map_err(|err| err.to_string())?;
    verify_event_chain(&events).map_err(|err| err.to_string())?;

    let command_id = new_command_id();
    let started_at = time::OffsetDateTime::now_utc();
    let cwd =
        repo_root.canonicalize().unwrap_or_else(|_| repo_root.to_path_buf()).display().to_string();

    store
        .append_typed_event(
            &session.session_id,
            EventKind::CommandStarted,
            EventPayload::CommandStarted(CommandStartedPayload {
                command_id: command_id.clone(),
                argv: argv.clone(),
                cwd,
                started_at,
            }),
        )
        .map_err(|err| err.to_string())?;

    let timer = Instant::now();
    let (exit_code, stdout_bytes, stderr_bytes) =
        execute_command(repo_root, &argv, options.use_shell, options.capture_output)?;
    let duration_ms = timer.elapsed().as_millis() as u64;

    let mut stdout_artifact = None;
    let mut stderr_artifact = None;
    if options.capture_output {
        if !stdout_bytes.is_empty() {
            let artifact = store
                .store_artifact(&session.session_id, &stdout_bytes, "text/plain", "command.stdout")
                .map_err(|err| err.to_string())?;
            stdout_artifact = Some(artifact.artifact_id);
        }
        if !stderr_bytes.is_empty() {
            let artifact = store
                .store_artifact(&session.session_id, &stderr_bytes, "text/plain", "command.stderr")
                .map_err(|err| err.to_string())?;
            stderr_artifact = Some(artifact.artifact_id);
        }
    }

    store
        .append_typed_event(
            &session.session_id,
            EventKind::CommandFinished,
            EventPayload::CommandFinished(CommandFinishedPayload {
                command_id: command_id.clone(),
                exit_code,
                duration_ms,
                stdout_artifact,
                stderr_artifact,
            }),
        )
        .map_err(|err| err.to_string())?;

    Ok(RunResult { exit_code, command_id, duration_ms })
}

fn execute_command(
    repo_root: &Path,
    argv: &[String],
    use_shell: bool,
    capture_output: bool,
) -> Result<(i32, Vec<u8>, Vec<u8>), String> {
    let mut command = if use_shell {
        let script = argv.join(" ");
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(script);
        cmd
    } else {
        let mut cmd = Command::new(&argv[0]);
        cmd.args(&argv[1..]);
        cmd
    };

    command.current_dir(repo_root);
    command.stdin(Stdio::inherit());

    if capture_output {
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
    } else {
        command.stdout(Stdio::inherit());
        command.stderr(Stdio::inherit());
    }

    let mut child =
        command.spawn().map_err(|err| format!("failed to run `{}`: {err}", argv.join(" ")))?;

    let (stdout_bytes, stderr_bytes) = if capture_output {
        let stdout = child.stdout.take().expect("stdout pipe");
        let stderr = child.stderr.take().expect("stderr pipe");
        let limit = default_output_limit();
        let stdout_handle = thread::spawn(move || capture_stream(stdout, io::stdout(), limit));
        let stderr_handle = thread::spawn(move || capture_stream(stderr, io::stderr(), limit));
        let stdout_bytes = stdout_handle
            .join()
            .map_err(|_| "stdout thread panicked".to_string())?
            .map_err(|err| err.to_string())?;
        let stderr_bytes = stderr_handle
            .join()
            .map_err(|_| "stderr thread panicked".to_string())?
            .map_err(|err| err.to_string())?;
        (stdout_bytes, stderr_bytes)
    } else {
        (Vec::new(), Vec::new())
    };

    let status =
        child.wait().map_err(|err| format!("failed waiting for `{}`: {err}", argv.join(" ")))?;

    Ok((exit_status_code(status), stdout_bytes, stderr_bytes))
}

fn capture_stream<R, W>(mut reader: R, mut writer: W, limit: u64) -> io::Result<Vec<u8>>
where
    R: Read,
    W: Write,
{
    let mut buffer = LimitedBuffer::new(limit);
    let mut chunk = [0u8; 8192];
    loop {
        let read = reader.read(&mut chunk)?;
        if read == 0 {
            break;
        }
        writer.write_all(&chunk[..read])?;
        writer.flush()?;
        buffer.push(&chunk[..read]);
    }
    Ok(buffer.into_bytes())
}

fn exit_status_code(status: ExitStatus) -> i32 {
    status.code().unwrap_or(1)
}

fn resolve_session(
    store: &Store,
    session_id: Option<String>,
) -> Result<attestack_core::Session, String> {
    if let Some(session_id) = session_id {
        return store.read_session(store.session_dir(&session_id)).map_err(|err| err.to_string());
    }

    store
        .find_open_session()
        .map_err(|err| err.to_string())?
        .ok_or_else(|| "no open session".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use attestack_store::{new_session, Store};
    use tempfile::tempdir;

    #[test]
    fn run_records_success_and_failure() {
        let dir = tempdir().unwrap();
        Store::init(dir.path()).unwrap();
        let store = Store::open(dir.path()).unwrap();
        let identity_id = store.config().unwrap().default_identity_id;
        let session = new_session("demo".into(), identity_id);
        store.create_session(session.clone()).unwrap();

        let ok = run_command(
            dir.path(),
            vec!["echo".into(), "hello".into()],
            RunOptions { session_id: None, capture_output: true, use_shell: false },
        )
        .unwrap();
        assert_eq!(ok.exit_code, 0);

        let fail = run_command(
            dir.path(),
            vec!["false".into()],
            RunOptions { session_id: None, capture_output: false, use_shell: false },
        )
        .unwrap();
        assert_ne!(fail.exit_code, 0);

        let events = store.read_events(&session.session_id).unwrap();
        assert_eq!(events.len(), 5);
    }
}
