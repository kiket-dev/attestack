# Attestack

Attestack is a local-first proof layer for AI-assisted software work.

It records what happened during development and CI, stores a tamper-evident local event log, and produces portable evidence bundles that can be verified offline.

## Problem

AI-assisted development and automation are making software work faster, but less attributable and less reviewable. Important evidence is scattered across Git history, terminal output, CI logs, PR comments, AI transcripts, and memory.

Attestack answers:

- What happened during this development session?
- Which commands and tests actually ran?
- What changed between the start and end of the work?
- Was the record tampered with after the fact?
- Can the proof be shared without requiring access to the original machine or SaaS account?

## What you get today

- Local CLI (`attestack`) and MCP server (`attestack-mcp`)
- Repository-local `.attestack/` store
- Hash-chained JSONL event log
- Session lifecycle: start, note, run command, snapshot, stop
- Signed evidence bundle export and offline verification
- Agent commands and CI helpers (`attestack agent`, `attestack ci`)

## Example

```bash
attestack init
attestack start "fix billing webhook"
attestack run -- pnpm test
attestack note "Reviewed AI-generated auth changes manually."
attestack snapshot
attestack stop
attestack bundle create
attestack verify .attestack/bundles/fix-billing-webhook.attestack.zip
```

## Documentation

**Public docs:** [kiket-dev.github.io/attestack](https://kiket-dev.github.io/attestack/) (built from `site/` with [Astro Starlight](https://starlight.astro.build/), deployed via GitHub Pages).

**Install:**

```bash
curl -fsSL https://raw.githubusercontent.com/kiket-dev/attestack/main/scripts/install.sh | bash
```

**Agent setup:** `./scripts/setup-agent.sh cursor --with-rules` — see the [agent setup guide](https://kiket-dev.github.io/attestack/agent-setup.html).

**Maintainers:** `./scripts/release-check.sh` — see [docs/releasing.md](docs/releasing.md).

Source material lives in `docs/` and is included into the published site. To preview locally:

```bash
cd site && npm ci && npm run dev
# or: ./scripts/build-docs.sh && cd site && npm run preview
```

## Non-goals

- No SaaS account requirement
- No blockchain dependency
- No automatic capture of private AI transcripts
- No always-on surveillance
- No claim that recorded commands prove code is correct or secure

## License

MIT. See `LICENSE`.
