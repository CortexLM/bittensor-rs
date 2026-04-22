# Account Queries

Module: `bittensor_chain::queries::account`

Balance, stake, delegation, alpha, and token flow queries for on-chain accounts.

```rust
use bittensor_chain::queries::account;
use bittensor_chain::prelude::*;
use bittensor_core::balance::Balance;
use bittensor_core::config::NetworkConfig;
```

All functions take `&OnlineClient<SubtensorConfig>` as the first argument and return `Result<T, BittensorError>`.

---

## `get_balance`

```rust
pub async fn get_balance(
    client: &OnlineClient<SubtensorConfig>,
    account_id: &subxt::utils::AccountId32,
) -> Result<Balance>
```

Fetches the free balance for an account. Reads from the System pallet's `Account` storage map.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `account_id` | `&AccountId32` | The account to query |

**Returns**

`Balance` -- The free (spendable) balance in rao-denominated units. Use `.to_tao()` for human-readable output.

**Example**

```rust
// Requires live node
let balance: Balance = account::get_balance(client.rpc(), &account_id).await?;
println!("Free balance: {balance}");
println!("In TAO: {}", balance.to_tao());
```

---

## `get_total_balance`

```rust
pub async fn get_total_balance(
    client: &OnlineClient<SubtensorConfig>,
    account_id: &subxt::utils::AccountId32,
) -> Result<Balance>
```

Fetches the total balance (free + reserved) for an account.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `account_id` | `&AccountId32` | The account to query |

**Returns**

`Balance` -- Total balance (free plus reserved).

**Example**

```rust
// Requires live node
let total: Balance = account::get_total_balance(client.rpc(), &account_id).await?;
println!("Total balance: {total}");
```

---

## `get_stake`

```rust
pub async fn get_stake(
    _client: &OnlineClient<SubtensorConfig>,
    _coldkey: &subxt::utils::AccountId32,
    _hotkey: &subxt::utils::AccountId32,
    _netuid: u16,
) -> Result<Balance>
```

