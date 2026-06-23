#!/usr/bin/env bash
set -euo pipefail

# Dogfood Attestack agent setup in the current git repo (Track A validation).
#
# Usage: ./scripts/dogfood-agent.sh [options] [agent]
# Default agent: cursor

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
AGENT="${1:-cursor}"
MERGE=0
EXTRA=()

usage() {
  cat <<'EOF'
Usage: dogfood-agent.sh [options] [agent]

Options:
  --merge     Merge attestack into existing Cursor MCP config
  -h, --help  Show help

Examples:
  ./scripts/dogfood-agent.sh cursor
  ./scripts/dogfood-agent.sh --merge cursor
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    -h | --help) usage; exit 0 ;;
    --merge) MERGE=1; shift ;;
    cursor | claude-code | claude-desktop | windsurf | cline | opencode | copilot | all)
      AGENT="$1"
      shift
      ;;
    *)
      echo "error: unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if ! git rev-parse --git-dir >/dev/null 2>&1; then
  echo "error: run from a git repository root" >&2
  exit 1
fi

echo "==> Dogfood agent setup ($AGENT) in $(pwd)"

SETUP_ARGS=(--with-rules --force)
if [[ "$MERGE" -eq 1 ]]; then
  SETUP_ARGS+=(--merge)
fi

"$ROOT/scripts/setup-agent.sh" "$AGENT" "${SETUP_ARGS[@]}"

case "$AGENT" in
  cursor)
    test -f .cursor/mcp.json
    test -f .attestack/config.toml
    grep -q "attestack-mcp" .cursor/mcp.json || grep -q "attestack" .cursor/mcp.json
    ;;
  *)
    test -f .attestack/config.toml
    ;;
esac

if ! grep -q '^\.attestack/' .gitignore 2>/dev/null; then
  if [[ -f .gitignore ]]; then
    {
      echo ""
      echo "# Attestack local evidence store"
      echo ".attestack/"
    } >> .gitignore
    echo "==> Appended .attestack/ to .gitignore"
  fi
fi

echo "==> Start a session before agent work:"
echo "    attestack start \"your task\""
echo "    # or: $ROOT/scripts/agent-session.sh start \"your task\""
echo ""
echo "Dogfood setup OK."
