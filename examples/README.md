# Bittensor Rust SDK Examples

This directory contains comprehensive examples demonstrating how to use the Bittensor Rust SDK. Each example is self-contained and can be run independently.

## Prerequisites

Before running the examples, ensure you have:
- Rust 1.70 or higher installed
- Network connectivity to a Bittensor RPC endpoint
- (Optional) Set `BITTENSOR_RPC` environment variable for custom endpoint (defaults to Finney)

## Running Examples

Execute any example using cargo:

```bash
cargo run --example <example_name>
```

For custom RPC endpoint:
```bash
BITTENSOR_RPC=wss://your-endpoint:443 cargo run --example chain_info
```

## Available Examples

### Basic Queries

#### chain_info
Query fundamental blockchain information including block height, runtime version, and chain properties.

```bash
cargo run --example chain_info
```

Output includes:
- Current block number
- Chain name and properties
- Runtime version
- Token decimals and symbols

#### wallets
Query wallet balances and account information for single or multiple addresses.

```bash
cargo run --example wallets
```

Features:
- Query balance for specific addresses
- Display free, reserved, and frozen balance
- Support for SS58 address format

#### subnets
List all subnets on the network with their configuration and statistics.

```bash
cargo run --example subnets
```

Displays:
- Subnet ID and owner
- Network parameters (tempo, emission ratio)
- Registration status and costs
- Number of neurons

### Neuron Operations

#### neurons
Query and display individual neuron information within subnets.

```bash
cargo run --example neurons
```

Shows:
- Neuron UID and addresses (hotkey/coldkey)
- Stake amounts and distribution
- Performance metrics (rank, trust, consensus)
- Network endpoints (axon/prometheus)

#### metagraph
Display subnet metagraph in a formatted table showing key metrics.

```bash
cargo run --example metagraph
```

Output format:
- Tabular view of all neurons
- Stake, rank, trust, consensus values
- Active status and validator permits
- Emission rates



### Network Components

#### delegates
Query delegate information and analyze delegation patterns.

```bash
cargo run --example delegates
```

Shows:
- Delegate addresses and metadata
- Total stake and nominator count
- Commission rates (take)
- Return calculations

#### voting
Interact with governance voting system and proposals.

```bash
cargo run --example voting
```

Features:
- List active proposals
- Query vote counts
- Check senate membership
- Display voting history

#### commitments
Query weight commitment data for commit-reveal schemes.

```bash
cargo run --example commitments
```

Displays:
- Current commitments
- Reveal status
- Block numbers for commits

### Advanced Features

#### identity
Query and display on-chain identity information.

```bash
cargo run --example identity
```

Shows:
- Identity fields (name, email, web, etc.)
- Verification status
- Identity judgements

#### liquidity
Query liquidity pool information and swap calculations.

```bash
cargo run --example liquidity
```

Features:
- Pool reserves
- Swap rate calculations
- Liquidity provider information

## Environment Variables

- `BITTENSOR_RPC` - Custom RPC endpoint (default: wss://entrypoint-finney.opentensor.ai:443)
- `SEED` - Random seed for subnet selection in examples

## Common Patterns

### Error Handling

All examples implement proper error handling:

```rust
match result {
    Ok(data) => process_data(data),
    Err(e) => eprintln!("Error: {}", e),
}
```

### Connection Management

Examples establish connection once and reuse:

```rust
let client = BittensorClient::new(rpc_url).await?;
// Multiple queries using same client
```

### Address Format

Examples use SS58 encoding for display:

```rust
use sp_core::crypto::Ss58Codec;
println!("Address: {}", account_id.to_ss58check());
```

## Extending Examples

To create new examples:

1. Create a new file in `examples/` directory
2. Add the example to `Cargo.toml` if needed
3. Follow the established patterns for error handling and output formatting
4. Document the example purpose and usage

## Performance Notes

- Use bulk query functions when fetching multiple items
- Cache results when appropriate
- Consider rate limiting for continuous monitoring
- Prefer specific queries over fetching all data

## Troubleshooting

### Connection Issues
- Verify RPC endpoint is accessible
- Check network connectivity
- Ensure WebSocket support for WSS endpoints

### Decoding Errors
- Update to latest SDK version
- Verify runtime compatibility
- Check for network upgrades

### Performance Problems
- Use bulk query examples as reference
- Implement caching for repeated queries
- Consider concurrent fetching patterns

## Additional Resources

- [API Documentation](../docs/queries.md)
- [Type Definitions](../docs/types.md)
- [SDK Architecture](../docs/architecture.md)