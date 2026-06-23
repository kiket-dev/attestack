# Test fixtures

Golden fixtures provide stable examples of Attestack session and bundle data for tests and documentation.

## `minimal-session/`

A closed session with `session.started`, `session.note_added`, and `session.stopped`.

## `command-session/`

A closed session that includes `command.started` and `command.finished` from `attestack run -- echo hello`.

## `valid-bundle/`

A signed portable bundle (`demo.attestack.zip`) with matching public identity (`default.public.json`).

Regenerate all fixtures with:

```bash
./scripts/generate-fixtures.sh
```

Tests verify event chains, bundle digests, and signatures — not exact timestamps across regenerations.
