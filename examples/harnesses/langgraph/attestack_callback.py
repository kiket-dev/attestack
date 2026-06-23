"""LangChain / LangGraph callback handler that records Attestack evidence."""

from __future__ import annotations

import hashlib
import json
import os
import subprocess
from contextlib import contextmanager
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterator, Mapping, Optional, Union

from langchain_core.callbacks import BaseCallbackHandler
from langchain_core.messages import BaseMessage
from langchain_core.outputs import LLMResult


def content_hash(value: Union[str, bytes]) -> str:
    """Return Attestack-style sha256 content hash."""
    data = value.encode("utf-8") if isinstance(value, str) else value
    return f"sha256:{hashlib.sha256(data).hexdigest()}"


def _stable_json(value: Any) -> str:
    if isinstance(value, str):
        return value
    return json.dumps(value, sort_keys=True, default=str)


def _message_blob(messages: list[Any]) -> str:
  parts: list[str] = []
  for message in messages:
    if isinstance(message, BaseMessage):
      parts.append(message.type)
      parts.append(message.content if isinstance(message.content, str) else _stable_json(message.content))
      if message.additional_kwargs:
        parts.append(_stable_json(message.additional_kwargs))
    elif isinstance(message, tuple) and len(message) == 2:
      parts.append(str(message[0]))
      parts.append(_stable_json(message[1]))
    else:
      parts.append(_stable_json(message))
  return "\n".join(parts)


def _llm_text(result: LLMResult) -> str:
  chunks: list[str] = []
  for generation_list in result.generations:
    for generation in generation_list:
      text = generation.text
      if text:
        chunks.append(text)
      message = getattr(generation, "message", None)
      if message is not None:
        content = getattr(message, "content", None)
        if content is not None:
          chunks.append(content if isinstance(content, str) else _stable_json(content))
  return "\n".join(chunks) if chunks else _stable_json(result.model_dump())


@dataclass
class AttestackSession:
  repo_root: Path
  attestack_bin: str


class AttestackCallbackHandler(BaseCallbackHandler):
  """Record LangChain tool and model events into the active Attestack session."""

  def __init__(
    self,
    *,
    repo_root: Optional[Path] = None,
    attestack_bin: Optional[str] = None,
    session_id: Optional[str] = None,
    raise_on_error: bool = True,
  ) -> None:
    self.repo_root = (repo_root or Path.cwd()).resolve()
    self.attestack_bin = attestack_bin or os.environ.get("ATTESTACK_BIN", "attestack")
    self.session_id = session_id
    self.raise_on_error = raise_on_error
    self._pending_tools: dict[Union[str, None], tuple[str, str]] = {}
    self._pending_prompts: dict[Union[str, None], tuple[str, Optional[str]]] = {}

  def _args(self, extra: list[str]) -> list[str]:
    args = [self.attestack_bin, *extra]
    if self.session_id:
      args.extend(["--session", self.session_id])
    return args

  def _run(self, extra: list[str]) -> None:
    try:
      subprocess.run(
        self._args(extra),
        cwd=self.repo_root,
        check=True,
        capture_output=True,
        text=True,
      )
    except subprocess.CalledProcessError as exc:
      stderr = (exc.stderr or "").strip()
      if self.raise_on_error:
        detail = stderr or str(exc)
        raise RuntimeError(f"attestack {' '.join(extra[:2])} failed: {detail}") from exc

  def on_tool_start(
    self,
    serialized: dict[str, Any],
    input_str: str,
    *,
    run_id: Optional[str] = None,
    **kwargs: Any,
  ) -> None:
    tool_name = serialized.get("name") or serialized.get("id") or "unknown_tool"
    self._pending_tools[run_id] = (tool_name, input_str)

  def on_tool_end(self, output: str, *, run_id: Optional[str] = None, **kwargs: Any) -> None:
    tool_name, input_str = self._pending_tools.pop(run_id, ("unknown_tool", ""))
    output_text = output if isinstance(output, str) else _stable_json(output)
    args = [
      "agent",
      "tool-call",
      "--tool",
      tool_name,
      "--input-hash",
      content_hash(input_str),
      "--output-hash",
      content_hash(output_text),
      "--summary",
      f"{tool_name} completed",
    ]
    self._run(args)

  def on_tool_error(self, error: BaseException, *, run_id: Optional[str] = None, **kwargs: Any) -> None:
    tool_name, input_str = self._pending_tools.pop(run_id, ("unknown_tool", ""))
    summary = f"{tool_name} failed: {error}"
    self._run(
      [
        "agent",
        "tool-call",
        "--tool",
        tool_name,
        "--input-hash",
        content_hash(input_str),
        "--summary",
        summary[:200],
      ]
    )

  def on_chat_model_start(
    self,
    serialized: Mapping[str, Any],
    messages: list[list[BaseMessage]],
    *,
    run_id: Optional[str] = None,
    **kwargs: Any,
  ) -> None:
    flat = [item for group in messages for item in group]
    model = serialized.get("name") or serialized.get("id")
    self._pending_prompts[run_id] = (_message_blob(flat), model)

  def on_llm_start(
    self,
    serialized: Mapping[str, Any],
    prompts: list[str],
    *,
    run_id: Optional[str] = None,
    **kwargs: Any,
  ) -> None:
    model = serialized.get("name") or serialized.get("id")
    self._pending_prompts[run_id] = ("\n".join(prompts), model)

  def _record_prompt_response(self, run_id: Optional[str], response_text: str) -> None:
    prompt_text, model = self._pending_prompts.pop(run_id, ("", None))
    if prompt_text:
      prompt_args = ["agent", "prompt", "--content-hash", content_hash(prompt_text)]
      if model:
        prompt_args.extend(["--model", str(model)])
      self._run(prompt_args)

    response_args = ["agent", "response", "--content-hash", content_hash(response_text)]
    if model:
      response_args.extend(["--model", str(model)])
    self._run(response_args)

  def on_chat_model_end(self, response: LLMResult, *, run_id: Optional[str] = None, **kwargs: Any) -> None:
    self._record_prompt_response(run_id, _llm_text(response))

  def on_llm_end(self, response: LLMResult, *, run_id: Optional[str] = None, **kwargs: Any) -> None:
    self._record_prompt_response(run_id, _llm_text(response))

  def record_decision(self, summary: str, rationale: Optional[str] = None) -> None:
    """Explicit helper for graph-level decisions (optional)."""
    args = ["agent", "decision", "--summary", summary]
    if rationale:
      args.extend(["--rationale", rationale])
    self._run(args)


@contextmanager
def attestack_session(
  title: str,
  *,
  repo_root: Optional[Path] = None,
  attestack_bin: Optional[str] = None,
  export_bundle: bool = True,
) -> Iterator[AttestackSession]:
  """Initialize (if needed), start a session, and export a bundle on exit."""
  root = (repo_root or Path.cwd()).resolve()
  binary = attestack_bin or os.environ.get("ATTESTACK_BIN", "attestack")

  def run_cli(*extra: str) -> None:
    subprocess.run([binary, *extra], cwd=root, check=True, capture_output=True, text=True)

  if not (root / ".attestack" / "config.toml").is_file():
    run_cli("init")

  run_cli("start", title)
  session = AttestackSession(repo_root=root, attestack_bin=binary)
  try:
    yield session
  finally:
    run_cli("stop")
    if export_bundle:
      run_cli("bundle", "create")
      bundles = sorted((root / ".attestack" / "bundles").glob("*.attestack.zip"))
      if bundles:
        run_cli("verify", str(bundles[-1]))
