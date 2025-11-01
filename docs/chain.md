# Chain Operations

Documentation for the core blockchain client and connection management in the Bittensor Rust SDK.

## Overview

The `BittensorClient` is the primary interface for connecting to and interacting with the Bittensor blockchain. It provides methods for queries, transactions, and event monitoring.

## Client Initialization

### Basic Connection

```rust
use bittensor_rs::chain::BittensorClient;

// Connect to default endpoint
let client = BittensorClient::new("wss://entrypoint-finney.opentensor.ai:443").await?;

// Connect to local node
let client = BittensorClient::new("ws://127.0.0.1:9944").await?;
```

### Connection Options

```rust
// With timeout
let client = BittensorClient::with_timeout(
    "wss://entrypoint-finney.opentensor.ai:443",
    Duration::from_secs(30)
).await?;

// With custom configuration
let client = BittensorClient::builder()
    .url("wss://entrypoint-finney.opentensor.ai:443")
    .timeout(Duration::from_secs(30))
    .build()
    .await?;
```

## Core Methods

### Block Information

```rust
// Get current block number
let block_number = client.get_block_number().await?;

// Get block hash
let block_hash = client.get_block_hash(block_number).await?;

// Get finalized block
let finalized_hash = client.get_finalized_block_hash().await?;
```

### Chain Properties

```rust
// Get chain name
let chain_name = client.get_chain_name().await?;

// Get runtime version
let runtime_version = client.get_runtime_version().await?;

// Get chain properties
let properties = client.get_chain_properties().await?;
```

### Storage Queries

Direct storage access for advanced use cases:

```rust
use subxt::dynamic::Value;

// Query storage with keys
let keys = vec![Value::u128(netuid as u128)];
let value = client.storage_with_keys(
    "SubtensorModule", 
    "SubnetworkN", 
    keys
).await?;

// Query storage at specific block
let value = client.storage_at_block(
    "SubtensorModule",
    "TotalStake",
    vec![],
    block_hash
).await?;
```

## Transaction Submission

### Creating Signers

```rust
use bittensor_rs::chain::{create_signer, create_signer_from_seed};
use sp_core::Pair;

// From seed phrase
let signer = create_signer_from_seed("//Alice")?;

// From secret key
let secret_key = [/* 32 bytes */];
let signer = create_signer(secret_key)?;

// From mnemonic
let mnemonic = "word1 word2 ... word12";
let signer = create_signer_from_mnemonic(mnemonic, None)?;
```

### Submitting Transactions

```rust
use bittensor_rs::chain::ExtrinsicWait;

// Build transaction
let tx = client.tx()
    .subtensor()
    .set_weights(netuid, uids, weights, version_key)?;

// Sign and submit
let events = client.sign_and_submit_then_watch(&tx, &signer, ExtrinsicWait::Finalized).await?;

// Get transaction hash
let tx_hash = events.extrinsic_hash();
```

### Transaction Options

```rust
pub enum ExtrinsicWait {
    /// Wait for block inclusion
    InBlock,
    /// Wait for finalization (recommended)
    Finalized,
    /// Don't wait, return immediately
    None,
}
```

## Event Monitoring

### Subscribe to Events

```rust
// Subscribe to all events
let mut event_sub = client.subscribe_events().await?;

while let Some(events) = event_sub.next().await {
    for event in events? {
        println!("Event: {:?}", event);
    }
}
```

### Filter Specific Events

```rust
// Listen for transfer events
let mut event_sub = client.subscribe_events().await?;

while let Some(events) = event_sub.next().await {
    for event in events? {
        if let Ok(transfer) = event.as_event::<TransferEvent>() {
            println!("Transfer: {} -> {} ({})", 
                transfer.from, transfer.to, transfer.amount
            );
        }
    }
}
```

## Advanced Usage

### Custom RPC Calls

```rust
// Raw RPC request
let result: serde_json::Value = client.rpc()
    .request("chain_getBlockHash", rpc_params![block_number])
    .await?;
```

### Runtime API Calls

```rust
// Call runtime API
let result = client.runtime_api_call(
    "NeuronInfoRuntimeApi",
    "get_neurons",
    Some(encoded_params)
).await?;
```

### Batch Queries

```rust
use futures::future::join_all;

// Parallel storage queries
let queries = vec![
    client.storage_with_keys("Module", "Storage1", keys1.clone()),
    client.storage_with_keys("Module", "Storage2", keys2.clone()),
    client.storage_with_keys("Module", "Storage3", keys3.clone()),
];

let results = join_all(queries).await;
```

## Connection Management

### Health Checks

```rust
// Check if connected
let is_connected = client.is_connected();

// Ping test
let latency = client.ping().await?;
```

### Reconnection

```rust
// Automatic reconnection
let client = BittensorClient::with_reconnect(url, max_retries).await?;

// Manual reconnection
if !client.is_connected() {
    client.reconnect().await?;
}
```

### Connection Pooling

```rust
// For high-throughput applications
struct ClientPool {
    clients: Vec<BittensorClient>,
    current: AtomicUsize,
}

impl ClientPool {
    async fn get(&self) -> &BittensorClient {
        let idx = self.current.fetch_add(1, Ordering::Relaxed) % self.clients.len();
        &self.clients[idx]
    }
}
```

## Error Handling

### Error Types

```rust
use bittensor_rs::Error;

match client.get_block_number().await {
    Ok(block) => println!("Block: {}", block),
    Err(Error::ConnectionError(e)) => eprintln!("Connection failed: {}", e),
    Err(Error::RpcError(e)) => eprintln!("RPC error: {}", e),
    Err(Error::DecodingError(e)) => eprintln!("Decode failed: {}", e),
    Err(e) => eprintln!("Other error: {}", e),
}
```

### Retry Logic

```rust
async fn with_retry<T, F, Fut>(f: F, max_retries: u32) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let mut retries = 0;
    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) if retries < max_retries => {
                retries += 1;
                tokio::time::sleep(Duration::from_secs(1 << retries)).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```

## Performance Optimization

### Caching Metadata

```rust
// Cache metadata to avoid repeated fetches
let metadata = client.metadata();
let cached_client = client.with_cached_metadata(metadata);
```

### Keep-Alive Settings

```rust
// Configure WebSocket keep-alive
let client = BittensorClient::builder()
    .url(url)
    .keep_alive_interval(Duration::from_secs(30))
    .build()
    .await?;
```

## Related Documentation

- [Query Operations](queries.md) - Using the client for queries
- [Validator Operations](validator.md) - Transaction examples
- [Type Definitions](types.md) - Data structures used with the client
