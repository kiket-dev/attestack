---
title: "Releasing"
description: "Maintainer release checklist and artifacts."
---
# Releasing Attestack

Maintainer checklist for shipping a version.

## Pre-release gate

From a clean checkout:

```bash
./scripts/release-check.sh
```

This runs fmt/clippy/tests, smoke test, agent setup smoke, and mdBook (if installed).

## Ship steps

1. **Update changelog** — edit `CHANGELOG.md` with the release date and highlights.
2. **Commit** — ensure the full tree is on `main`.
3. **Tag and push:**

   ```bash
   git tag -a vX.Y.Z -m "Attestack vX.Y.Z"
   git push origin main
   git push origin vX.Y.Z
   ```

4. **Verify GitHub Actions** — `release.yml` publishes matrix artifacts; `docs.yml` updates GitHub Pages (requires a **public** repository on the free GitHub plan).
5. **Smoke the release artifact** on a machine without Rust:

   ```bash
   curl -fsSL https://raw.githubusercontent.com/kiket-dev/attestack/main/scripts/install.sh | bash
   # Private repo: gh auth login first, then the same command (install.sh falls back to gh)
   attestack --help
   attestack doctor
   ```

6. **Dogfood agents** — in a real project:

   ```bash
   ./scripts/setup-agent.sh cursor --with-rules
   attestack start "dogfood task"
   # use Cursor MCP tools, then:
   attestack stop && attestack bundle create
   ```

## After release

- Announce with the [quick start](https://kiket-dev.github.io/attestack/quickstart.html) and [agent setup](https://kiket-dev.github.io/attestack/agent-setup.html) links.
- Open issues for follow-up work (PR summaries, Sigstore signing, SDKs).

## Rollback

Delete the GitHub Release and tag if a bad artifact shipped:

```bash
git tag -d vX.Y.Z
git push origin :refs/tags/vX.Y.Z
```

Fix, re-tag a new version (never re-use a published tag).
