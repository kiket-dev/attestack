"""Shared Attestack CLI helpers for Python harness examples."""

from __future__ import annotations

import hashlib
import json
import os
import subprocess
from contextlib import contextmanager
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterator, Optional, Union


def content_hash(value: Union[str, bytes]) -> str:
    data = value.encode("utf-8") if isinstance(value, str) else value
    return f"sha256:{hashlib.sha256(data).hexdigest()}"


def stable_json(value: Any) -> str:
    if isinstance(value, str):
        return value
    return json.dumps(value, sort_keys=True, default=str)


def resolve_repo_root(start: Optional[Path] = None) -> Path:
    env_root = os.environ.get("ATTESTACK_REPO_ROOT")
    if env_root:
        return Path(env_root).resolve()
    start = (start or Path.cwd()).resolve()
    try:
        completed = subprocess.run(
            ["git", "rev-parse", "--show-toplevel"],
            cwd=start,
            check=True,
            capture_output=True,
            text=True,
        )
        return Path(completed.stdout.strip())
    except (subprocess.CalledProcessError, FileNotFoundError):
        return start


@dataclass
class AttestackCli:
    repo_root: Path
    attestack_bin: str
    session_id: Optional[str] = None

    @classmethod
    def from_env(cls, repo_root: Optional[Path] = None) -> AttestackCli:
        root = resolve_repo_root(repo_root)
        binary = os.environ.get("ATTESTACK_BIN", "attestack")
        return cls(repo_root=root, attestack_bin=binary)

    def run(self, extra: list[str], *, check: bool = True) -> subprocess.CompletedProcess[str]:
        args = [self.attestack_bin, *extra]
        if self.session_id:
            args.extend(["--session", self.session_id])
        return subprocess.run(
            args,
            cwd=self.repo_root,
            check=check,
            capture_output=True,
            text=True,
        )

    def init_if_needed(self) -> None:
        if not (self.repo_root / ".attestack" / "config.toml").is_file():
            self.run(["init"])

    def start(self, title: str) -> None:
        self.init_if_needed()
        self.run(["start", title])

    def stop(self) -> None:
        self.run(["stop"])

    def bundle_create_and_verify(self, *, strict: bool = True) -> Path:
        self.run(["bundle", "create"])
        bundles = sorted((self.repo_root / ".attestack" / "bundles").glob("*.attestack.zip"))
        if not bundles:
            raise RuntimeError("no bundle created")
        bundle = bundles[-1]
        args = ["verify", str(bundle)]
        if strict:
            args.append("--strict")
        self.run(args)
        return bundle

    def agent_tool_call(
        self,
        tool: str,
        *,
        input_hash: Optional[str] = None,
        output_hash: Optional[str] = None,
        summary: Optional[str] = None,
    ) -> None:
        args = ["agent", "tool-call", "--tool", tool]
        if input_hash:
            args.extend(["--input-hash", input_hash])
        if output_hash:
            args.extend(["--output-hash", output_hash])
        if summary:
            args.extend(["--summary", summary])
        self.run(args)

    def agent_prompt(self, content_hash_value: str, model: Optional[str] = None) -> None:
        args = ["agent", "prompt", "--content-hash", content_hash_value]
        if model:
            args.extend(["--model", model])
        self.run(args)

    def agent_response(self, content_hash_value: str, model: Optional[str] = None) -> None:
        args = ["agent", "response", "--content-hash", content_hash_value]
        if model:
            args.extend(["--model", model])
        self.run(args)

    def agent_decision(self, summary: str, rationale: Optional[str] = None) -> None:
        args = ["agent", "decision", "--summary", summary]
        if rationale:
            args.extend(["--rationale", rationale])
        self.run(args)


@contextmanager
def attestack_session(
    title: str,
    *,
    repo_root: Optional[Path] = None,
    export_bundle: bool = True,
) -> Iterator[AttestackCli]:
    cli = AttestackCli.from_env(repo_root)
    cli.start(title)
    try:
        yield cli
    finally:
        cli.stop()
        if export_bundle:
            cli.bundle_create_and_verify()
