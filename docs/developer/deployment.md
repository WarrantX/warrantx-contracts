# Deployment

Use `scripts/deploy-testnet.sh` with a funded testnet identity. Deploy treasury WASM first, install its hash in the factory, then deploy and initialize treasury instances with the correct admin and asset. Record contract IDs and WASM hashes in release evidence. Never commit a secret key.

