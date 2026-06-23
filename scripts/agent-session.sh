#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

resolve_attestack() {
  if command -v attestack >/dev/null 2>&1; then
    command -v attestack
    return
  fi
  if [[ -x "$ROOT/target/release/attestack" ]]; then
    echo "$ROOT/target/release/attestack"
    return
  fi
  if [[ -x "$ROOT/target/debug/attestack" ]]; then
    echo "$ROOT/target/debug/attestack"
    return
  fi
  echo "error: attestack not found; install it or run from the Attestack repo" >&2
  exit 1
}

ATTESTACK="$(resolve_attestack)"
CMD="${1:-}"
TITLE="${2:-agent session}"

usage() {
  cat <<'EOF'
Usage: agent-session.sh <command> [title]

Simple session helpers for any AI agent (no MCP required).

Commands:
  start [title]   attestack init (if needed) + start session
  stop            stop the active session
  finish          stop + bundle create + verify
  status          show active session
  note <text>     append a note

Examples:
  ./scripts/agent-session.sh start "fix billing webhook"
  ./scripts/agent-session.sh note "Agent applied JWT refactor"
  ./scripts/agent-session.sh finish
EOF
}

case "$CMD" in
  start)
    if [[ ! -f .attestack/config.toml ]]; then
      "$ATTESTACK" init
    fi
    "$ATTESTACK" start "$TITLE"
    "$ATTESTACK" status || true
    ;;
  stop)
    "$ATTESTACK" stop
    ;;
  finish)
    "$ATTESTACK" stop
    "$ATTESTACK" bundle create
    BUNDLE="$(ls -1 .attestack/bundles/*.attestack.zip 2>/dev/null | tail -1 || true)"
    if [[ -n "$BUNDLE" ]]; then
      "$ATTESTACK" verify "$BUNDLE"
    fi
    ;;
  status)
    "$ATTESTACK" status
    ;;
  note)
    shift || true
    TEXT="${*:-}"
    if [[ -z "$TEXT" ]]; then
      echo "error: note requires text" >&2
      exit 1
    fi
    "$ATTESTACK" note "$TEXT"
    ;;
  -h | --help | "")
    usage
    ;;
  *)
    echo "error: unknown command: $CMD" >&2
    usage >&2
    exit 1
    ;;
esac
