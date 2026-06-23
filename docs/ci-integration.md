# CI Integration

Attestack can record CI runs as tamper-evident sessions and export signed bundles as workflow artifacts.

## GitHub Actions (recommended)

Copy `.github/workflows/attestack-evidence.yml` into your repo or call it as a reusable workflow from CI.

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

## Pipeline wrappers (Dagger, Earthly, Nix)

Wrap the pipeline entrypoint with `attestack ci` so the whole run is one signed bundle.

### Dagger

```bash
attestack ci start --title "Dagger ${DAGGER_SESSION:-local}"
attestack ci run -- dagger call test
attestack ci finish
```

Run from your git checkout root. Upload `.attestack/bundles/*.attestack.zip` as a CI artifact.

### Earthly

```bash
attestack ci start --title "Earthly ${EARTHLY_TARGET:-+test}"
attestack ci run -- earthly +test
attestack ci finish
```

For monorepos, use one session per target or one session per pipeline job.

### Nix flake check

```bash
attestack ci start --title "nix flake check"
attestack ci run -- nix flake check
attestack ci finish
```

In a `flake.nix` CI derivation, call the same three commands around your test attribute — Attestack stays outside Nix; only the wrapped command runs in the sandbox.

### GitHub Actions reusable workflow

See `.github/workflows/attestack-evidence.yml` and `examples/github-actions/attestack-evidence.yml`.

After `attestack ci finish`, generate a PR body snippet:

```bash
BUNDLE="$(ls .attestack/bundles/*.attestack.zip | tail -1)"
attestack pr-summary --bundle "$BUNDLE"
```

Paste the Markdown into your pull request description, or pipe it to `gh pr edit --body-file -`.

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
