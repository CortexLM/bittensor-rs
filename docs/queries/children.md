# Childkey Queries

Module: `bittensor_chain::queries::children`

Childkey hierarchy, take rates, and cooldown tracking.

```rust
use bittensor_chain::queries::children;
use bittensor_chain::prelude::*;
```

All functions take `&OnlineClient<SubtensorConfig>` as the first argument and return `Result<T, BittensorError>`.

---

## `get_children`

```rust
pub async fn get_children(
    client: &OnlineClient<SubtensorConfig>,
    hotkey: &subxt::utils::AccountId32,
    netuid: u16,
) -> Result<Vec<(u64, AccountId32)>>
```

Fetches the child keys for a hotkey in a subnet. Each element is a `(proportion, child_account)` pair, where `proportion` is the share of the parent's stake that flows to this child key.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `hotkey` | `&AccountId32` | Parent hotkey account ID |
| `netuid` | `u16` | Subnet unique identifier |

**Returns**

`Vec<(u64, AccountId32)>` -- List of `(proportion, child_account)` pairs. Empty if no child keys are registered.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::children;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let hotkey = subxt::utils::AccountId32::from([0u8; 32]);
    let child_keys = children::get_children(client.rpc(), &hotkey, 1).await?;
    for (proportion, child) in child_keys {
        println!("Child {} with proportion {}", child, proportion);
    }

    Ok(())
}
```

---

## `get_childkey_take`

```rust
pub async fn get_childkey_take(
    client: &OnlineClient<SubtensorConfig>,
    hotkey: &subxt::utils::AccountId32,
    netuid: u16,
) -> Result<u16>
```

Fetches the childkey take rate for a hotkey in a subnet. This is the fraction of the child's emission that the parent claims, expressed in basis points (e.g., 1200 = 12%).

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `hotkey` | `&AccountId32` | Hotkey account ID |
| `netuid` | `u16` | Subnet unique identifier |

**Returns**

`u16` -- Take rate in parts-per-ten-thousand. Returns 0 if not set.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::children;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let hotkey = subxt::utils::AccountId32::from([0u8; 32]);
    let take = children::get_childkey_take(client.rpc(), &hotkey, 1).await?;
    println!("Childkey take: {}bps ({:.2}%)", take, take as f64 / 100.0);

    Ok(())
}
```

---

## `get_pending_child_key_cooldown`

```rust
pub async fn get_pending_child_key_cooldown(
    client: &OnlineClient<SubtensorConfig>,
) -> Result<u64>
```

Fetches the global pending child key cooldown period in blocks. After revoking a child key relationship, a hotkey must wait this many blocks before establishing a new child key link.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |

**Returns**

`u64` -- Cooldown period in blocks.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::children;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let cooldown = children::get_pending_child_key_cooldown(client.rpc()).await?;
    println!("Child key cooldown: {} blocks", cooldown);

    Ok(())
}
```

---

## `get_parent_keys`

```rust
pub async fn get_parent_keys(
    client: &OnlineClient<SubtensorConfig>,
    hotkey: &subxt::utils::AccountId32,
    netuid: u16,
) -> Result<Vec<(u64, AccountId32)>>
```

Fetches the parent keys for a hotkey in a subnet. This is the reverse lookup of `get_children`: given a child hotkey, returns the parent hotkeys and their proportions. Each element is a `(proportion, parent_account)` pair.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `hotkey` | `&AccountId32` | Child hotkey account ID |
| `netuid` | `u16` | Subnet unique identifier |

**Returns**

`Vec<(u64, AccountId32)>` -- List of `(proportion, parent_account)` pairs. Empty if no parent keys exist.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::children;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let hotkey = subxt::utils::AccountId32::from([0u8; 32]);
    let parents = children::get_parent_keys(client.rpc(), &hotkey, 1).await?;
    for (proportion, parent) in parents {
        println!("Parent {} with proportion {}", parent, proportion);
    }

    Ok(())
}
```

---

## `get_pending_child_keys`

```rust
pub async fn get_pending_child_keys(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    hotkey: &subxt::utils::AccountId32,
) -> Result<(Vec<(u64, AccountId32)>, u64)>
```

Fetches the pending child key assignments for a subnet and hotkey. Returns a tuple containing the list of pending `(proportion, child_account)` pairs and a cooldown value indicating when the pending assignment can be finalized.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `netuid` | `u16` | Subnet unique identifier |
| `hotkey` | `&AccountId32` | Hotkey account ID |

**Returns**

`(Vec<(u64, AccountId32)>, u64)` -- A tuple of:
- `Vec<(u64, AccountId32)>` -- Pending child key assignments as `(proportion, child_account)` pairs.
- `u64` -- Cooldown remaining before the pending assignment can be finalized.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::children;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let hotkey = subxt::utils::AccountId32::from([0u8; 32]);
    let (pending, cooldown) = children::get_pending_child_keys(client.rpc(), 1, &hotkey).await?;
    println!("Pending children: {}", pending.len());
    println!("Cooldown remaining: {} blocks", cooldown);
    for (proportion, child) in pending {
        println!("  Pending child {} with proportion {}", child, proportion);
    }

    Ok(())
}
```

---

## Full Example

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::children;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;
    let rpc = client.rpc();

    // Requires live node
    let hotkey = subxt::utils::AccountId32::from([0u8; 32]);

    // Child key hierarchy
    let child_keys = children::get_children(rpc, &hotkey, 1).await?;
    println!("Child keys: {} entries", child_keys.len());

    // Take rate
    let take = children::get_childkey_take(rpc, &hotkey, 1).await?;
    println!("Childkey take: {}bps", take);

    // Cooldown
    let cooldown = children::get_pending_child_key_cooldown(rpc).await?;
    println!("Global child key cooldown: {} blocks", cooldown);

    // Parent lookup
    let parents = children::get_parent_keys(rpc, &hotkey, 1).await?;
    println!("Parent keys: {} entries", parents.len());

    // Pending assignments
    let (pending, pending_cooldown) = children::get_pending_child_keys(rpc, 1, &hotkey).await?;
    println!("Pending children: {} (cooldown: {} blocks)", pending.len(), pending_cooldown);

    Ok(())
}
```
