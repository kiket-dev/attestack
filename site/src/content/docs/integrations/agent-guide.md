---
title: "Agent guide"
description: "attestack agent commands and MCP tool reference."
---
# Agent Integration Guide

> **New:** Step-by-step setup for Cursor, Claude Code, Copilot, and more — see **[Agent setup guide](/integrations/agent-setup/)**.  
> Run `./scripts/setup-agent.sh cursor` from your project root for automated configuration.

This guide covers practical ways to connect AI coding agents to Attestack today.

## Design principles

- **Structured evidence, not surveillance** — record tool calls, decisions, and approvals as data
- **Hash-first privacy** — prompts and responses stored as SHA-256 hashes by default
- **No arbitrary execution** — MCP tools append events only; they never run shell commands from agent input
- **Vendor-neutral core** — tool-specific adapters stay outside `attestack-core`

## CLI: `attestack agent`

Record agent activity into the active session:

```bash
attestack start "feature work"

# Hash-only tool call (preferred)
attestack agent tool-call \
  --tool read_file \
  --input-hash sha256:… \
  --output-hash sha256:… \
  --summary "Read src/auth.rs"

# Human or agent decision
attestack agent decision \
  --summary "Reuse JWT middleware" \
  --rationale "Matches existing patterns"

# Approval / rejection
attestack agent approval \
  --subject "Generated SQL migration" \
  --approved

# Prompt/response hashes (never raw content by default)
attestack agent prompt --content-hash sha256:… --model claude-sonnet
attestack agent response --content-hash sha256:… --model claude-sonnet

attestack stop
```

All commands support `--json` and `--session <id>`.

## MCP server

The `attestack-mcp` binary exposes Attestack to MCP-capable agents (Cursor, Claude Desktop, etc.).

**Quick setup:** `./scripts/setup-agent.sh cursor` (or `claude-code`, `windsurf`, `all`).  
See [Agent setup guide](/integrations/agent-setup/) for every supported agent.

### Build

```bash
cargo build --release -p attestack-mcp
```

### Configure Cursor

Use the setup script (recommended):

```bash
./scripts/setup-agent.sh cursor --with-rules
```

Or copy `examples/agents/cursor/mcp.json` → `.cursor/mcp.json` and set the binary path.  
See `examples/agents/README.md`.

### MCP tools

| Tool | Description |
|------|-------------|
| `attestack_status` | Active session status |
| `attestack_note` | Append a session note |
| `attestack_agent_tool_call` | Record tool name + input/output hashes |
| `attestack_agent_decision` | Record a decision summary |
| `attestack_snapshot` | Capture Git snapshot (requires Git repo) |

## Orchestration harnesses (LangGraph)

For LangGraph, LangChain, and similar frameworks, attach a callback handler that hashes tool and model I/O and calls `attestack agent` subprocesses.

**Reference example:** `examples/harnesses/langgraph/`

```python
from attestack_callback import AttestackCallbackHandler, attestack_session

handler = AttestackCallbackHandler()

with attestack_session("my graph run"):
    graph.invoke(
        {"messages": [("user", "…")]},
        config={"callbacks": [handler]},
    )
```

See [harness examples](../examples/harnesses/README.md) for patterns (callback adapter, session wrapper, CI).

**Shell wrappers:** [OpenHands](../examples/harnesses/openhands/), [Aider](../examples/harnesses/aider/).

## Shell wrapper pattern

For agents without MCP, wrap critical commands:

```bash
attestack run -- npm test
attestack note "Agent: tests passed after auth refactor"
```

## Transcript import (planned)

Post-hoc importers for Claude Code, Copilot CLI, and OpenCode will convert transcripts into normalized Attestack events. Imported events are labeled separately from live-recorded events.

## Bundles with agent metadata

Export as usual:

```bash
attestack bundle create
attestack verify .attestack/bundles/*.attestack.zip
```

Agent events appear in the session timeline and report. Raw prompts are not included unless you explicitly attach them as artifacts.
