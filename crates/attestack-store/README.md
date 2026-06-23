# attestack-store

Local `.attestack/` file store: sessions, append-only event logs, artifacts, Git snapshots, bundles, and health checks.

## Responsibilities

- Initialize and open the repository-local store
- Append hash-chained events and store artifacts
- Capture Git snapshot metadata
- Create signed `.attestack.zip` bundles
- Verify local sessions and bundles
- Generate session reports and run `doctor` checks

## Dependencies

Built on **`attestack-core`** for types, hashing, and signatures. Used by **`attestack-cli`**.
