# Subnet Queries

Module: `bittensor_chain::queries::subnet`

Subnet metadata, hyperparameters, existence checks, and per-subnet parameters.

```rust
use bittensor_chain::queries::subnet;
use bittensor_chain::prelude::*;
use bittensor_core::types::{SubnetInfo, SubnetHyperparameters};
```

All functions take `&OnlineClient<SubtensorConfig>` as the first argument and return `Result<T, BittensorError>`.

---

## `get_subnet_info`

```rust
pub async fn get_subnet_info(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<Option<SubnetInfo>>
```

Fetches subnet metadata including owner, tempo, identity, and UID count. Returns `None` if the subnet does not exist.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `netuid` | `u16` | Subnet ID |

**Returns**

`Option<SubnetInfo>` -- Subnet metadata, or `None` if subnet does not exist.

**Example**

```rust
// Requires live node
if let Some(info) = subnet::get_subnet_info(client.rpc(), 1).await? {
    println!("Subnet 1: {} (tempo={})", info.name, info.tempo);
}
```

---

## `get_subnet_hyperparameters`

```rust
pub async fn get_subnet_hyperparameters(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<Option<SubnetHyperparameters>>
```

Fetches the full hyperparameter set for a subnet. Returns `None` if the subnet does not exist.

**Returns**

`Option<SubnetHyperparameters>` -- All hyperparameters including rho, kappa, difficulty, burn, immunity ratio, alpha values, and more.

---

## `get_total_subnets`

```rust
pub async fn get_total_subnets(client: &OnlineClient<SubtensorConfig>) -> Result<u16>
```

Fetches the total number of subnets on chain.

---

## `subnet_exists`

```rust
pub async fn subnet_exists(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<bool>
```

Checks whether a subnet exists on chain.

---

## `get_subnet_owner`

```rust
pub async fn get_subnet_owner(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<Option<String>>
```

Fetches the owner coldkey of a subnet as an SS58 string. Returns `None` if no owner is set.

---

## `get_subnet_name`

```rust
pub async fn get_subnet_name(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<Option<String>>
```

Fetches the display name of a subnet from its on-chain identity. Returns `None` if no identity is set.

---

## `get_subnet_owner_hotkey`

```rust
pub async fn get_subnet_owner_hotkey(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<Option<subxt::utils::AccountId32>>
```

Fetches the hotkey that owns the subnet. Returns `None` if no owner hotkey is set.

---

## `get_tempo`

```rust
pub async fn get_tempo(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u16>
```

Blocks per epoch for a subnet.

---

## `get_subnetwork_n`

```rust
pub async fn get_subnetwork_n(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u16>
```

Current number of UIDs in a subnet.

---

## `get_subnet_mechanism`

```rust
pub async fn get_subnet_mechanism(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u16>
```

Subnet mechanism type (Yuma variant identifier).

---

## `get_is_network_member`

```rust
pub async fn get_is_network_member(
    client: &OnlineClient<SubtensorConfig>,
    hotkey: &subxt::utils::AccountId32,
    netuid: u16,
) -> Result<bool>
```

Checks if a hotkey is a member of a subnet.

---

## `get_network_registration_allowed`

```rust
pub async fn get_network_registration_allowed(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<bool>
```

Whether burn-based registration is allowed for the subnet.

---

## `get_network_pow_registration_allowed`

```rust
pub async fn get_network_pow_registration_allowed(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<bool>
```

Whether PoW registration is allowed for the subnet.

---

## `get_network_registered_at`

```rust
pub async fn get_network_registered_at(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64>
```

Block at which the subnet was registered.

---

## Capacity and Limit Queries

### `get_min_allowed_uids`
```rust
pub async fn get_min_allowed_uids(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u16>
```
Minimum allowed UIDs.

### `get_max_allowed_uids`
```rust
pub async fn get_max_allowed_uids(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u16>
```
Maximum allowed UIDs (default 256).

### `get_max_allowed_validators`
```rust
pub async fn get_max_allowed_validators(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u16>
```
Maximum allowed validators (default 128).

---

## Immunity and Activity

### `get_immunity_period`
```rust
pub async fn get_immunity_period(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u16>
```
Immunity period in blocks (default 4096).

### `get_activity_cutoff`
```rust
pub async fn get_activity_cutoff(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u16>
```
Activity cutoff in blocks (default 5000).

---

## Weight Parameters

### `get_max_weights_limit`
```rust
pub async fn get_max_weights_limit(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u16>
```
Maximum total weight a neuron can distribute (u16 scale, default 65535).

### `get_min_allowed_weights`
```rust
pub async fn get_min_allowed_weights(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u16>
```
Minimum allowed weight value (default 1024).

---

## Adjustment Parameters

### `get_adjustment_interval`
```rust
pub async fn get_adjustment_interval(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u16>
```
Blocks between difficulty/burn adjustments (default 100).

### `get_bonds_moving_average`
```rust
pub async fn get_bonds_moving_average(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u64>
```
Bonds moving average denominator (default 900000).

### `get_bonds_penalty`
```rust
pub async fn get_bonds_penalty(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u16>
```
Bonds penalty value.

### `get_bonds_reset_on`
```rust
pub async fn get_bonds_reset_on(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<bool>
```
Whether bonds reset on registration.

### `get_scaling_law_power`
```rust
pub async fn get_scaling_law_power(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u16>
```
Scaling law exponent (default 50 = 0.5).

