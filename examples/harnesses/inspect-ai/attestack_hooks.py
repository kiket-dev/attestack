"""Inspect AI hooks that record Attestack evidence during eval runs."""

from __future__ import annotations

import sys
from pathlib import Path

_SHARED = Path(__file__).resolve().parents[1] / "_shared"
if str(_SHARED) not in sys.path:
    sys.path.insert(0, str(_SHARED))

from attestack_cli import AttestackCli, content_hash, stable_json  # noqa: E402

from inspect_ai.event._model import ModelEvent  # noqa: E402
from inspect_ai.event._tool import ToolEvent  # noqa: E402
from inspect_ai.hooks import Hooks, RunEnd, RunStart, SampleEvent, hooks  # noqa: E402


@hooks("attestack", "Record hash-chained Attestack evidence from Inspect AI evals")
class AttestackHooks(Hooks):
    """Map completed Inspect AI tool/model events to Attestack ai.* events."""

    def __init__(self) -> None:
        self.cli = AttestackCli.from_env()
        self._task_names: list[str] = []

    async def on_run_start(self, data: RunStart) -> None:
        self._task_names = list(data.task_names)
        title = "Inspect: " + ",".join(data.task_names)
        self.cli.start(title)

    async def on_sample_event(self, data: SampleEvent) -> None:
        event = data.event
        if isinstance(event, ToolEvent) and not event.pending:
            args_text = stable_json(event.arguments)
            result_text = "" if event.result is None else stable_json(event.result)
            self.cli.agent_tool_call(
                event.function,
                input_hash=content_hash(args_text),
                output_hash=content_hash(result_text),
                summary=f"{event.function} completed",
            )
        elif isinstance(event, ModelEvent) and not event.pending:
            input_text = "" if event.input is None else stable_json(event.input)
            output_text = "" if event.output is None else stable_json(event.output)
            model = event.model or "unknown"
            self.cli.agent_prompt(content_hash(input_text), model=model)
            self.cli.agent_response(content_hash(output_text), model=model)

    async def on_run_end(self, data: RunEnd) -> None:
        task_names = ",".join(self._task_names) if self._task_names else "eval"
        if data.exception is None:
            summary = f"Inspect eval completed: {task_names}"
            rationale = None
        else:
            summary = f"Inspect eval failed: {task_names}"
            rationale = str(data.exception)
        self.cli.agent_decision(summary, rationale=rationale)
        self.cli.stop()
        self.cli.bundle_create_and_verify()
