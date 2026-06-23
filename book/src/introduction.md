<div class="hero">

<p class="hero-eyebrow">Local-first · Tamper-evident · Offline verification</p>

<h1 class="hero-title">Proof for AI-assisted development</h1>

<p class="hero-lead">
Attestack records what happened during a session — commands, notes, Git state — and packages it into signed evidence bundles anyone can verify without your machine or a SaaS account.
</p>

<div class="hero-actions">
<a class="hero-btn hero-btn-primary" href="quickstart.html">Get started</a>
<a class="hero-btn hero-btn-secondary" href="https://github.com/kiket-dev/attestack">View on GitHub</a>
</div>

</div>

## How it works

<ul class="workflow">
<li>init</li>
<li>start</li>
<li>run</li>
<li>note</li>
<li>snapshot</li>
<li>stop</li>
<li>bundle</li>
<li>verify</li>
</ul>

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

## Why Attestack

<div class="feature-grid">

<div class="feature-card">
<span class="feature-icon">⛓</span>
<h3>Hash-chained events</h3>
<p>Each event links to the previous one. Tampering with the log breaks verification immediately.</p>
</div>

<div class="feature-card">
<span class="feature-icon">📦</span>
<h3>Portable bundles</h3>
<p>Export a signed zip archive with session metadata, events, reports, and selected artifacts.</p>
</div>

<div class="feature-card">
<span class="feature-icon">🔒</span>
<h3>Private by default</h3>
<p>Data stays local. Command output and diffs are opt-in. Path redaction is available on export.</p>
</div>

<div class="feature-card">
<span class="feature-icon">✓</span>
<h3>Offline verify</h3>
<p>Recompute digests, validate the chain, and check Ed25519 signatures — no network required.</p>
</div>

</div>

## Questions it answers

- What happened during this development session?
- Which commands and tests actually ran?
- What changed between the start and end of the work?
- Was the record tampered with after the fact?
- Can the proof be shared without access to the original machine?

> [!NOTE]
> Attestack records **what happened**, not whether code is correct or secure. It is evidence tooling, not a substitute for review or testing.

## What's included

Attestack ships as a developer CLI with a repository-local `.attestack/` store, hash-chained JSONL events, session lifecycle commands, signed bundle export, and `attestack verify`. The `attestack-mcp` server and `attestack agent` commands connect AI coding tools to the same evidence log.

See the [agent setup guide](agent-setup.md) and [CI integration guide](ci-integration.md) for common workflows.

## Non-goals

- No SaaS account requirement
- No blockchain dependency
- No automatic capture of private AI transcripts
- No always-on surveillance

## License

MIT. See the [repository LICENSE](https://github.com/kiket-dev/attestack/blob/main/LICENSE).
