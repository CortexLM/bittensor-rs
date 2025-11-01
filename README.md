<div align="center">

<pre>


██████╗░██╗████████╗████████╗███████╗███╗░░██╗░██████╗░█████╗░██████╗░  ░██████╗██████╗░██╗░░██╗
██╔══██╗██║╚══██╔══╝╚══██╔══╝██╔════╝████╗░██║██╔════╝██╔══██╗██╔══██╗  ██╔════╝██╔══██╗██║░██╔╝
██████╦╝██║░░░██║░░░░░░██║░░░█████╗░░██╔██╗██║╚█████╗░██║░░██║██████╔╝  ╚█████╗░██║░░██║█████═╝░
██╔══██╗██║░░░██║░░░░░░██║░░░██╔══╝░░██║╚████║░╚═══██╗██║░░██║██╔══██╗  ░╚═══██╗██║░░██║██╔═██╗░
██████╦╝██║░░░██║░░░░░░██║░░░███████╗██║░╚███║██████╔╝╚█████╔╝██║░░██║  ██████╔╝██████╔╝██║░╚██╗
╚═════╝░╚═╝░░░╚═╝░░░░░░╚═╝░░░╚══════╝╚═╝░░╚══╝╚═════╝░░╚════╝░╚═╝░░╚═╝  ╚═════╝░╚═════╝░╚═╝░░╚═╝
</pre>

**Bittensor SDK for Rust.**

[![GitHub Stars](https://img.shields.io/github/stars/CortexLM/bittensor-rs?style=flat-square&logo=github)](https://github.com/CortexLM/bittensor-rs/stargazers) [![License](https://img.shields.io/github/license/CortexLM/bittensor-rs?style=flat-square)](https://github.com/CortexLM/bittensor-rs/blob/master/LICENSE) [![Version](https://img.shields.io/badge/version-0.1.0-blue?style=flat-square)](https://github.com/CortexLM/bittensor-rs/releases)

![Alt](https://repobeats.axiom.co/api/embed/233c07ffcbc977111ef312ccfaeeeee736e29a5b.svg "Repobeats analytics image")

</div>


## Installation

The SDK is currently at version **0.1.0**.

### Using cargo add (Recommended)

```bash
cargo add --git https://github.com/CortexLM/bittensor-rs bittensor-rs
```

To install a specific version or branch:

```bash
# Install from a specific branch
cargo add --git https://github.com/CortexLM/bittensor-rs --branch main bittensor-rs

# Install from a specific tag (when tags are available)
cargo add --git https://github.com/CortexLM/bittensor-rs --tag v0.1.0 bittensor-rs
```

### Manual Installation

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
    let client = BittensorClient::with_default().await?;
    
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

