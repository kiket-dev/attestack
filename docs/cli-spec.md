# CLI Specification

## CLI Name

Primary binary: `attestack`

Short alias may be added in a future release.

## Global Requirements

- All commands must be non-interactive by default.
- Every command should support `--help`.
- Errors should be actionable and include the path or command that failed.
- Machine-readable output should be available through `--json` for commands that produce structured results.
- The CLI must never upload data to remote services.

## Commands

### `attestack init`

Initializes Attestack in the current Git repository or current directory.

Behavior:

- Create `.attestack/config.toml`.
- Create `.attestack/sessions/`.
- Create `.attestack/bundles/`.
- Create or link a local signing identity.
- Add `.attestack/runs/`, `.attestack/tmp/`, and `.attestack/keys/` to `.gitignore` only if the user passes `--update-gitignore`.

Flags:

- `--force`: overwrite existing config.
- `--update-gitignore`: update `.gitignore`.
- `--json`: emit structured result.

### `attestack start <title>`

Starts a new session.

Behavior:

- Fail if another session is open unless `--allow-parallel` is set.
- Capture initial Git snapshot when inside a Git repo.
- Write `session.started`.
- Print session ID and next suggested commands.

Flags:

- `--allow-parallel`
- `--no-git`
- `--json`

### `attestack status`

Shows current session status.

Behavior:

- Print open session, event count, last event, and current Git state.
- Exit with code `0` when a session is open.
- Exit with code `2` when no session is open.

Flags:

- `--json`

### `attestack note <text>`

Adds a human note to the active session.

Behavior:

- Write `session.note_added`.
- Do not interpret Markdown or execute content.

Flags:

- `--session <id>`
- `--json`

### `attestack run -- <command...>`

Runs a command and records start, finish, exit code, duration, and optional output artifacts.

Behavior:

- Write `command.started`.
- Execute the command.
- Capture stdout and stderr to local artifact files.
- Stream stdout and stderr to the terminal while capturing.
- Write `command.finished`.
- Return the wrapped command exit code.

Flags:

- `--session <id>`
- `--capture-output`: include output artifacts in the session store.
- `--no-capture-output`: record metadata only.
- `--shell`: execute through the user's shell.
- `--json`: emit metadata after command completes.

Default:

- Capture metadata.
- Store output locally.
- Exclude output from bundles unless explicitly requested during bundle creation.

### `attestack snapshot`

Captures a Git snapshot for the active session.

Behavior:

- Collect HEAD, branch, dirty status, staged diff hash, unstaged diff hash, and untracked file list hash.
- Write `git.snapshot`.

Flags:

- `--include-diff`: store full diff as local artifact.
- `--session <id>`
- `--json`

### `attestack stop`

Closes the active session.

Behavior:

- Capture final Git snapshot.
- Write `session.stopped`.
- Mark session as closed.
- Generate a Markdown report.

Flags:

- `--session <id>`
- `--no-report`
- `--json`

### `attestack report [session-id]`

Generates or prints a human-readable report.

Behavior:

- Summarize timeline, commands, exit codes, notes, snapshots, and verification status.
- Do not include sensitive artifacts by default.

Flags:

- `--output <path>`
- `--include-command-output`
- `--json`

### `attestack bundle create [session-id]`

Creates a portable evidence bundle.

Behavior:

- Validate session chain before bundling.
- Write bundle manifest.
- Include session metadata, event log, report, and selected artifacts.
- Sign the bundle manifest.
- Write `.attestack/bundles/<slug>.attestack.zip`.

Flags:

- `--output <path>`
- `--include-diff`
- `--include-command-output`
- `--redact-paths`
- `--json`

### `attestack verify <bundle-or-session-path>`

Verifies a bundle or local session.

Behavior:

- Never execute files.
- Verify schema versions.
- Verify file digests.
- Verify event hash chain.
- Verify signatures when present.
- Print clear pass/fail output.

Exit codes:

- `0`: verified.
- `1`: verification failed.
- `2`: invalid input or unsupported schema.

Flags:

- `--public-key <path>`: verify bundle signatures with a specific public identity file.
- `--json`
- `--strict`: fail when verification succeeds with warnings (for example, unsigned bundle manifests).

### `attestack agent`

Records AI agent activity into the active session.

Subcommands:

- `tool-call` — record tool name and optional input/output hashes
- `decision` — record a decision summary
- `approval` — record approval or rejection
- `prompt` — record a prompt content hash
- `response` — record a response content hash

All subcommands support `--session <id>` and `--json`.

### `attestack ci`

CI workflow helpers.

Subcommands:

- `ci start` — initialize (if needed) and start a CI session
- `ci run -- <command...>` — run a command inside the active CI session
- `ci finish` — stop the session and export a redacted bundle

Environment variables used for session titles: `GITHUB_WORKFLOW`, `GITHUB_RUN_ID`, `CI_PIPELINE_ID`, and `CI`.

### `attestack doctor`

Checks local installation and repository readiness.

Behavior:

- Check config.
- Check signing identity.
- Check Git availability.
- Check active session consistency.

Flags:

- `--json`

## Output Style

Human output should be short and useful:

```text
Session started
ID: ses_20260622_abc123
Store: .attestack/sessions/ses_20260622_abc123

Next:
  attestack run -- pnpm test
  attestack note "Reviewed auth path"
  attestack stop
```

## Privacy Defaults

By default:

- Store command output locally.
- Do not include command output in exported bundles.
- Do not include full diffs in exported bundles.
- Do not capture environment variables.
- Do not capture AI prompts automatically.

Users can opt in to richer capture.
