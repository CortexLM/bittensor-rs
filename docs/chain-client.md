# SubtensorClient API Reference

The `bittensor-chain` crate provides `SubtensorClient`, a typed WebSocket client for the Bittensor Subtensor chain built on subxt 0.50.

```toml
[dependencies]
bittensor-chain = "0.1"
bittensor-core = "0.1"
```

```rust
use bittensor_chain::prelude::*;
use bittensor_core::config::NetworkConfig;
```

---

## SubtensorClient

### `SubtensorClient::from_config`

```rust
pub async fn from_config(config: NetworkConfig) -> Result<Self, BittensorError>
```

Connects to the Subtensor chain using a `NetworkConfig`. If the primary WebSocket endpoint fails and an archive endpoint is configured, the client automatically falls back to the archive.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `config` | `NetworkConfig` | Network configuration with WebSocket URL and optional archive URL |

**Returns**

`Result<SubtensorClient, BittensorError>`

**Failover behavior**

When `config.archive_endpoint` is `Some`, the client tries the archive endpoint first, then the primary. This matches the Python SDK's behavior for archive node failover. When `archive_endpoint` is `None`, only the primary endpoint is tried.

**Example**

```rust
// Requires live node
use bittensor_chain::prelude::*;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;
    println!("Connected to Finney");
    Ok(())
}
```

### `SubtensorClient::from_url`

```rust
pub async fn from_url(url: &str) -> Result<Self, BittensorError>
```

Connects to a single WebSocket URL with no failover.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `url` | `&str` | WebSocket endpoint URL |

**Example**

```rust
// Requires live node
let client = SubtensorClient::from_url("wss://entrypoint-finney.opentensor.ai:443").await?;
```

### `SubtensorClient::rpc`

```rust
pub fn rpc(&self) -> &OnlineClient<SubtensorConfig>
```

Returns a reference to the underlying subxt `OnlineClient`. Use this for advanced queries, custom storage fetches, or passing into query functions.

**Example**

```rust
let rpc = client.rpc();
let balance = bittensor_chain::queries::account::get_balance(rpc, &account_id).await?;
```

### `SubtensorClient::at_current_block`

```rust
pub async fn at_current_block(&self) -> Result<ClientAtBlock, BittensorError>
```

Returns a block-specific client pinned to the current best block. Use this for historical or consistent queries within a single block.

**Returns**

`Result<ClientAtBlock, BittensorError>`

**Example**

```rust
// Requires live node
let at = client.at_current_block().await?;
println!("Block hash: {:?}", at.block_hash());
```

### `SubtensorClient::get_block_hash`

```rust
pub async fn get_block_hash(
    &self,
    block_number: u64,
) -> Result<Option<subxt::utils::H256>, BittensorError>
```

Looks up the block hash for a given block number. Returns `None` if the block has been pruned or not yet produced.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `block_number` | `u64` | Block number to look up |

**Returns**

`Result<Option<H256>, BittensorError>` -- The 32-byte block hash, or `None` if unavailable.

**Example**

```rust
// Requires live node
if let Some(hash) = client.get_block_hash(1_000_000).await? {
    println!("Block 1M hash: {hash:?}");
} else {
    println!("Block 1M not available (pruned or not yet produced)");
}
```

---

## ClientAtBlock

A type alias for a subxt client pinned to a specific block:

```rust
pub type ClientAtBlock = subxt::client::ClientAtBlock<
    SubtensorConfig,
    subxt::client::OnlineClientAtBlockImpl<SubtensorConfig>,
>;
```

Provides methods:

- `block_hash()` -- Returns the hash of the pinned block.
- `block_number()` -- Returns the block number of the pinned block.
- `storage()` -- Returns a storage accessor scoped to this block.

---

## NetworkConfig

```rust
use bittensor_core::config::NetworkConfig;
```

```rust
pub struct NetworkConfig {
    pub name: String,
    pub ws_endpoint: String,
    pub archive_endpoint: Option<String>,
    pub chain_id: u16,
}
```

| Field | Type | Description |
|-------|------|-------------|
| `name` | `String` | Human-readable network name |
| `ws_endpoint` | `String` | Primary WebSocket endpoint URL |
| `archive_endpoint` | `Option<String>` | Archive node endpoint, used for failover |
| `chain_id` | `u16` | SS58 address prefix (42 for Bittensor) |

### Preset configurations

| Method | WebSocket URL | Archive | Description |
|--------|-------------|---------|-------------|
| `NetworkConfig::finney()` | `wss://entrypoint-finney.opentensor.ai:443` | `None` | Finney mainnet |
| `NetworkConfig::test()` | `wss://test.finney.opentensor.ai:443` | `None` | Testnet |
| `NetworkConfig::local()` | `ws://127.0.0.1:9944` | `None` | Local dev node |
| `NetworkConfig::archive()` | `wss://archive.finney.opentensor.ai:443` | `Some(...)` | Archive node (failover enabled) |
| `NetworkConfig::latent_lite()` | `wss://lite.finney.opentensor.ai:443` | `None` | Latent lite endpoint |

**Example**

```rust
let finney = NetworkConfig::finney();
assert_eq!(finney.name, "finney");
assert_eq!(finney.chain_id, 42);

let archive = NetworkConfig::archive();
assert!(archive.archive_endpoint.is_some());
```

---

## SubtensorConfig

```rust
use bittensor_core::config::SubtensorConfig;
```

The subxt `Config` implementation for Bittensor's Subtensor chain. Inherits Blake2-256 hashing, 32-byte account IDs, and standard Substrate extrinsic params from `subxt::config::substrate::SubstrateConfig`.

```rust
impl subxt::Config for SubtensorConfig {
    type AccountId = subxt::utils::AccountId32;
    type Address = subxt::utils::MultiAddress<Self::AccountId, u32>;
    type Signature = subxt::utils::MultiSignature;
    type Hasher = subxt::config::substrate::DynamicHasher256;
    type Header = subxt::config::substrate::SubstrateHeader<...>;
    type TransactionExtensions = subxt::config::substrate::SubstrateExtrinsicParams<Self>;
    type AssetId = u32;
}
```

---

## Full Example: Connect and Query

```rust
// Requires live node
use bittensor_chain::prelude::*;
use bittensor_core::config::NetworkConfig;
use bittensor_core::balance::Balance;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Get current block
    let block = client.at_current_block().await?;
    println!("Connected at block {} ({:?})", block.block_number(), block.block_hash());

    // Query balance
    let account_id = subxt::utils::AccountId32::from([0u8; 32]); // replace with real key
    let balance: Balance = bittensor_chain::queries::account::get_balance(
        client.rpc(), &account_id
    ).await?;
    println!("Balance: {balance}");

    Ok(())
}
```
