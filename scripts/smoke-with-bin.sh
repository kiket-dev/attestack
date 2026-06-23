#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="${ATTESTACK_BIN:-${ROOT}/target/release/attestack}"

if [[ ! -x "$BIN" ]]; then
  echo "attestack binary not found at: $BIN"
  exit 1
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

"$BIN" init
"$BIN" start "smoke"
"$BIN" run -- echo ok
"$BIN" snapshot
"$BIN" stop
"$BIN" bundle create
bundle="$(find .attestack/bundles -name '*.attestack.zip' | head -1)"
"$BIN" verify "$bundle"
"$BIN" doctor

echo "Smoke test passed."