Fetches the stake for a coldkey/hotkey pair in a specific subnet. This is currently a stub that always returns zero.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `_client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client (unused) |
| `_coldkey` | `&AccountId32` | Coldkey account (unused) |
| `_hotkey` | `&AccountId32` | Hotkey account (unused) |
| `_netuid` | `u16` | Subnet ID (unused) |

**Returns**

`Balance::ZERO` -- Always returns zero in the current implementation.

**Example**

```rust
let stake = account::get_stake(client.rpc(), &coldkey, &hotkey, 1).await?;
assert_eq!(stake, Balance::ZERO);
```

---

## `get_stake_info_for_coldkey`

```rust
pub async fn get_stake_info_for_coldkey(
    client: &OnlineClient<SubtensorConfig>,
    coldkey: &subxt::utils::AccountId32,
) -> Result<Vec<StakeInfo>>
```

Fetches stake info for all hotkeys owned by a coldkey. Returns a `StakeInfo` for each owned hotkey with the stake field currently set to zero.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `coldkey` | `&AccountId32` | The coldkey to look up |

**Returns**

`Vec<StakeInfo>` -- List of stake entries. Each `StakeInfo` contains `hotkey`, `coldkey`, and `stake` fields.

**Example**

```rust
// Requires live node
let stakes = account::get_stake_info_for_coldkey(client.rpc(), &coldkey).await?;
for s in &stakes {
    println!("Hotkey: {} (stake: {})", s.hotkey, s.stake);
}
```

---

## `get_total_network_stake`

```rust
pub async fn get_total_network_stake(client: &OnlineClient<SubtensorConfig>) -> Result<Balance>
```

Fetches the total stake across the entire network. Reads from the `TotalStake` storage entry.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |

**Returns**

`Balance` -- Total network-wide stake.

**Example**

```rust
// Requires live node
let total = account::get_total_network_stake(client.rpc()).await?;
println!("Network total stake: {}", total.to_tao());
```

---

## `get_owned_hotkeys`

```rust
pub async fn get_owned_hotkeys(
    client: &OnlineClient<SubtensorConfig>,
    coldkey: &subxt::utils::AccountId32,
) -> Result<Vec<subxt::utils::AccountId32>>
```

Fetches all hotkeys owned by a coldkey. Reads from the `OwnedHotkeys` double map.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `coldkey` | `&AccountId32` | The owning coldkey |

**Returns**

`Vec<AccountId32>` -- List of hotkey account IDs owned by the coldkey.

**Example**

```rust
// Requires live node
let hotkeys = account::get_owned_hotkeys(client.rpc(), &coldkey).await?;
println!("Owns {} hotkeys", hotkeys.len());
```

---

## `get_total_hotkey_alpha`

```rust
pub async fn get_total_hotkey_alpha(
    client: &OnlineClient<SubtensorConfig>,
    hotkey: &subxt::utils::AccountId32,
    netuid: u16,
) -> Result<u64>
```

Fetches the total alpha tokens held by a hotkey in a subnet.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `hotkey` | `&AccountId32` | The hotkey account |
| `netuid` | `u16` | Subnet ID |

**Returns**

`u64` -- Total alpha for the hotkey in the subnet.

---

## `get_total_hotkey_alpha_last_epoch`

```rust
pub async fn get_total_hotkey_alpha_last_epoch(
    client: &OnlineClient<SubtensorConfig>,
    hotkey: &subxt::utils::AccountId32,
    netuid: u16,
) -> Result<u64>
```

Fetches the total alpha tokens held by a hotkey at the end of the last epoch in a subnet.

**Parameters**

Same as `get_total_hotkey_alpha`.

**Returns**

`u64` -- Alpha at the last epoch boundary.

---

## `get_token_symbol`

```rust
pub async fn get_token_symbol(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<Vec<u8>>
```

Fetches the token symbol bytes for a subnet. The raw bytes can be converted to a UTF-8 string.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `netuid` | `u16` | Subnet ID |

**Returns**

`Vec<u8>` -- Token symbol as raw bytes.

**Example**

```rust
let sym_bytes = account::get_token_symbol(client.rpc(), 1).await?;
let symbol = String::from_utf8_lossy(&sym_bytes);
println!("Subnet 1 token: {symbol}");
```

---

## `get_subnet_tao`

```rust
pub async fn get_subnet_tao(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<Balance>
```

Fetches the total TAO staked into a subnet.

**Returns**

`Balance` -- Total TAO in the subnet.

---

## `get_subnet_tao_provided`

```rust
pub async fn get_subnet_tao_provided(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64>
```

Fetches the subnet TAO provided value (raw rao).

**Returns**

`u64` -- Subnet TAO provided in rao.

---

## `get_subnet_alpha_in`

```rust
pub async fn get_subnet_alpha_in(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64>
```

Fetches the subnet alpha inflow (alpha entering the subnet).

**Returns**

`u64` -- Alpha in value.

---

## `get_subnet_alpha_in_provided`

```rust
pub async fn get_subnet_alpha_in_provided(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64>
```

Fetches the subnet alpha in provided value.

**Returns**

`u64` -- Alpha in provided.

---

## `get_subnet_alpha_out`

```rust
pub async fn get_subnet_alpha_out(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64>
```

Fetches the subnet alpha outflow (alpha leaving the subnet).

**Returns**

`u64` -- Alpha out value.

---

## `get_subnet_alpha_in_emission`

```rust
pub async fn get_subnet_alpha_in_emission(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64>
```

Fetches the alpha inflow emission for a subnet.

**Returns**

`u64` -- Alpha in emission rate.

---

## `get_subnet_alpha_out_emission`

```rust
pub async fn get_subnet_alpha_out_emission(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64>
```

Fetches the alpha outflow emission for a subnet.

**Returns**

`u64` -- Alpha out emission rate.

---

## `get_subnet_tao_in_emission`

```rust
pub async fn get_subnet_tao_in_emission(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64>
```

Fetches the TAO inflow emission for a subnet.

**Returns**

`u64` -- TAO in emission rate.

---

## `get_root_alpha_dividends_per_subnet`

```rust
pub async fn get_root_alpha_dividends_per_subnet(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    hotkey: &subxt::utils::AccountId32,
) -> Result<u64>
```

Fetches the root alpha dividends allocated to a hotkey from a specific subnet.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `netuid` | `u16` | The subnet paying dividends |
| `hotkey` | `&AccountId32` | The hotkey receiving dividends |

**Returns**

`u64` -- Root alpha dividend amount.

---

## `get_total_issuance`

```rust
pub async fn get_total_issuance(
    client: &OnlineClient<SubtensorConfig>,
) -> Result<Balance>
```

Fetches the global total token issuance.

**Returns**

`Balance` -- Total issuance across all subnets.

**Example**

```rust
// Requires live node
let issuance = account::get_total_issuance(client.rpc()).await?;
println!("Total issuance: {} TAO", issuance.to_tao());
```

---

## Full Example

```rust
// Requires live node
use bittensor_chain::prelude::*;
use bittensor_chain::queries::account;
use bittensor_core::config::NetworkConfig;
use bittensor_core::balance::Balance;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;
    let rpc = client.rpc();

    // Replace with a real account ID
    let account_id = subxt::utils::AccountId32::from([0u8; 32]);

    let free = account::get_balance(rpc, &account_id).await?;
    let total = account::get_total_balance(rpc, &account_id).await?;
    let network_stake = account::get_total_network_stake(rpc).await?;
    let issuance = account::get_total_issuance(rpc).await?;

    println!("Free:   {} TAO", free.to_tao());
    println!("Total:  {} TAO", total.to_tao());
    println!("Stake:  {} TAO", network_stake.to_tao());
    println!("Issued: {} TAO", issuance.to_tao());

    Ok(())
}
```
