# Bittensor Rust SDK v2

A Rust SDK for interacting with the Bittensor network, designed to match the Python SDK's interface and functionality.

## Features

- **Subtensor Client**: Connect to the Bittensor blockchain and query state
- **Metagraph**: Access subnet state including neurons, stakes, and rankings
- **Chain Data Types**: NeuronInfo, AxonInfo, SubnetInfo, etc.
- **Query Functions**: Query neurons, subnets, stakes, and more
- **Balance Type**: Handle TAO/RAO conversions with operator support
- **SS58 Utilities**: Encode/decode Substrate addresses

## Quick Start

```rust
use bittensor_rs::{Subtensor, Metagraph};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Connect to finney (mainnet)
    let subtensor = Subtensor::new("finney").await?;

    // Get metagraph for subnet 1
    let metagraph = subtensor.metagraph(1).await?;

    println!("Subnet 1 has {} neurons", metagraph.n);
    println!("Total stake: {}", metagraph.total_stake());

    // Access neuron data by UID
    for uid in 0..metagraph.n.min(5) as usize {
        println!(
            "UID {}: hotkey={}, stake={}, incentive={:.4}",
            uid,
            metagraph.hotkeys[uid],
            metagraph.stake[uid],
            metagraph.incentive[uid]
        );
    }

    Ok(())
}
```

## Networks

The SDK supports the following networks:

- `finney` - Bittensor mainnet (default)
- `test` - Test network
- `archive` - Archive node with full history
- `local` - Local node (ws://127.0.0.1:9944)
- `latent-lite` - Latent lite network

You can also pass a WebSocket URL directly:

```rust
let subtensor = Subtensor::new("wss://custom-node.example.com").await?;
```

## Modules

### `config`

Network configuration and settings:

```rust
use bittensor_rs::config::{Network, FINNEY_ENTRYPOINT, DEFAULTS};
```

### `types`

Chain data types matching the Python SDK:

- `AxonInfo` - Axon endpoint information
- `NeuronInfo` / `NeuronInfoLite` - Neuron metadata
- `PrometheusInfo` - Prometheus endpoint info
- `SubnetInfo` / `SubnetHyperparameters` - Subnet data
- `DelegateInfo` - Delegate information

### `metagraph`

The Metagraph struct provides tensor-like arrays for neuron attributes:

```rust
let mg = subtensor.metagraph(1).await?;

// Property accessors (matching Python SDK)
mg.s();  // stake
mg.r();  // ranks
mg.i();  // incentive
mg.e();  // emission
mg.c();  // consensus
mg.t();  // trust
mg.tv(); // validator_trust
mg.d();  // dividends
mg.w();  // weights
mg.b();  // bonds

// Utility methods
mg.total_stake();
mg.get_neuron(uid);
mg.get_uid("hotkey_address");
mg.validators();
mg.active_neurons();
```

### `utils`

Utility functions:

- `Balance` - TAO/RAO amount handling
- `ss58_encode` / `ss58_decode` - Address encoding
- `is_valid_ss58_address` - Address validation

## Differences from Python SDK

This is a read-only SDK focused on querying blockchain state. It does **not** include:

- Wallet operations (stake, transfer, etc.)
- Extrinsic submission
- Key management

For write operations, use the Python SDK or submit extrinsics directly via subxt.

## Dependencies

- `subxt` - Substrate client
- `sp-core` - Substrate primitives
- `tokio` - Async runtime
- `serde` - Serialization

## License

MIT
