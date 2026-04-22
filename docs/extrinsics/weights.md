# Weights Extrinsics

Module path: `bittensor_chain::extrinsics::weights`

Weights extrinsics control how validators signal their trust in other neurons on a subnet. Setting weights determines the emission distribution within a subnet. The module supports direct weight setting, as well as commit-and-reveal schemes that hide weight assignments until the reveal phase to reduce copycat behavior.

## Transaction Result

All weights functions return `Result<TxSuccess>`. The `TxSuccess` struct:

```rust
pub struct TxSuccess {
    pub block_hash: H256,
    pub extrinsic_hash: H256,
}
```

- **`block_hash`**: The hash of the block that included the transaction.
- **`extrinsic_hash`**: The hash of the extrinsic, used for transaction tracking.

---

## set_weights

Set weights directly on a subnet. Each UID in the `uids` vector gets a corresponding weight from the `weights` vector. Weights must sum to a maximum of `u16::MAX` (65535) across all entries. The `version_key` field is used by the subnet validator to identify the algorithm version that produced the weights.

### Signature

```rust
pub async fn set_weights(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    netuid: u16,
    uids: Vec<u16>,
    weights: Vec<u16>,
    version_key: u64,
) -> Result<TxSuccess>
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected Subtensor client |
| `signer` | `&Keypair` | Hotkey of the validator setting weights |
| `netuid` | `u16` | Subnet ID where weights are being set |
| `uids` | `Vec<u16>` | List of neuron UIDs on the subnet |
| `weights` | `Vec<u16>` | Weight values for each UID. Must be same length as `uids`. Sum must not exceed 65535 |
| `version_key` | `u64` | Version identifier for the weight-setting algorithm. Used by subnet owners for compatibility checks |

### Returns

`Result<TxSuccess>` containing the block hash and extrinsic hash on success.

### Example

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::extrinsics::weights;

async fn set_subnet_weights() -> Result<()> {
    let client = OnlineClient::from_url("wss://entrypoint-finney.opentensor.ai:443").await?;
    let signer = subxt_signer::sr25519::Keypair::from_uri("//MyValidatorHotkey")?;

    let netuid: u16 = 1;

    // UIDs of neurons being ranked
    let uids = vec![0, 1, 2, 3, 4];

    // Corresponding weights (must sum to <= 65535)
    let weights = vec![30000, 15000, 10000, 5000, 5535];

    let version_key: u64 = 1; // Algorithm version 1

    let result = weights::set_weights(
        &client,
        &signer,
        netuid,
        uids,
        weights,
        version_key,
    ).await?;

    println!("Weights set on subnet {}, block: {}", netuid, result.block_hash);

    Ok(())
}
```

### Weight Normalization

Weights are typically normalized so their sum equals `u16::MAX` (65535). Raw scores must be scaled before passing them to `set_weights`:

```rust
fn normalize_weights(raw_scores: Vec<f64>) -> Vec<u16> {
    let total: f64 = raw_scores.iter().sum();
    if total == 0.0 {
        return vec![0; raw_scores.len()];
    }
    raw_scores
        .iter()
        .map(|s| ((s / total) * u16::MAX as f64) as u16)
        .collect()
}

let raw = vec![0.4, 0.3, 0.2, 0.1];
let normalized = normalize_weights(raw);
// normalized: [26214, 19660, 13107, 6553] (approximately)
```

---

## commit_weights

Commit a hashed version of weights without revealing the actual values. This is the first step of the commit-and-reveal weight setting scheme. The commitment is a 32-byte hash of the weight data. Weights remain hidden until `reveal_weights` is called in a later block.

### Signature

```rust
pub async fn commit_weights(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    netuid: u16,
    commitment: [u8; 32],
) -> Result<TxSuccess>
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected Subtensor client |
| `signer` | `&Keypair` | Hotkey of the validator committing weights |
| `netuid` | `u16` | Subnet ID where weights are being committed |
| `commitment` | `[u8; 32]` | Hash commitment of the weight data. Must be computed from the UIDs, weights, and a salt |

### Returns

`Result<TxSuccess>` containing the block hash and extrinsic hash on success.

### Example

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::extrinsics::weights;
use sha2::{Sha256, Digest};

fn compute_commitment(uids: &[u16], weights: &[u16], salt: &[u16]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    for uid in uids {
        hasher.update(uid.to_le_bytes());
    }
    for weight in weights {
        hasher.update(weight.to_le_bytes());
    }
    for s in salt {
        hasher.update(s.to_le_bytes());
    }
    let hash = hasher.finalize();
    let mut result = [0u8; 32];
    result.copy_from_slice(&hash);
    result
}

async fn commit_my_weights() -> Result<()> {
    let client = OnlineClient::from_url("wss://entrypoint-finney.opentensor.ai:443").await?;
    let signer = subxt_signer::sr25519::Keypair::from_uri("//MyValidatorHotkey")?;

    let netuid: u16 = 1;
    let uids = vec![0, 1, 2];
    let weights = vec![30000, 20000, 15535];
    let salt = vec![42, 17, 91];

    let commitment = compute_commitment(&uids, &weights, &salt);

    let result = weights::commit_weights(
        &client,
        &signer,
        netuid,
        commitment,
    ).await?;

    println!("Weight commitment submitted, block: {}", result.block_hash);

    // Store uids, weights, and salt for later reveal
    Ok(())
}
```

---

## reveal_weights

Reveal previously committed weights on a subnet. The chain verifies that the revealed UIDs, weights, and salt match the commitment hash submitted earlier. This is the second step of the commit-and-reveal scheme.

### Signature

