# Metagraph Queries

Module: `bittensor_chain::queries::metagraph`

Subnet-wide neural graph snapshots, pending emission data, mechanism step tracking, and recycle-or-burn settings.

```rust
use bittensor_chain::queries::metagraph;
use bittensor_chain::prelude::*;
use bittensor_core::types::MetagraphInfo;
use bittensor_core::balance::Balance;
```

All functions take `&OnlineClient<SubtensorConfig>` as the first argument and return `Result<T, BittensorError>`.

---

## `get_metagraph`

```rust
pub async fn get_metagraph(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<MetagraphInfo>
```

Fetches a metagraph snapshot for a subnet. Aggregates subnet UID count, total issuance, and total stake from the `subtensor_module` storage into a `MetagraphInfo` struct.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `netuid` | `u16` | Subnet unique identifier |

**Returns**

`MetagraphInfo` -- Struct containing:

| Field | Type | Description |
|-------|------|-------------|
| `netuid` | `u16` | Subnet unique identifier |
| `block` | `u64` | Block number at which this snapshot was taken |
| `n` | `u16` | Number of neurons (UIDs) in the subnet |
| `stake` | `Balance` | Total stake across all neurons in the subnet |
| `total_issuance` | `Balance` | Total issuance for the subnet |
| `total_weight` | `u64` | Sum of all weight values (zero until full matrix fetch is implemented) |
| `total_bond` | `u64` | Sum of all bond values (zero until full matrix fetch is implemented) |

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::metagraph;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let info = metagraph::get_metagraph(client.rpc(), 1).await?;
    println!("Subnet {} at block {}", info.netuid, info.block);
    println!("Neurons: {}", info.n);
    println!("Total stake: {} TAO", info.stake.to_tao());
    println!("Total issuance: {} TAO", info.total_issuance.to_tao());

    Ok(())
}
```

---

## `get_selective_metagraph`

```rust
pub async fn get_selective_metagraph(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<MetagraphInfo>
```

Fetches a selective metagraph. Currently delegates to `get_metagraph` and returns the same full snapshot. Reserved for future optimization where only a subset of neuron data is fetched.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `netuid` | `u16` | Subnet unique identifier |

**Returns**

`MetagraphInfo` -- Same as `get_metagraph`.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::metagraph;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let info = metagraph::get_selective_metagraph(client.rpc(), 8).await?;
    println!("Subnet 8 has {} neurons", info.n);

    Ok(())
}
```

---

## `get_subnet_owner_cut`

```rust
pub async fn get_subnet_owner_cut(
    client: &OnlineClient<SubtensorConfig>,
) -> Result<u16>
```

Fetches the subnet owner cut, a global parameter in basis points. For example, a value of 1800 means the subnet owner receives 18% of the emission.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |

**Returns**

`u16` -- Owner cut in parts-per-ten-thousand (basis points).

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::metagraph;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let cut = metagraph::get_subnet_owner_cut(client.rpc()).await?;
    println!("Owner cut: {}bps ({:.2}%)", cut, cut as f64 / 100.0);

    Ok(())
}
```

---

## `get_root_prop`

```rust
pub async fn get_root_prop(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<Option<subtensor::runtime_types::substrate_fixed::FixedU128<...>>>
```

Fetches the root proposal weight for a subnet as a `FixedU128` type. This is a fixed-point decimal value used in root network weighting. Returns `None` if the value is not set for the given subnet.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `netuid` | `u16` | Subnet unique identifier |

**Returns**

`Option<FixedU128<...>>` -- The fixed-point root proposal weight, or `None` if unset.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::metagraph;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    if let Some(prop) = metagraph::get_root_prop(client.rpc(), 1).await? {
        println!("Root prop for subnet 1 is set");
    } else {
        println!("Root prop for subnet 1 is not set");
    }

    Ok(())
}
```

---

## `get_first_emission_block_number`

```rust
pub async fn get_first_emission_block_number(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64>
```

Fetches the block number at which emission first started for a subnet. Returns 0 if emission has not started.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `netuid` | `u16` | Subnet unique identifier |

**Returns**

`u64` -- The first emission block number, or 0 if not set.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::metagraph;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let block = metagraph::get_first_emission_block_number(client.rpc(), 1).await?;
    println!("First emission for subnet 1 at block {}", block);

    Ok(())
}
```

---

## `get_pending_server_emission`

```rust
pub async fn get_pending_server_emission(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64>
```

Fetches the accumulated pending server emission for a subnet, in rao. This is the emission earned by servers (miners) that has accumulated but not yet been distributed during the next mechanism step.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `netuid` | `u16` | Subnet unique identifier |

**Returns**

`u64` -- Pending server emission in rao.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::metagraph;
use bittensor_core::config::NetworkConfig;
use bittensor_core::balance::Balance;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let pending = metagraph::get_pending_server_emission(client.rpc(), 1).await?;
    println!("Pending server emission: {} TAO", Balance::from_rao(pending).to_tao());

    Ok(())
}
```

---

## `get_pending_validator_emission`

```rust
pub async fn get_pending_validator_emission(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64>
```

Fetches the accumulated pending validator emission for a subnet, in rao. This is the emission earned by validators that has accumulated but not yet been distributed during the next mechanism step.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `netuid` | `u16` | Subnet unique identifier |

**Returns**

`u64` -- Pending validator emission in rao.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::metagraph;
use bittensor_core::config::NetworkConfig;
use bittensor_core::balance::Balance;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let pending = metagraph::get_pending_validator_emission(client.rpc(), 1).await?;
    println!("Pending validator emission: {} TAO", Balance::from_rao(pending).to_tao());

    Ok(())
}
```

---

## `get_pending_root_alpha_divs`

```rust
pub async fn get_pending_root_alpha_divs(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64>
```

Fetches the accumulated pending root alpha dividends for a subnet, in rao. These dividends flow from the root network to subnets based on their weight in the root weights matrix.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `netuid` | `u16` | Subnet unique identifier |

**Returns**

`u64` -- Pending root alpha dividends in rao.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::metagraph;
use bittensor_core::config::NetworkConfig;
use bittensor_core::balance::Balance;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let divs = metagraph::get_pending_root_alpha_divs(client.rpc(), 1).await?;
    println!("Pending root alpha dividends: {} TAO", Balance::from_rao(divs).to_tao());

    Ok(())
}
```

---

## `get_pending_owner_cut`

```rust
pub async fn get_pending_owner_cut(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64>
```

Fetches the accumulated pending owner cut for a subnet, in rao. This is the subnet owner's share of emission that has accumulated but not yet been claimed.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `netuid` | `u16` | Subnet unique identifier |

**Returns**

`u64` -- Pending owner cut in rao.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::metagraph;
use bittensor_core::config::NetworkConfig;
use bittensor_core::balance::Balance;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let cut = metagraph::get_pending_owner_cut(client.rpc(), 1).await?;
    println!("Pending owner cut: {} TAO", Balance::from_rao(cut).to_tao());

    Ok(())
}
```

---

## `get_blocks_since_last_step`

```rust
pub async fn get_blocks_since_last_step(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64>
```

Fetches the number of blocks that have elapsed since the last mechanism step for a subnet. The mechanism step is the point at which emission distribution and weight processing occur.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `netuid` | `u16` | Subnet unique identifier |

**Returns**

`u64` -- Number of blocks since the last mechanism step.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::metagraph;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let blocks = metagraph::get_blocks_since_last_step(client.rpc(), 1).await?;
    println!("Blocks since last mechanism step: {}", blocks);

    Ok(())
}
```

---

## `get_last_mechanism_step_block`

```rust
pub async fn get_last_mechanism_step_block(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64>
```

Fetches the block number at which the last mechanism step occurred for a subnet. Returns 0 if no mechanism step has occurred yet.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `netuid` | `u16` | Subnet unique identifier |

**Returns**

`u64` -- Block number of the last mechanism step, or 0 if not set.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::metagraph;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let block = metagraph::get_last_mechanism_step_block(client.rpc(), 1).await?;
    println!("Last mechanism step at block {}", block);

    Ok(())
}
```

---

## `get_recycle_or_burn`

```rust
pub async fn get_recycle_or_burn(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<Option<subtensor::runtime_types::pallet_subtensor::pallet::RecycleOrBurnEnum>>
```

Fetches the recycle-or-burn setting for a subnet. This determines whether registration fees are recycled back into the subnet's emission pool or burned. Returns `None` if the subnet does not have this setting configured.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `netuid` | `u16` | Subnet unique identifier |

**Returns**

`Option<RecycleOrBurnEnum>` -- The recycle-or-burn setting, or `None` if not set.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::metagraph;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    match metagraph::get_recycle_or_burn(client.rpc(), 1).await? {
        Some(setting) => println!("Subnet 1 recycle-or-burn is configured"),
        None => println!("Subnet 1 has no recycle-or-burn setting"),
    }

    Ok(())
}
```

---

## Full Example

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::metagraph;
use bittensor_core::config::NetworkConfig;
use bittensor_core::balance::Balance;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;
    let rpc = client.rpc();

    // Requires live node
    // Metagraph snapshot
    let info = metagraph::get_metagraph(rpc, 1).await?;
    println!("Subnet {} at block {} with {} neurons", info.netuid, info.block, info.n);
    println!("Stake: {} TAO", info.stake.to_tao());

    // Owner cut
    let cut = metagraph::get_subnet_owner_cut(rpc).await?;
    println!("Global owner cut: {}bps", cut);

    // Pending emissions
    let server_em = metagraph::get_pending_server_emission(rpc, 1).await?;
    let val_em = metagraph::get_pending_validator_emission(rpc, 1).await?;
    let root_divs = metagraph::get_pending_root_alpha_divs(rpc, 1).await?;
    let owner_cut = metagraph::get_pending_owner_cut(rpc, 1).await?;
    println!("Pending server: {} TAO", Balance::from_rao(server_em).to_tao());
    println!("Pending validator: {} TAO", Balance::from_rao(val_em).to_tao());
    println!("Pending root alpha divs: {} TAO", Balance::from_rao(root_divs).to_tao());
    println!("Pending owner cut: {} TAO", Balance::from_rao(owner_cut).to_tao());

    // Mechanism step tracking
    let first_block = metagraph::get_first_emission_block_number(rpc, 1).await?;
    let last_step = metagraph::get_last_mechanism_step_block(rpc, 1).await?;
    let since_step = metagraph::get_blocks_since_last_step(rpc, 1).await?;
    println!("First emission block: {}", first_block);
    println!("Last mechanism step: {}", last_step);
    println!("Blocks since last step: {}", since_step);

    // Recycle or burn
    if let Some(_setting) = metagraph::get_recycle_or_burn(rpc, 1).await? {
        println!("Subnet 1 recycle-or-burn is configured");
    }

    Ok(())
}
```
