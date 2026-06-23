#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EXAMPLES="$ROOT/examples/agents"
FORCE=0
INIT_ONLY=0
WITH_RULES=0
MERGE=0
AGENTS=()

usage() {
  cat <<'EOF'
Usage: setup-agent.sh [options] [agent...]

Configure Attestack for popular AI coding agents.

Agents:
  cursor          Cursor IDE (.cursor/mcp.json)
  claude-code     Claude Code (.mcp.json + CLAUDE.md snippet)
  claude-desktop  Claude Desktop (prints merge instructions)
  windsurf        Windsurf / Codeium MCP
  cline           Cline VS Code extension
  opencode        OpenCode MCP
  copilot         GitHub Copilot (instructions + CLI workflow)
  all             All of the above (except claude-desktop merge)

Options:
  --force         Overwrite existing config files
  --merge         Merge into existing Cursor MCP config (cursor only)
  --with-rules    Install Cursor / Copilot instruction files
  --init-only     Only build MCP + run attestack init (no agent config)
  -h, --help      Show this help

Examples:
  ./scripts/setup-agent.sh cursor
  ./scripts/setup-agent.sh cursor --merge --with-rules
  ./scripts/setup-agent.sh cursor claude-code --with-rules
  ./scripts/setup-agent.sh all --force
EOF
}

log() { printf '==> %s\n' "$*"; }
warn() { printf 'warning: %s\n' "$*" >&2; }

while [[ $# -gt 0 ]]; do
  case "$1" in
    -h | --help) usage; exit 0 ;;
    --force) FORCE=1; shift ;;
    --merge) MERGE=1; shift ;;
    --with-rules) WITH_RULES=1; shift ;;
    --init-only) INIT_ONLY=1; shift ;;
    cursor | claude-code | claude-desktop | windsurf | cline | opencode | copilot | all)
      AGENTS+=("$1")
      shift
      ;;
    *)
      echo "error: unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if [[ ${#AGENTS[@]} -eq 0 && "$INIT_ONLY" -eq 0 ]]; then
  echo "error: specify at least one agent or --init-only" >&2
  usage >&2
  exit 1
fi

if [[ " ${AGENTS[*]:-} " == *" all "* ]]; then
  AGENTS=(cursor claude-code claude-desktop windsurf cline opencode copilot)
fi

resolve_attestack() {
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
  echo ""
}

resolve_mcp() {
  if command -v attestack-mcp >/dev/null 2>&1; then
    command -v attestack-mcp
    return
  fi
  if [[ -x "$ROOT/target/release/attestack-mcp" ]]; then
    echo "$ROOT/target/release/attestack-mcp"
    return
  fi
  if [[ -x "$ROOT/target/debug/attestack-mcp" ]]; then
    echo "$ROOT/target/debug/attestack-mcp"
    return
  fi
  log "Building attestack-mcp (release)..."
  (cd "$ROOT" && cargo build --release -p attestack-mcp -p attestack-cli -q)
  echo "$ROOT/target/release/attestack-mcp"
}

write_file() {
  local dest="$1"
  local src="$2"
  if [[ -f "$dest" && "$FORCE" -eq 0 ]]; then
    warn "$dest already exists (use --force to overwrite)"
    return 1
  fi
  mkdir -p "$(dirname "$dest")"
  cp "$src" "$dest"
  log "Wrote $dest"
}

render_mcp_json() {
  local template="$1"
  local dest="$2"
  local mcp_bin="$3"
  local repo_root="$4"
  if [[ -f "$dest" && "$FORCE" -eq 0 ]]; then
    warn "$dest already exists (use --force to overwrite)"
    return 1
  fi
  mkdir -p "$(dirname "$dest")"
  sed \
    -e "s|__ATTESTACK_MCP_BIN__|$mcp_bin|g" \
    -e "s|__ATTESTACK_REPO_ROOT__|$repo_root|g" \
    "$template" > "$dest"
  log "Wrote $dest"
}

merge_cursor_mcp() {
  local dest="$1"
  local mcp_bin="$2"
  if [[ ! -f "$dest" ]]; then
    render_mcp_json "$EXAMPLES/cursor/mcp.json" "$dest" "$mcp_bin" ""
    return
  fi
  PROJECT_ROOT="$PROJECT_ROOT" MCP_BIN="$mcp_bin" python3 <<'PY'
import json
import os
from pathlib import Path

dest = Path(os.environ["PROJECT_ROOT"]) / ".cursor" / "mcp.json"
mcp_bin = os.environ["MCP_BIN"]
data = json.loads(dest.read_text(encoding="utf-8"))
servers = data.setdefault("mcpServers", {})
servers["attestack"] = {
    "command": mcp_bin,
    "args": [],
    "env": {"ATTESTACK_REPO_ROOT": "${workspaceFolder}"},
}
dest.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
print(dest)
PY
  log "Merged attestack into $dest"
}

PROJECT_ROOT="$(pwd)"
MCP_BIN="$(resolve_mcp)"
ATTESTACK_BIN="$(resolve_attestack)"

if [[ -z "$ATTESTACK_BIN" ]]; then
  log "Building attestack CLI..."
  (cd "$ROOT" && cargo build --release -p attestack-cli -q)
  ATTESTACK_BIN="$ROOT/target/release/attestack"
fi

if [[ ! -f "$PROJECT_ROOT/.attestack/config.toml" ]]; then
  log "Initializing Attestack in $PROJECT_ROOT"
  "$ATTESTACK_BIN" init
else
  log "Attestack already initialized in $PROJECT_ROOT"
fi

if [[ "$INIT_ONLY" -eq 1 ]]; then
  log "MCP binary: $MCP_BIN"
  log "Done (--init-only)"
  exit 0
