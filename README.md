# WarrantX Contracts

[![CI](https://github.com/Michealshodipo56/warrantx-contracts/actions/workflows/ci.yml/badge.svg)](https://github.com/Michealshodipo56/warrantx-contracts/actions/workflows/ci.yml)
[![Stellar](https://img.shields.io/badge/Stellar-Soroban-black)](https://stellar.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

[Live application](https://warrantx-app.vercel.app) · [Documentation](https://entity-6.gitbook.io/warrantx-documentation) · [Testnet contract](https://stellar.expert/explorer/testnet/contract/CATX47HYXQMHZALH3HYM6JKILXYGPVU324RKQZBPKEJH7TOVAEPMYT2R)

WarrantX is an open-source policy engine for Stellar treasuries. Its Soroban contracts enforce member roles, recurring spending limits, approval thresholds, policy versioning, and token transfers. This repository contains the treasury contract and its deployment factory; the companion `warrantx-app` repository provides the dashboard, API, indexer, and TypeScript SDK.

> **Release status:** v0.1.1 is a testnet-only preview. The contracts are unaudited. Do not use this release to custody material mainnet funds.

## Contract architecture

```text
Factory
  └─ deploys Treasury instances from an approved WASM hash

Treasury
  ├─ members and roles
  ├─ versioned spending policies
  ├─ recurring allowance accounting
  ├─ requests and approvals
  └─ Soroban token transfers
```

## Quick start

Requires stable Rust, the `wasm32-unknown-unknown` target, and Stellar CLI.

```bash
rustup target add wasm32-unknown-unknown
cargo test --workspace
./scripts/build.sh
```

The [structured documentation](docs/SUMMARY.md) covers architecture, lifecycles, public functions, security boundaries, deployment, and testing.

## Quality gates

Pull requests must pass the `contracts-ci` check: formatting, Clippy with warnings denied, the complete workspace test suite, and release WASM compilation. Contract changes require tests and a review of authorization, storage, events, overflow, token transfers, and compatibility.

## Maintainer

| Maintainer | GitHub |
| --- | --- |
| Micheal Shodipo | [@Michealshodipo56](https://github.com/Michealshodipo56) |

## Contributing and security

See [CONTRIBUTING.md](CONTRIBUTING.md) and [SECURITY.md](SECURITY.md). By participating, you agree to [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

## Contributors

<a href="https://github.com/Michealshodipo56/warrantx-contracts/graphs/contributors"><img src="https://contrib.rocks/image?repo=Michealshodipo56/warrantx-contracts" alt="WarrantX contributors" /></a>

## License

MIT. See [LICENSE](LICENSE).
