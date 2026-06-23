# Scenarios

Step-by-step workflows for common Attestack situations.

## Scenario 1: AI-assisted feature work

**Goal:** Record a full development session and share evidence with reviewers.

```bash
attestack init
attestack start "auth middleware refactor"
attestack agent tool-call --tool read_file --input-hash sha256:abc… --summary "Read auth module"
attestack run -- cargo test
attestack note "Reviewed token validation path manually"
attestack agent decision --summary "Use existing JWT helper" --rationale "Matches repo conventions"
attestack snapshot
attestack stop
attestack bundle create --redact-paths
attestack verify .attestack/bundles/*.attestack.zip
```

Attach the bundle to your PR description or upload as a CI artifact.

## Scenario 2: Reviewer verifying a bundle

**Goal:** Confirm evidence was not tampered with.

```bash
attestack verify ./evidence/fix-auth.attestack.zip
attestack report --output /tmp/review.md
```

If verification fails, the CLI lists digest mismatches or hash-chain breaks. Do not trust the bundle.

## Scenario 3: CI test run with evidence export

**Goal:** Capture CI test execution in a signed bundle.

See [CI integration](ci-integration.md) and `examples/github-actions/attestack-evidence.yml`.

Summary:

1. `attestack ci start` — opens a session titled from `GITHUB_*` env vars
2. `attestack ci run -- npm test` — records the test command
3. `attestack ci finish` — stops session, creates bundle, uploads artifact

## Scenario 4: Agent via MCP (Cursor, Claude Desktop, etc.)

**Goal:** Let an MCP-capable agent append notes and tool-call hashes without shell wrappers.

1. Build `attestack-mcp`
2. Add to your MCP config (see `examples/mcp/cursor-mcp.json`)
3. Agent calls `attestack_note`, `attestack_agent_tool_call`, etc.

The MCP server never executes arbitrary shell commands from agent input.

## Scenario 5: Non-Git project

**Goal:** Use Attestack outside a Git repository.

```bash
attestack init
attestack start "config migration" --no-git
attestack run -- make check
attestack note "Validated migration script output"
attestack stop
```

`snapshot` will fail with a clear message; other commands work normally.

## Scenario 6: Parallel sessions (advanced)

```bash
attestack start "experiment A" --allow-parallel
attestack start "experiment B" --allow-parallel
attestack note "A only" --session ses_…
```

Use `--session <id>` to target a specific open session.

## Scenario 7: Tamper detection demo

1. Create and verify a valid bundle
2. Copy the zip and edit `events.jsonl` inside
3. Run `attestack verify` on the copy — verification must fail

This demonstrates fail-closed verification.

## Scenario 8: Health check before a demo

```bash
attestack doctor --json
```

Fix any `[fail]` items before recording a session (missing identity, broken chain, etc.).
