# Attestack evidence (GitHub Copilot)

This project uses [Attestack](https://kiket-dev.github.io/attestack/) to record what happened during AI-assisted development.

## Workflow

1. Start a session before substantive Copilot-assisted work:
   ```bash
   attestack start "describe the task"
   ```
2. Run tests and builds through Attestack so commands are recorded:
   ```bash
   attestack run -- npm test
   ```
3. Add notes for important review points:
   ```bash
   attestack note "Reviewed Copilot suggestion for auth module"
   ```
4. Close and export when done:
   ```bash
   attestack stop
   attestack bundle create
   ```

## Privacy

Do not paste secrets into notes. Attestack stores data locally under `.attestack/`.

## Setup

Run from the repo root:

```bash
./scripts/setup-agent.sh copilot --with-rules
```
