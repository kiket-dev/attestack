# Inspect AI + Attestack

Record [Inspect AI](https://inspect.aisi.org.uk/) evaluation runs as verifiable Attestack bundles using the hooks API.

**No API key required** — the demo uses `mockllm/model`.

## Prerequisites

- Python 3.10+
- Attestack CLI
- Git repository root (for snapshots)

## Quick start

```bash
cd examples/harnesses/inspect-ai
./run_demo.sh
```

## Use in your eval

1. Copy or import `attestack_hooks.py` (and `_shared/attestack_cli.py`).
2. Import the hooks module from your task file so `@hooks` registers:

```python
import my_project.attestack_hooks  # noqa: F401

from inspect_ai import Task, task
```

3. Run eval as usual — hooks start/stop the Attestack session automatically:

```bash
export ATTESTACK_BIN=/path/to/attestack
inspect eval my_task.py --model openai/gpt-4o
```

Optional: require the hook via `INSPECT_REQUIRED_HOOKS=attestack`.

## What gets recorded

| Inspect AI hook | Attestack event |
|-----------------|-----------------|
| `on_run_start` | `session.started` |
| `on_sample_event` (ToolEvent) | `ai.tool_call` |
| `on_sample_event` (ModelEvent) | `ai.prompt` + `ai.response` |
| `on_run_end` | `ai.decision`, `session.stopped`, bundle export |

Only **completed** tool/model events are recorded (not pending).

## Verify

```bash
attestack verify .attestack/bundles/*.attestack.zip --strict
attestack pr-summary --bundle .attestack/bundles/*.attestack.zip
```

## Files

| File | Purpose |
|------|---------|
| `attestack_hooks.py` | `AttestackHooks` Inspect AI subscriber |
| `demo_eval.py` | Minimal eval task |
| `../_shared/attestack_cli.py` | Shared subprocess helpers |
| `run_demo.sh` | venv + demo runner |
