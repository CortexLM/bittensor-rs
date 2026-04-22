# Weight Queries

Module: `bittensor_chain::queries::weights`

Weight matrix rows, commit-reveal state, weight-setting rate limits, and CRV3 commit data.

```rust
use bittensor_chain::queries::weights;
use bittensor_chain::prelude::*;
use bittensor_core::types::WeightCommitInfo;
```

All functions take `&OnlineClient<SubtensorConfig>` as the first argument and return `Result<T, BittensorError>`.

---

## `get_weights`

```rust
pub async fn get_weights(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    uid: u16,
) -> Result<Vec<(u16, u16)>>
```

Fetches the weight row for a given UID in a subnet. Each element is a `(destination_uid, weight_value)` pair representing the weight assigned by this UID to other neurons.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `netuid` | `u16` | Subnet unique identifier |
| `uid` | `u16` | Neuron UID within the subnet |

**Returns**

`Vec<(u16, u16)>` -- Weight assignments as `(destination_uid, weight_value)` pairs. Empty if no weights are set.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::weights;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let w = weights::get_weights(client.rpc(), 1, 0).await?;
    for (dest, val) in w {
        println!("UID 0 -> UID {}: weight {}", dest, val);
    }

    Ok(())
}
```

---

## `get_weights_min`

```rust
pub async fn get_weights_min(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u16>
```

Fetches the minimum allowed weight value for a subnet. Weights below this threshold are rejected when setting weights.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `netuid` | `u16` | Subnet unique identifier |

**Returns**

`u16` -- Minimum allowed weight value.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::weights;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let min = weights::get_weights_min(client.rpc(), 1).await?;
    println!("Subnet 1 min weight: {}", min);

    Ok(())
}
```

---

## `get_weights_max`

```rust
pub async fn get_weights_max(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u16>
```

Fetches the maximum weight limit for a subnet. Weight values are capped at this limit when submitted on chain.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `netuid` | `u16` | Subnet unique identifier |

**Returns**

`u16` -- Maximum weight limit (typically `u16::MAX` = 65535).

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::weights;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let max = weights::get_weights_max(client.rpc(), 1).await?;
    println!("Subnet 1 max weight limit: {}", max);

    Ok(())
}
```

---

## `get_weights_set_rate_limit`

```rust
pub async fn get_weights_set_rate_limit(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64>
```

Fetches the weight-set rate limit for a subnet in blocks. A neuron must wait this many blocks between successive weight-set transactions.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `netuid` | `u16` | Subnet unique identifier |

**Returns**

`u64` -- Rate limit in blocks between weight updates.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::weights;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let limit = weights::get_weights_set_rate_limit(client.rpc(), 1).await?;
    println!("Subnet 1 weight-set rate limit: {} blocks", limit);

    Ok(())
}
```

---

## `get_weight_commits`

```rust
pub async fn get_weight_commits(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    hotkey: &subxt::utils::AccountId32,
) -> Result<Option<WeightCommitInfo>>
```

Fetches the committed weight hash for a hotkey in a subnet. Returns the first commit if multiple exist, wrapped in a `WeightCommitInfo` struct. Returns `None` if no commits exist.

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

Returns `None` if no commits exist for this hotkey/subnet pair.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::weights;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let hotkey = subxt::utils::AccountId32::from([0u8; 32]);
    if let Some(commit) = weights::get_weight_commits(client.rpc(), 1, &hotkey).await? {
        println!("Commit for {} in subnet {} at reveal block {}",
            commit.hotkey, commit.netuid, commit.reveal_round);
    }

    Ok(())
}
```

---

## `get_timelocked_weight_commits`

```rust
pub async fn get_timelocked_weight_commits(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    block: u64,
) -> Result<Vec<(AccountId32, u64, BoundedVec<u8>, u64)>>
```

Fetches all timelocked weight commits for a subnet at a specific block. Returns a vector of tuples, each containing the delegator account, a u64 value, the commit data as a `BoundedVec<u8>`, and a second u64 value.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `netuid` | `u16` | Subnet unique identifier |
| `block` | `u64` | Block number to query commits at |

**Returns**

`Vec<(AccountId32, u64, BoundedVec<u8>, u64)>` -- List of timelocked weight commits. Each tuple is `(delegator, value1, commit_data, value2)`.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::weights;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let commits = weights::get_timelocked_weight_commits(client.rpc(), 1, 1_000_000).await?;
    for (account, v1, data, v2) in commits {
        println!("Timelocked commit from {:?} ({} bytes)", account, data.0.len());
    }

    Ok(())
}
```

---

## `get_crv3_weight_commits`

```rust
pub async fn get_crv3_weight_commits(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    block: u64,
) -> Result<Vec<(AccountId32, BoundedVec<u8>, u64)>>
```

Fetches all CRV3 (commit-reveal v3) weight commits for a subnet at a specific block. Returns a vector of tuples with the account, commit data, and a u64 value.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `netuid` | `u16` | Subnet unique identifier |
| `block` | `u64` | Block number to query commits at |

