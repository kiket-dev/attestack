#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if ! command -v node >/dev/null 2>&1; then
  echo "Node.js is required to build docs."
  exit 1
fi

node site/scripts/sync-content.mjs

cd site
if [[ ! -d node_modules ]]; then
  npm ci
fi
npm run build

echo "Built site at site/dist/index.html"
echo "Preview with: cd site && npm run preview"
