# OpenHands + Attestack

OpenHands runs agents in a sandbox with a tool loop. Attestack does **not** hook into OpenHands internals in v0.1 — wrap the run in a session and record shell steps with `attestack run --`.

## Pattern

```
attestack start → OpenHands run → attestack note / agent tool-call (optional) → attestack stop → bundle
```

The harness orchestrates; Attestack records. Never pass agent-generated shell through MCP.

## Quick start

From your **project root** (git repo):

```bash
/path/to/attestack/examples/harnesses/openhands/run-with-evidence.sh \
  "Fix the failing auth test" \
  openhands cli --task "Fix the failing auth test"
```

Or step by step:

```bash
attestack init   # once per repo
attestack start "OpenHands: fix auth test"

# Run OpenHands (CLI, Docker, or local install — your choice)
openhands cli --task "Fix the failing auth test"

attestack note "OpenHands session completed"
attestack snapshot
attestack stop
attestack bundle create
attestack verify .attestack/bundles/*.attestack.zip --strict
```

## Recording sandbox commands

If you control the command wrapper around OpenHands actions, prefix subprocess steps:

```bash
attestack run -- npm test
attestack run -- git diff --stat
```

Each recorded command appears in the bundle as `command.started` / `command.finished`.

## Optional: hash tool calls manually

When you log tool invocations from OpenHands hooks or post-run scripts:

```bash
INPUT_HASH="$(printf '%s' "$tool_input" | sha256sum | awk '{print "sha256:"$1}')"
OUTPUT_HASH="$(printf '%s' "$tool_output" | sha256sum | awk '{print "sha256:"$1}')"
attestack agent tool-call \
  --tool "$tool_name" \
  --input-hash "$INPUT_HASH" \
  --output-hash "$OUTPUT_HASH" \
  --summary "OpenHands $tool_name"
```

Prefer hashes over raw tool I/O. A thin Python shim calling the CLI is enough — no changes to `attestack-core`.

## Docker

```bash
export ATTESTACK_HOST_BIN="$(command -v attestack)"
docker run --rm -v "$PWD:/workspace" -v "$ATTESTACK_HOST_BIN:/usr/local/bin/attestack:ro" \
  -w /workspace \
  your-openhands-image \
  /workspace/examples/harnesses/openhands/run-with-evidence.sh "task title" openhands ...
```

Mount the repo so `.attestack/` persists on the host.

## Verify

```bash
attestack verify .attestack/bundles/*.attestack.zip --strict
attestack pr-summary --bundle .attestack/bundles/*.attestack.zip
```

## Future

A transcript importer for OpenHands export JSON may land in Track C Tier 3. Live hooks stay wrapper-first until export formats stabilize.
