# bittensor-rs

Rust SDK for the Bittensor decentralized AI network. Wallet management, chain interaction, neuron serving, and subnet monitoring, built on subxt 0.50 with typed storage queries and extrinsics.

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

## Feature Flags

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
| Chain client | substrate-interface | subxt 0.50, typed storage + extrinsics |

## Getting Started

### Requirements

- Rust 1.85+ (Edition 2024)
- A running Subtensor node (finney, test, or local)

### Add Dependencies

```toml
[dependencies]
bittensor-core = "0.1"
bittensor-chain = "0.1"
bittensor-wallet = "0.1"
```

Or via cargo:

```sh
cargo add bittensor-core bittensor-chain bittensor-wallet
```

### Connect and Query

```rust,no_run
use bittensor_chain::prelude::SubtensorClient;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;
    let block = client.at_current_block().await?;
    println!("Connected at block {:?}", block.block_hash());
    Ok(())
}
```

### Query Balance

```rust,no_run
use bittensor_chain::prelude::*;
use bittensor_core::config::NetworkConfig;
use bittensor_core::balance::Balance;

let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;
let balance: Balance = bittensor_chain::queries::account::get_balance(
    client.rpc(), &account_id
).await?;
println!("Balance: {balance}");
```

### Transfer TAO

```rust,no_run
use bittensor_chain::prelude::*;
use bittensor_core::config::NetworkConfig;
use bittensor_core::balance::Balance;

let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;
let signer = subxt_signer::sr25519::Keypair::from_uri("//Alice")?;
let dest = subxt_signer::sr25519::PublicKey::from_uri("//Bob")?;
let amount = Balance::from_tao(1.0);

bittensor_chain::extrinsics::transfer::transfer(
    client.rpc(), &signer, &dest, amount.to_rao()
).await?;
```

### Create a Wallet

```rust
use bittensor_wallet::prelude::*;

let mut wallet = Wallet::new("default");
let mnemonic = wallet.create_coldkey("my-password")?;
println!("Back up this mnemonic: {mnemonic}");
println!("Coldkey address: {}", wallet.get_coldkeypub()?);

let hotkey = wallet.create_hotkey()?;
println!("Hotkey address: {}", hotkey.ss58_address());
```

### Network Endpoints

| Network | WebSocket URL |
|---|---|
| Finney (mainnet) | `wss://entrypoint-finney.opentensor.ai:443` |
| Testnet | `wss://test.finney.opentensor.ai:443` |
| Local | `ws://127.0.0.1:9944` |
| Archive | `wss://archive.finney.opentensor.ai:443` |

You can also connect directly with `SubtensorClient::from_url`:

```rust,no_run
let client = SubtensorClient::from_url("wss://entrypoint-finney.opentensor.ai:443").await?;
```

## Documentation

Full documentation is available in the `docs/` directory:

- [Documentation Index](docs/index.md)
- [Getting Started](docs/getting-started.md)
- [Architecture](docs/architecture.md)

## Metadata

Chain metadata is stored at `metadata/finney.scale` and auto-loaded at compile time. When the Finney runtime upgrades, regenerate the metadata and API bindings:

```sh
cargo install subxt-cli@0.50.0 --locked
subxt metadata --url wss://entrypoint-finney.opentensor.ai:443 -f bytes > metadata/finney.scale
cargo check -p bittensor-chain
```

## License

MIT
