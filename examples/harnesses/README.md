# Orchestration harness examples

Reference integrations for **LangGraph**, **Inspect AI**, CI wrappers, and other orchestration layers that run outside an editor MCP session.

Attestack records structured evidence (tool calls, prompt/response hashes, decisions) into a hash-chained session. The harness orchestrates; Attestack appends events — it never executes agent input.

## Integration patterns

| Pattern | When to use | Examples |
|---------|-------------|----------|
| **Callback adapter** | Framework exposes hooks or LangChain callbacks | [langgraph/](langgraph/), [inspect-ai/](inspect-ai/) |
| **Session wrapper** | Shell-driven agents, eval runners | [openhands/](openhands/), [aider/](aider/), `scripts/agent-session.sh` |
| **MCP sidecar** | IDE agents (Cursor, Claude Code) | [examples/agents/](../agents/) |
| **CI helpers** | GitHub Actions, Dagger entrypoints | [examples/github-actions/](../github-actions/) |

Shared Python helpers: [`_shared/attestack_cli.py`](_shared/attestack_cli.py).

## Callback adapters

### LangGraph

```bash
cd examples/harnesses/langgraph
./run_demo.sh
```

### Inspect AI

```bash
cd examples/harnesses/inspect-ai
./run_demo.sh
```

See each directory's README for hook/callback wiring.

## Session wrapper (OpenHands, Aider, others)

```bash
./examples/harnesses/openhands/run-with-evidence.sh "fix auth" openhands cli --task "..."
./examples/harnesses/aider/run-with-evidence.sh "refactor billing" aider --message "..."
./scripts/agent-session.sh start "eval run" && ./scripts/agent-session.sh finish
```

## Dogfood in your repo

```bash
/path/to/attestack/scripts/dogfood-agent.sh --merge cursor
```

Use `--merge` when `.cursor/mcp.json` already lists other MCP servers.

## Verify output

```bash
attestack verify .attestack/bundles/*.attestack.zip --strict
```

## Harness status

| Harness | Status |
|---------|--------|
| LangGraph | Callback example + CI smoke |
| Inspect AI | Hooks example + CI smoke |
| OpenHands | Shell wrapper + docs |
| Aider | Shell wrapper + docs |
