# Query Operations

The Bittensor Rust SDK provides comprehensive query capabilities for interacting with the Bittensor network. All query functions are organized by domain and optimized for performance.

## Overview

Query operations are the primary interface for retrieving data from the Bittensor blockchain. These operations are read-only and do not require transaction signing. All on-chain balances, stake, and emission values are returned in RAO (`u128`); convert to TAO for display only.

## Parity Notes (Python SDK)

- Storage key names and indices must match Subtensor runtime metadata. `NetUidStorageIndex` is `u16`, computed as `mechanism_id * 4096 + netuid`.
- Emission, stake, and balances are RAO on-chain; TAO formatting is for display only.
- Commit-reveal and CRv4 queries depend on runtime storage entries (e.g., `CRV3WeightCommitsV2`, `TimelockedWeightCommits`). Validate against runtime metadata.
- See `docs/parity_checklist.md` for detailed parity gaps.
- Finney metadata reference: `metadata_hash=0x31a1392ead4c198c974610bc078f69346261648d306def22607e95fc521baf50`, `spec_version=377`.
## Parity Notes (Python SDK)

- Storage key names and indices must match Subtensor runtime metadata. `NetUidStorageIndex` is `u16`, computed as `mechanism_id * 4096 + netuid`.
- Emission, stake, and balances are RAO on-chain; TAO formatting is for display only.
- Commit-reveal and CRv4 queries depend on runtime storage entries (e.g., `CRV3WeightCommitsV2`, `TimelockedWeightCommits`). Validate against runtime metadata.
- See `docs/parity_checklist.md` for detailed parity gaps.

## Query Categories

### Network Queries

- **[Neurons](queries/neurons.md)** - Query individual neurons and their properties
- **[Subnets](queries/subnets.md)** - Retrieve subnet configuration and statistics
- **[Delegates](queries/delegates.md)** - Access delegate information and voting power
- **[Staking](queries/staking.md)** - Query stake distributions and delegations
- **[Wallets](queries/wallets.md)** - Check account balances and information
- **[Commitments](queries/commitments.md)** - Commit-reveal and timelocked commitment queries

### System Queries

- **Chain Information** - Block height, runtime version, chain properties
- **Storage Queries** - Direct access to chain storage
- **Metadata** - Runtime metadata and type information

## Common Patterns

### Basic Query Structure

All queries follow a consistent pattern:

```rust
use bittensor_rs::chain::BittensorClient;
use bittensor_rs::queries;

// Connect to the network
let client = BittensorClient::with_default().await?;

// Execute query
let result = queries::module::function(&client, parameters).await?;
```

### Error Handling

All query functions return `Result<T, Error>` where errors can include:

- Connection errors
- Decoding errors
- Storage not found errors
- Invalid parameters

```rust
match queries::neurons::get_neuron(&client, netuid, uid).await {
    Ok(Some(neuron)) => println!("Found neuron: {:?}", neuron),
    Ok(None) => println!("Neuron not found"),
    Err(e) => eprintln!("Query error: {}", e),
}
```

### Block Parameter

Most queries accept an optional block parameter to query historical state:

```rust
// Query at latest block
let neurons = queries::neurons::neurons(&client, netuid, None).await?;

// Query at specific block
let block_number = 1234567;
let neurons = queries::neurons::neurons(&client, netuid, Some(block_number)).await?;
```

## Performance Considerations

### Bulk Queries

For retrieving multiple items, use bulk query functions when available:

```rust
// Efficient: Single bulk query
let all_neurons = queries::neurons_bulk::neurons_bulk(&client, netuid, None).await?;

// Inefficient: Multiple individual queries
for uid in 0..n {
    let neuron = queries::neurons::get_neuron(&client, netuid, uid).await?;
}
```

### Caching

Query results can be cached locally to reduce network calls:

```rust
use std::time::{Duration, Instant};

struct CachedData<T> {
    data: T,
    timestamp: Instant,
}

impl<T> CachedData<T> {
    fn is_valid(&self, max_age: Duration) -> bool {
        self.timestamp.elapsed() < max_age
    }
}
```

## Storage Keys

For advanced use cases, you can query storage directly:

```rust
use subxt::dynamic::Value;

// Direct storage query
let key = vec![Value::u128(netuid as u128)];
let value = client.storage_with_keys("SubtensorModule", "SubnetworkN", key).await?;
```

## Type Decoding

The SDK provides utilities for decoding storage values:

```rust
use bittensor_rs::utils::decoders;

// Decode various types from storage Value
let u64_value = decoders::decode_u64(&value)?;
let account = decoders::decode_account_id32(&value)?;
let vec_u16 = decoders::decode_vec_u16(&value)?;
```

## Next Steps

- Review specific query documentation for detailed API references
- See the examples directory for working code samples
- Consult type definitions for data structure details
- Track parity updates in [Parity Checklist](parity_checklist.md)