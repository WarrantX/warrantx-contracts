# Testing

Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and a release WASM build. Tests cover initialization, members, deposits, automatic and approved payments, recurring periods, allowance rejection, stale policy versions, and policy validation.

