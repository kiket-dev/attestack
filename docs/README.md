# Documentation sources

User-facing documentation is published at **[kiket-dev.github.io/attestack](https://kiket-dev.github.io/attestack/)**.

## Layout

| Location | Purpose |
|----------|---------|
| `docs/` | Canonical markdown sources (architecture, CLI spec, security, etc.) |
| `book/` | [mdBook](https://rust-lang.github.io/mdBook/) site configuration, navigation, and `theme/custom.css` |

The `book/src/*.md` files mostly `{{#include}}` pages from this directory so content stays in one place.

## Preview locally

```bash
cargo install mdbook
mdbook serve book
```

Open http://localhost:3000/attestack/ (the `/attestack/` prefix matches GitHub Pages project-site routing).

## Publish

Pushes to `main` run `.github/workflows/docs.yml`, which builds mdBook and deploys to GitHub Pages.

One-time repo setup in GitHub: **Settings → Pages → Build and deployment → Source: GitHub Actions**. GitHub Pages requires a **public** repository on the free plan.
