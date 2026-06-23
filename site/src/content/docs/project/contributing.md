---
title: "Contributing"
description: "How to contribute to Attestack."
---
Thank you for contributing to Attestack.

## Development setup

```bash
git clone https://github.com/kiket-dev/attestack.git
cd attestack
rustup toolchain install stable
cargo build
./scripts/check.sh
```

Install pre-commit hooks (optional):

```bash
pip install pre-commit
pre-commit install
```

## Project layout

| Path | Purpose |
|------|---------|
| `crates/attestack-core` | Types, hashing, verification, signatures |
| `crates/attestack-store` | Local `.attestack/` store and bundles |
| `crates/attestack-cli` | User-facing CLI |
| `crates/attestack-mcp` | MCP server for agent integrations |
| `docs/` | Canonical documentation sources |
| `site/` | Astro Starlight docs site (synced from `docs/`) |
| `examples/` | Workflows and integration examples |
| `testdata/` | Golden fixtures for tests |

## Quality gate

Before opening a PR:

```bash
./scripts/check.sh        # fmt + clippy + tests
./scripts/smoke.sh        # release binary e2e (optional)
./scripts/build-docs.sh   # docs site build (optional)
```

## Tests

- Unit tests live in each crate's `src/` (`#[cfg(test)]`)
- CLI integration tests: `crates/attestack-cli/tests/`
- Golden fixtures: `testdata/`

## Documentation

Edit files in `docs/`, then run `node site/scripts/sync-content.mjs`. Preview with `cd site && npm run dev`.

## Security

Report vulnerabilities privately — see `SECURITY.md` in the repository root.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
