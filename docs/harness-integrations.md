# Harness integrations

Orchestration frameworks (LangGraph, eval runners, shell agents) integrate with Attestack via **callback adapters** or **session wrappers**. Editor agents (Cursor, Claude Code) should use [Agent setup](agent-setup.md) and MCP instead.

## Callback adapter (LangGraph)

LangGraph and LangChain emit tool and model lifecycle events through `BaseCallbackHandler`. The reference handler hashes payloads and appends Attestack `ai.*` events via the CLI.

```python
from attestack_callback import AttestackCallbackHandler, attestack_session

handler = AttestackCallbackHandler()

with attestack_session("my graph run"):
    graph.invoke(
        {"messages": [("user", "…")]},
        config={"callbacks": [handler]},
    )
```

**Example:** [`examples/harnesses/langgraph/`](https://github.com/kiket-dev/attestack/tree/main/examples/harnesses/langgraph)

| LangChain callback | Attestack event |
|--------------------|-----------------|
| Tool end | `ai.tool_call` (input + output hashes) |
| Model start / end | `ai.prompt` + `ai.response` hashes |
| `record_decision()` | `ai.decision` |

Run the demo (no API key):

```bash
cd examples/harnesses/langgraph
./run_demo.sh
attestack verify .attestack/bundles/*.attestack.zip --strict
```

## Session wrapper (OpenHands, Aider, eval runners)

Any harness that runs shell commands can wrap a session without code changes to Attestack:

| Harness | Example |
|---------|---------|
| OpenHands | [`examples/harnesses/openhands/`](https://github.com/kiket-dev/attestack/tree/main/examples/harnesses/openhands) |
| Aider | [`examples/harnesses/aider/`](https://github.com/kiket-dev/attestack/tree/main/examples/harnesses/aider) |
| Generic | `scripts/agent-session.sh` |

```bash
./scripts/agent-session.sh start "eval run"
attestack run -- npm test          # optional: record subprocess steps
./scripts/agent-session.sh finish  # stop + bundle create + verify
```

OpenHands and Aider include `run-with-evidence.sh` wrappers you can copy or call from your project root.

## CI entrypoints

GitHub Actions, Dagger, Earthly, and Nix: see [CI integration](ci-integration.md) and `examples/github-actions/`.

Pull requests on GitHub get an automated evidence summary comment when the CI evidence workflow runs (see `.github/workflows/attestack-evidence.yml`).

## Verify

Every integration should produce a bundle that passes:

```bash
attestack verify .attestack/bundles/*.attestack.zip --strict
```

Raw prompts and tool I/O are not stored by default — only SHA-256 hashes.
