use std::io;
use std::path::PathBuf;

use attestack_core::{Event, EventKind, EventPayload, Session, VerificationReport};

use crate::bundle::verify_local_session;
use crate::Store;

#[derive(Debug, Clone, Default)]
pub struct ReportOptions {
    pub include_command_output: bool,
}

pub struct SessionReport {
    pub markdown: String,
}

pub fn render_session_report(
    session: &Session,
    events: &[Event],
    verification: Option<&VerificationReport>,
    options: &ReportOptions,
) -> SessionReport {
    let mut lines = vec![
        format!("# {}", session.title),
        String::new(),
        "## Summary".into(),
        format!("- Session ID: `{}`", session.session_id),
        format!("- Status: {:?}", session.status),
        format!("- Created: {}", session.created_at),
    ];

    if let Some(closed_at) = session.closed_at {
        lines.push(format!("- Closed: {closed_at}"));
    }
    if let Some(repo) = &session.repo {
        lines.push(format!("- Repository: {}", repo.root));
        if let Some(head) = &repo.initial_head {
            lines.push(format!("- Initial HEAD: `{head}`"));
        }
    }
    lines.push(format!("- Events: {}", events.len()));
    lines.push(String::new());

    lines.push("## Commands".into());
    let commands: Vec<_> = events
        .iter()
        .filter_map(|event| match &event.payload {
            EventPayload::CommandFinished(payload) => Some((event.recorded_at, payload)),
            _ => None,
        })
        .collect();
    if commands.is_empty() {
        lines.push("- None recorded".into());
    } else {
        for (recorded_at, payload) in commands {
            let mut line = format!(
                "- `{recorded_at}` `{}` exit {} ({} ms)",
                payload.command_id, payload.exit_code, payload.duration_ms
            );
            if options.include_command_output {
                if let Some(stdout) = &payload.stdout_artifact {
                    line.push_str(&format!(", stdout={stdout}"));
                }
                if let Some(stderr) = &payload.stderr_artifact {
                    line.push_str(&format!(", stderr={stderr}"));
                }
            }
            lines.push(line);
        }
    }
    lines.push(String::new());

    lines.push("## Notes".into());
    let notes: Vec<_> = events
        .iter()
        .filter_map(|event| match &event.payload {
            EventPayload::SessionNoteAdded(payload) => Some(payload.text.clone()),
            _ => None,
        })
        .collect();
    if notes.is_empty() {
        lines.push("- None recorded".into());
    } else {
        for note in notes {
            lines.push(format!("- {note}"));
        }
    }
    lines.push(String::new());

    lines.push("## Git Snapshots".into());
    let snapshots: Vec<_> = events
        .iter()
        .filter_map(|event| match &event.payload {
            EventPayload::GitSnapshot(payload) => Some(payload),
            _ => None,
        })
        .collect();
    if snapshots.is_empty() {
        lines.push("- None recorded".into());
    } else {
        for snapshot in snapshots {
            let head = snapshot.head.as_deref().unwrap_or("unknown");
            let branch = snapshot.branch.as_deref().unwrap_or("unknown");
            lines.push(format!("- HEAD `{head}` on `{branch}` (dirty={})", snapshot.dirty));
        }
    }
    lines.push(String::new());

    lines.push("## Timeline".into());
    for event in events {
        lines.push(format!(
            "- `{}` seq {} — {}",
            event.recorded_at,
            event.sequence,
            event_kind_label(event.kind)
        ));
    }
    lines.push(String::new());

    lines.push("## Verification".into());
    match verification {
        Some(report) if report.verified => lines.push("- Event chain: verified".into()),
        Some(report) => {
            lines.push("- Event chain: failed".into());
            for error in &report.errors {
                lines.push(format!("  - {error}"));
            }
        }
        None => lines.push("- Event chain: not checked".into()),
    }

    SessionReport { markdown: lines.join("\n") }
}

pub struct PrSummaryOptions {
    pub bundle_path: Option<PathBuf>,
    pub bundle_sha256: Option<String>,
}

/// Compact Markdown suitable for pasting into a PR description.
pub fn render_pr_summary(
    session: &Session,
    events: &[Event],
    verification: Option<&VerificationReport>,
    options: &PrSummaryOptions,
) -> String {
    let mut lines = vec![
        "## Attestack evidence".into(),
        String::new(),
        format!("**Session:** `{}`", session.session_id),
        format!("**Title:** {}", session.title),
        format!("**Status:** {:?}", session.status),
    ];

    if let Some(repo) = &session.repo {
        lines.push(format!("**Repository:** {}", repo.root));
    }
    lines.push(String::new());

    lines.push("### Commands".into());
    let commands: Vec<_> = events
        .iter()
        .filter_map(|event| match &event.payload {
            EventPayload::CommandFinished(payload) => Some(payload),
            _ => None,
        })
        .collect();
    if commands.is_empty() {
        lines.push("- None recorded".into());
    } else {
        for payload in commands {
            lines.push(format!(
                "- exit **{}** in {} ms (`{}`)",
                payload.exit_code, payload.duration_ms, payload.command_id
            ));
        }
    }
    lines.push(String::new());

    let notes: Vec<_> = events
        .iter()
        .filter_map(|event| match &event.payload {
            EventPayload::SessionNoteAdded(payload) => Some(payload.text.as_str()),
            _ => None,
        })
        .collect();
    if !notes.is_empty() {
        lines.push("### Notes".into());
        for note in notes {
            lines.push(format!("- {note}"));
        }
        lines.push(String::new());
    }

    let snapshots: Vec<_> = events
        .iter()
        .filter_map(|event| match &event.payload {
            EventPayload::GitSnapshot(payload) => Some(payload),
            _ => None,
        })
        .collect();
    if !snapshots.is_empty() {
        lines.push("### Git".into());
        for snapshot in snapshots {
            let head = snapshot.head.as_deref().unwrap_or("unknown");
            let branch = snapshot.branch.as_deref().unwrap_or("unknown");
            lines.push(format!("- `{head}` on `{branch}` (dirty={})", snapshot.dirty));
        }
        lines.push(String::new());
    }

    lines.push("### Verification".into());
    match verification {
        Some(report) if report.verified => lines.push("- Local session chain: **verified**".into()),
        Some(report) => {
            lines.push("- Local session chain: **failed**".into());
            for error in &report.errors {
                lines.push(format!("  - {error}"));
            }
        }
        None => lines.push("- Local session chain: not checked".into()),
    }

    if let Some(path) = &options.bundle_path {
        lines.push(String::new());
        lines.push("### Verify bundle".into());
        lines.push("```bash".into());
        lines.push(format!("attestack verify {}", path.display()));
        lines.push("```".into());
        if let Some(digest) = &options.bundle_sha256 {
            lines.push(format!("Bundle SHA256: `{digest}`"));
        }
    }

    lines.join("\n")
}

