# Delegate Queries

Module: `bittensor_chain::queries::delegate`

Delegate list, take rates, delegation info, childkey hierarchy, and staking rate limits.

```rust
use bittensor_chain::queries::delegate;
use bittensor_chain::prelude::*;
use bittensor_core::types::DelegateInfo;
```

All functions take `&OnlineClient<SubtensorConfig>` as the first argument and return `Result<T, BittensorError>`.

---

## `get_delegates`

```rust
pub async fn get_delegates(client: &OnlineClient<SubtensorConfig>) -> Result<Vec<DelegateInfo>>
```

Fetches all delegates that have a non-zero take across any subnet. Iterates through every subnet and UID to find hotkeys with a `delegates` storage entry greater than zero.

**Returns**

`Vec<DelegateInfo>` -- List of delegates with their take, hotkey, and owner info. Stake and nominator fields are populated as zero/empty in the current implementation.

---

## `get_delegate_take`

```rust
pub async fn get_delegate_take(
    client: &OnlineClient<SubtensorConfig>,
    hotkey: &subxt::utils::AccountId32,
) -> Result<u16>
```

Fetches the delegate take rate for a hotkey in basis points (e.g. 1800 = 18%).

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `hotkey` | `&AccountId32` | Delegate hotkey |

**Returns**

`u16` -- Take rate in parts-per-ten-thousand.

---

## `get_delegated_info`

```rust
pub async fn get_delegated_info(
    client: &OnlineClient<SubtensorConfig>,
    coldkey: &subxt::utils::AccountId32,
) -> Result<Vec<DelegateInfo>>
```

Fetches delegate info for all hotkeys owned by a coldkey.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `coldkey` | `&AccountId32` | Owning coldkey |

**Returns**

`Vec<DelegateInfo>` -- Delegates owned by the coldkey with their take rates.

---

## `get_max_delegate_take`

```rust
pub async fn get_max_delegate_take(client: &OnlineClient<SubtensorConfig>) -> Result<u16>
```

Global maximum delegate take in basis points (default 11796 = ~18%).

---

## `get_min_delegate_take`

```rust
pub async fn get_min_delegate_take(client: &OnlineClient<SubtensorConfig>) -> Result<u16>
```

Global minimum delegate take in basis points (default 0).

---

## Childkey Take

### `get_childkey_take`
```rust
pub async fn get_childkey_take(
    client: &OnlineClient<SubtensorConfig>,
    hotkey: &subxt::utils::AccountId32,
    netuid: u16,
) -> Result<u16>
```
Childkey take for a hotkey in a subnet, in basis points.

### `get_max_childkey_take`
```rust
pub async fn get_max_childkey_take(client: &OnlineClient<SubtensorConfig>) -> Result<u16>
```
Global maximum childkey take (default 11796).

### `get_min_childkey_take`
```rust
pub async fn get_min_childkey_take(client: &OnlineClient<SubtensorConfig>) -> Result<u16>
```
Global minimum childkey take (default 0).

---

## Key Hierarchy

### `get_pending_child_keys`
```rust
pub async fn get_pending_child_keys(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    hotkey: &subxt::utils::AccountId32,
) -> Result<(Vec<(u64, subxt::utils::AccountId32)>, u64)>
```
Pending child keys for a subnet and hotkey. Returns `(pending_list, cooldown_block)`. Each entry in the list is `(expiry_block, child_account)`.

### `get_child_keys`
```rust
pub async fn get_child_keys(
    client: &OnlineClient<SubtensorConfig>,
    hotkey: &subxt::utils::AccountId32,
    netuid: u16,
) -> Result<Vec<(u64, subxt::utils::AccountId32)>>
```
Active child keys for a hotkey in a subnet. Each entry is `(proportion, child_account)`.

### `get_parent_keys`
```rust
pub async fn get_parent_keys(
    client: &OnlineClient<SubtensorConfig>,
    hotkey: &subxt::utils::AccountId32,
    netuid: u16,
) -> Result<Vec<(u64, subxt::utils::AccountId32)>>
```
Parent keys for a hotkey in a subnet. Each entry is `(proportion, parent_account)`.

