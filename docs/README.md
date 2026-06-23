# Documentation sources

User-facing documentation is published at **[kiket-dev.github.io/attestack](https://kiket-dev.github.io/attestack/)**.

## Layout

| Location | Purpose |
| --- | --- |
| `docs/` | Canonical markdown sources (architecture, CLI spec, security, etc.) |
| `site/` | [Astro Starlight](https://starlight.astro.build/) docs site — theme, components, landing page |
| `site/src/content/docs/` | Published pages (synced from `docs/` via `site/scripts/sync-content.mjs`) |

Hand-maintained pages: `site/src/content/docs/index.mdx` (landing) and `getting-started/quickstart.mdx`.

## Preview locally

```bash
cd site
npm ci
npm run dev
```

Open http://localhost:4321/attestack/ (matches GitHub Pages project-site routing).

Or build static output:

```bash
./scripts/build-docs.sh
cd site && npm run preview
```

## Publish

Pushes to `main` run `.github/workflows/docs.yml`, which syncs content, builds Starlight, and deploys to GitHub Pages.

One-time repo setup in GitHub: **Settings → Pages → Build and deployment → Source: GitHub Actions**. GitHub Pages requires a **public** repository on the free plan.

## Updating content

1. Edit the relevant file in `docs/`.
2. Run `node site/scripts/sync-content.mjs` (or `./scripts/build-docs.sh`, which runs sync first).
3. Preview with `cd site && npm run dev`.

When editing the landing page or quickstart demo GIF, update `site/src/content/docs/index.mdx` or `./scripts/render-demos.sh` directly.
