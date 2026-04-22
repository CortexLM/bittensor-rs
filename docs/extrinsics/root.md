# Root Extrinsics

Module path: `bittensor_chain::extrinsics::root`

Root extrinsics operate on the root subnet (netuid 0), which controls global emission distribution across all subnets on the Bittensor network. Only registered root members (typically senators) can call these functions. Root weights determine how much TAO each subnet receives per block.

## Transaction Result

All root functions return `Result<TxSuccess>`. The `TxSuccess` struct:

```rust
pub struct TxSuccess {
    pub block_hash: H256,
    pub extrinsic_hash: H256,
}
```

- **`block_hash`**: The hash of the block that included the transaction.
- **`extrinsic_hash`**: The hash of the extrinsic, used for transaction tracking.

## Root Subnet

The root subnet (netuid 0) is the top-level subnet that governs how emissions are distributed across all other subnets. Each registered root member sets weights over subnet UIDs, and the aggregate of these weights determines the emission split. Root extrinsics require the signer to be registered on the root subnet, which is restricted to senators and other authorized accounts.

---

## root_set_weights

Set weights on the root subnet to influence emission distribution across subnets. Each UID in the `uids` vector corresponds to a subnet ID, and each weight in the `weights` vector determines the relative emission share for that subnet. The sum of weights must not exceed `u16::MAX` (65535).

### Signature

```rust
pub async fn root_set_weights(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    uids: Vec<u16>,
    weights: Vec<u16>,
    version_key: u64,
) -> Result<TxSuccess>
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected Subtensor client |
| `signer` | `&Keypair` | Hotkey of the root member setting weights |
| `uids` | `Vec<u16>` | Subnet UIDs being weighted (these are netuid values, not neuron UIDs) |
| `weights` | `Vec<u16>` | Weight values for each subnet. Must be same length as `uids`. Sum must not exceed 65535 |
| `version_key` | `u64` | Version identifier for the weight-setting algorithm |

### Returns

`Result<TxSuccess>` containing the block hash and extrinsic hash on success.

### Example

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::extrinsics::root;

async fn set_root_weights() -> Result<()> {
    let client = OnlineClient::from_url("wss://entrypoint-finney.opentensor.ai:443").await?;
    let signer = subxt_signer::sr25519::Keypair::from_uri("//MySenateHotkey")?;

    // Subnet IDs being weighted
    let uids = vec![1, 2, 3, 5, 8];

    // Relative weights for each subnet (sum <= 65535)
    // Subnet 1 gets the most emission, subnet 8 gets the least
    let weights = vec![20000, 15000, 12000, 10000, 8535];

    let version_key: u64 = 1;

    let result = root::root_set_weights(
        &client,
        &signer,
        uids,
        weights,
        version_key,
    ).await?;

    println!("Root weights set, block: {}", result.block_hash);

    Ok(())
}
```

### Weight Normalization for Root

Root weights work the same way as subnet weights: they are u16 values that must sum to at most 65535. Normalize raw scores before passing them:

```rust
fn normalize_root_weights(raw_scores: Vec<f64>) -> Vec<u16> {
    let total: f64 = raw_scores.iter().sum();
    if total == 0.0 {
        return vec![0; raw_scores.len()];
    }
    raw_scores
        .iter()
        .map(|s| ((s / total) * u16::MAX as f64) as u16)
        .collect()
}

// Example: 5 subnets with different perceived value
let raw = vec![0.35, 0.25, 0.20, 0.15, 0.05];
let normalized = normalize_root_weights(raw);
```

---

## root_claim

Claim membership on the root subnet. This is used when a hotkey meets the criteria for root membership (such as being a senator) but is not yet registered. Calling `root_claim` registers the signer on netuid 0, enabling them to set root weights.

### Signature

```rust
pub async fn root_claim(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
) -> Result<TxSuccess>
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected Subtensor client |
| `signer` | `&Keypair` | Hotkey claiming root membership |

### Returns

`Result<TxSuccess>` containing the block hash and extrinsic hash on success.

### Example

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::extrinsics::root;

async fn claim_root_membership() -> Result<()> {
    let client = OnlineClient::from_url("wss://entrypoint-finney.opentensor.ai:443").await?;
    let signer = subxt_signer::sr25519::Keypair::from_uri("//MySenateHotkey")?;

    let result = root::root_claim(&client, &signer).await?;
    println!("Root membership claimed, block: {}", result.block_hash);

    Ok(())
}
```

### When to Use root_claim

You only need to call `root_claim` once, when the hotkey first becomes eligible for root membership. After claiming, the hotkey remains registered on the root subnet. Check current root membership before calling:

```rust
// Check if already registered on root (pseudocode; use chain query module for actual API)
let is_root_member = client
    .storage()
    .fetch(&subtensor::storage().is_registered(0, &hotkey), None)
    .await?
    .unwrap_or(false);

if !is_root_member {
    let result = root::root_claim(&client, &signer).await?;
}
```

---

## Important Notes

### Senate-Only Access

Root extrinsics are restricted. Only accounts that are members of the senate (or otherwise authorized by the chain) can register on the root subnet and set root weights. Attempting to call `root_set_weights` or `root_claim` without authorization will result in a transaction failure.

### Root Weights and Emission

The aggregate of all root members' weights determines how much TAO each subnet receives per block. A subnet with higher total root weight receives a larger share of the daily emission. This makes root weight setting a governance-level decision that affects the entire network economy.

### Weight Setting Frequency

Root members can set weights once per epoch on the root subnet. The epoch length is configurable but typically aligns with the tempo of the root subnet. Setting weights more frequently than the epoch allows has no effect; the transaction will be rejected.

### Version Key for Root

The `version_key` parameter in `root_set_weights` serves the same purpose as in subnet-level `set_weights`: it identifies the algorithm version. The Bittensor governance may require a specific version key for root weight submissions. Check the latest governance proposals for the expected value.

### Senate Composition Changes

When the senate composition changes (members added or removed), the emission distribution adjusts at the next epoch boundary. New senate members should call `root_claim` and then `root_set_weights` promptly to participate in emission governance.

### Zero Weights

Setting a weight of zero for a subnet effectively votes to remove that subnet's emission. While this is valid, it should be used cautiously as it can significantly impact the subnet's validator and miner incentives.
