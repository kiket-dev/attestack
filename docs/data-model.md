# Data Model

## Goals

The data model must support:

- Local append-only event recording.
- Tamper-evident session history.
- Offline verification.
- Portable evidence bundles.
- Future SDK, CI, and agent integrations.

## Identifiers

Use opaque, URL-safe IDs:

- `session_id`: `ses_<timestamp>_<random>`
- `event_id`: `evt_<session_id>_<sequence>`
- `artifact_id`: `art_<digest-prefix>`
- `identity_id`: `id_<public-key-fingerprint>`

Avoid embedding user names or repo names in IDs.

## Session

```json
{
  "schema_version": "attestack.session.v1",
  "session_id": "ses_20260622_abc123",
  "title": "fix billing webhook",
  "created_at": "2026-06-22T13:00:00Z",
  "closed_at": null,
  "repo": {
    "root": "/path/to/repo",
    "vcs": "git",
    "initial_head": "abc123"
  },
  "identity_id": "id_abc123",
  "status": "open"
}
```

Store local paths in session files, but allow redaction or omission during bundle export.

## Event Envelope

All event kinds share the same envelope:

```json
{
  "schema_version": "attestack.event.v1",
  "event_id": "evt_ses_20260622_abc123_000001",
  "session_id": "ses_20260622_abc123",
  "sequence": 1,
  "recorded_at": "2026-06-22T13:00:01Z",
  "kind": "session.started",
  "payload": {},
  "prev_event_hash": null,
  "event_hash": "sha256:...",
  "signature": {
    "alg": "Ed25519",
    "key_id": "id_abc123",
    "value": "base64..."
  }
}
```

`event_hash` is computed over the canonical event with `event_hash` and `signature` omitted.

## Event kinds

Core event kinds:

- `session.started`
- `session.note_added`
- `command.started`
- `command.finished`
- `git.snapshot`
- `artifact.attached`
- `bundle.created`
- `session.stopped`

AI agent event kinds (hash-first by default):

- `ai.tool_call` — tool name plus optional input/output hashes and summary
- `ai.decision` — decision summary and optional rationale
- `ai.approval` — approval or rejection of generated work
- `ai.prompt` — prompt content hash and optional model id
- `ai.response` — response content hash and optional model id

Future event kinds:

- `policy.evaluated`
- `ci.job_recorded`
- `release.artifact_recorded`
- `redaction.applied`

## Command Events

`command.started`:

```json
{
  "command_id": "cmd_abc123",
  "argv": ["pnpm", "test"],
  "cwd": "/path/to/repo",
  "started_at": "2026-06-22T13:01:00Z"
}
```

`command.finished`:

```json
{
  "command_id": "cmd_abc123",
  "exit_code": 0,
  "duration_ms": 1234,
  "stdout_artifact": "art_abc123",
  "stderr_artifact": "art_def456"
}
```

By default, command output may be stored locally but should be excluded from exported bundles unless the user opts in.

## AI Agent Events

`ai.tool_call`:

```json
{
  "tool": "read_file",
  "input_hash": "sha256:...",
  "output_hash": "sha256:...",
  "summary": "Read auth module"
}
```

`ai.decision`:

```json
{
  "summary": "Reuse JWT middleware",
  "rationale": "Matches existing patterns"
}
```

`ai.approval`:

```json
{
  "subject": "Generated SQL migration",
  "approved": true
}
```

`ai.prompt` / `ai.response`:

```json
{
  "content_hash": "sha256:...",
  "model": "claude-sonnet"
}
```

Raw prompts and responses are not stored by default — only content hashes unless explicitly attached as artifacts.

## Git Snapshot

```json
{
  "repo_root_hash": "sha256:...",
  "head": "abc123",
  "branch": "main",
  "dirty": true,
  "staged_diff_hash": "sha256:...",
  "unstaged_diff_hash": "sha256:...",
  "untracked_files_hash": "sha256:..."
}
```

Diff hashes are computed from `git diff`, `git diff --staged`, and a stable sorted list of untracked files.

## Artifact

Artifacts are files referenced by events or bundles.

```json
{
  "artifact_id": "art_abc123",
  "path": "artifacts/art_abc123",
  "sha256": "abc123...",
  "size_bytes": 1024,
  "media_type": "text/plain",
  "purpose": "command.stdout",
  "redacted": false
}
```

## Bundle Manifest

```json
{
  "schema_version": "attestack.bundle.v1",
  "bundle_id": "bun_abc123",
  "created_at": "2026-06-22T13:30:00Z",
  "sessions": ["ses_20260622_abc123"],
  "files": [
    {
      "path": "sessions/ses_20260622_abc123/events.jsonl",
      "sha256": "abc123...",
      "size_bytes": 4096,
      "media_type": "application/jsonl"
    }
  ],
  "signature": {
    "alg": "Ed25519",
    "key_id": "id_abc123",
    "value": "base64..."
  }
}
```

## Compatibility

Use explicit schema versions. Fail closed on unknown major schema versions and warn on unknown minor-compatible fields.

Do not add compatibility shims for unshipped schemas during early development. Replace them until the first public release.
