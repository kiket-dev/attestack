# Changelog

All notable changes to Attestack are documented here.

## [0.1.0] - 2026-06-22

### Added

- Local-first CLI with session lifecycle: `init`, `start`, `run`, `note`, `snapshot`, `stop`
- Hash-chained JSONL event log with Ed25519-signed bundle export
- `attestack verify` with optional `--public-key` and `--strict` for offline bundle verification
- `attestack report` and `attestack doctor` for human-readable summaries and health checks
- `attestack agent` subcommands for AI tool calls, decisions, approvals, and prompt/response hashes
- `attestack ci start|run|finish` for CI evidence capture
- `attestack-mcp` MCP server for Cursor and other MCP clients
- GitHub Actions example workflow (`examples/github-actions/attestack-evidence.yml`)
- Install script (`scripts/install.sh`) and multi-platform release artifacts
- mdBook documentation site with use cases, scenarios, and integration guides
- Golden test fixtures under `testdata/`

### Security

- Atomic writes for session metadata; locked append for event logs
- Zip-slip protection and bundle size limits on verify
- Path redaction option for portable bundle export

[0.1.0]: https://github.com/kiket-dev/attestack/releases/tag/v0.1.0
