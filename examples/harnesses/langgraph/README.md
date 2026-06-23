# LangGraph + Attestack

Record LangGraph agent runs as verifiable Attestack evidence using a LangChain callback handler.

**No API key required** — the demo uses `FakeListChatModel` and a single tool call.

## Prerequisites

- Python 3.10+
- [Attestack CLI](https://kiket-dev.github.io/attestack/quickstart.html) (or build from this repo)
- Git repository (for snapshots when using `attestack init` in a real project)

## Quick start

From this directory:

```bash
./run_demo.sh
```

Or manually:

```bash
python -m venv .venv && source .venv/bin/activate
pip install -r requirements.txt
export ATTESTACK_BIN=/path/to/attestack   # optional
python demo_graph.py
```

The demo initializes Attestack (if needed), starts a session, runs a small two-node LangGraph (tool lookup → model summary) with `AttestackCallbackHandler`, records a decision, then exports and verifies a bundle.

## Use in your graph

```python
from attestack_callback import AttestackCallbackHandler, attestack_session

handler = AttestackCallbackHandler()

with attestack_session("production task"):
    graph.invoke(
        {"messages": [("user", "…")]},
        config={"callbacks": [handler]},
    )
```

Set `ATTESTACK_BIN` when the CLI is not on `PATH`.

### What gets recorded

| LangChain event | Attestack event |
|-----------------|-----------------|
| `on_tool_end` | `ai.tool_call` (input + output hashes) |
| `on_chat_model_start` / `on_chat_model_end` | `ai.prompt` + `ai.response` hashes |
| `handler.record_decision(…)` | `ai.decision` |

Raw prompts and tool I/O are **never** stored — only SHA-256 hashes (`sha256:…`).

### Graph lifecycle (interrupt / resume)

LangGraph 1.1+ exposes `GraphCallbackHandler` for interrupt/resume. You can subclass it alongside `AttestackCallbackHandler` and call `record_decision()` on pause/resume boundaries. This example focuses on tool and model callbacks, which cover most agent evidence.

## Verify

```bash
attestack verify .attestack/bundles/*.attestack.zip --strict
attestack pr-summary --bundle .attestack/bundles/*.attestack.zip
```

Expect `ai.tool_call`, `ai.prompt`, `ai.response`, and `ai.decision` events in the session timeline.

## Files

| File | Purpose |
|------|---------|
| `attestack_callback.py` | `AttestackCallbackHandler` + `attestack_session()` |
| `demo_graph.py` | Two-node LangGraph demo (tool + fake model) |
| `run_demo.sh` | venv setup + demo runner |
| `requirements.txt` | `langgraph`, `langchain-core` |
