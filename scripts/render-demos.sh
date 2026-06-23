#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if ! command -v vhs >/dev/null 2>&1; then
  echo "error: vhs not installed — see https://github.com/charmbracelet/vhs" >&2
  exit 1
fi

if [[ ! -x "$ROOT/target/release/attestack" ]]; then
  echo "==> Building attestack CLI for demo"
  cargo build --release -p attestack-cli -q
fi

export PATH="$ROOT/target/release:$PATH"

echo "==> Rendering quickstart-demo.gif"
vhs scripts/demos/quickstart.tape

echo "==> Done: book/src/img/quickstart-demo.gif"
