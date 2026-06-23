#!/usr/bin/env bash
set -euo pipefail

# Dogfood Attestack agent setup in the current git repo (Track A validation).
#
# Usage: ./scripts/dogfood-agent.sh [agent]
# Default agent: cursor

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
AGENT="${1:-cursor}"

if ! git rev-parse --git-dir >/dev/null 2>&1; then
  echo "error: run from a git repository root" >&2
  exit 1
fi

echo "==> Dogfood agent setup ($AGENT) in $(pwd)"

"$ROOT/scripts/setup-agent.sh" "$AGENT" --with-rules --force

case "$AGENT" in
  cursor)
    test -f .cursor/mcp.json
    test -f .attestack/config.toml
    grep -q "attestack-mcp" .cursor/mcp.json
    ;;
  *)
    test -f .attestack/config.toml
    ;;
esac

echo "==> Start a session before agent work:"
echo "    attestack start \"your task\""
echo "    # or: $ROOT/scripts/agent-session.sh start \"your task\""
echo ""
echo "Dogfood setup OK."
