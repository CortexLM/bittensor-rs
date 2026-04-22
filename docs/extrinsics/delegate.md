# Delegate Extrinsics

Module path: `bittensor_chain::extrinsics::take`

Delegate extrinsics manage the take rate, which is the percentage of emission that a delegate (validator) retains before distributing the remainder to delegators. A delegate can increase or decrease their take, or become a delegate for the first time.

## Transaction Result

All delegate functions return `Result<TxSuccess>`. The `TxSuccess` struct:

```rust
pub struct TxSuccess {
    pub block_hash: H256,
    pub extrinsic_hash: H256,
}
```

- **`block_hash`**: The hash of the block that included the transaction.
- **`extrinsic_hash`**: The hash of the extrinsic, used for tracking transaction status.

## Take Rate

The take rate is expressed as a value out of 10,000 (basis points). For example:
- A take of `1000` means the delegate keeps 10% of emission, and 90% goes to delegators.
- A take of `5000` means a 50/50 split.
- A take of `10000` means the delegate keeps 100% and delegators receive nothing.

The maximum take is `10000` (100%). The minimum depends on subnet rules but is typically `0` (0%).

---

## increase_take

Raise the take rate for a hotkey. The new take must be higher than the current take. This reduces the share of emission distributed to delegators.

### Signature

```rust
pub async fn increase_take(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    hotkey: AccountId32,
    take: u16,
) -> Result<TxSuccess>
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected Subtensor client |
| `signer` | `&Keypair` | Coldkey that owns the hotkey |
| `hotkey` | `AccountId32` | Delegate hotkey whose take is being increased |
| `take` | `u16` | New take value in basis points (0-10000). Must be greater than the current take |

### Returns

`Result<TxSuccess>` containing the block hash and extrinsic hash on success. Returns an error if the new take is not higher than the current take, or if the value exceeds 10000.

### Example

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::extrinsics::take;

async fn raise_delegate_take() -> Result<()> {
    let client = OnlineClient::from_url("wss://entrypoint-finney.opentensor.ai:443").await?;
    let signer = subxt_signer::sr25519::Keypair::from_uri("//MyColdkey")?;

    let hotkey: AccountId32 = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
        .parse()?;

    // Increase take from current value to 15% (1500 basis points)
    let new_take: u16 = 1500;

    let result = take::increase_take(&client, &signer, hotkey, new_take).await?;
    println!("Take increased to 15%, block: {}", result.block_hash);

    Ok(())
}
```

---

## decrease_take

Lower the take rate for a hotkey. The new take must be lower than the current take. This increases the share of emission distributed to delegators.

### Signature

```rust
pub async fn decrease_take(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    hotkey: AccountId32,
    take: u16,
) -> Result<TxSuccess>
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected Subtensor client |
| `signer` | `&Keypair` | Coldkey that owns the hotkey |
| `hotkey` | `AccountId32` | Delegate hotkey whose take is being decreased |
| `take` | `u16` | New take value in basis points (0-10000). Must be less than the current take |

### Returns

`Result<TxSuccess>` containing the block hash and extrinsic hash on success. Returns an error if the new take is not lower than the current take.

### Example

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::extrinsics::take;

async fn lower_delegate_take() -> Result<()> {
    let client = OnlineClient::from_url("wss://entrypoint-finney.opentensor.ai:443").await?;
    let signer = subxt_signer::sr25519::Keypair::from_uri("//MyColdkey")?;

    let hotkey: AccountId32 = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
        .parse()?;

    // Decrease take from current value to 5% (500 basis points)
    let new_take: u16 = 500;

    let result = take::decrease_take(&client, &signer, hotkey, new_take).await?;
    println!("Take decreased to 5%, block: {}", result.block_hash);

    Ok(())
}
```

---

## become_delegate

Register a hotkey as a delegate on the network with an initial take rate. Only coldkeys that own a registered hotkey can become delegates. Once a hotkey is a delegate, other coldkeys can stake to it and receive emission proportional to their stake minus the delegate's take.

### Signature

```rust
pub async fn become_delegate(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    hotkey: AccountId32,
    take: u16,
) -> Result<TxSuccess>
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected Subtensor client |
| `signer` | `&Keypair` | Coldkey that owns the hotkey |
| `hotkey` | `AccountId32` | Hotkey to register as a delegate |
| `take` | `u16` | Initial take value in basis points (0-10000) |

### Returns

`Result<TxSuccess>` containing the block hash and extrinsic hash on success. Returns an error if the hotkey is already a delegate, or if the hotkey is not registered on any subnet.

### Example

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::extrinsics::take;

async fn register_as_delegate() -> Result<()> {
    let client = OnlineClient::from_url("wss://entrypoint-finney.opentensor.ai:443").await?;
    let signer = subxt_signer::sr25519::Keypair::from_uri("//MyColdkey")?;

    let hotkey: AccountId32 = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
        .parse()?;

    // Become a delegate with a 10% take
    let initial_take: u16 = 1000;

    let result = take::become_delegate(&client, &signer, hotkey, initial_take).await?;
    println!("Registered as delegate with 10% take, block: {}", result.block_hash);

    Ok(())
}
```

### Delegation and Staking

After becoming a delegate, other coldkeys can stake to the delegate's hotkey using `staking::add_stake`. The delegate earns emission proportional to their total stake, and distributes the portion after their take to all delegators weighted by each delegator's stake proportion.

---

## Important Notes

### Directional Constraints

The chain enforces strict directionality on take changes:
- `increase_take` only accepts a value higher than the current take. Call it when you want to take more.
- `decrease_take` only accepts a value lower than the current take. Call it when you want to take less.
- There is no generic "set_take" function. This is an intentional design choice to prevent large, sudden swings in take rate that would harm delegators.

### Take Rate Transparency

Delegators can query a delegate's current take rate from the chain before staking. Sudden increases in take reduce delegator returns, so delegates should communicate planned take changes to their delegators in advance.

### Global Take

The take rate applies globally across all subnets where the delegate has stake. Changing the take rate on one subnet affects all subnets simultaneously.

### Only the Owner Can Change Take

Only the coldkey that owns the delegate hotkey can call `increase_take`, `decrease_take`, or `become_delegate`. Proxy accounts with the appropriate proxy type can also adjust take on behalf of the owner.

### Maximum and Minimum Values

The take rate is bounded between 0 and 10000 basis points. The effective minimum may be higher than 0 on some subnets. Check subnet-specific constraints before setting a very low take.
