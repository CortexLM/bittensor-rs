# Neuron Queries

Module: `bittensor_chain::queries::neuron`

UID lookups, neuron info, neuron count, and per-UID metric vectors.

```rust
use bittensor_chain::queries::neuron;
use bittensor_chain::prelude::*;
use bittensor_core::types::{NeuronInfo, NeuronInfoLite};
```

All functions take `&OnlineClient<SubtensorConfig>` as the first argument and return `Result<T, BittensorError>`.

---

## `get_neuron`

```rust
pub async fn get_neuron(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    uid: u16,
) -> Result<Option<NeuronInfo>>
```

Fetches full `NeuronInfo` for a UID in a subnet, including weights, bonds, axon info, and prometheus info. Returns `None` if the UID is not active.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `netuid` | `u16` | Subnet ID |
| `uid` | `u16` | Neuron UID within the subnet |

**Returns**

`Option<NeuronInfo>` -- Full neuron metadata, or `None` if inactive.

**Example**

```rust
// Requires live node
if let Some(info) = neuron::get_neuron(client.rpc(), 1, 0).await? {
    println!("UID {} rank: {}", info.uid, info.rank);
    println!("Weights count: {}", info.weights.len() / 2);
}
```

---

## `get_neuron_lite`

```rust
pub async fn get_neuron_lite(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    uid: u16,
) -> Result<Option<NeuronInfoLite>>
```

Fetches a lightweight `NeuronInfoLite` for a UID in a subnet. Excludes weight and bond vectors, making it faster for list queries. Returns `None` if the UID is not active.

**Parameters**

Same as `get_neuron`.

**Returns**

`Option<NeuronInfoLite>` -- Lightweight neuron metadata, or `None`.

---

## `get_uid_for_hotkey`

```rust
pub async fn get_uid_for_hotkey(
    client: &OnlineClient<SubtensorConfig>,
    hotkey: &subxt::utils::AccountId32,
    netuid: u16,
) -> Result<Option<u16>>
```

Resolves the UID for a hotkey in a subnet. Returns `None` if the hotkey is not a member.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `hotkey` | `&AccountId32` | Hotkey account ID |
| `netuid` | `u16` | Subnet ID |

**Returns**

`Option<u16>` -- The neuron UID, or `None` if not a subnet member.

---

## `get_neuron_for_pubkey_and_subnet`

```rust
pub async fn get_neuron_for_pubkey_and_subnet(
    client: &OnlineClient<SubtensorConfig>,
    hotkey: &subxt::utils::AccountId32,
    netuid: u16,
) -> Result<Option<NeuronInfo>>
```

Fetches full `NeuronInfo` for a hotkey in a subnet. Internally resolves the UID first, then fetches the full info.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `hotkey` | `&AccountId32` | Hotkey account ID |
| `netuid` | `u16` | Subnet ID |

**Returns**

`Option<NeuronInfo>` -- Full neuron metadata, or `None`.

---

## `get_neuron_count`

```rust
pub async fn get_neuron_count(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u16>
```

Fetches the current number of registered UIDs in a subnet.

**Returns**

`u16` -- Number of neurons.

---

## `get_max_neurons`

```rust
pub async fn get_max_neurons(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u16>
```

Fetches the maximum allowed UIDs for a subnet.

**Returns**

`u16` -- Maximum allowed neurons.

---

## `get_neurons`

```rust
pub async fn get_neurons(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<Vec<NeuronInfoLite>>
```

Fetches all neurons in a subnet as a list of `NeuronInfoLite`. Iterates over every UID from 0 to `subnetwork_n - 1`.

**Returns**

`Vec<NeuronInfoLite>` -- All neurons in the subnet.

**Example**

```rust
// Requires live node
let neurons = neuron::get_neurons(client.rpc(), 1).await?;
println!("Subnet 1 has {} neurons", neurons.len());
for n in &neurons {
    if n.active {
        println!("  UID {} rank={}", n.uid, n.rank);
    }
}
```

