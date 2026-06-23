# Installation

Attestack ships as a single static CLI binary. Choose the method that fits your environment.

## Requirements

| Requirement | Required for |
|-------------|--------------|
| Linux, macOS, or Windows | Running the CLI |
| Git | `snapshot` and repository metadata |
| Rust (stable) | Building from source only |

Private signing keys are stored under `~/.attestack/keys/` and are never written into your project tree by default.

## Install from GitHub Releases

Download the archive for your platform from [GitHub Releases](https://github.com/kiket-dev/attestack/releases), verify the SHA256 checksum, and place `attestack` on your `PATH`:

```bash
curl -fsSL https://raw.githubusercontent.com/kiket-dev/attestack/main/scripts/install.sh | bash
```

Or manually:

```bash
VERSION=v0.1.1
curl -LO "https://github.com/kiket-dev/attestack/releases/download/${VERSION}/attestack-linux-x86_64.tar.gz"
curl -LO "https://github.com/kiket-dev/attestack/releases/download/${VERSION}/attestack-linux-x86_64.tar.gz.sha256"
sha256sum -c attestack-linux-x86_64.tar.gz.sha256
tar -xzf attestack-linux-x86_64.tar.gz
sudo install -m 0755 attestack /usr/local/bin/attestack
attestack --help
```

## Build from source

```bash
git clone https://github.com/kiket-dev/attestack.git
cd attestack
cargo build --release -p attestack-cli
export PATH="$PWD/target/release:$PATH"
attestack doctor
```

Build the MCP adapter (optional, for AI agent integrations):

```bash
cargo build --release -p attestack-mcp
```

Connect your editor agent (Cursor, Claude Code, Windsurf, …):

```bash
./scripts/setup-agent.sh cursor --with-rules
```

See [Agent setup guide](agent-setup.md) for all supported agents.

## Initialize a repository

```bash
cd your-project
attestack init
attestack doctor
```

Use `attestack init --update-gitignore` to append recommended `.attestack/` ignore rules.

## Upgrade

Re-run the install script or replace the binary from a newer release. Your `~/.attestack/keys/` identity persists across upgrades.

## Uninstall

Remove the binary from your `PATH` and optionally delete:

- `~/.attestack/keys/` — signing identities
- `.attestack/` in each initialized repository — local evidence stores
