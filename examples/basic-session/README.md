# Basic Session Example

This example describes the first user flow Attestack should support.

## Flow

```bash
attestack init
attestack start "basic session"
attestack run -- echo "hello from attestack"
attestack note "This is a human note."
attestack snapshot
attestack stop
attestack bundle create
attestack verify .attestack/bundles/*.attestack.zip
```

## Expected Result

The run should produce:

- A local `.attestack/` store.
- One closed session.
- A hash-chained `events.jsonl`.
- A Markdown report.
- A portable `.attestack.zip` bundle.
- A successful offline verification result.

## Tamper Demo

After bundle creation, manually edit a copied bundle's event log and run verification again. Verification should fail with a clear error identifying the broken event chain or digest mismatch.
