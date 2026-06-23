---
title: "Agent setup"
description: "Connect Cursor, Claude, Copilot, and other agents via MCP."
---
# Agent Setup Guide

Connect Attestack to your AI coding agent in a few minutes. Every integration follows the same pattern:

1. **Install** Attestack (`attestack` + `attestack-mcp`)
2. **Initialize** the repo (`attestack init`)
3. **Connect** the agent (MCP config or session wrapper)
4. **Start a session** before agent work (`attestack start "…"`)
5. **Stop and export** when done (`attestack stop && attestack bundle create`)

:::tip
Run `./scripts/setup-agent.sh --help` for automated setup. Use `./scripts/agent-session.sh start` before agent work.
:::


## Quick setup (automated)

From your project root (after cloning or installing Attestack):

```bash
# One agent
./scripts/setup-agent.sh cursor

# Several at once
./scripts/setup-agent.sh cursor claude-code windsurf

# MCP binary + repo init only (any agent)
./scripts/setup-agent.sh --init-only
```

The script will:

- Build or locate `attestack-mcp`
- Run `attestack init` if needed
- Write the correct MCP config for each agent
- Optionally install Cursor rules (see below)

## Session workflow (all agents)

```bash
attestack start "fix auth bug"
# … agent work …
attestack stop
attestack bundle create
attestack verify .attestack/bundles/*.attestack.zip
```

Or use the helper:

```bash
./scripts/agent-session.sh start "fix auth bug"
# … agent work …
./scripts/agent-session.sh finish   # stop + bundle create
```

---

## Cursor

**Best for:** MCP-native recording inside the editor.

### Automated

```bash
./scripts/setup-agent.sh cursor
```

### Manual

1. Build MCP: `cargo build --release -p attestack-mcp`
2. Copy `examples/agents/cursor/mcp.json` → `.cursor/mcp.json`
3. Replace `/path/to/attestack-mcp` with your binary path (or `~/.local/bin/attestack-mcp`)
4. Restart Cursor (or reload MCP)
5. Optional: copy `examples/agents/cursor/rules.mdc` → `.cursor/rules/attestack.mdc`

### Before agent work

```bash
attestack init
attestack start "feature work"
```

Ask the agent to use MCP tools: `attestack_status`, `attestack_note`, `attestack_agent_tool_call`, `attestack_agent_decision`, `attestack_snapshot`.

---

## Claude Code

**Best for:** Terminal agent with project-local MCP.

### Automated

```bash
./scripts/setup-agent.sh claude-code
```

### Manual

1. Build MCP: `cargo build --release -p attestack-mcp`
2. Copy `examples/agents/claude-code/mcp.json` → `.mcp.json` in the project root  
   (or merge into `~/.claude/settings.json` under `mcpServers`)
3. Add `CLAUDE.md` snippet from `examples/agents/claude-code/CLAUDE.md.snippet`
4. Restart Claude Code

### CLI alternative (no MCP)

Wrap your session:

```bash
./scripts/agent-session.sh start "claude code session"
claude "implement the billing fix"
./scripts/agent-session.sh finish
```

Record decisions manually:

```bash
attestack agent decision --summary "Used existing JWT middleware"
```

---

## Claude Desktop

**Best for:** Desktop app with global MCP.

### Automated

```bash
./scripts/setup-agent.sh claude-desktop
```

Writes a config fragment to `examples/agents/claude-desktop/generated-config.json` and prints where to merge it.

### Manual

1. Open Claude Desktop → Settings → Developer → Edit Config
2. Merge `examples/agents/claude-desktop/mcp.json` into `mcpServers`
3. Use an **absolute path** to `attestack-mcp`
4. Set `ATTESTACK_REPO_ROOT` to your project path (or open Claude from that directory)

Config file locations:

| OS | Path |
|----|------|
| macOS | `~/Library/Application Support/Claude/claude_desktop_config.json` |
| Linux | `~/.config/Claude/claude_desktop_config.json` |
| Windows | `%APPDATA%\Claude\claude_desktop_config.json` |

---

## Windsurf

**Best for:** Codeium Windsurf (Cursor-like MCP).

### Automated

```bash
./scripts/setup-agent.sh windsurf
```

### Manual

1. Copy `examples/agents/windsurf/mcp.json` into Windsurf MCP settings  
   (global: `~/.codeium/windsurf/mcp_config.json`, or project MCP UI)
2. Set absolute `attestack-mcp` path and `ATTESTACK_REPO_ROOT`

Same session workflow as Cursor.

---

## VS Code + GitHub Copilot

Copilot does not expose MCP in all setups. Use the **CLI wrapper** pattern:

### Automated

```bash
./scripts/setup-agent.sh copilot
```

Installs `.github/copilot-instructions.md` (optional) and prints wrapper commands.

### Manual workflow

```bash
attestack init
attestack start "copilot session"

# Run tests/build through Attestack so commands are recorded
attestack run -- npm test
attestack note "Copilot: refactored auth module"

attestack stop
attestack bundle create
```

Add `examples/agents/copilot/copilot-instructions.md` to your repo as `.github/copilot-instructions.md` so Copilot knows to remind you to record evidence.

---

## Cline (VS Code extension)

**Best for:** MCP in VS Code.

### Automated

```bash
./scripts/setup-agent.sh cline
```

### Manual

1. Open Cline → MCP Servers → Configure
2. Merge `examples/agents/cline/mcp.json`
3. Use absolute `attestack-mcp` path

Or paste the JSON from `examples/agents/cline/mcp.json` into Cline's MCP settings UI.

---

## OpenCode

**Best for:** Open-source terminal agents with MCP.

### Automated

```bash
./scripts/setup-agent.sh opencode
```

### Manual

1. Add `examples/agents/opencode/mcp.json` to OpenCode's MCP config (see OpenCode docs for config path)
2. Set `ATTESTACK_REPO_ROOT` to the project root

Fallback: use `./scripts/agent-session.sh` like Claude Code CLI.

---

## Any agent (shell-only)

No MCP required. Works everywhere:

```bash
attestack init
attestack start "manual agent session"

attestack run -- make test          # records command + output metadata
attestack note "Agent reviewed diff"
attestack agent tool-call --tool apply_patch --summary "Updated auth.rs"
attestack snapshot

attestack stop
attestack bundle create
```

---

## Verify MCP is working

```bash
attestack start "mcp test"
attestack status
```

In the agent, invoke **`attestack_status`** (or ask: “call attestack_status MCP tool”). You should see the open session ID.

If MCP fails:

- Confirm `attestack-mcp` is on PATH or path in config is absolute
- Confirm `ATTESTACK_REPO_ROOT` points at an initialized repo (`attestack init`)
- Confirm an open session exists (`attestack start "…"`)
- Restart the editor/agent after config changes

---

## What gets recorded

| Method | Records |
|--------|---------|
| MCP `attestack_note` | Session notes |
| MCP `attestack_agent_tool_call` | Tool name + optional hashes |
| MCP `attestack_agent_decision` | Decision summary |
| MCP `attestack_snapshot` | Git snapshot |
| `attestack run -- …` | Shell commands + exit codes |
| `attestack agent …` | AI events from CLI |

Raw prompts are **not** stored by default — only hashes if you use `attestack agent prompt --content-hash …`.

---

## Next steps

- [Agent guide](/integrations/agent-guide/) — MCP tools and CLI reference
- [CI integration](/integrations/ci/) — record CI runs
- [Scenarios](/getting-started/scenarios/) — end-to-end workflows
