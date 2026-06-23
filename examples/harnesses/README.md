# Orchestration harness examples

Reference integrations for **LangGraph**, CI wrappers, and other orchestration layers that run outside an editor MCP session.

Attestack records structured evidence (tool calls, prompt/response hashes, decisions) into a hash-chained session. The harness orchestrates; Attestack appends events — it never executes agent input.

## Integration patterns

| Pattern | When to use | Examples |
|---------|-------------|----------|
| **Callback adapter** | Framework exposes LangChain-style callbacks | [langgraph/](langgraph/) |
| **Session wrapper** | Shell-driven agents, eval runners | [openhands/](openhands/), [aider/](aider/), `scripts/agent-session.sh` |
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

## Session wrapper (OpenHands, Aider, others)

```bash
# OpenHands
./examples/harnesses/openhands/run-with-evidence.sh "fix auth" openhands cli --task "..."

# Aider
./examples/harnesses/aider/run-with-evidence.sh "refactor billing" aider --message "..."

# Generic
./scripts/agent-session.sh start "eval run"
./scripts/agent-session.sh finish
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
| LangGraph | Reference callback example + CI smoke |
| OpenHands | Shell wrapper + docs |
| Aider | Shell wrapper + docs |
| Inspect AI | Planned (callback adapter) |
