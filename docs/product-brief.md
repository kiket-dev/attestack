# Product Brief

## One-Line Description

Attestack is a local-first proof layer for AI-assisted software work.

## Target User

Start with individual developers and small engineering teams that use AI coding assistants, CLI agents, or automation but still want a reviewable record of what happened.

Secondary users:

- Staff engineers and team leads reviewing AI-assisted changes.
- Security engineers responsible for secure SDLC evidence.
- Platform teams that need release evidence.
- Compliance teams that eventually need exportable proof.

## Core Job

Help a developer create a trustworthy, portable record of a development session or CI run without changing their normal workflow too much.

## User Promise

Developers should be able to say:

> I can show what changed, what commands ran, what evidence was captured, and whether the record was tampered with.

## Business Promise

Teams should be able to say:

> We can adopt AI-assisted development without losing accountability, release evidence, or auditability.

## Current capabilities

Attestack is intentionally local and explicit:

- `attestack init` creates local configuration.
- `attestack start` opens a session.
- `attestack run -- <command>` records command execution.
- `attestack note` records human context.
- `attestack snapshot` captures Git state and diff hashes.
- `attestack stop` closes the session.
- `attestack bundle create` exports a portable evidence bundle.
- `attestack verify <bundle>` verifies integrity offline.
- `attestack agent` and `attestack-mcp` record AI tool activity with hash-first privacy defaults.

## Adoption ladder

1. Explicit CLI sessions.
2. Git hooks for commit and diff evidence.
3. CI integration for PR and release evidence.
4. Shell integration for transparent command capture.
5. SDKs, MCP adapter, and transcript importers for AI agents such as Claude Code, Copilot CLI, and OpenCode.
6. Optional hosted team layer.

## Positioning

Position Attestack as developer infrastructure:

- A flight recorder for AI-assisted development.
- Local proof logs for software work.
- Offline-verifiable evidence for PRs and releases.

## Differentiation

Attestack should be:

- Local-first and useful without an account.
- CLI-first and developer friendly.
- Built on boring cryptography and transparent file formats.
- Practical before it is complete.
- Careful about privacy and redaction.

## Future Monetization

Open-source local usage should stay free. Paid features can appear later where teams naturally need shared infrastructure:

- Hosted encrypted bundle storage.
- Team search and retention.
- GitHub/GitLab app.
- Policy gates for PRs and releases.
- Organization identity and signing.
- SSO, RBAC, audit exports.
