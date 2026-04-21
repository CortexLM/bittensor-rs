# bittensor-rs

Rust SDK for the Bittensor network — wallet management, chain interaction, neuron serving, and subnet monitoring.

## Architecture

```
                    ┌─────────────────┐
                    │  bittensor-cli  │
                    │   (btcli-rs)    │
                    └────────┬────────┘
                             │
            ┌────────────────┼────────────────┐
            │                │                │
     ┌──────┴──────┐  ┌──────┴──────┐  ┌──────┴──────┐
     │ bittensor-  │  │ bittensor-  │  │ bittensor-  │
     │   wallet    │  │   tui       │  │   pyo3      │
     └──────┬──────┘  └──────┬──────┘  └──────┬──────┘
            │                │                │
            │         ┌──────┴──────┐         │
            │         │ bittensor-  │         │
            │         │ metagraph   │         │
            │         └──────┬──────┘         │
            │                │                │
            └────────────────┼────────────────┘
                             │
                      ┌──────┴──────┐
                      │ bittensor-  │
                      │   chain     │
                      └──────┬──────┘
                             │
         ┌───────────────────┼───────────────────┐
         │                   │                   │
  ┌──────┴──────┐     ┌──────┴──────┐     ┌──────┴──────┐
  │ bittensor-  │     │ bittensor-  │     │ bittensor-  │
  │   axon      │     │  dendrite   │     │   core      │
  └──────┬──────┘     └──────┬──────┘     └─────────────┘
         │                   │                   ▲
         │            ┌──────┴──────┐            │
         │            │ bittensor-  │            │
         └────────────┤  synapse   ├────────────┘
                      └─────────────┘

  ┌─────────────────────────────────────────────────────┐
  │                   bittensor-wasm                      │
  │  (standalone: reimplements core types for wasm-bindgen)│
  └─────────────────────────────────────────────────────┘

  ┌─────────────────────────────────────────────────────┐
  │                  bittensor-examples                  │
  │  (depends on all native crates for runnable samples) │
  └─────────────────────────────────────────────────────┘
```

## Crates

| Crate | Description |
|---|---|
| **bittensor-core** | Balance arithmetic, error types, config, POW, shared types |
| **bittensor-wallet** | Wallet, keypair, keyfile, mnemonic, SS58 |
| **bittensor-chain** | Subtensor client, queries, extrinsics, events |
| **bittensor-synapse** | Protocol types, headers, hashing, signing, streaming |
| **bittensor-axon** | Axum-based neuron server with middleware and routing |
| **bittensor-dendrite** | HTTP client with request signing and streaming |
| **bittensor-metagraph** | Subnet graph sync, iteration, and serialization |
| **bittensor-cli** | `btcli-rs` command-line tool |
| **bittensor-tui** | Terminal UI dashboard |
| **bittensor-pyo3** | Python bindings (`bittensor_rs` package) |
| **bittensor-wasm** | WASM bindings for browser usage |
| **bittensor-examples** | Runnable code samples |

## Getting Started

Add to your `Cargo.toml`:

```toml
[dependencies]
bittensor-core = "0.1"
bittensor-chain = "0.1"
bittensor-wallet = "0.1"
```

Connect and query:

```rust,no_run
use bittensor_chain::prelude::SubtensorClient;
use bittensor_core::config::NetworkConfig;

let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;
```

## Requirements

- Rust 1.85+ (Edition 2024)
- A running Subtensor node (finney, test, or local)

## License

MIT
