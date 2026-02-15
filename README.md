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

[![GitHub Stars](https://img.shields.io/github/stars/CortexLM/bittensor-rs?style=flat-square&logo=github)](https://github.com/CortexLM/bittensor-rs/stargazers) [![License](https://img.shields.io/github/license/CortexLM/bittensor-rs?style=flat-square)](https://github.com/CortexLM/bittensor-rs/blob/master/LICENSE) [![Version](https://img.shields.io/badge/version-0.1.0-blue?style=flat-square)](https://github.com/CortexLM/bittensor-rs/releases) ![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
[![CI](https://github.com/CortexLM/bittensor-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/CortexLM/bittensor-rs/actions/workflows/ci.yml)

![Alt](https://repobeats.axiom.co/api/embed/233c07ffcbc977111ef312ccfaeeeee736e29a5b.svg "Repobeats analytics image")

</div>

> [!WARNING]
> This code is currently under active development and may contain bugs. Please thoroughly test the code before using it in production environments.

## Performance Benchmarks

The Rust SDK significantly outperforms the Python SDK for both local and network operations.

### Local Operations (CPU-bound)

| Operation                              | Python SDK   | Rust SDK     | Speedup   |
| -------------------------------------- | ------------ | ------------ | --------- |
| rao_to_tao (1000 ops)                  | 0.341 ms     | 0.003 ms     | **113x**  |
| tao_to_rao (1000 ops)                  | 0.402 ms     | 0.002 ms     | **201x**  |
| normalize_max_weight (256 neurons)     | 0.006 ms     | 0.001 ms     | **6x**    |
| convert_weights_and_uids (256 neurons) | 0.046 ms     | 0.001 ms     | **46x**   |
| normalize_max_weight (1000 neurons)    | 0.008 ms     | 0.001 ms     | **8x**    |
| u16_normalized_float (10000 ops)       | 0.203 ms     | 0.008 ms     | **25x**   |
| convert_to_tensor (128 uids)           | 0.010 ms     | 0.000 ms     | **>100x** |
| **Total**                              | **1.016 ms** | **0.015 ms** | **~68x**  |

### Network Operations (I/O-bound)

| Operation         | Python SDK | Rust SDK | Speedup  |
| ----------------- | ---------- | -------- | -------- |
| Connection time   | 2642 ms    | 1017 ms  | **2.6x** |
| get_current_block | 107 ms     | 183 ms   | 0.6x     |
| get_total_subnets | 214 ms     | 182 ms   | **1.2x** |
| subnet_exists     | 217 ms     | 183 ms   | **1.2x** |
| tempo             | 448 ms     | 188 ms   | **2.4x** |
| subnetwork_n      | 461 ms     | 212 ms   | **2.2x** |
| get_delegates     | 10289 ms   | 7403 ms  | **1.4x** |

> **Note**: Network operations are dominated by RPC latency (~180ms per call). The Rust SDK uses optimized connection pooling, async I/O, and direct SCALE decoding for maximum performance. Module loading time is excluded from benchmarks (Python: ~800ms, Rust: ~0ms).

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
## Quick Start

### Basic Connection

```rust
use bittensor_rs::chain::BittensorClient;
### Basic Connection

```rust
use bittensor_rs::chain::BittensorClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = BittensorClient::with_default().await?;

    // Query chain information
    let block_number = client.block_number().await?;
    println!("Current block: {}", block_number);

    Ok(())
}
```

The default RPC endpoint is `wss://entrypoint-finney.opentensor.ai:443`. Override it with the
`BITTENSOR_RPC` environment variable or by passing a custom URL to `BittensorClient::new`.

### Querying Neurons

```rust
use bittensor_rs::queries::neurons;

let neurons = neurons::neurons(&client, 1, None).await?;
for neuron in neurons.iter() {
    println!("UID: {}, Stake: {}", neuron.uid, neuron.stake);
}
```

### Balance Utilities

```rust
use bittensor_rs::utils::balance::{Balance, rao_to_tao, tao_to_rao};

// Convert between TAO (display) and RAO (on-chain)
let rao = tao_to_rao(1.5);  // 1_500_000_000 RAO (truncated)
let tao = rao_to_tao(1_000_000_000);  // 1.0 TAO

// Use Balance struct (RAO is the source of truth)
let balance = Balance::from_rao(2_500_000_000);
println!("Balance: {} TAO ({} RAO)", balance.as_tao(), balance.as_rao());
```

### Weight Normalization

```rust
use bittensor_rs::utils::weights::{normalize_max_weight, normalize_weights};

// Normalize weights with max limit (same as Python SDK)
let weights = vec![0.1, 0.2, 0.3, 0.4];
let normalized = normalize_max_weight(&weights, 0.35);

// Convert to u16 format for chain submission
let uids = vec![0u64, 1, 2];
let weights = vec![0.25f32, 0.5, 0.25];
let (uid_vec, weight_vec) = normalize_weights(&uids, &weights)?;
```

## Examples

The `examples/` directory contains working code examples for all major SDK features. See the [Examples Documentation](examples/README.md) for detailed descriptions.

## Architecture

The SDK is organized into modular components:

- **chain** - Core blockchain client implementation
- **config** - Configuration management (network, axon, logging)
- **core** - Core constants and protocol definitions
- **metagraph** - Metagraph synchronization and types
- **queries** - All network query functions organized by domain
- **types** - Rust structs representing Bittensor data types
  - **Synapse** - Network message types
  - **DynamicInfo** - Comprehensive subnet information
  - **MetagraphInfo** - Complete metagraph data structure
  - **NeuronInfo** - Neuron data with all fields
- **utils** - Utility functions for common operations
  - **weights** - Weight normalization and processing
  - **balance** - TAO/RAO conversion
  - **crypto** - Cryptographic utilities
- **validator** - Validator-specific operations (staking, weights, registration)

## Python SDK Compatibility

This SDK is designed to be compatible with the Python Bittensor SDK. Key features include:

- Same data structures (NeuronInfo, SubnetInfo, DelegateInfo, etc.)
- Compatible weight normalization algorithms
- Same balance conversion (1 TAO = 1e9 RAO)
- Same SS58 address format

## Building from Source

### Prerequisites

- Rust 1.70 or higher
- Cargo

## CI/CD

Automated GitHub Actions run on push and pull requests:

- Build on `stable` and `nightly` toolchains
- Run unit and doc tests with `--all-features`
- Enforce formatting with `cargo fmt --check`
- Enforce lints with `cargo clippy -D warnings`

Workflow file: `.github/workflows/ci.yml`. Check the latest status via the CI badge above.

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

## Testing

```bash
# Run all unit tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test weight_comparison
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