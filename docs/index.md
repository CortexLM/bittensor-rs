# bittensor-rs Documentation

Welcome to the bittensor-rs SDK documentation. This Rust SDK provides typed, memory-safe access to the Bittensor decentralized AI network, mirroring the capabilities of the Python SDK while offering native performance, compile-time guarantees, and WASM compatibility.

## Table of Contents

### Getting Started

- [Getting Started](getting-started.md) - Installation, prerequisites, your first program, connecting to the network

### Architecture

- [Architecture Overview](architecture.md) - Crate structure, data flows, design principles, query and transaction pipelines

### Crate References

| Crate | Documentation | Description |
|---|---|---|
| **bittensor-core** | [Architecture](architecture.md) | Balance arithmetic, error types, config, POW, shared types |
| **bittensor-wallet** | [Wallet](wallet.md) | Wallet, keypair, keyfile, mnemonic, SS58 |
| **bittensor-chain** | [Chain Client](chain-client.md) | Subtensor client, queries, extrinsics, events |
| **bittensor-synapse** | [Synapse](synapse.md) | Protocol types, headers, hashing, signing, streaming |
| **bittensor-axon** | [Axon](axon.md) | Axum-based neuron server with middleware and routing |
| **bittensor-dendrite** | [Dendrite](dendrite.md) | HTTP client with request signing and streaming |
| **bittensor-metagraph** | [Metagraph](metagraph-lib.md) | Subnet graph sync, iteration, and serialization |
| **bittensor-cli** | [CLI](cli.md) | `btcli-rs` command-line tool |
| **bittensor-tui** | [TUI](tui.md) | Terminal UI dashboard |
| **bittensor-pyo3** | [Python Bindings](python-bindings.md) | Python bindings (`bittensor_rs` package) |
| **bittensor-wasm** | [WASM Bindings](wasm-bindings.md) | Browser bindings via wasm-bindgen |
| **bittensor-examples** | Source only | Runnable code samples demonstrating each subsystem |

### Chain Queries

Read-only storage queries against the Subtensor chain:

- [Account Queries](queries/account.md) - Balance, stake info, hotkey ownership
- [Delegate Queries](queries/delegate.md) - Delegate info, take rates, delegated stake
- [Metagraph Queries](queries/metagraph.md) - Subnet graph state queries
- [Neuron Queries](queries/neuron.md) - Individual neuron lookup by UID
- [Subnet Queries](queries/subnet.md) - Subnet info and hyperparameters
- [Staking Queries](queries/staking.md) - Stake balances, delegation info
- [Children Queries](queries/children.md) - Child hotkey and take rate queries
- [Commit Queries](queries/commit.md) - Weight commit and reveal queries
- [Identity Queries](queries/identity.md) - Subnet identity queries
- [Network Queries](queries/network.md) - Network constants, version, tempo
- [Proxy Queries](queries/proxy.md) - Proxy account queries
- [Weights Queries](queries/weights.md) - Weight matrix queries

### Chain Extrinsics

Signed transactions that modify chain state:

- [Staking](extrinsics/staking.md) - add_stake, remove_stake, move_stake, swap_stake, transfer_stake
- [Transfer](extrinsics/transfer.md) - Transfer TAO between accounts
- [Registration](extrinsics/registration.md) - POW register, burned register, root register
- [Weights](extrinsics/weights.md) - set_weights, commit_weights, reveal_weights, commit_timelocked_weights
- [Serving](extrinsics/serving.md) - serve_axon, serve_axon_tls
- [Delegate](extrinsics/delegate.md) - increase_take, decrease_take, become_delegate
- [Children](extrinsics/children.md) - set_children, set_childkey_take
- [Proxy](extrinsics/proxy.md) - add_proxy, remove_proxy
- [Root](extrinsics/root.md) - root_set_weights, root_claim
- [Coldkey Swap](extrinsics/coldkey-swap.md) - swap_coldkey

### Migration and Reference

- [Migration Guide](migration-guide.md) - Migrating from the Python bittensor SDK to Rust
- [Glossary](glossary.md) - Bittensor and Rust SDK terminology
- [FAQ](faq.md) - Frequently asked questions

## Network Endpoints

| Network | WebSocket URL | Use Case |
|---|---|---|
| Finney (mainnet) | `wss://entrypoint-finney.opentensor.ai:443` | Production |
| Testnet | `wss://test.finney.opentensor.ai:443` | Testing |
| Local | `ws://127.0.0.1:9944` | Development |
| Archive | `wss://archive.finney.opentensor.ai:443` | Historical queries |

## Feature Flags

Feature flags control optional functionality at compile time.

### bittensor-chain

| Feature | Default | Description |
|---|---|---|
| `storage-subscriptions` | yes | Enable `subscribe_storage` event stream |
| `drand` | no | Drand randomness beacon verification |
| `mev-shield` | no | Post-quantum MEV protection for extrinsics |
| `integration-tests` | no | Integration test suite (requires local node) |

### bittensor-metagraph

| Feature | Default | Description |
|---|---|---|
| `ml-backend` | no | ML-based weight scoring backend |

### bittensor-cli

| Feature | Default | Description |
|---|---|---|
| `mev` | no | MEV-shield protected transactions |

## Python SDK vs Rust SDK

| Feature | Python SDK | Rust SDK |
|---|---|---|
| Performance | Interpreter overhead | Native binary, zero-cost abstractions |
| Memory safety | GC managed | Compile-time borrow checking |
| Concurrency | GIL-limited async | Native async with tokio, no GIL |
| WASM support | No | Full wasm-bindgen support |
| Python bindings | Native | Via bittensor-pyo3 (`bittensor_rs`) |
| Typing | Runtime duck typing | Static types, subxt-generated metadata |
| Keyfile format | NaCl secretbox | Same NaCl secretbox, cross-compatible |
| Chain client | substrate-interface | subxt 0.50, typed storage and extrinsics |
