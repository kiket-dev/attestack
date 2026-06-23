# Quick start

Record your first Attestack session in a few minutes.

> [!TIP]
> Run `attestack doctor` anytime to check store initialization, signing identity, Git availability, and active session health.

## Prerequisites

| Requirement | Notes |
|-------------|-------|
| Shell | Linux or macOS today; Windows support is planned |
| [Git](https://git-scm.com/) | Required for `snapshot`; recommended for real projects |
| [Rust](https://rustup.rs/) | Stable toolchain — build from source until crates.io publish |

## Install

See the [Installation guide](installation.md) for release binaries, `scripts/install.sh`, or building from source.

Attestack is not on crates.io yet. Clone and build the CLI:

```bash
git clone https://github.com/kiket-dev/attestack.git
cd attestack
cargo build --release -p attestack-cli
export PATH="$PWD/target/release:$PATH"
attestack --help
```

## Walkthrough

<ol class="steps">

<li>

### Initialize a project

Create a Git repo and initialize Attestack in it:

```bash
mkdir my-project && cd my-project
git init -b main
echo "hello" > README.md
git add README.md && git commit -m "init"

attestack init
attestack doctor
```

</li>

<li>

### Start a session

Open a session with a descriptive title. Attestack captures an initial Git snapshot when inside a repo:

```bash
attestack start "demo session"
attestack status
```

</li>

<li>

### Record work

Run commands, add notes, and capture Git state as you go:

```bash
attestack run -- echo hello
attestack note "Reviewed generated change"
attestack snapshot
```

</li>

<li>

### Close and review

Stopping the session writes a Markdown report under `.attestack/sessions/<id>/reports/`:

```bash
attestack stop
attestack report
```

</li>

<li>

### Export and verify

Create a signed bundle and verify it offline:

```bash
attestack bundle create
attestack verify .attestack/bundles/*.attestack.zip
```

> [!TIP]
> Use `--redact-paths` when sharing bundles outside your machine:
>
> ```bash
> attestack bundle create --redact-paths
> ```

</li>

</ol>

## Scripting with JSON

Most commands accept `--json` for automation:

```bash
attestack status --json
attestack verify .attestack/bundles/demo.attestack.zip --json
attestack doctor --json
```

## Next steps

- [Installation](installation.md) — release binaries and install script
- [Use cases](use-cases.md) — who Attestack is for and why
- [CI integration](ci-integration.md) — GitHub Actions and generic CI
- [Agent guide](agent-guide.md) — `attestack agent` and MCP server
- [CLI reference](cli-spec.md) — every command, flag, and exit code
- [Data model](data-model.md) — sessions, events, and bundles
- [Security model](security-model.md) — keys, privacy, and verification
