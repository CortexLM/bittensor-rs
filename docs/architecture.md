# SDK Architecture

This document describes the high-level architecture and organization of the Bittensor Rust SDK.

## Module Structure

The SDK is organized into modular components:

### Core Modules

- **chain** - Core blockchain client implementation using subxt
- **queries** - Network query functions organized by domain (neurons, subnets, delegates, etc.)
- **types** - Rust structs representing Bittensor data types
- **utils** - Utility functions for encoding, decoding, address conversion, and weight normalization
- **validator** - Validator-specific operations (staking, weights, registration, serving)

### Module Details

#### chain

The `chain` module provides the main `BittensorClient` that manages connections to Bittensor nodes via WebSocket RPC. It handles:

- Connection management
- Storage queries
- Transaction submission
- Event monitoring
- Runtime API calls

**Key Components:**

- `BittensorClient` - Main client struct
- `with_default()` - Initialize with default or environment-specified RPC endpoint
- Storage query helpers
- Transaction builders

#### queries

The `queries` module provides organized functions for querying network state:

- **neurons.rs** - Neuron information queries (bulk, single, lightweight)
- **subnets.rs** - Subnet configuration and state queries
- **delegates.rs** - Delegate information and nominator queries
- **wallets.rs** - Balance and account queries
- **stakes.rs** - Stake distribution queries
- **voting.rs** - Governance and voting queries
- **liquidity.rs** - Liquidity pool queries

All query functions are optimized for performance using bulk operations and concurrent fetching where possible.

#### types

The `types` module defines all data structures:

- **neuron.rs** - `NeuronInfo`, `NeuronInfoLite`
- **subnet.rs** - `SubnetInfo`, `SubnetConfigInfo`, `SubnetIdentity`
- **delegate.rs** - `DelegateInfo`, `DelegatedInfo`
- **axon.rs** - `AxonInfo`
- **prometheus.rs** - `PrometheusInfo`
- **commitment.rs** - `WeightCommitInfo`
- **liquidity.rs** - `LiquidityPosition`
- **proposal_vote.rs** - `ProposalVoteData`

All types implement `Serialize` and `Deserialize` for JSON compatibility.

#### utils

The `utils` module provides helper functions:

- **weights.rs** - Weight normalization and denormalization
- **ss58.rs** - SS58 address encoding/decoding
- **encode.rs** - SCALE encoding utilities
- **scale_decode.rs** - SCALE decoding utilities
- **decoders.rs** - Value type decoding from subxt
- **crypto.rs** - Cryptographic utilities (commitment hashing)
- **balance_newtypes.rs** - RAO/TAO conversion

#### validator

The `validator` module provides operations for validators:

- **weights.rs** - Set, commit, and reveal weights
- **staking.rs** - Add, remove, move, and swap stake
- **registration.rs** - Register neurons on subnets
- **serving.rs** - Serve axon and Prometheus endpoints
- **take.rs** - Delegate take management
- **transfer.rs** - Token and stake transfers
- **mechanism.rs** - Mechanism-specific weight operations
- **root.rs** - Root subnet operations
- **children.rs** - Child subnet operations
- **liquidity.rs** - Liquidity pool operations

## Design Principles

### Performance

- **Bulk Operations**: Use storage queries that fetch multiple items at once
- **Concurrent Execution**: Use `FuturesUnordered` for parallel requests
- **Minimal Network Calls**: Batch related queries together
- **Efficient Encoding**: Use SCALE encoding for Substrate compatibility

### Type Safety

- **Strong Typing**: All data structures are strongly typed
- **AccountId32**: Use Substrate-compatible account IDs throughout
- **Error Handling**: All operations return `Result<T>` for proper error handling

### Compatibility

- **Subtensor Format**: All extrinsic calls match Subtensor's expected format
- **Weights Format**: UIDs and weights as `Vec<u16>`, scaled by `u16::MAX`
- **IP Encoding**: IPv4 as `u32` within `u128`, IPv6 direct as `u128`
- **SCALE Encoding**: All serialization uses SCALE encoding

## Data Flow

### Query Flow

1. **Client Initialization**: Create `BittensorClient` with RPC endpoint
2. **Storage Query**: Query storage using subxt storage API
3. **Value Decoding**: Decode SCALE-encoded `Value` types to Rust structs
4. **Type Conversion**: Convert to SDK types (`NeuronInfo`, `SubnetInfo`, etc.)

### Transaction Flow

1. **Transaction Building**: Build extrinsic call using subxt
2. **Signing**: Sign transaction with key pair
3. **Submission**: Submit transaction to chain
4. **Monitoring**: Wait for inclusion and finality

## Error Handling

All operations use Rust's `Result<T, E>` type for error handling:

- **anyhow::Result** - Used for general error propagation
- **Custom Errors** - Domain-specific error types where appropriate
- **Error Context** - Errors include context about the operation

## Configuration

### Environment Variables

- **BITTENSOR_RPC** - Custom RPC endpoint (defaults to `wss://entrypoint-finney.opentensor.ai:443`)

### Client Initialization

```rust
use bittensor_rs::chain::BittensorClient;

// Use default or environment variable
let client = BittensorClient::with_default().await?;

// Or specify custom endpoint
let client = BittensorClient::new(Some("wss://custom.endpoint:443")).await?;
```

## Dependencies

### Core Dependencies

- **subxt** - Substrate client library
- **sp-core** - Substrate core types
- **serde** - Serialization framework
- **anyhow** - Error handling
- **tokio** - Async runtime

### Optional Dependencies

- **sp-keyring** - Key management (for signing)
- **scale-codec** - SCALE encoding/decoding

## Testing

The SDK includes:

- **Unit Tests** - Test individual functions and types
- **Integration Tests** - Test against live or mock networks
- **Example Programs** - Working examples in `examples/` directory

## Extension Points

The architecture supports extension through:

- **Custom Query Functions** - Add new query functions following existing patterns
- **Custom Types** - Add new types in the `types` module
- **Custom Validator Operations** - Add new validator operations in the `validator` module
- **Utility Functions** - Add helper functions in the `utils` module

## Performance Considerations

### Bulk Queries

Use bulk query functions when fetching multiple items:

- `neurons()` - Fetch all neurons in a subnet
- `get_all_subnets()` - Fetch all subnet information

### Concurrent Requests

The SDK uses `FuturesUnordered` for concurrent requests where safe:

- Fetching per-neuron data (axon info, Prometheus info)
- Fetching stake distributions
- Fetching delegate information

### Caching

Consider caching frequently accessed data:

- Subnet information
- Delegate information
- Neuron metadata

## Best Practices

1. **Connection Reuse**: Reuse `BittensorClient` instances when possible
2. **Bulk Operations**: Use bulk query functions for multiple items
3. **Error Handling**: Always handle `Result` types appropriately
4. **Type Safety**: Use strongly-typed structs instead of raw values
5. **Documentation**: Follow existing patterns when adding new functionality
