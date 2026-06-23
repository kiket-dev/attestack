#!/usr/bin/env bash
set -euo pipefail

# Wrap any OpenHands (or similar) command in an Attestack session.
#
# Usage:
#   run-with-evidence.sh "session title" openhands cli --task "..."

if [[ $# -lt 2 ]]; then
  echo "Usage: run-with-evidence.sh <title> <command...>" >&2
  exit 1
fi

TITLE="$1"
shift

resolve_attestack() {
  if [[ -n "${ATTESTACK_BIN:-}" && -x "$ATTESTACK_BIN" ]]; then
    echo "$ATTESTACK_BIN"
    return
  fi
  if command -v attestack >/dev/null 2>&1; then
    command -v attestack
    return
  fi
  echo "error: attestack not found; set ATTESTACK_BIN or install attestack" >&2
  exit 1
}

ATTESTACK="$(resolve_attestack)"

if [[ ! -f .attestack/config.toml ]]; then
  "$ATTESTACK" init
fi

"$ATTESTACK" start "$TITLE"
cleanup() {
  "$ATTESTACK" stop || true
}
trap cleanup EXIT

set +e
"$ATTESTACK" run -- "$@"
EXIT=$?
set -e

"$ATTESTACK" note "Harness command exit code: $EXIT"
"$ATTESTACK" snapshot || true
trap - EXIT
"$ATTESTACK" stop
"$ATTESTACK" bundle create
BUNDLE="$(ls -1 .attestack/bundles/*.attestack.zip 2>/dev/null | tail -1 || true)"
if [[ -n "$BUNDLE" ]]; then
  "$ATTESTACK" verify "$BUNDLE" --strict
  echo "Bundle: $BUNDLE"
fi

exit "$EXIT"
