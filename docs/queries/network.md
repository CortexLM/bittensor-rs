# Network Queries

Module: `bittensor_chain::queries::network`

Block number, hash rate, issuance, subnet limit, and network-level global parameters.

```rust
use bittensor_chain::queries::network;
use bittensor_chain::prelude::*;
use bittensor_core::balance::Balance;
```

All functions take `&OnlineClient<SubtensorConfig>` as the first argument and return `Result<T, BittensorError>`.

---

## `get_network_block`

```rust
pub async fn get_network_block(
    client: &OnlineClient<SubtensorConfig>,
) -> Result<u64>
```

Fetches the current best block number from the chain.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |

**Returns**

`u64` -- Current best block number.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::network;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let block = network::get_network_block(client.rpc()).await?;
    println!("Current block: {}", block);

    Ok(())
}
```

---

## `get_network_hash_rate`

```rust
pub async fn get_network_hash_rate(
    client: &OnlineClient<SubtensorConfig>,
) -> Result<u64>
```

Fetches the current network hash rate. This is a stub implementation that always returns 0. Reserved for future integration with mining difficulty data.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |

**Returns**

`u64` -- Always returns 0 (stub).

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::network;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node (but always returns 0 currently)
    let hash_rate = network::get_network_hash_rate(client.rpc()).await?;
    println!("Network hash rate: {} (stub, always 0)", hash_rate);

    Ok(())
}
```

---

## `get_current_weight`

```rust
pub async fn get_current_weight(
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

`Vec<(u16, u16)>` -- Weight assignments as `(destination_uid, weight_value)` pairs. Empty if the UID has no weights set.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::network;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let weights = network::get_current_weight(client.rpc(), 1, 0).await?;
    for (dest_uid, weight) in weights {
        println!("UID 0 weights UID {} with value {}", dest_uid, weight);
    }

    Ok(())
}
```

---

## `get_total_issuance`

```rust
pub async fn get_total_issuance(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<Balance>
```

Fetches the total token issuance. Currently reads the global `total_issuance` from the `subtensor_module` storage. The `netuid` parameter is reserved for future per-subnet issuance queries.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `netuid` | `u16` | Subnet unique identifier (currently unused, reads global total) |

**Returns**

`Balance` -- Total issuance in rao-denominated Balance.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::network;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let issuance = network::get_total_issuance(client.rpc(), 0).await?;
    println!("Total issuance: {} TAO", issuance.to_tao());

    Ok(())
}
```

---

## `get_block_hash`

```rust
pub async fn get_block_hash(
    client: &OnlineClient<SubtensorConfig>,
    block_number: u64,
) -> Result<Option<subxt::utils::H256>>
```

Fetches the block hash for a given block number. Returns `None` if the block number does not exist on chain.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `block_number` | `u64` | Block number to look up |

**Returns**

`Option<H256>` -- The block hash, or `None` if the block does not exist.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::network;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    if let Some(hash) = network::get_block_hash(client.rpc(), 1_000_000).await? {
        println!("Block 1000000 hash: {:?}", hash);
    } else {
        println!("Block 1000000 not found");
    }

    Ok(())
}
```

---

## `get_total_networks`

```rust
pub async fn get_total_networks(
    client: &OnlineClient<SubtensorConfig>,
) -> Result<u16>
```

Fetches the total number of networks (subnets) currently registered on chain.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |

**Returns**

`u16` -- Total number of subnets.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::network;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let total = network::get_total_networks(client.rpc()).await?;
    println!("Total subnets: {}", total);

    Ok(())
}
```

---

## `get_block_emission`

```rust
pub async fn get_block_emission(
    client: &OnlineClient<SubtensorConfig>,
) -> Result<Balance>
```

Fetches the global block emission rate in rao per block. This is the total new TAO minted per block across all subnets.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |

**Returns**

`Balance` -- Block emission in rao.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::network;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let emission = network::get_block_emission(client.rpc()).await?;
    println!("Block emission: {} TAO", emission.to_tao());

    Ok(())
}
```

---

## `get_subnet_limit`

```rust
pub async fn get_subnet_limit(
    client: &OnlineClient<SubtensorConfig>,
) -> Result<u16>
```

Fetches the maximum number of subnets allowed on chain (the subnet limit).

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |

**Returns**

