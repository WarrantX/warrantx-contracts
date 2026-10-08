#!/usr/bin/env bash
set -eo pipefail

echo "========================================="
echo "  WarrantX Soroban Testnet Deployment    "
echo "========================================="

RPC_URL="${STELLAR_RPC_URL:-https://soroban-testnet.stellar.org:443}"
NETWORK_PASSPHRASE="${STELLAR_NETWORK_PASSPHRASE:-Test SDF Network ; September 2015}"

if [ -z "$STELLAR_SECRET_KEY" ]; then
    echo "Notice: STELLAR_SECRET_KEY not set. Preparing deployment configurations and output structure."
    echo "To execute live deployment, export STELLAR_SECRET_KEY=<testnet-secret-key>"
    mkdir -p deployments
    cat <<EOF > deployments/testnet.json
{
  "network": "testnet",
  "rpcUrl": "${RPC_URL}",
  "status": "ready_for_key_authorization",
  "contracts": {
    "treasury_wasm_hash": "pending",
    "factory_id": "pending"
  },
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF
    echo "Created deployments/testnet.json template."
    exit 0
fi

echo "Deploying contracts using Stellar CLI..."
cargo build --target wasm32-unknown-unknown --release

TREASURY_WASM="target/wasm32-unknown-unknown/release/warrantx_treasury.wasm"
FACTORY_WASM="target/wasm32-unknown-unknown/release/warrantx_factory.wasm"

echo "Installing Treasury WASM..."
WASM_HASH=$(stellar contract install --wasm "$TREASURY_WASM" --source "$STELLAR_SECRET_KEY" --rpc-url "$RPC_URL" --network-passphrase "$NETWORK_PASSPHRASE")
echo "Treasury WASM Hash: $WASM_HASH"

echo "Deploying Factory Contract..."
FACTORY_ID=$(stellar contract deploy --wasm "$FACTORY_WASM" --source "$STELLAR_SECRET_KEY" --rpc-url "$RPC_URL" --network-passphrase "$NETWORK_PASSPHRASE")
echo "Factory Contract ID: $FACTORY_ID"

mkdir -p deployments
cat <<EOF > deployments/testnet.json
{
  "network": "testnet",
  "rpcUrl": "${RPC_URL}",
  "status": "deployed",
  "contracts": {
    "treasury_wasm_hash": "${WASM_HASH}",
    "factory_id": "${FACTORY_ID}"
  },
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

echo "Deployment records saved to deployments/testnet.json!"
