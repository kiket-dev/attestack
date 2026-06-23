# Security Model

## Security Goals

Attestack should provide:

- Tamper evidence for local session logs.
- Offline verification of exported bundles.
- Clear identity binding for signatures.
- Privacy-preserving defaults.
- Safe handling of untrusted bundles.

## Non-Goals

Attestack does not prove:

- The code is correct.
- The code is secure.
- A human truly reviewed something unless the review event was signed by that identity.
- A command was not also run outside Attestack.
- The local machine was not compromised during recording.

## Threat Model

Attestack defends against:

- Accidental modification of event logs.
- Post-hoc editing of a session or bundle.
- Reordering, deleting, or inserting events without detection.
- Bundle file substitution.
- Basic misrepresentation of command results inside a sealed bundle.

Attestack does not fully defend against:

- Malware on the developer machine.
- A malicious user who controls the signing key.
- False notes intentionally entered by a user.
- Terminal commands run outside Attestack.
- AI tools that hide or misreport their own behavior.

## Hash Chain

Each event links to the previous event hash. Verification recomputes all hashes and checks the chain.

This detects:

- Deleted events.
- Reordered events.
- Modified event payloads.
- Inserted events without recomputing and resigning subsequent events.

## Signatures

Use Ed25519 signatures.

Sign:

- Events, if per-event signing is enabled.
- The final session seal.
- Bundle manifests.

Bundle manifests are signed by default. The event hash chain is always present.

## Key Handling

Do not store private keys in the project repository.

Preferred order:

1. OS keychain.
2. User-local encrypted key file.
3. User-local unencrypted key file only with explicit warning.

Public identity metadata can be stored in `.attestack/identities/`.

## Privacy Defaults

Attestack should not silently capture:

- Environment variables.
- Secret files.
- Full command output into shareable bundles.
- Full diffs.
- AI prompts or transcripts.
- Clipboard contents.

The user may opt in to capturing more evidence. Bundle creation should clearly state what is included.

## Redaction

Path redaction options:

- `--redact-paths` replaces absolute paths with repo-relative paths or placeholders.
- Command output is excluded from bundles unless explicitly included.
- Full diffs are excluded unless explicitly included.

Planned enhancements:

- Pattern-based redaction.
- Secret scanner integration.
- Redaction receipts that prove a bundle was derived from a fuller local record.

## Verification Safety

Verification must treat bundles as untrusted input.

Rules:

- Never execute bundle contents.
- Defend against zip slip path traversal.
- Enforce maximum file sizes.
- Enforce maximum event counts unless overridden.
- Parse JSON with strict schemas.
- Fail closed on malformed signatures or hashes.

## Transparency Logs Later

Transparency logs are optional.

Future options:

- Sigstore/Rekor for CI and release bundles.
- Witnessed append-only logs for organizations.
- Timestamping services for stronger time evidence.

## Security Review Checklist

Before the first public release:

- Fuzz bundle verification inputs.
- Add test vectors for canonicalization and signatures.
- Test zip path traversal defenses.
- Test tampered event logs.
- Test missing and duplicated sequence numbers.
- Test unknown schema versions.
- Review key storage behavior on Linux, macOS, and Windows.