---

## Per-UID Vector Queries

These functions return one entry per UID in the subnet, indexed by UID position.

### `get_rank`

```rust
pub async fn get_rank(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<Vec<u16>>
```

Rank scores (0-65535 scaled) for each UID.

### `get_trust`

```rust
pub async fn get_trust(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<Vec<u16>>
```

Trust scores for each UID.

### `get_consensus`

```rust
pub async fn get_consensus(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<Vec<u16>>
```

Consensus scores for each UID.

### `get_incentive`

```rust
pub async fn get_incentive(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<Vec<u16>>
```

Incentive scores for each UID.

### `get_dividends`

```rust
pub async fn get_dividends(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<Vec<u16>>
```

Dividend scores for each UID.

### `get_emission`

```rust
pub async fn get_emission(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<Vec<u64>>
```

Emission in rao per tempo for each UID.

### `get_active`

```rust
pub async fn get_active(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<Vec<bool>>
```

Active status for each UID. `true` means the neuron is serving.

### `get_last_update`

```rust
pub async fn get_last_update(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<Vec<u64>>
```

Block number of the last weight update for each UID.

### `get_validator_permit`

```rust
pub async fn get_validator_permit(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<Vec<bool>>
```

Validator permit flags for each UID.

### `get_validator_trust`

```rust
pub async fn get_validator_trust(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<Vec<u16>>
```

Validator trust scores for each UID.

### `get_pruning_scores`

```rust
pub async fn get_pruning_scores(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<Vec<u16>>
```

Pruning scores for each UID. Lower scores are more likely to be pruned.

### `get_stake_weight`

```rust
pub async fn get_stake_weight(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<Vec<u16>>
```

Stake weight values for each UID.

---

## Bond and Key Queries

### `get_bonds`

```rust
pub async fn get_bonds(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    uid: u16,
) -> Result<Vec<(u16, u16)>>
```

Bond vector for a UID. Each tuple is `(peer_uid, bond_value)`.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `netuid` | `u16` | Subnet ID |
| `uid` | `u16` | Neuron UID |

### `get_block_at_registration`

```rust
pub async fn get_block_at_registration(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    uid: u16,
) -> Result<u64>
```

Block at which a UID was registered in the subnet.

### `get_uids`

```rust
pub async fn get_uids(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    hotkey: &subxt::utils::AccountId32,
) -> Result<u16>
```

UID for a hotkey in a subnet. Returns the raw storage value; may return `u16::MAX` if the hotkey has no UID.

### `get_keys`

```rust
pub async fn get_keys(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    uid: u16,
) -> Result<Option<subxt::utils::AccountId32>>
```

Hotkey account ID for a UID in a subnet. Returns `None` if the UID is unassigned.

### `get_loaded_emission`

```rust
pub async fn get_loaded_emission(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<Vec<(subxt::utils::AccountId32, u64, u64)>>
```

Loaded emission vector for a subnet. Each entry is `(hotkey, server_emission, validator_emission)`.

---

## Full Example

```rust
// Requires live node
use bittensor_chain::prelude::*;
use bittensor_chain::queries::neuron;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;
    let rpc = client.rpc();

    let count = neuron::get_neuron_count(rpc, 1).await?;
    println!("Subnet 1 has {} neurons", count);

    let ranks = neuron::get_rank(rpc, 1).await?;
    let incentives = neuron::get_incentive(rpc, 1).await?;
    for uid in 0..count as usize {
        println!("UID {uid}: rank={}, incentive={}", ranks[uid], incentives[uid]);
    }

    if let Some(info) = neuron::get_neuron(rpc, 1, 0).await? {
        println!("UID 0 hotkey: {}", info.hotkey);
        println!("UID 0 validator_trust: {}", info.validator_trust);
    }

    Ok(())
}
```
