#!/usr/bin/env bash
set -eo pipefail

echo "==> Building WarrantX Soroban Smart Contracts (WASM)..."

cargo build --target wasm32-unknown-unknown --release --manifest-path Cargo.toml

echo "==> Optimization with soroban contract optimize (if available)..."
if command -v soroban >/dev/null 2>&1; then
    mkdir -p deployments/wasm
    cp target/wasm32-unknown-unknown/release/warrantx_treasury.wasm deployments/wasm/warrantx_treasury.wasm || true
    cp target/wasm32-unknown-unknown/release/warrantx_factory.wasm deployments/wasm/warrantx_factory.wasm || true
    echo "WASM artifacts copied to deployments/wasm/"
fi

echo "==> Build complete!"
