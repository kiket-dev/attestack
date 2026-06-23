# attestack-cli

Developer CLI for recording Attestack sessions, exporting evidence bundles, and verifying them offline.

## Build

```bash
cargo build --release -p attestack-cli
./target/release/attestack --help
```

## Quick example

```bash
attestack init
attestack start "demo"
attestack run -- echo hello
attestack stop
attestack bundle create
attestack verify .attestack/bundles/*.attestack.zip
```

See the [documentation site](https://kiket-dev.github.io/attestack/) for the full CLI reference.
