#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$ROOT/../../.." && pwd)"

resolve_attestack() {
  if [[ -n "${ATTESTACK_BIN:-}" && -x "$ATTESTACK_BIN" ]]; then
    echo "$ATTESTACK_BIN"
    return
  fi
  if command -v attestack >/dev/null 2>&1; then
    command -v attestack
    return
  fi
  if [[ -x "$REPO_ROOT/target/release/attestack" ]]; then
    echo "$REPO_ROOT/target/release/attestack"
    return
  fi
  echo "error: attestack not found" >&2
  exit 1
}

export ATTESTACK_BIN="$(resolve_attestack)"
export ATTESTACK_REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"

if [[ ! -d "$ROOT/.venv" ]]; then
  python3 -m venv "$ROOT/.venv"
fi
# shellcheck disable=SC1091
source "$ROOT/.venv/bin/activate"
python -m pip install --upgrade pip -q
pip install -r "$ROOT/requirements.txt" -q

cd "$ATTESTACK_REPO_ROOT"
TASK_PATH="${ROOT#$ATTESTACK_REPO_ROOT/}/demo_eval.py"
inspect eval "$TASK_PATH" --model mockllm/model

echo "Demo complete. Bundle:"
ls -1 "$ATTESTACK_REPO_ROOT"/.attestack/bundles/*.attestack.zip | tail -1
