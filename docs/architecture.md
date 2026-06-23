# Architecture

## Design Principles

- Local-first: every core feature works without a server.
- Verifiable: bundles can be checked offline.
- Transparent: data formats are inspectable and documented.
- Private by default: raw sensitive content is not captured unless the user explicitly opts in.
- Incremental: explicit CLI capture comes before shell, editor, or agent integrations.

## Recommended Implementation Stack

Start with Rust for the CLI and core libraries.

Reasons:

- Single static binary distribution.
- Good performance and portability.
- Strong type system for schema and verifier correctness.
- Mature cryptography crates.
- Good fit for file-system-heavy local tooling.

Initial crates:

- `attestack-core`: event types, canonicalization, hashing, signatures, bundle manifest types.
- `attestack-store`: local `.attestack/` file store, append-only event writing, locking, atomic writes.
- `attestack-cli`: command parsing and user-facing CLI.
- `attestack-mcp`: MCP server for agent integrations.

Additional packages (planned or in progress):

- TypeScript SDK for AI tools and build scripts.
- GitHub Action wrapper.
- Agent transcript importers for tools such as Claude Code, Copilot CLI, and OpenCode.

## Agent integration

Agent integrations sit on top of the local event store, verifier, and MCP server rather than bypassing them. The core schema stays vendor-neutral: tool-specific code can parse Claude Code, Copilot CLI, OpenCode, or other transcript formats, but `attestack-core` only exposes normalized AI event types.

Default agent metadata is privacy-preserving. Store prompt, response, and transcript content as hashes and metadata unless the user explicitly opts into raw content capture.

## Local Repository Layout

```text
.attestack/
  config.toml
  identities/
    default.public.json
  sessions/
    <session-id>/
      session.json
      events.jsonl
      checkpoints/
      reports/
  bundles/
    <bundle-name>.attestack.zip
```

Private keys should not be stored inside the repository by default. Use the OS keychain when available, or a user-local directory such as `~/.attestack/keys/`.

## Core Flow

1. User runs `attestack start`.
2. CLI captures initial Git state if inside a Git repo.
3. CLI creates a session file and first hash-chained event.
4. User records commands through `attestack run -- <command>`.
5. User adds notes and snapshots as needed.
6. User runs `attestack stop`.
7. CLI seals the session and writes a report.
8. User exports a bundle.
9. Verifier checks manifest, hashes, signatures, and event chain.

## Event Store

Use append-only JSONL for the session event log. Every event includes:

- Stable schema version.
- Session ID.
- Monotonic sequence number.
- Timestamp.
- Event kind.
- Payload.
- Previous event hash.
- Current event hash.
- Optional signature.

The current event hash is computed over a canonical representation of the event without the signature.

## Canonicalization

Use deterministic JSON canonicalization. Prefer an existing RFC 8785 implementation if it is trustworthy and maintained. If none is suitable, implement the smallest required subset and cover it with cross-platform test vectors.

## Signatures

Use Ed25519 for local signing.

Do not invent signature schemes. Keep the signed payload small and explicit:

- Event canonical bytes for event signatures.
- Bundle manifest canonical bytes for bundle signatures.

Keyless Sigstore signing can be added later for CI and release bundles.

## Git Integration

Git capture relies on Git commands and hashes:

- Current branch.
- HEAD commit.
- Worktree dirty state.
- Diff hash.
- Staged diff hash.
- Untracked file list hash.

Avoid storing full diffs by default. Offer `--include-diff` for explicit capture.

## Bundle Format

An Attestack bundle is a zip archive:

```text
bundle.json
sessions/<session-id>/session.json
sessions/<session-id>/events.jsonl
sessions/<session-id>/report.md
artifacts/<artifact-id>
signatures/
```

`bundle.json` is the manifest. It lists every included file, its SHA-256 digest, size, media type, and purpose.

## Verification

Verifier steps:

1. Parse bundle manifest.
2. Check manifest schema version.
3. Recompute each file digest.
4. Recompute event hashes in order.
5. Check sequence numbers and previous hash links.
6. Verify signatures when present.
7. Emit a human-readable summary and machine-readable result.

Verification should never execute bundle contents.

## Hosted Layer Later

The optional hosted product should not be required by the CLI. Its job is to provide shared value:

- Upload bundles.
- Search sessions.
- Retain evidence.
- Bind organization identities.
- Enforce policies.
- Integrate with GitHub/GitLab.

The local bundle format should remain useful without the hosted service.
