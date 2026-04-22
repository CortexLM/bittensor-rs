# Proxy Queries

Module: `bittensor_chain::queries::proxy`

Proxy account lookups, permission checks, and the `ProxyType` enum for Bittensor's proxy delegation system.

```rust
use bittensor_chain::queries::proxy;
use bittensor_chain::prelude::*;
```

All functions take `&OnlineClient<SubtensorConfig>` as the first argument and return `Result<T, BittensorError>`.

---

## ProxyType Enum

The Bittensor runtime defines 18 proxy types for fine-grained permission delegation. Each type corresponds to a specific set of allowed extrinsics.

| Index | Variant | Description |
|-------|---------|-------------|
| 0 | `Any` | Full access -- the delegate can perform any action on behalf of the delegator |
| 1 | `Owner` | Owner-level permissions (subnet ownership operations) |
| 2 | `NonCritical` | Non-critical operations that cannot cause irreversible state changes |
| 3 | `NonTransfer` | All operations except token transfers |
| 4 | `Senate` | Senate voting and governance participation |
| 5 | `NonFungible` | Non-fungible token operations |
| 6 | `Triumvirate` | Triumvirate governance operations |
| 7 | `Governance` | General governance operations (voting, proposals) |
| 8 | `Staking` | Staking and unstaking operations |
| 9 | `Registration` | Neuron registration in subnets |
| 10 | `Transfer` | Token transfer operations |
| 11 | `SmallTransfer` | Limited-amount transfers |
| 12 | `RootWeights` | Setting weights on the root network |
| 13 | `ChildKeys` | Managing child key relationships |
| 14 | `SudoUncheckedSetCode` | Sudo-level runtime code updates (reserved) |
| 15 | `SwapHotkey` | Hotkey swap operations |
| 16 | `SubnetLeaseBeneficiary` | Managing subnet lease beneficiaries |
| 17 | `RootClaim` | Root network claim operations |

---

## `get_proxies`

```rust
pub async fn get_proxies(
    client: &OnlineClient<SubtensorConfig>,
    delegator: &subxt::utils::AccountId32,
    _delegate: &subxt::utils::AccountId32,
    _proxy_type: Option<u8>,
) -> Result<Vec<(subxt::utils::AccountId32, u8)>>
```

Fetches the proxy list for a delegator. Returns all proxy definitions associated with the delegator from the `proxy` pallet's `proxies` storage. Each result is a `(delegate_account, proxy_type_u8)` pair.

Note: The `delegate` and `proxy_type` parameters are currently unused and reserved for future filtering.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `delegator` | `&AccountId32` | The account that has set up proxies |
| `_delegate` | `&AccountId32` | Reserved for future filtering (currently unused) |
| `_proxy_type` | `Option<u8>` | Reserved for future filtering (currently unused) |

**Returns**

`Vec<(AccountId32, u8)>` -- List of `(delegate_account, proxy_type_index)` pairs. Empty if no proxies are configured.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::proxy;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let delegator = subxt::utils::AccountId32::from([0u8; 32]);
    let delegate = subxt::utils::AccountId32::from([1u8; 32]);
    let proxies = proxy::get_proxies(client.rpc(), &delegator, &delegate, None).await?;
    for (account, ptype) in proxies {
        println!("Delegate {} has proxy type index {}", account, ptype);
    }

    Ok(())
}
```

---

## `get_pure_proxy`

```rust
pub async fn get_pure_proxy(
    _client: &OnlineClient<SubtensorConfig>,
    _proxy_account: &subxt::utils::AccountId32,
) -> Result<Option<subxt::utils::AccountId32>>
```

Fetches the pure (anonymous) proxy for a given proxy account. This is a stub implementation that always returns `None`. Pure proxies are anonymous proxy accounts that are derived from the delegator and proxy type; this query will be implemented in a future release.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `_client` | `&OnlineClient<SubtensorConfig>` | Currently unused (stub) |
| `_proxy_account` | `&AccountId32` | Currently unused (stub) |

**Returns**

`Option<AccountId32>` -- Always returns `None` (stub).

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::proxy;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    let proxy_account = subxt::utils::AccountId32::from([0u8; 32]);
    // Currently returns None (stub)
    if let Some(pure) = proxy::get_pure_proxy(client.rpc(), &proxy_account).await? {
        println!("Pure proxy: {}", pure);
    } else {
        println!("Pure proxy lookup not yet implemented");
    }

    Ok(())
}
```

