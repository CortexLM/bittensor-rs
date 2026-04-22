# Commit Queries

Module: `bittensor_chain::queries::commit`

Weight commit and reveal hashes, storage commitment data, and usage tracking from the commitments pallet.

```rust
use bittensor_chain::queries::commit;
use bittensor_chain::prelude::*;
use bittensor_core::types::WeightCommitInfo;
```

All functions take `&OnlineClient<SubtensorConfig>` as the first argument and return `Result<T, BittensorError>`.

---

## `get_weight_commit`

```rust
pub async fn get_weight_commit(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    hotkey: &subxt::utils::AccountId32,
) -> Result<Option<WeightCommitInfo>>
```

Fetches the committed weight hash for a hotkey in a subnet. Reads the `weight_commits` storage entry from the `subtensor_module` and returns the first commit wrapped in a `WeightCommitInfo` struct. Returns `None` if no commits exist.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `netuid` | `u16` | Subnet unique identifier |
| `hotkey` | `&AccountId32` | Hotkey account ID |

**Returns**

`Option<WeightCommitInfo>` -- Committed weight info with fields:

| Field | Type | Description |
|-------|------|-------------|
| `hotkey` | `String` | SS58-encoded hotkey |
| `commit` | `Vec<u8>` | Opaque commit bytes (hash of weight vector + salt) |
| `reveal_round` | `u64` | Block at which the commit can be revealed |
| `netuid` | `u16` | Subnet the commit targets |

Returns `None` if no commits exist.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::commit;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let hotkey = subxt::utils::AccountId32::from([0u8; 32]);
    if let Some(c) = commit::get_weight_commit(client.rpc(), 1, &hotkey).await? {
        println!("Commit by {} in subnet {} at reveal block {}",
            c.hotkey, c.netuid, c.reveal_round);
    } else {
        println!("No weight commit found");
    }

    Ok(())
}
```

---

## `get_weight_reveal`

```rust
pub async fn get_weight_reveal(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    hotkey: &subxt::utils::AccountId32,
) -> Result<Option<Vec<u8>>>
```

Fetches the revealed weight data for a hotkey in a subnet at the current block. Queries the `crv3_weight_commits_v2` storage to find a matching reveal entry. Returns the raw weight data bytes if found, or `None` if no reveal exists for this hotkey at the current block.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `netuid` | `u16` | Subnet unique identifier |
| `hotkey` | `&AccountId32` | Hotkey account ID |

**Returns**

`Option<Vec<u8>>` -- The revealed weight data bytes, or `None` if no reveal exists.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::commit;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let hotkey = subxt::utils::AccountId32::from([0u8; 32]);
    if let Some(data) = commit::get_weight_reveal(client.rpc(), 1, &hotkey).await? {
        println!("Revealed {} bytes of weight data", data.len());
    } else {
        println!("No weight reveal found at current block");
    }

    Ok(())
}
```

---

## `get_commitment_of`

```rust
pub async fn get_commitment_of(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    hotkey: &subxt::utils::AccountId32,
) -> Result<Option<subtensor::runtime_types::pallet_commitments::types::Registration<u64, u32>>>
```

Fetches the storage commitment registration for a hotkey in a subnet from the `commitments` pallet. This contains the on-chain commitment data (such as model hashes or IPFS CIDs) that a neuron has registered for their subnet role. Returns `None` if no commitment exists.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `netuid` | `u16` | Subnet unique identifier |
| `hotkey` | `&AccountId32` | Hotkey account ID |

**Returns**

`Option<Registration<u64, u32>>` -- The commitment registration struct from the commitments pallet, or `None` if not set.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::commit;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let hotkey = subxt::utils::AccountId32::from([0u8; 32]);
    if let Some(reg) = commit::get_commitment_of(client.rpc(), 1, &hotkey).await? {
        println!("Subnet 1 commitment found for hotkey");
    } else {
        println!("No commitment registered");
    }

    Ok(())
}
```

---

## `get_last_commitment`

```rust
pub async fn get_last_commitment(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    hotkey: &subxt::utils::AccountId32,
) -> Result<u32>
```

Fetches the block number at which a hotkey last committed data to a subnet. Returns 0 if no prior commitment exists.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `netuid` | `u16` | Subnet unique identifier |
| `hotkey` | `&AccountId32` | Hotkey account ID |

**Returns**

`u32` -- Block number of the last commitment, or 0 if not set.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::commit;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let hotkey = subxt::utils::AccountId32::from([0u8; 32]);
    let last = commit::get_last_commitment(client.rpc(), 1, &hotkey).await?;
    println!("Last commitment at block {}", last);

    Ok(())
}
```

---

## `get_used_space_of`

```rust
pub async fn get_used_space_of(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    hotkey: &subxt::utils::AccountId32,
) -> Result<Option<subtensor::runtime_types::pallet_commitments::types::UsageTracker>>
```

Fetches the storage usage tracker for a hotkey in a subnet from the `commitments` pallet. This tracks how much on-chain storage space a neuron has consumed with their commitment data. Returns `None` if no usage record exists.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `netuid` | `u16` | Subnet unique identifier |
| `hotkey` | `&AccountId32` | Hotkey account ID |

**Returns**

`Option<UsageTracker>` -- The usage tracker struct from the commitments pallet, or `None` if not set.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::commit;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let hotkey = subxt::utils::AccountId32::from([0u8; 32]);
    if let Some(tracker) = commit::get_used_space_of(client.rpc(), 1, &hotkey).await? {
        println!("Subnet 1 storage usage tracker found");
    } else {
        println!("No usage tracker record");
    }

    Ok(())
}
```

---

## `get_max_space`

```rust
pub async fn get_max_space(
    client: &OnlineClient<SubtensorConfig>,
) -> Result<u32>
```

Fetches the global maximum allowed storage space for commitments, in bytes. This is the upper bound on how much data a neuron can store on-chain per commitment.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |

**Returns**

`u32` -- Maximum storage space in bytes.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::commit;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let max = commit::get_max_space(client.rpc()).await?;
    println!("Max commitment space: {} bytes", max);

    Ok(())
}
```

---

## Full Example

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::commit;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;
    let rpc = client.rpc();

    // Requires live node
    let hotkey = subxt::utils::AccountId32::from([0u8; 32]);

    // Weight commit
    if let Some(c) = commit::get_weight_commit(rpc, 1, &hotkey).await? {
        println!("Weight commit: reveal at block {}", c.reveal_round);
    }

    // Weight reveal
    if let Some(data) = commit::get_weight_reveal(rpc, 1, &hotkey).await? {
        println!("Revealed weight data: {} bytes", data.len());
    }

    // Storage commitment
    if let Some(reg) = commit::get_commitment_of(rpc, 1, &hotkey).await? {
        println!("Commitment registered on-chain");
    }

    let last = commit::get_last_commitment(rpc, 1, &hotkey).await?;
    println!("Last commitment block: {}", last);

    // Usage tracking
    if let Some(tracker) = commit::get_used_space_of(rpc, 1, &hotkey).await? {
        println!("Usage tracker present");
    }

    let max_space = commit::get_max_space(rpc).await?;
    println!("Max commitment space: {} bytes", max_space);

    Ok(())
}
```
