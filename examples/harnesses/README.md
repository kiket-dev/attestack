# Orchestration harness examples

Reference integrations for **LangGraph**, CI wrappers, and other orchestration layers that run outside an editor MCP session.

Attestack records structured evidence (tool calls, prompt/response hashes, decisions) into a hash-chained session. The harness orchestrates; Attestack appends events — it never executes agent input.

## Integration patterns

| Pattern | When to use | Examples |
|---------|-------------|----------|
| **Callback adapter** | Framework exposes LangChain-style callbacks | [langgraph/](langgraph/) |
| **Session wrapper** | Shell-driven agents, eval runners | `scripts/agent-session.sh` |
| **MCP sidecar** | IDE agents (Cursor, Claude Code) | [examples/agents/](../agents/) |
| **CI helpers** | GitHub Actions, Dagger entrypoints | [examples/github-actions/](../github-actions/) |

## Callback adapter (LangGraph)

1. Start a session before the graph runs (`attestack start` or `attestack_session()`).
2. Attach `AttestackCallbackHandler` to `config["callbacks"]` on `graph.invoke()`.
3. On tool and model events, the handler hashes payloads and calls `attestack agent …`.
4. Stop and export a bundle when the run finishes.

```bash
cd examples/harnesses/langgraph
python -m venv .venv && source .venv/bin/activate
pip install -r requirements.txt
./run_demo.sh
```

See [langgraph/README.md](langgraph/README.md) for details.

## Session wrapper (any harness)

```bash
./scripts/agent-session.sh start "eval run"
# your harness / agent runs here
./scripts/agent-session.sh finish   # stop + bundle create + verify
```

Use `attestack run --` inside the harness for subprocess steps you want in the evidence chain.

## Verify output

Every example should produce a bundle that passes strict verification:

```bash
attestack verify .attestack/bundles/*.attestack.zip --strict
```

## More harnesses

| Harness | Status |
|---------|--------|
| LangGraph | Reference callback example |
| Inspect AI | Planned |
| OpenHands | Shell wrapper (doc only) |
| Aider | `agent-session.sh` |
