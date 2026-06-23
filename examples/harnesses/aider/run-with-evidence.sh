#!/usr/bin/env bash
set -euo pipefail

# Wrap an Aider (or any terminal agent) command in an Attestack session.
#
# Usage:
#   run-with-evidence.sh "session title" aider --message "..."

if [[ $# -lt 2 ]]; then
  echo "Usage: run-with-evidence.sh <title> <command...>" >&2
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
SESSION="$ROOT/scripts/agent-session.sh"

resolve_attestack() {
  if [[ -n "${ATTESTACK_BIN:-}" && -x "$ATTESTACK_BIN" ]]; then
    echo "$ATTESTACK_BIN"
    return
  fi
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
  echo "error: attestack not found" >&2
  exit 1
}

ATTESTACK="$(resolve_attestack)"
TITLE="$1"
shift

"$SESSION" start "$TITLE"

set +e
"$ATTESTACK" run -- "$@"
EXIT=$?
set -e

"$ATTESTACK" note "Aider harness exit code: $EXIT"
"$ATTESTACK" snapshot || true
"$SESSION" finish

exit "$EXIT"
