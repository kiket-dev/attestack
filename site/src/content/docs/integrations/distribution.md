---
title: "Distribution"
description: "Release channels, install paths, and packaging."
---
How Attestack is built, released, and published.

## Channels

| Channel | Status | Notes |
|---------|--------|-------|
| GitHub Releases | Active | Linux/macOS/Windows tarballs/zip + SHA256 |
| `install.sh` | Active | Downloads latest release binary |
| crates.io | Planned | `attestack-cli`, `attestack-core`, `attestack-store` |
| npm / TypeScript SDK | Planned | For agent and automation authors |

## Release process

1. Update `CHANGELOG.md`
2. Tag `vX.Y.Z` and push
3. `.github/workflows/release.yml` builds matrix artifacts, runs smoke tests, publishes to GitHub Releases
4. `.github/workflows/docs.yml` updates the documentation site

## Artifact layout

```
attestack-linux-x86_64.tar.gz
attestack-linux-x86_64.tar.gz.sha256
attestack-macos-aarch64.tar.gz
attestack-macos-x86_64.tar.gz
attestack-windows-x86_64.zip
```

Each Unix archive contains `attestack` and `attestack-mcp`. Windows releases ship `attestack.exe` only (build MCP from source or use WSL).

## Checksums

Every release asset has a companion `.sha256` file. Verify before install:

```bash
sha256sum -c attestack-linux-x86_64.tar.gz.sha256
```

## Code signing

Release binaries are checksum-verified. Cosign/Sigstore signing is planned for a future release.

## Building docs locally

```bash
./scripts/build-docs.sh
# output: site/dist/
```

## Publishing to crates.io (maintainers)

```bash
cargo publish -p attestack-core
cargo publish -p attestack-store
cargo publish -p attestack-cli
```

Ensure `[package.metadata]` repository URLs are set in each crate manifest.
