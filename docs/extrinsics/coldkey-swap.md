# Coldkey Swap Extrinsics

Module path: `bittensor_chain::extrinsics::coldkey_swap`

Coldkey swap extrinsics handle the replacement of a coldkey across the entire Bittensor state. This is the most security-sensitive operation on the network: it transfers ownership of all stake, delegation, and balance from the old coldkey to a new one. It is used when a coldkey is compromised or needs rotation.

## Transaction Result

The coldkey swap function returns `Result<TxSuccess>`. The `TxSuccess` struct:

```rust
pub struct TxSuccess {
    pub block_hash: H256,
    pub extrinsic_hash: H256,
}
```

- **`block_hash`**: The hash of the block that included the transaction.
- **`extrinsic_hash`**: The hash of the extrinsic, used for transaction tracking.

## Irreversible Operation

Coldkey swap is irreversible. Once the swap is finalized, the old coldkey loses all access to its stake, balance, and delegation relationships. The new coldkey becomes the sole owner. There is no undo, no grace period after finalization, and no recovery if the new coldkey is also lost.

---

## swap_coldkey

Replace the signer's coldkey with a new coldkey. The chain migrates all balances, stake, and delegation records from the old coldkey to the new one.

### Signature

```rust
pub async fn swap_coldkey(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    new_coldkey: AccountId32,
) -> Result<TxSuccess>
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected Subtensor client |
| `signer` | `&Keypair` | The current coldkey being replaced |
| `new_coldkey` | `AccountId32` | The new coldkey that will receive ownership |

### Returns

`Result<TxSuccess>` containing the block hash and extrinsic hash on success.

### Example

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::extrinsics::coldkey_swap;

async fn rotate_compromised_coldkey() -> Result<()> {
    let client = OnlineClient::from_url("wss://entrypoint-finney.opentensor.ai:443").await?;

    // Sign with the OLD coldkey (the one being replaced)
    let old_signer = subxt_signer::sr25519::Keypair::from_uri("//CompromisedColdkey")?;

    // The NEW coldkey address (generated offline, private key kept secure)
    let new_coldkey: AccountId32 = "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92TjN5Yv7U2Z"
        .parse()?;

    let result = coldkey_swap::swap_coldkey(&client, &old_signer, new_coldkey).await?;
    println!("Coldkey swapped, block: {}", result.block_hash);
    println!("Old coldkey has been fully deactivated.");

    Ok(())
}
```

### Generating the New Coldkey

The new coldkey should be generated offline and its private key stored securely before initiating the swap. You can generate a new keypair using the wallet CLI or programmatically:

```rust
use subxt_signer::sr25519::Keypair;

// Generate a new random keypair
let new_keypair = Keypair::generate();

// The AccountId32 for the swap call
let new_coldkey: AccountId32 = new_keypair.public_key().into();

// IMPORTANT: Store the new keypair's secret bytes securely
let secret_bytes = new_keypair.into_bytes();
// Write to an offline hardware device or air-gapped storage
```

### Swap with URI-Based Key

If you have the new coldkey's URI (for development or testing):

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::extrinsics::coldkey_swap;

async fn swap_to_new_uri() -> Result<()> {
    let client = OnlineClient::from_url("wss://entrypoint-finney.opentensor.ai:443").await?;
    let old_signer = subxt_signer::sr25519::Keypair::from_uri("//OldColdkey")?;

    // Create the new keypair from URI to get its public key
    let new_keypair = subxt_signer::sr25519::Keypair::from_uri("//NewColdkey")?;
    let new_coldkey: AccountId32 = new_keypair.public_key().into();

    let result = coldkey_swap::swap_coldkey(&client, &old_signer, new_coldkey).await?;
    println!("Swapped to new coldkey: {}", new_coldkey);
    println!("Block: {}", result.block_hash);

    Ok(())
}
```

---

## What Gets Migrated

The coldkey swap transfers all of the following from the old coldkey to the new one:

| Asset | Behavior |
|---|---|
| Free balance | Migrated in full |
| Staked balance (all subnets) | Migrated in full |
| Delegation relationships | Ownership transferred; delegate take rates unchanged |
| Hotkey associations | All hotkey ownership records updated |
| Children assignments | Parent-child relationships preserved under new coldkey |
| Proxy relationships | Proxies where old coldkey is the delegator are updated |

After the swap, the old coldkey retains no assets, no stake, and no authority on the network.

---

## Important Notes

### Security Precautions

This is the highest-risk operation in the Bittensor SDK. Follow these precautions:

1. **Generate the new key offline.** Never create the new coldkey on a machine connected to the internet.
2. **Verify the new address twice.** A typo in the `new_coldkey` parameter sends all your assets to the wrong address. There is no recovery.
3. **Back up the new keypair.** After the swap, the old coldkey is useless. If you lose the new keypair, you lose everything.
4. **Act quickly if compromised.** If your coldkey is compromised, swap it before the attacker can drain funds or restake.
5. **Do not reuse old keys.** After swapping, never use the old coldkey again. It has no value, and any transactions signed by it will fail.

### Transaction Cost

The coldkey swap transaction itself requires a fee paid from the old coldkey's free balance. Ensure the old coldkey has enough free balance to cover the fee before submitting. If the free balance is zero, the transaction cannot be submitted.

### No Partial Swap

You cannot swap a subset of your assets. The swap is all-or-nothing: every balance, stake entry, and delegation record transfers at once. If you only want to move specific stake, use `staking::transfer_stake` instead.

### Hotkey Keys Are Unaffected

The coldkey swap does not rotate hotkeys. All hotkey keypairs remain the same; only the coldkey that owns them changes. Hotkeys continue to function normally under the new coldkey's authority.

### Proxy Considerations

If the old coldkey has granted proxy authority to other accounts, those proxy relationships are updated to reference the new coldkey. The proxy delegates retain their authority, now acting on behalf of the new coldkey. If you want to revoke proxies during the swap, remove them before calling `swap_coldkey`.

### Verification After Swap

After the swap completes, verify the migration by querying the new coldkey's balance and stake. Also confirm that the old coldkey's balance is zero:

```rust
// Query new coldkey balance (pseudocode; use chain query module for actual API)
let new_balance = client
    .storage()
    .fetch(&subtensor::storage().account_balance(&new_coldkey), None)
    .await?;

// Query old coldkey balance (should be zero)
let old_balance = client
    .storage()
    .fetch(&subtensor::storage().account_balance(&old_coldkey), None)
    .await?;

assert_eq!(old_balance, 0, "Old coldkey should have zero balance after swap");
```
