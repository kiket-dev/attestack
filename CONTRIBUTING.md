# Contributing to Attestack

Thank you for helping improve Attestack. This file summarizes how to get started; the full guide lives in [docs/contributing.md](docs/contributing.md) and on the [documentation site](https://kiket-dev.github.io/attestack/project/contributing/).

## Development setup

```bash
git clone https://github.com/kiket-dev/attestack.git
cd attestack
cargo build --workspace
./scripts/check.sh
```

Install pre-commit hooks (optional):

```bash
pre-commit install
```

## Quality gates

Before opening a pull request, ensure:

- `./scripts/check.sh` passes (fmt, clippy, tests)
- New behavior has integration or unit tests where practical
- Public docs in `docs/` are updated when user-facing behavior changes; run `node site/scripts/sync-content.mjs` and verify with `cd site && npm run build`

## Pull requests

- Keep changes focused and explain the motivation in the PR description
- Link related issues when applicable
- Include a test plan for CLI or verification changes

## Security

Report vulnerabilities privately — see [SECURITY.md](SECURITY.md).

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