### `get_target_registrations_per_interval`
```rust
pub async fn get_target_registrations_per_interval(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u16>
```
Target registrations per adjustment interval (default 2).

### `get_adjustment_alpha`
```rust
pub async fn get_adjustment_alpha(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u64>
```
EMA adjustment alpha for difficulty/burn.

---

## Alpha and Yuma Parameters

### `get_liquid_alpha_on`
```rust
pub async fn get_liquid_alpha_on(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<bool>
```
Whether liquid alpha is enabled.

### `get_yuma3_on`
```rust
pub async fn get_yuma3_on(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<bool>
```
Whether Yuma3 consensus is enabled.

### `get_alpha_values`
```rust
pub async fn get_alpha_values(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<(u16, u16)>
```
Alpha values as `(alpha_high, alpha_low)`.

### `get_subtoken_enabled`
```rust
pub async fn get_subtoken_enabled(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<bool>
```
Whether the subtoken economy is enabled.

---

## Serving and Registration

### `get_serving_rate_limit`
```rust
pub async fn get_serving_rate_limit(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u64>
```
Blocks between serve operations (default 50).

### `get_burn`
```rust
pub async fn get_burn(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u64>
```
Current burn cost for registration in rao.

### `get_difficulty`
```rust
pub async fn get_difficulty(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u64>
```
PoW difficulty for registration.

### `get_min_burn`
```rust
pub async fn get_min_burn(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u64>
```
Minimum registration burn in rao.

### `get_max_burn`
```rust
pub async fn get_max_burn(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u64>
```
Maximum registration burn in rao.

### `get_min_difficulty`
```rust
pub async fn get_min_difficulty(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u64>
```
Minimum PoW difficulty.

### `get_max_difficulty`
```rust
pub async fn get_max_difficulty(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u64>
```
Maximum PoW difficulty.

---

## Registration Tracking

### `get_last_adjustment_block`
```rust
pub async fn get_last_adjustment_block(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u64>
```
Last block at which difficulty/burn was adjusted.

### `get_registrations_this_interval`
```rust
pub async fn get_registrations_this_interval(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u16>
```
Registrations in the current adjustment interval.

### `get_registrations_this_block`
```rust
pub async fn get_registrations_this_block(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u16>
```
Registrations in the current block.

### `get_rao_recycled_for_registration`
```rust
pub async fn get_rao_recycled_for_registration(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u64>
```
Rao recycled (burned) for registrations in the current interval.

---

## Global Parameters

### `get_tx_rate_limit`
```rust
pub async fn get_tx_rate_limit(client: &OnlineClient<SubtensorConfig>) -> Result<u64>
```
Global transaction rate limit in blocks (default 1000).

### `get_ema_price_halving_blocks`
```rust
pub async fn get_ema_price_halving_blocks(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u64>
```
Blocks for EMA price halving (default 201600).

---

## Consensus Parameters

### `get_rho`
```rust
pub async fn get_rho(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u16>
```
Rho: overall incentive power (default 10).

### `get_kappa`
```rust
pub async fn get_kappa(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u16>
```
Kappa: temperature for sigmoid trust (default 32767).

### `get_alpha_sigmoid_steepness`
```rust
pub async fn get_alpha_sigmoid_steepness(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<i16>
```
Signed steepness of the alpha sigmoid function (default 1000).

---

## Voting and Locking

### `get_voting_power`
```rust
pub async fn get_voting_power(client: &OnlineClient<SubtensorConfig>, netuid: u16, hotkey: &subxt::utils::AccountId32) -> Result<u64>
```
Voting power for a hotkey in a subnet.

### `get_voting_power_tracking_enabled`
```rust
pub async fn get_voting_power_tracking_enabled(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<bool>
```
Whether voting power tracking is enabled.

### `get_max_registrations_per_block`
```rust
pub async fn get_max_registrations_per_block(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u16>
```
Max registrations per block (default 1).

### `get_validator_prune_len`
```rust
pub async fn get_validator_prune_len(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u64>
```
Validator prune length (default 1).

### `get_subnet_locked`
```rust
pub async fn get_subnet_locked(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u64>
```
Amount locked in the subnet in rao.

### `get_largest_locked`
```rust
pub async fn get_largest_locked(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u64>
```
Largest single lock in the subnet in rao.

### `get_transfer_toggle`
```rust
pub async fn get_transfer_toggle(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<bool>
```
Whether transfers are enabled for the subnet.

---

## Full Example

```rust
// Requires live node
use bittensor_chain::prelude::*;
use bittensor_chain::queries::subnet;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;
    let rpc = client.rpc();

    let total = subnet::get_total_subnets(rpc).await?;
    println!("Total subnets: {total}");

    for netuid in 0..total {
        if subnet::subnet_exists(rpc, netuid).await? {
            let tempo = subnet::get_tempo(rpc, netuid).await?;
            let n = subnet::get_subnetwork_n(rpc, netuid).await?;
            println!("Subnet {netuid}: tempo={tempo}, neurons={n}");
        }
    }

    if let Some(hp) = subnet::get_subnet_hyperparameters(rpc, 1).await? {
        println!("Subnet 1 rho={} kappa={}", hp.rho, hp.kappa);
        println!("  liquid_alpha={}", hp.liquid_alpha_enabled);
    }

    Ok(())
}
```
