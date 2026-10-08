#!/usr/bin/env bash
set -eo pipefail

echo "==> Running WarrantX Soroban Contract Tests..."
cargo test --manifest-path Cargo.toml -- --nocapture
echo "==> All contract tests passed successfully!"