---

## `get_proxy`

```rust
pub async fn get_proxy(
    client: &OnlineClient<SubtensorConfig>,
    delegator: &subxt::utils::AccountId32,
) -> Result<Option<(BoundedVec<ProxyDef>, u64)>>
```

Fetches the raw proxy definition entry for a delegator from the `proxy` pallet's `proxies` storage. Returns the full `(proxies_vec, deposit)` tuple, where `proxies_vec` is a `BoundedVec` of `ProxyDefinition` structs and `deposit` is the total reserve balance held for the proxy registrations.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `delegator` | `&AccountId32` | The delegator account |

**Returns**

`Option<(BoundedVec<ProxyDef>, u64)>` -- The proxy list and deposit, or `None` if the delegator has no proxy entries.

**Type Aliases**

- `ProxyDef` = `ProxyDefinition<AccountId32, ProxyType, u32>` -- Contains fields: `delegate` (AccountId32), `proxy_type` (ProxyType enum), and `delay` (u32).
- The second element of the tuple is the `deposit` (u64) reserved for the proxy registrations.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::proxy;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let delegator = subxt::utils::AccountId32::from([0u8; 32]);
    if let Some((proxy_vec, deposit)) = proxy::get_proxy(client.rpc(), &delegator).await? {
        println!("{} proxy entries, deposit: {}", proxy_vec.0.len(), deposit);
    } else {
        println!("No proxy entries for delegator");
    }

    Ok(())
}
```

---

## `get_check_permissions`

```rust
pub async fn get_check_permissions(
    client: &OnlineClient<SubtensorConfig>,
    delegator: &subxt::utils::AccountId32,
    delegate: &subxt::utils::AccountId32,
    proxy_type: u8,
) -> Result<bool>
```

Checks whether a delegate has a specific proxy type permission for a delegator. Looks up the delegator's proxy list and returns `true` only if the delegate is found with the matching proxy type index.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `delegator` | `&AccountId32` | The account that set up the proxy |
| `delegate` | `&AccountId32` | The account being delegated to |
| `proxy_type` | `u8` | The proxy type index to check (see ProxyType table above) |

**Returns**

`bool` -- `true` if the delegate has the specified proxy type permission, `false` otherwise.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::proxy;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let delegator = subxt::utils::AccountId32::from([0u8; 32]);
    let delegate = subxt::utils::AccountId32::from([1u8; 32]);

    // Check if delegate has Staking permission (index 8)
    let has_staking = proxy::get_check_permissions(client.rpc(), &delegator, &delegate, 8).await?;
    println!("Delegate has Staking permission: {}", has_staking);

    // Check if delegate has full Any permission (index 0)
    let has_any = proxy::get_check_permissions(client.rpc(), &delegator, &delegate, 0).await?;
    println!("Delegate has Any permission: {}", has_any);

    Ok(())
}
```

---

## Full Example

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::proxy;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;
    let rpc = client.rpc();

    // Requires live node
    let delegator = subxt::utils::AccountId32::from([0u8; 32]);
    let delegate = subxt::utils::AccountId32::from([1u8; 32]);

    // Get all proxies for a delegator
    let proxies = proxy::get_proxies(rpc, &delegator, &delegate, None).await?;
    println!("Delegator has {} proxy entries", proxies.len());
    for (account, ptype) in &proxies {
        println!("  Delegate {} with proxy type {}", account, ptype);
    }

    // Get raw proxy definition
    if let Some((proxy_vec, deposit)) = proxy::get_proxy(rpc, &delegator).await? {
        println!("Raw proxy list: {} entries, deposit: {}", proxy_vec.0.len(), deposit);
    }

    // Check specific permissions
    let can_stake = proxy::get_check_permissions(rpc, &delegator, &delegate, 8).await?;
    println!("Can stake: {}", can_stake);

    // Pure proxy lookup (stub)
    let pure = proxy::get_pure_proxy(rpc, &delegator).await?;
    println!("Pure proxy: {:?}", pure);

    Ok(())
}
```
