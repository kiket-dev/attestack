# CI Integration

Attestack can record CI runs as tamper-evident sessions and export signed bundles as workflow artifacts.

## GitHub Actions (recommended)

Copy `examples/github-actions/attestack-evidence.yml` into `.github/workflows/` or call it as a reusable workflow.

The example workflow:

1. Builds or installs `attestack`
2. Runs `attestack ci start` (session title from `GITHUB_WORKFLOW` / `GITHUB_RUN_ID`)
3. Runs your test command via `attestack ci run -- …`
4. Runs `attestack ci finish` (stop + bundle create)
5. Uploads `.attestack/bundles/*.attestack.zip` as an artifact

## CI commands

| Command | Purpose |
|---------|---------|
| `attestack ci start` | `init` (if needed) + `start` with CI-derived title |
| `attestack ci run -- <cmd>` | Run a command inside the active CI session |
| `attestack ci finish` | `stop` + `bundle create` + print bundle path |

Environment variables used for session titles:

- `GITHUB_WORKFLOW`
- `GITHUB_RUN_ID`
- `CI` (generic fallback)

## Generic CI (GitLab, Buildkite, etc.)

```bash
attestack init
attestack start "CI ${CI_PIPELINE_ID:-local}"
attestack run -- make test
attestack stop
attestack bundle create
attestack verify .attestack/bundles/*.attestack.zip
```

Upload the bundle from your CI artifact step.

## Verifying CI bundles locally

```bash
attestack verify ./downloaded/ci-bundle.attestack.zip
```

If you no longer have the original repository identity, pass the public key:

```bash
attestack verify ./bundle.attestack.zip --public-key .attestack/identities/default.public.json
```

## Security notes

- Do not commit `.attestack/` private material; use `--redact-paths` on bundle export for external sharing
- CI secrets are never captured by `attestack run` (environment variables are excluded by default)
- Bundles are signed with your Ed25519 identity; protect `~/.attestack/keys/`

## Future: Sigstore signing

Optional Sigstore signing for release and CI bundles is planned for a future release.
