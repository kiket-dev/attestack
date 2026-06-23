#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LANGGRAPH_DIR="$ROOT/examples/harnesses/langgraph"

if ! command -v python3 >/dev/null 2>&1; then
  echo "warning: python3 not found; skipping LangGraph harness smoke"
  exit 0
fi

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

(
  cd "$TMP"
  git init -b main -q
  git config user.email "harness@test.local"
  git config user.name "Harness Smoke"
  echo demo > README.md
  git add README.md
  git commit -q -m init

  cp -r "$LANGGRAPH_DIR/." .
  chmod +x run_demo.sh

  if [[ ! -x "$ROOT/target/release/attestack" ]]; then
    (cd "$ROOT" && cargo build --release -p attestack-cli -q)
  fi
  export ATTESTACK_BIN="$ROOT/target/release/attestack"

  ./run_demo.sh >/dev/null

  BUNDLE="$(ls -1 .attestack/bundles/*.attestack.zip | tail -1)"
  "$ATTESTACK_BIN" verify "$BUNDLE" --strict

  EVENTS="$(unzip -p "$BUNDLE" 'sessions/*/events.jsonl')"
  echo "$EVENTS" | grep -q '"kind":"ai.tool_call"'
  echo "$EVENTS" | grep -q '"kind":"ai.prompt"'
  echo "$EVENTS" | grep -q '"kind":"ai.response"'
  echo "$EVENTS" | grep -q '"kind":"ai.decision"'
)

echo "LangGraph harness smoke passed."
