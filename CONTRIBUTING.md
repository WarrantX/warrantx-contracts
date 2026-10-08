# Contributing to WarrantX Contracts

Start with a scoped public issue. Branch from `main`, keep changes focused, and include tests for every behavior change.

Before opening a pull request, run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace --target wasm32-unknown-unknown --release
```

Use conventional commit subjects. Never commit identities, secret keys, or production deployment credentials. Report security findings through `SECURITY.md`.

