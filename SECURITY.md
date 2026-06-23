# Security Policy

## Supported versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | Yes       |

## Reporting a vulnerability

Please **do not** open public GitHub issues for security vulnerabilities.

Email security reports to the maintainers via GitHub private vulnerability reporting on [kiket-dev/attestack](https://github.com/kiket-dev/attestack/security/advisories/new), or contact the repository owners directly.

Include:

- A description of the issue and impact
- Steps to reproduce
- Affected versions
- Suggested fix if you have one

We aim to acknowledge reports within a few business days.

## Scope

In scope:

- Tamper-evidence bypass (hash chain, signatures, bundle verification)
- Path traversal or unsafe file writes in bundle import/export
- Private key exposure through the local store or bundles
- Command injection via CLI arguments or MCP tools

Out of scope:

- Attacks requiring full compromise of the developer machine
- Social engineering
- Issues in third-party AI agents or CI platforms

## Key handling

- Private keys live in `~/.attestack/keys/` by default — never commit them
- Use `attestack bundle create --redact-paths` when sharing bundles externally
- Verify bundles with `attestack verify` before trusting exported evidence

## Secure development

Contributors should run `./scripts/check.sh` and follow guidance in [docs/security-model.md](docs/security-model.md).