---

## Staking Hotkeys

### `get_staking_hotkeys`
```rust
pub async fn get_staking_hotkeys(
    client: &OnlineClient<SubtensorConfig>,
    coldkey: &subxt::utils::AccountId32,
) -> Result<Vec<subxt::utils::AccountId32>>
```
Hotkeys that a coldkey has staked to.

### `get_num_staking_coldkeys`
```rust
pub async fn get_num_staking_coldkeys(client: &OnlineClient<SubtensorConfig>) -> Result<u64>
```
Global count of staking coldkeys.

---

## Auto-stake and Rate Limits

### `get_auto_stake_destination`
```rust
pub async fn get_auto_stake_destination(
    client: &OnlineClient<SubtensorConfig>,
    hotkey: &subxt::utils::AccountId32,
    netuid: u16,
) -> Result<Option<subxt::utils::AccountId32>>
```
Auto-stake destination for a hotkey in a subnet. Returns `None` if no destination is set.

### `get_last_tx_block`
```rust
pub async fn get_last_tx_block(
    client: &OnlineClient<SubtensorConfig>,
    hotkey: &subxt::utils::AccountId32,
) -> Result<u64>
```
Last block at which a hotkey submitted a transaction.

### `get_last_tx_block_child_key_take`
```rust
pub async fn get_last_tx_block_child_key_take(
    client: &OnlineClient<SubtensorConfig>,
    hotkey: &subxt::utils::AccountId32,
) -> Result<u64>
```
Last block at which a hotkey's childkey take was modified.

### `get_last_tx_block_delegate_take`
```rust
pub async fn get_last_tx_block_delegate_take(
    client: &OnlineClient<SubtensorConfig>,
    hotkey: &subxt::utils::AccountId32,
) -> Result<u64>
```
Last block at which a hotkey's delegate take was modified.

### `get_staking_operation_rate_limiter`
```rust
pub async fn get_staking_operation_rate_limiter(
    client: &OnlineClient<SubtensorConfig>,
    coldkey: &subxt::utils::AccountId32,
    hotkey: &subxt::utils::AccountId32,
    netuid: u16,
) -> Result<bool>
```
Whether the staking rate limiter is active for a coldkey/hotkey pair in a subnet. Returns `true` if the pair is rate-limited.

### `get_tx_delegate_take_rate_limit`
```rust
pub async fn get_tx_delegate_take_rate_limit(client: &OnlineClient<SubtensorConfig>) -> Result<u64>
```
Global rate limit for delegate take changes in blocks (default 216000).

### `get_tx_childkey_take_rate_limit`
```rust
pub async fn get_tx_childkey_take_rate_limit(client: &OnlineClient<SubtensorConfig>) -> Result<u64>
```
Global rate limit for childkey take changes in blocks (default 216000).

---

## Ownership

### `get_owner`
```rust
pub async fn get_owner(
    client: &OnlineClient<SubtensorConfig>,
    hotkey: &subxt::utils::AccountId32,
) -> Result<Option<subxt::utils::AccountId32>>
```
Fetches the coldkey that owns a hotkey. Returns `None` if the hotkey has no owner on-chain.

---

## Full Example

```rust
// Requires live node
use bittensor_chain::prelude::*;
use bittensor_chain::queries::delegate;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;
    let rpc = client.rpc();

    let delegates = delegate::get_delegates(rpc).await?;
    println!("Found {} delegates", delegates.len());
    for d in &delegates {
        println!("  {} take={}bps", d.delegate_hotkey, d.take);
    }

    let max_take = delegate::get_max_delegate_take(rpc).await?;
    println!("Max delegate take: {max_take} bps");

    let hotkey = subxt::utils::AccountId32::from([0u8; 32]); // replace with real key
    if let Some(owner) = delegate::get_owner(rpc, &hotkey).await? {
        println!("Owner of hotkey: {owner:?}");
    }

    Ok(())
}
```
