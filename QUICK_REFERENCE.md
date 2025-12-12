# Bittensor Rust SDK Quick Reference

## Installation

```toml
[dependencies]
bittensor-rs = { git = "https://github.com/CortexLM/bittensor-rs" }
```

## Basic Connection

```rust
use bittensor_rs::chain::BittensorClient;

let client = BittensorClient::with_default().await?;
```

## Common Queries

### Network Information

```rust
use bittensor_rs::queries::{chain_info, subnets};

// Chain info
let block_number = client.get_block_number().await?;
let chain_name = chain_info::get_chain_name(&client).await?;

// Subnet count
let total = subnets::total_subnets(&client).await?;
```

### Neuron Queries

```rust
use bittensor_rs::queries::neurons;

// Get all neurons
let neurons = neurons::neurons(&client, netuid, None).await?;

// Get specific neuron
let neuron = neurons::get_neuron(&client, netuid, uid).await?;

// Optimized bulk fetch
use bittensor_rs::queries::neurons_bulk;
let all_neurons = neurons_bulk::neurons_bulk(&client, netuid, None).await?;
```

### Wallet Balance

```rust
use bittensor_rs::queries::wallets;

let balance = wallets::get_balance(&client, &account_id).await?;
println!("Balance: {} TAO", balance as f64 / 1e9);
```

### Delegate Information

```rust
use bittensor_rs::queries::delegates;

// Get all delegates
let delegates = delegates::get_all_delegates_info(&client).await?;

// Get specific delegate
let delegate = delegates::get_delegate(&client, &delegate_address).await?;
```

## Address Conversion

```rust
use sp_core::crypto::Ss58Codec;

// To SS58
let ss58 = account_id.to_ss58check_with_version(
    sp_core::crypto::Ss58AddressFormat::custom(42)
);

// From SS58
let account_id = AccountId32::from_ss58check(&ss58_string)?;
```

## Transaction Example

```rust
use bittensor_rs::chain::{create_signer_from_seed, ExtrinsicWait};
use bittensor_rs::validator::set_weights;

// Create signer
let signer = create_signer_from_seed("//Alice")?;

// Set weights
let tx_hash = set_weights(
    &client,
    &signer,
    netuid,
    &uids,
    &weights,
    None,
    ExtrinsicWait::Finalized
).await?;
```

## Error Handling

```rust
use bittensor_rs::Error;

match query_result {
    Ok(data) => process(data),
    Err(Error::StorageNotFound) => handle_not_found(),
    Err(Error::DecodingError(e)) => handle_decode_error(e),
    Err(e) => handle_generic_error(e),
}
```

## Common Patterns

### Optional Block Parameter

```rust
// Latest block
let data = query(&client, None).await?;

// Specific block
let data = query(&client, Some(block_number)).await?;
```

### Bulk Operations

```rust
use futures::stream::{FuturesUnordered, StreamExt};

let mut futures = FuturesUnordered::new();
for item in items {
    futures.push(async_operation(item));
}

while let Some(result) = futures.next().await {
    process_result(result?);
}
```

### Value Decoding

```rust
use bittensor_rs::utils::decoders;

let u64_val = decoders::decode_u64(&value)?;
let account = decoders::decode_account_id32(&value)?;
let vec_u16 = decoders::decode_vec_u16(&value)?;
```

## Environment Variables

- `BITTENSOR_RPC` - RPC endpoint URL
- `RUST_LOG` - Logging level (debug, info, warn, error)

## Useful Constants

```rust
use bittensor_rs::core::{SS58_FORMAT, RAO_PER_TAO};

// SS58 format for Bittensor
let format = sp_core::crypto::Ss58AddressFormat::custom(SS58_FORMAT);

// Convert TAO to RAO
let rao = tao * RAO_PER_TAO;
```
