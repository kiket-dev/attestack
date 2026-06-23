# Use Cases

Attestack helps teams answer **what happened**, **what was verified**, and **whether the record was tampered with** — without requiring a SaaS account or always-on surveillance.

## Individual developer with AI assistants

You use Cursor, Claude Code, Copilot, or other agents to implement features. Attestack records:

- Commands you ran (`attestack run`)
- Human review notes (`attestack note`)
- Git state at start and end (`snapshot`)
- Agent tool calls and decisions (`attestack agent …`)

When you finish, export a signed bundle and attach it to a PR or ticket.

**Outcome:** Reviewers see structured evidence instead of reconstructing work from chat logs.

## Tech lead reviewing AI-assisted changes

A developer attaches an `.attestack.zip` bundle to a pull request. You run:

```bash
attestack verify path/to/bundle.attestack.zip
attestack report --output review.md
```

**Outcome:** You verify the event chain and signature offline, then read a human-friendly report.

## Security and compliance evidence

Security teams need proof that tests ran and changes were reviewed before merge. Attestack captures:

- Test command exit codes
- Git HEAD and dirty-state hashes
- Optional bundle export with path redaction

**Outcome:** Portable, tamper-evident artifacts suitable for audit trails. Attestack does not claim code correctness — it records what happened.

## CI pipeline evidence

GitHub Actions (or other CI) runs tests inside an Attestack session, exports a bundle, and uploads it as a workflow artifact. See [CI integration](ci-integration.md).

**Outcome:** Reproducible CI evidence linked to workflow run IDs.

## Incident investigation

After a production incident, reconstruct what commands and snapshots were captured during a hotfix session.

**Outcome:** Timeline with hash chain integrity; tampering is detectable.

## Open-source maintainers

Demonstrate reproducible contributor workflows: init → session → tests → bundle → verify.

**Outcome:** Transparent evidence for security-conscious adopters evaluating the project.

## What Attestack is not

- Not a replacement for code review or static analysis
- Not automatic capture of private AI transcripts (opt-in only)
- Not proof that generated code is secure or correct
