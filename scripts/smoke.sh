#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

BIN="${ROOT}/target/release/attestack"
if [[ ! -x "$BIN" ]]; then
  echo "Building release binary..."
  cargo build --release -p attestack-cli
fi

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

cd "$tmpdir"
git init -b main >/dev/null
git config user.email "smoke@example.com"
git config user.name "Smoke Test"
echo hello > README.md
git add README.md
git commit -m "init" >/dev/null

echo "==> init"
"$BIN" init
echo "==> start"
"$BIN" start "smoke"
echo "==> run"
"$BIN" run -- echo ok
echo "==> snapshot"
"$BIN" snapshot
echo "==> stop"
"$BIN" stop
echo "==> bundle create"
"$BIN" bundle create
echo "==> verify"
bundle="$(find .attestack/bundles -name '*.attestack.zip' | head -1)"
"$BIN" verify "$bundle"
echo "==> doctor"
"$BIN" doctor

echo "Smoke test passed."
