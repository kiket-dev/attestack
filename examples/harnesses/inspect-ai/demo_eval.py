"""Minimal Inspect AI eval with Attestack hooks (mockllm — no API key)."""

from __future__ import annotations

import attestack_hooks  # noqa: F401 — registers @hooks subscriber

from inspect_ai import Task, task
from inspect_ai.dataset import Sample
from inspect_ai.scorer import includes
from inspect_ai.solver import generate


@task
def hello_eval() -> Task:
    return Task(
        dataset=[Sample(input="Say hello in one word.", target="hello")],
        solver=[generate()],
        scorer=includes(),
    )