```rust
pub async fn reveal_weights(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    netuid: u16,
    uids: Vec<u16>,
    weights: Vec<u16>,
    salt: Vec<u16>,
) -> Result<TxSuccess>
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected Subtensor client |
| `signer` | `&Keypair` | Hotkey of the validator revealing weights |
| `netuid` | `u16` | Subnet ID where weights are being revealed |
| `uids` | `Vec<u16>` | Neuron UIDs matching the committed data |
| `weights` | `Vec<u16>` | Weight values matching the committed data. Same length as `uids` |
| `salt` | `Vec<u16>` | Salt values used in the original commitment hash |

### Returns

`Result<TxSuccess>` containing the block hash and extrinsic hash on success. Returns an error if the revealed data does not match the commitment, or if no commitment exists.

### Example

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::extrinsics::weights;

async fn reveal_my_weights() -> Result<()> {
    let client = OnlineClient::from_url("wss://entrypoint-finney.opentensor.ai:443").await?;
    let signer = subxt_signer::sr25519::Keypair::from_uri("//MyValidatorHotkey")?;

    let netuid: u16 = 1;

    // These must match the data used in commit_weights
    let uids = vec![0, 1, 2];
    let weights = vec![30000, 20000, 15535];
    let salt = vec![42, 17, 91];

    let result = weights::reveal_weights(
        &client,
        &signer,
        netuid,
        uids,
        weights,
        salt,
    ).await?;

    println!("Weights revealed, block: {}", result.block_hash);

    Ok(())
}
```

### Reveal Timing

There is a minimum delay between committing and revealing. The exact number of blocks depends on subnet configuration. Revealing too early will cause the transaction to fail. Check the subnet's `commit_reveal_interval` before calling `reveal_weights`.

---

## commit_timelocked_weights

Commit weights with a time-lock that prevents early reveals. The `reveal_round` parameter specifies the earliest epoch at which the weights can be revealed. This adds an extra layer of protection against front-running, since other validators cannot see the weights until the specified round is reached.

### Signature

```rust
pub async fn commit_timelocked_weights(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    netuid: u16,
    commitment: [u8; 32],
    reveal_round: u64,
) -> Result<TxSuccess>
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected Subtensor client |
| `signer` | `&Keypair` | Hotkey of the validator committing weights |
| `netuid` | `u16` | Subnet ID where weights are being committed |
| `commitment` | `[u8; 32]` | Hash commitment of the weight data (same format as `commit_weights`) |
| `reveal_round` | `u64` | The earliest epoch number at which the reveal is allowed. Must be greater than the current epoch plus the subnet's minimum interval |

### Returns

`Result<TxSuccess>` containing the block hash and extrinsic hash on success.

### Example

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::extrinsics::weights;
use sha2::{Sha256, Digest};

fn compute_commitment(uids: &[u16], weights: &[u16], salt: &[u16]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    for uid in uids {
        hasher.update(uid.to_le_bytes());
    }
    for weight in weights {
        hasher.update(weight.to_le_bytes());
    }
    for s in salt {
        hasher.update(s.to_le_bytes());
    }
    let hash = hasher.finalize();
    let mut result = [0u8; 32];
    result.copy_from_slice(&hash);
    result
}

async fn commit_timelocked() -> Result<()> {
    let client = OnlineClient::from_url("wss://entrypoint-finney.opentensor.ai:443").await?;
    let signer = subxt_signer::sr25519::Keypair::from_uri("//MyValidatorHotkey")?;

    let netuid: u16 = 1;
    let uids = vec![0, 1, 2, 3];
    let weights = vec![25000, 20000, 15000, 5535];
    let salt = vec![100, 200, 300, 400];

    let commitment = compute_commitment(&uids, &weights, &salt);

    // Query current epoch (pseudocode; use the chain query module for actual API)
    let current_epoch: u64 = 150;
    let reveal_round = current_epoch + 10; // Reveal allowed 10 epochs from now

    let result = weights::commit_timelocked_weights(
        &client,
        &signer,
        netuid,
        commitment,
        reveal_round,
    ).await?;

    println!(
        "Time-locked commitment submitted. Reveal allowed at epoch {}",
        reveal_round
    );
    println!("Block: {}", result.block_hash);

    Ok(())
}
```

---

## Important Notes

### Weight Setting Frequency

Each subnet enforces a minimum interval between weight-setting operations for the same hotkey. Calling `set_weights` or `reveal_weights` too frequently will result in the transaction being rejected. The typical cooldown is one epoch per subnet, but this is configurable by the subnet owner.

### Commit-and-Reveal Rationale

The commit-and-reveal scheme prevents validators from copying each other's weight assignments. When a validator commits weights, other participants see only the hash, not the actual values. After the minimum interval passes, the validator reveals the actual weights, and the chain verifies the hash matches. Time-locked commitments add further protection by specifying the earliest reveal epoch.

### UID vs. Hotkey

The `uids` parameter uses integer UIDs, not hotkey AccountIds. UIDs are assigned sequentially when a neuron registers on a subnet. You can look up a UID for a given hotkey by querying the subnet's UID mapping:

```rust
// Query UID for a hotkey on a subnet (pseudocode; use chain query module for actual API)
let uid = client
    .storage()
    .fetch(&subtensor::storage().uid_for_hotkey(netuid, &hotkey), None)
    .await?;
```

### Version Key

The `version_key` is an opaque value that subnet owners use to verify compatibility between the weight-setting algorithm and the subnet's expectations. Subnet owners may reject weights that use an incompatible version key. Check the subnet's documentation or on-chain metadata for the expected version key value.

### Weight Sum Constraint

Weights passed to `set_weights` must sum to at most `u16::MAX` (65535). If the sum exceeds this limit, the transaction will fail validation. The chain does not automatically normalize weights. You must normalize them before calling the function.
