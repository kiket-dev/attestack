#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "==> Attestack release check"
echo ""

echo "==> ./scripts/check.sh"
./scripts/check.sh

echo ""
echo "==> ./scripts/smoke.sh"
./scripts/smoke.sh

echo ""
echo "==> Build attestack-mcp"
cargo build --release -p attestack-mcp -q

echo ""
echo "==> Agent setup smoke (cursor MCP config)"
TMP="$(mktemp -d)"
(
  cd "$TMP"
  "$ROOT/scripts/setup-agent.sh" cursor --force >/dev/null
  test -f .cursor/mcp.json
  test -f .attestack/config.toml
  grep -q "attestack-mcp" .cursor/mcp.json
)
rm -rf "$TMP"
echo "Agent setup smoke passed."

echo ""
echo "==> Dogfood agent setup (cursor --with-rules)"
TMP_DOGFOOD="$(mktemp -d)"
(
  cd "$TMP_DOGFOOD"
  git init -q -b main
  git config user.email "dogfood@test.local"
  git config user.name "Dogfood"
  echo demo > README.md
  git add README.md
  git commit -q -m init
  "$ROOT/scripts/dogfood-agent.sh" cursor
)
rm -rf "$TMP_DOGFOOD"

echo ""
echo "==> LangGraph harness smoke"
./scripts/harness-langgraph-smoke.sh

echo ""
echo "==> Inspect AI harness smoke"
./scripts/harness-inspect-ai-smoke.sh

echo ""
echo "==> Docs site build"
if command -v node >/dev/null 2>&1; then
  ./scripts/build-docs.sh
else
  echo "warning: node not installed; skipping docs build (CI will build on push)"
fi

echo ""
echo "Release check passed."
echo ""
echo "Next steps:"
echo "  1. Update CHANGELOG.md and tag a release (see docs/releasing.md)"
echo "  2. git push origin main && git push origin vX.Y.Z"
echo "  3. Verify GitHub Release assets and smoke install.sh"
