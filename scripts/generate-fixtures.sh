#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cargo build --quiet -p attestack-cli --manifest-path "$ROOT/Cargo.toml"
ATTESTACK="$ROOT/target/debug/attestack"

minimal_session() {
  local dest="$ROOT/testdata/minimal-session"
  local tmp
  tmp="$(mktemp -d)"
  (
    cd "$tmp"
    "$ATTESTACK" init
    "$ATTESTACK" start "golden fixture" --no-git
    "$ATTESTACK" note "fixture note"
    "$ATTESTACK" stop --no-report
    mkdir -p "$dest"
    session_dir="$(find .attestack/sessions -mindepth 1 -maxdepth 1 -type d | head -1)"
    cp "$session_dir/session.json" "$dest/"
    cp "$session_dir/events.jsonl" "$dest/"
  )
  rm -rf "$tmp"
  echo "Updated $dest"
}

command_session() {
  local dest="$ROOT/testdata/command-session"
  local tmp
  tmp="$(mktemp -d)"
  (
    cd "$tmp"
    git init -b main >/dev/null 2>&1
    git config user.email "fixture@example.com"
    git config user.name "Fixture"
    echo "fixture" > README.md
    git add README.md
    git commit -m "init" >/dev/null 2>&1
    "$ATTESTACK" init
    "$ATTESTACK" start "command fixture"
    "$ATTESTACK" run -- echo hello
    "$ATTESTACK" stop --no-report
    mkdir -p "$dest"
    session_dir="$(find .attestack/sessions -mindepth 1 -maxdepth 1 -type d | head -1)"
    cp "$session_dir/session.json" "$dest/"
    cp "$session_dir/events.jsonl" "$dest/"
  )
  rm -rf "$tmp"
  echo "Updated $dest"
}

valid_bundle() {
  local dest="$ROOT/testdata/valid-bundle"
  local tmp
  tmp="$(mktemp -d)"
  (
    cd "$tmp"
    git init -b main >/dev/null 2>&1
    git config user.email "fixture@example.com"
    git config user.name "Fixture"
    echo "fixture" > README.md
    git add README.md
    git commit -m "init" >/dev/null 2>&1
    "$ATTESTACK" init
    "$ATTESTACK" start "bundle fixture"
    "$ATTESTACK" run -- echo hello
    "$ATTESTACK" stop
    mkdir -p "$dest"
    "$ATTESTACK" bundle create --output "$dest/demo.attestack.zip"
    cp .attestack/identities/default.public.json "$dest/default.public.json"
  )
  rm -rf "$tmp"
  echo "Updated $dest"
}

rm -rf "$ROOT/testdata/minimal-session" "$ROOT/testdata/command-session" "$ROOT/testdata/valid-bundle"
minimal_session
command_session
valid_bundle
echo "All fixtures regenerated under $ROOT/testdata/"
