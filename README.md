# Bittensor Rust SDK

A high-performance Rust SDK for interacting with the Bittensor network, providing comprehensive blockchain queries and network operations.

## Overview

The Bittensor Rust SDK offers a complete interface to the Bittensor blockchain, enabling developers to query network state, retrieve neuron information, manage wallets, and interact with various network components. Built on top of Substrate's subxt library, it provides type-safe, asynchronous access to all Bittensor functionality.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
bittensor-rs = { git = "https://github.com/CortexLM/bittensor-rs" }
```

## Documentation

### API Reference

- [Chain Operations](docs/chain.md) - Blockchain client and connection management
- [Query Operations](docs/queries.md) - Comprehensive network queries
- [Type Definitions](docs/types.md) - Core data structures
- [Utilities](docs/utils.md) - Encoding, decoding, and helper functions
- [Validator Operations](docs/validator.md) - Validator-specific functionality

### Query Guides

- [Neuron Queries](docs/queries/neurons.md) - Retrieve neuron information
- [Subnet Queries](docs/queries/subnets.md) - Query subnet configuration and state
- [Delegate Queries](docs/queries/delegates.md) - Access delegate information
- [Wallet Queries](docs/queries/wallets.md) - Check balances and account data
- [Staking Queries](docs/queries/staking.md) - Query stake distributions

## Quick Start

### Basic Connection

```rust
use bittensor_rs::chain::BittensorClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = BittensorClient::new("wss://entrypoint-finney.opentensor.ai:443").await?;
    
    // Query chain information
    let block_number = client.get_block_number().await?;
    println!("Current block: {}", block_number);
    
    Ok(())
}
```

### Querying Neurons

```rust
use bittensor_rs::queries::neurons;

let neurons = neurons::neurons(&client, netuid, None).await?;
for neuron in neurons.iter() {
    println!("UID: {}, Stake: {}", neuron.uid, neuron.stake);
}
```

## Examples

The `examples/` directory contains working code examples for all major SDK features. See the [Examples Documentation](examples/README.md) for detailed descriptions.

## Architecture

The SDK is organized into modular components:

- **chain** - Core blockchain client implementation
- **queries** - All network query functions organized by domain
- **types** - Rust structs representing Bittensor data types
- **utils** - Utility functions for common operations
- **validator** - Validator-specific operations (staking, weights, registration)

## Building from Source

### Prerequisites

- Rust 1.70 or higher
- Cargo

### Build Commands

```bash
# Build the library
cargo build --release

# Run tests
cargo test

# Build documentation
cargo doc --open

# Run a specific example
cargo run --example chain_info
```

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test queries::tests
```

### Code Style

The project follows standard Rust formatting conventions. Run `cargo fmt` before submitting pull requests.

## Contributing

Contributions are welcome. Please submit pull requests against the main branch. Ensure all tests pass and add appropriate documentation for new features.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Support

For issues and feature requests, please use the GitHub issue tracker.

---

Developed by [Cortex Foundation](https://github.com/CortexLM)

