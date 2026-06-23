# Attestack MCP

stdio MCP server that exposes Attestack session tools to AI agents.

## Tools

| Tool | Description |
|------|-------------|
| `attestack_status` | Active session status |
| `attestack_note` | Append a session note |
| `attestack_agent_tool_call` | Record tool name + input/output hashes |
| `attestack_agent_decision` | Record a decision summary |
| `attestack_snapshot` | Capture a Git snapshot |

## Build

```bash
cargo build --release -p attestack-mcp
```

## Cursor configuration

See `examples/mcp/cursor-mcp.json` in the repository root.

Run from an initialized repository (or set `ATTESTACK_REPO_ROOT`).

## Security

MCP tools append structured events only — they never execute shell commands from agent input.