fn event_kind_label(kind: EventKind) -> &'static str {
    match kind {
        EventKind::SessionStarted => "session.started",
        EventKind::SessionNoteAdded => "session.note_added",
        EventKind::CommandStarted => "command.started",
        EventKind::CommandFinished => "command.finished",
        EventKind::GitSnapshot => "git.snapshot",
        EventKind::ArtifactAttached => "artifact.attached",
        EventKind::BundleCreated => "bundle.created",
        EventKind::SessionStopped => "session.stopped",
        EventKind::AiToolCall => "ai.tool_call",
        EventKind::AiDecision => "ai.decision",
        EventKind::AiApproval => "ai.approval",
        EventKind::AiPrompt => "ai.prompt",
        EventKind::AiResponse => "ai.response",
    }
}

impl Store {
    pub fn resolve_session_for_report(&self, session_id: Option<String>) -> crate::Result<Session> {
        if let Some(session_id) = session_id {
            return self.read_session(self.session_dir(&session_id));
        }
        if let Some(open) = self.find_open_session()? {
            return Ok(open);
        }
        self.find_latest_closed_session()?.ok_or(crate::StoreError::NoOpenSession)
    }

    pub fn generate_session_report(
        &self,
        session: &Session,
        options: &ReportOptions,
    ) -> crate::Result<SessionReport> {
        let events = self.read_events(&session.session_id)?;
        let verification = verify_local_session(&self.session_dir(&session.session_id)).ok();
        Ok(render_session_report(session, &events, verification.as_ref(), options))
    }

    pub fn generate_pr_summary(
        &self,
        session: &Session,
        options: &PrSummaryOptions,
    ) -> crate::Result<String> {
        let events = self.read_events(&session.session_id)?;
        let verification = verify_local_session(&self.session_dir(&session.session_id)).ok();
        Ok(render_pr_summary(session, &events, verification.as_ref(), options))
    }

    pub fn write_session_report(
        &self,
        session: &Session,
        events: &[Event],
        options: &ReportOptions,
    ) -> Result<PathBuf, io::Error> {
        let verification = verify_local_session(&self.session_dir(&session.session_id)).ok();
        let report = render_session_report(session, events, verification.as_ref(), options);
        let report_dir = self.session_dir(&session.session_id).join("reports");
        std::fs::create_dir_all(&report_dir)?;
        let report_path = report_dir.join("report.md");
        std::fs::write(&report_path, report.markdown)?;
        Ok(report_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{new_session, Store};
    use attestack_core::{EventKind, SessionNoteAddedPayload, SessionStoppedPayload};
    use tempfile::tempdir;

    #[test]
    fn report_includes_notes_and_commands() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path()).unwrap();
        let identity_id = store.config().unwrap().default_identity_id;
        let session = new_session("billing fix".into(), identity_id);
        store.create_session(session.clone()).unwrap();
        store
            .append_typed_event(
                &session.session_id,
                EventKind::SessionNoteAdded,
                EventPayload::SessionNoteAdded(SessionNoteAddedPayload {
                    text: "reviewed auth".into(),
                }),
            )
            .unwrap();
        store
            .append_typed_event(
                &session.session_id,
                EventKind::SessionStopped,
                EventPayload::SessionStopped(SessionStoppedPayload { report_path: None }),
            )
            .unwrap();

        let events = store.read_events(&session.session_id).unwrap();
        let report = render_session_report(&session, &events, None, &ReportOptions::default());
        assert!(report.markdown.contains("billing fix"));
        assert!(report.markdown.contains("reviewed auth"));
        assert!(report.markdown.contains("session.stopped"));
    }

    #[test]
    fn pr_summary_is_compact() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path()).unwrap();
        let identity_id = store.config().unwrap().default_identity_id;
        let session = new_session("PR demo".into(), identity_id);
        store.create_session(session.clone()).unwrap();
        store
            .append_typed_event(
                &session.session_id,
                EventKind::SessionNoteAdded,
                EventPayload::SessionNoteAdded(SessionNoteAddedPayload { text: "ship it".into() }),
            )
            .unwrap();

        let summary = store
            .generate_pr_summary(
                &session,
                &PrSummaryOptions { bundle_path: None, bundle_sha256: None },
            )
            .unwrap();
        assert!(summary.contains("## Attestack evidence"));
        assert!(summary.contains("ship it"));
        assert!(!summary.contains("## Timeline"));
    }
}
