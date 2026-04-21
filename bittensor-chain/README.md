# bittensor-chain

Subtensor chain client: typed queries, extrinsic submission, and event monitoring.

## Quick Start

```rust,no_run
use bittensor_chain::prelude::*;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Query balance
    let balance = bittensor_chain::queries::account::get_balance(
        client.rpc(), &account_id
    ).await?;

    // Transfer
    let signer = subxt_signer::sr25519::Keypair::from_uri("//Alice")?;
    bittensor_chain::extrinsics::transfer::transfer(
        client.rpc(), &signer, &dest, amount_rao
    ).await?;

    Ok(())
}
```

## Feature Flags

| Feature | Description |
|---|---|
| `storage-subscriptions` (default) | Enable `subscribe_storage` event stream |
| `drand` | Drand randomness beacon verification |
| `mev-shield` | Post-quantum MEV protection for extrinsics |
| `integration-tests` | Enable integration test suite (requires local node) |

## API Overview

| Module | Purpose |
|---|---|
| `client` | `SubtensorClient` — connect, health-check, RPC access |
| `queries` | Read-only chain queries (balance, neurons, metagraph, etc.) |
| `extrinsics` | Signed transactions (transfer, stake, weights, register, etc.) |
| `events` | Event monitoring and filtering (subscribe, decode, dispatch) |
| `generated` | Auto-generated subxt metadata bindings |

## Refreshing Metadata

When the Finney runtime upgrades, regenerate the metadata and API bindings:

```bash
cargo install subxt-cli@0.50.0 --locked
subxt metadata --url wss://entrypoint-finney.opentensor.ai:443 -f bytes > metadata/finney.scale
cargo check -p bittensor-chain
```

Alternative endpoint: `wss://finney.opentensor.ai:443`
