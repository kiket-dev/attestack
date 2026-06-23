---
title: "Testing strategy"
description: "How Attestack is tested across platforms."
---
## Quality Bar

Attestack handles evidence, hashes, signatures, and local files. The implementation should be boring, deterministic, and heavily tested.

## Test Layers

### Unit Tests

Cover:

- Event hashing.
- Canonicalization.
- Sequence validation.
- Signature creation and verification.
- Bundle manifest digest calculation.
- Path redaction.
- CLI argument parsing.

### Golden Tests

Maintain fixtures for:

- A valid minimal session.
- A valid session with command events.
- A valid bundle.
- Tampered event payload.
- Deleted event.
- Reordered events.
- Invalid signature.
- Mismatched artifact digest.
- Unsupported schema version.

Golden tests should verify exact expected outputs where reasonable.

### Integration Tests

Exercise real CLI flows:

```bash
attestack init
attestack start "test session"
attestack run -- echo hello
attestack note "manual review"
attestack snapshot
attestack stop
attestack bundle create
attestack verify .attestack/bundles/*.attestack.zip
```

Use temporary directories and temporary Git repositories.

### Security Tests

Cover:

- Zip path traversal attempts.
- Oversized bundle files.
- Malformed JSON.
- Duplicate event sequence numbers.
- Missing previous hashes.
- Unknown signature algorithms.
- Non-canonical JSON encodings.

### Cross-Platform Tests

The CLI should work on:

- Linux.
- macOS.
- Windows.

Normalize path handling and line endings in hash inputs. Be explicit when hashing raw command output versus normalized metadata.

## CI

Initial CI should run:

- Format check.
- Lint.
- Unit tests.
- Integration tests on Linux.

Later CI should add macOS and Windows.

## Release Checks

Before a tagged release:

- Build binaries.
- Run all tests.
- Generate checksums.
- Sign release artifacts.
- Verify the generated artifacts from a clean checkout.

## Manual Smoke Test

Use this before publishing:

```bash
cargo build --release
tmpdir="$(mktemp -d)"
cd "$tmpdir"
git init
echo hello > README.md
git add README.md
git commit -m "init"
attestack init
attestack start "smoke"
attestack run -- echo ok
attestack snapshot
attestack stop
attestack bundle create
attestack verify .attestack/bundles/*.attestack.zip
```

Adjust for Windows once packaging exists.
