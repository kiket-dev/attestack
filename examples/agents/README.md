# Agent integration examples

Copy-paste configs and snippets for popular AI coding agents.

## Automated setup (recommended)

From your **project root**:

```bash
/path/to/attestack/scripts/setup-agent.sh cursor
/path/to/attestack/scripts/setup-agent.sh claude-code --with-rules
/path/to/attestack/scripts/setup-agent.sh all
```

See [Agent setup guide](../../docs/agent-setup.md) for full instructions.

## Per-agent files

| Agent | Files | Setup command |
|-------|-------|---------------|
| [Cursor](cursor/) | `.cursor/mcp.json`, optional `rules.mdc` | `setup-agent.sh cursor --with-rules` |
| [Claude Code](claude-code/) | `.mcp.json`, `CLAUDE.md` snippet | `setup-agent.sh claude-code` |
| [Claude Desktop](claude-desktop/) | merge into desktop config | `setup-agent.sh claude-desktop` |
| [Windsurf](windsurf/) | global or `.windsurf/mcp.json` | `setup-agent.sh windsurf` |
| [Cline](cline/) | paste into Cline MCP UI | `setup-agent.sh cline` |
| [OpenCode](opencode/) | `opencode.mcp.json` | `setup-agent.sh opencode` |
| [Copilot](copilot/) | `.github/copilot-instructions.md` | `setup-agent.sh copilot --with-rules` |

## Session helper (any agent)

```bash
./scripts/agent-session.sh start "my task"
./scripts/agent-session.sh finish
```

Templates use placeholders `__ATTESTACK_MCP_BIN__` and `__ATTESTACK_REPO_ROOT__` — the setup script replaces them automatically.
