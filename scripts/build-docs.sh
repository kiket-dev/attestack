#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if ! command -v mdbook >/dev/null 2>&1; then
  echo "mdbook is not installed."
  echo "Install with: cargo install mdbook"
  exit 1
fi

mdbook build book
echo "Built site at book/book/index.html"