`u16` -- Maximum number of subnets.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::network;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let limit = network::get_subnet_limit(client.rpc()).await?;
    println!("Subnet limit: {}", limit);

    Ok(())
}
```

---

## `get_network_immunity_period`

```rust
pub async fn get_network_immunity_period(
    client: &OnlineClient<SubtensorConfig>,
) -> Result<u64>
```

Fetches the global network immunity period in blocks. During this period after subnet creation, certain actions (such as deregistration) are restricted.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |

**Returns**

`u64` -- Immunity period in blocks.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::network;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let period = network::get_network_immunity_period(client.rpc()).await?;
    println!("Immunity period: {} blocks", period);

    Ok(())
}
```

---

## `get_network_rate_limit`

```rust
pub async fn get_network_rate_limit(
    client: &OnlineClient<SubtensorConfig>,
) -> Result<u64>
```

Fetches the global network rate limit in blocks. This is the minimum number of blocks that must pass between certain network-level operations such as subnet registration.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |

**Returns**

`u64` -- Rate limit in blocks.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::network;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let rate_limit = network::get_network_rate_limit(client.rpc()).await?;
    println!("Network rate limit: {} blocks", rate_limit);

    Ok(())
}
```

---

## `get_nominator_min_required_stake`

```rust
pub async fn get_nominator_min_required_stake(
    client: &OnlineClient<SubtensorConfig>,
) -> Result<Balance>
```

Fetches the global minimum required stake for a nominator, in rao. Nominators with stake below this threshold may not receive staking rewards.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |

**Returns**

`Balance` -- Minimum required stake in rao.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::network;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let min_stake = network::get_nominator_min_required_stake(client.rpc()).await?;
    println!("Min nominator stake: {} TAO", min_stake.to_tao());

    Ok(())
}
```

---

## `get_subnetwork_n`

```rust
pub async fn get_subnetwork_n(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u16>
```

Fetches the number of UIDs (neurons) registered in a given subnet. Equivalent to the subnet's `subnetwork_n` storage value.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `netuid` | `u16` | Subnet unique identifier |

**Returns**

`u16` -- Number of registered UIDs in the subnet.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::network;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let n = network::get_subnetwork_n(client.rpc(), 1).await?;
    println!("Subnet 1 has {} neurons", n);

    Ok(())
}
```

---

## `get_networks_added`

```rust
pub async fn get_networks_added(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<bool>
```

Checks whether a given subnet exists (has been added) on chain. Returns `true` if the subnet is registered, `false` otherwise.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `netuid` | `u16` | Subnet unique identifier |

**Returns**

`bool` -- `true` if the subnet exists, `false` otherwise.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::network;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let exists = network::get_networks_added(client.rpc(), 1).await?;
    println!("Subnet 1 exists: {}", exists);

    Ok(())
}
```

---

## Full Example

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::network;
use bittensor_core::config::NetworkConfig;
use bittensor_core::balance::Balance;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;
    let rpc = client.rpc();

    // Requires live node
    // Block info
    let block = network::get_network_block(rpc).await?;
    println!("Current block: {}", block);

    // Hash rate (stub)
    let hash_rate = network::get_network_hash_rate(rpc).await?;
    println!("Network hash rate: {} (stub)", hash_rate);

    // Issuance and emission
    let issuance = network::get_total_issuance(rpc, 0).await?;
    let emission = network::get_block_emission(rpc).await?;
    println!("Total issuance: {} TAO", issuance.to_tao());
    println!("Block emission: {} TAO", emission.to_tao());

    // Subnet counts
    let total = network::get_total_networks(rpc).await?;
    let limit = network::get_subnet_limit(rpc).await?;
    println!("Total subnets: {} / limit {}", total, limit);

    // Network parameters
    let immunity = network::get_network_immunity_period(rpc).await?;
    let rate_limit = network::get_network_rate_limit(rpc).await?;
    let min_stake = network::get_nominator_min_required_stake(rpc).await?;
    println!("Immunity period: {} blocks", immunity);
    println!("Rate limit: {} blocks", rate_limit);
    println!("Min nominator stake: {} TAO", min_stake.to_tao());

    // Subnet queries
    let n = network::get_subnetwork_n(rpc, 1).await?;
    let exists = network::get_networks_added(rpc, 1).await?;
    println!("Subnet 1 has {} neurons, exists: {}", n, exists);

    // Block hash lookup
    if let Some(hash) = network::get_block_hash(rpc, block).await? {
        println!("Block {} hash: {:?}", block, hash);
    }

    Ok(())
}
```