**Returns**

`Vec<(AccountId32, BoundedVec<u8>, u64)>` -- List of CRV3 weight commits.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::weights;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let commits = weights::get_crv3_weight_commits(client.rpc(), 1, 1_000_000).await?;
    for (account, data, val) in commits {
        println!("CRV3 commit from {:?} ({} bytes)", account, data.0.len());
    }

    Ok(())
}
```

---

## `get_crv3_weight_commits_v2`

```rust
pub async fn get_crv3_weight_commits_v2(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    block: u64,
) -> Result<Vec<(AccountId32, u64, BoundedVec<u8>, u64)>>
```

Fetches all CRV3 v2 weight commits for a subnet at a specific block. This is the latest version of the commit-reveal scheme, adding an extra u64 field compared to v1. Returns a vector of tuples with the account, two u64 values, the commit data, and the salt/nonce.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `netuid` | `u16` | Subnet unique identifier |
| `block` | `u64` | Block number to query commits at |

**Returns**

`Vec<(AccountId32, u64, BoundedVec<u8>, u64)>` -- List of CRV3 v2 weight commits. Each tuple is `(account, value1, commit_data, value2)`.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::weights;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let commits = weights::get_crv3_weight_commits_v2(client.rpc(), 1, 1_000_000).await?;
    for (account, v1, data, v2) in commits {
        println!("CRV3 v2 commit from {:?} ({} bytes)", account, data.0.len());
    }

    Ok(())
}
```

---

## `get_commit_reveal_weights_enabled`

```rust
pub async fn get_commit_reveal_weights_enabled(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<bool>
```

Checks whether commit-reveal weights is enabled for a subnet. When enabled, neurons must submit a hash commit first and reveal the actual weights in a subsequent transaction.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `netuid` | `u16` | Subnet unique identifier |

**Returns**

`bool` -- `true` if commit-reveal is enabled for this subnet.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::weights;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let enabled = weights::get_commit_reveal_weights_enabled(client.rpc(), 1).await?;
    println!("Subnet 1 commit-reveal: {}", if enabled { "enabled" } else { "disabled" });

    Ok(())
}
```

---

## `get_commit_reveal_weights_version`

```rust
pub async fn get_commit_reveal_weights_version(
    client: &OnlineClient<SubtensorConfig>,
) -> Result<u16>
```

Fetches the global commit-reveal weights version. This version number determines which commit-reveal scheme the runtime uses (e.g., v3 or v4) and affects how commits and reveals are processed.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |

**Returns**

`u16` -- The commit-reveal weights version number.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::weights;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let version = weights::get_commit_reveal_weights_version(client.rpc()).await?;
    println!("Commit-reveal weights version: {}", version);

    Ok(())
}
```

---

## `get_reveal_period_epochs`

```rust
pub async fn get_reveal_period_epochs(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64>
```

Fetches the reveal period in epochs for a subnet. After committing weights, a neuron must wait this many epochs before revealing the actual weight values. One epoch equals one tempo (subnet-specific block interval).

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `netuid` | `u16` | Subnet unique identifier |

**Returns**

`u64` -- Reveal period in epochs.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::weights;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let period = weights::get_reveal_period_epochs(client.rpc(), 1).await?;
    println!("Subnet 1 reveal period: {} epochs", period);

    Ok(())
}
```

---

## `get_weights_version_key`

```rust
pub async fn get_weights_version_key(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64>
```

Fetches the weights version key for a subnet. The version key is a nonce that must be incremented each time a neuron sets new weights, preventing replay of old weight transactions.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `netuid` | `u16` | Subnet unique identifier |

**Returns**

`u64` -- The current weights version key. Returns 0 if not set.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::weights;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let key = weights::get_weights_version_key(client.rpc(), 1).await?;
    println!("Subnet 1 weights version key: {}", key);

    Ok(())
}
```

---

## Full Example

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::weights;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;
    let rpc = client.rpc();

    // Requires live node
    // Weight row
    let w = weights::get_weights(rpc, 1, 0).await?;
    println!("UID 0 has {} weight entries", w.len());

    // Bounds
    let min = weights::get_weights_min(rpc, 1).await?;
    let max = weights::get_weights_max(rpc, 1).await?;
    println!("Weight range: [{}, {}]", min, max);

    // Rate limit
    let rate_limit = weights::get_weights_set_rate_limit(rpc, 1).await?;
    println!("Weight-set rate limit: {} blocks", rate_limit);

    // Commit-reveal config
    let cr_enabled = weights::get_commit_reveal_weights_enabled(rpc, 1).await?;
    let cr_version = weights::get_commit_reveal_weights_version(rpc).await?;
    let reveal_period = weights::get_reveal_period_epochs(rpc, 1).await?;
    println!("Commit-reveal: {}, version: {}, reveal period: {} epochs",
        cr_enabled, cr_version, reveal_period);

    // Version key
    let version_key = weights::get_weights_version_key(rpc, 1).await?;
    println!("Weights version key: {}", version_key);

    Ok(())
}
```
