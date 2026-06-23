# Aider + Attestack

[Aider](https://aider.chat/) edits code via the terminal. Wrap each coding session with Attestack so commits, test runs, and notes are hash-chained and exportable.

## Pattern

Use `scripts/agent-session.sh` from the Attestack repo (or copy it into your project):

```bash
/path/to/attestack/scripts/agent-session.sh start "Aider: refactor billing"
aider --message "Refactor the billing webhook handler"
/path/to/attestack/scripts/agent-session.sh note "Aider applied webhook refactor"
/path/to/attestack/scripts/agent-session.sh finish
```

`finish` runs `stop`, `bundle create`, and `verify`.

## Wrapper script

From your project root:

```bash
/path/to/attestack/examples/harnesses/aider/run-with-evidence.sh \
  "Refactor billing webhook" \
  aider --message "Refactor the billing webhook handler"
```

## Recording test commands

Run tests through Attestack during the session:

```bash
attestack run -- npm test
attestack run -- cargo test
```

Aider's own git commits are captured on `attestack snapshot` (call before `stop` or use `finish` which snapshots via stop flow when configured).

## Manual session flow

```bash
attestack init
attestack start "Aider session"
aider
attestack note "Reviewed Aider diff for auth module"
attestack snapshot
attestack stop
attestack bundle create
attestack verify .attestack/bundles/*.attestack.zip --strict
```

## Hash-only agent events (optional)

If you script Aider tool metadata post-hoc:

```bash
attestack agent decision \
  --summary "Aider chose to edit src/billing.rs" \
  --rationale "User asked to refactor webhook handler"
```

## Verify

```bash
attestack pr-summary --bundle .attestack/bundles/*.attestack.zip
```

## See also

- [Agent setup guide](../../../docs/agent-setup.md) for MCP-based editors
- [Harness integrations](../../../docs/harness-integrations.md)
