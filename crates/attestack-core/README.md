# attestack-core

Core types, canonical JSON hashing, hash-chain verification, Ed25519 signing, and bundle manifest primitives for Attestack.

## Responsibilities

- Session, event, artifact, bundle, and identity types
- Deterministic event hashing via `serde_jcs`
- Hash chain validation (`verify_event_chain`)
- Bundle manifest signing and verification

## Crates

- **`attestack-store`** — local `.attestack/` persistence
- **`attestack-cli`** — user-facing CLI binary

This crate is intended to stay free of filesystem and CLI dependencies so SDKs and verifiers can depend on it directly.