fi

setup_cursor() {
  if [[ "$MERGE" -eq 1 ]]; then
    merge_cursor_mcp "$PROJECT_ROOT/.cursor/mcp.json" "$MCP_BIN"
  else
    render_mcp_json "$EXAMPLES/cursor/mcp.json" "$PROJECT_ROOT/.cursor/mcp.json" "$MCP_BIN" ""
  fi
  if [[ "$WITH_RULES" -eq 1 ]]; then
    write_file "$PROJECT_ROOT/.cursor/rules/attestack.mdc" "$EXAMPLES/cursor/rules.mdc" || true
  fi
  cat <<EOF

Cursor next steps:
  1. Restart Cursor or reload MCP servers
  2. Run: attestack start "your task title"
  3. Ask the agent to use attestack MCP tools (attestack_status, attestack_note, …)

EOF
}

setup_claude_code() {
  render_mcp_json "$EXAMPLES/claude-code/mcp.json" "$PROJECT_ROOT/.mcp.json" "$MCP_BIN" "$PROJECT_ROOT"
  if [[ -f "$PROJECT_ROOT/CLAUDE.md" ]]; then
    if ! grep -q "Attestack" "$PROJECT_ROOT/CLAUDE.md" 2>/dev/null; then
      if [[ "$FORCE" -eq 1 ]]; then
        cat "$EXAMPLES/claude-code/CLAUDE.md.snippet" >> "$PROJECT_ROOT/CLAUDE.md"
        log "Appended Attestack snippet to CLAUDE.md"
      else
        warn "Add examples/agents/claude-code/CLAUDE.md.snippet to CLAUDE.md manually"
      fi
    fi
  else
    cp "$EXAMPLES/claude-code/CLAUDE.md.snippet" "$PROJECT_ROOT/CLAUDE.md"
    log "Wrote $PROJECT_ROOT/CLAUDE.md"
  fi
  cat <<EOF

Claude Code next steps:
  1. Restart Claude Code
  2. Run: attestack start "your task title"
  3. MCP tools: attestack_status, attestack_note, attestack_agent_tool_call, …

EOF
}

setup_claude_desktop() {
  local out="$EXAMPLES/claude-desktop/generated-config.json"
  render_mcp_json "$EXAMPLES/claude-desktop/mcp.json" "$out" "$MCP_BIN" "$PROJECT_ROOT"
  cat <<EOF

Claude Desktop next steps:
  1. Open Claude Desktop → Settings → Developer → Edit Config
  2. Merge mcpServers from:
       $out
  3. Set ATTESTACK_REPO_ROOT to: $PROJECT_ROOT
  4. Run in that project: attestack start "your task title"

Config file locations:
  macOS: ~/Library/Application Support/Claude/claude_desktop_config.json
  Linux: ~/.config/Claude/claude_desktop_config.json

EOF
}

setup_windsurf() {
  local dest="$HOME/.codeium/windsurf/mcp_config.json"
  render_mcp_json "$EXAMPLES/windsurf/mcp.json" "$dest" "$MCP_BIN" "$PROJECT_ROOT" || true
  render_mcp_json "$EXAMPLES/windsurf/mcp.json" "$PROJECT_ROOT/.windsurf/mcp.json" "$MCP_BIN" "$PROJECT_ROOT" || true
  cat <<EOF

Windsurf next steps:
  1. Restart Windsurf
  2. Confirm MCP config (global or project .windsurf/mcp.json)
  3. Run: attestack start "your task title"

EOF
}

setup_cline() {
  local rendered="$PROJECT_ROOT/.cline/attestack-mcp.json"
  mkdir -p "$PROJECT_ROOT/.cline"
  render_mcp_json "$EXAMPLES/cline/mcp.json" "$rendered" "$MCP_BIN" "$PROJECT_ROOT" || true
  cat <<EOF

Cline next steps:
  1. Open Cline → MCP Servers → paste or import:
       $rendered
  2. Run: attestack start "your task title"

EOF
}

setup_opencode() {
  render_mcp_json "$EXAMPLES/opencode/mcp.json" "$PROJECT_ROOT/opencode.mcp.json" "$MCP_BIN" "$PROJECT_ROOT" || true
  cat <<EOF

OpenCode next steps:
  1. Add opencode.mcp.json to OpenCode MCP config (see OpenCode docs)
  2. Run: attestack start "your task title"

EOF
}

setup_copilot() {
  if [[ "$WITH_RULES" -eq 1 || "$FORCE" -eq 1 ]]; then
    write_file "$PROJECT_ROOT/.github/copilot-instructions.md" "$EXAMPLES/copilot/copilot-instructions.md" || true
  else
    warn "Run with --with-rules to install .github/copilot-instructions.md"
  fi
  cat <<EOF

GitHub Copilot next steps (CLI wrapper — no MCP required):
  attestack start "copilot session"
  attestack run -- npm test
  attestack note "Copilot: describe what changed"
  attestack stop && attestack bundle create

Optional: copy examples/agents/copilot/copilot-instructions.md
  → .github/copilot-instructions.md

EOF
}

for agent in "${AGENTS[@]}"; do
  log "Setting up $agent"
  case "$agent" in
    cursor) setup_cursor ;;
    claude-code) setup_claude_code ;;
    claude-desktop) setup_claude_desktop ;;
    windsurf) setup_windsurf ;;
    cline) setup_cline ;;
    opencode) setup_opencode ;;
    copilot) setup_copilot ;;
  esac
done

log "MCP binary: $MCP_BIN"
log "Start a session before agent work: attestack start \"your task\""
log "Or use: ./scripts/agent-session.sh start \"your task\""
