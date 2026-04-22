# Staking Extrinsics

Module path: `bittensor_chain::extrinsics::staking`

Staking extrinsics manage how coldkeys allocate stake to hotkeys across subnets. All amounts are expressed in RAO, where 1 TAO equals 1,000,000,000 RAO.

## Transaction Result

All staking functions return `Result<TxSuccess>` (or `Result<Vec<TxSuccess>>` for batch operations). The `TxSuccess` struct is defined as:

```rust
pub struct TxSuccess {
    pub block_hash: H256,
    pub extrinsic_hash: H256,
}
```

- **`block_hash`**: The hash of the block that included the transaction.
- **`extrinsic_hash`**: The hash of the extrinsic itself, useful for tracking transaction status.

If the transaction fails to be included (insufficient balance, invalid state, rate limit hit), the function returns an error.

## The Submit and Watch Pattern

Every extrinsic in this module follows the same three-step pattern:

1. Build a typed call via the Subtensor runtime API.
2. Sign the call with an sr25519 keypair.
3. Submit and watch for inclusion on chain.

The high-level wrapper functions in this module handle all three steps internally. You pass the client, signer, and parameters, and get back a `Result<TxSuccess>`.

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::extrinsics::staking;

// Connect to the network
let client = subxt::OnlineClient::from_url("wss://entrypoint-finney.opentensor.ai:443").await?;

// Create a signer from a URI string (development only)
let signer = subxt_signer::sr25519::Keypair::from_uri("//Alice")?;

// Or from wallet keypair bytes:
// let signer = subxt_signer::sr25519::Keypair::from_bytes(&wallet_keypair_bytes)?;

// Call the extrinsic
let result = staking::add_stake(&client, &signer, hotkey, netuid, amount).await?;
println!("Included in block: {}", result.block_hash);
```

## Amount Conversion

All stake amounts use RAO (u64). Convert TAO to RAO with `Balance::from_tao()` or the helper `tao_to_rao()`:

```rust
// 1.0 TAO in RAO
let one_tao: u64 = Balance::from_tao(1.0); // 1_000_000_000

// Or using the conversion function
let one_tao: u64 = tao_to_rao(1.0);

// 0.5 TAO
let half_tao: u64 = Balance::from_tao(0.5); // 500_000_000
```

---

## add_stake

Stake TAO from the signer's coldkey to a hotkey on a specific subnet.

### Signature

```rust
pub async fn add_stake(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    hotkey: AccountId32,
    netuid: u16,
    amount_staked: u64,
) -> Result<TxSuccess>
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected Subtensor client |
| `signer` | `&Keypair` | Coldkey that owns the stake |
| `hotkey` | `AccountId32` | Hotkey to stake to |
| `netuid` | `u16` | Subnet ID to stake on |
| `amount_staked` | `u64` | Amount in RAO to stake |

### Returns

`Result<TxSuccess>` containing the block hash and extrinsic hash on success.

### Example

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::extrinsics::staking;

async fn stake_to_validator() -> Result<()> {
    let client = OnlineClient::from_url("wss://entrypoint-finney.opentensor.ai:443").await?;
    let signer = subxt_signer::sr25519::Keypair::from_uri("//Alice")?;

    let hotkey: AccountId32 = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
        .parse()?;

    let netuid: u16 = 1;
    let amount = Balance::from_tao(1.0); // 1 TAO in RAO

    let result = staking::add_stake(&client, &signer, hotkey, netuid, amount).await?;
    println!("Staked 1 TAO, extrinsic: {}", result.extrinsic_hash);

    Ok(())
}
```

---

## add_stake_multiple

Stake TAO to multiple hotkeys on the same subnet in a single transaction. Each hotkey receives a corresponding amount from the `amounts` vector.

### Signature

```rust
pub async fn add_stake_multiple(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    hotkeys: Vec<AccountId32>,
    netuid: u16,
    amounts: Vec<u64>,
) -> Result<Vec<TxSuccess>>
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected Subtensor client |
| `signer` | `&Keypair` | Coldkey that owns the stake |
| `hotkeys` | `Vec<AccountId32>` | List of hotkeys to stake to |
| `netuid` | `u16` | Subnet ID to stake on |
| `amounts` | `Vec<u64>` | Amounts in RAO for each hotkey. Must be same length as `hotkeys` |

### Returns

`Result<Vec<TxSuccess>>` with one entry per hotkey. Returns an error if `hotkeys` and `amounts` have different lengths.

### Example

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::extrinsics::staking;

async fn stake_to_multiple() -> Result<()> {
    let client = OnlineClient::from_url("wss://entrypoint-finney.opentensor.ai:443").await?;
    let signer = subxt_signer::sr25519::Keypair::from_uri("//Alice")?;

    let hotkey_a: AccountId32 = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".parse()?;
    let hotkey_b: AccountId32 = "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92TjN5Yv7U2Z".parse()?;

    let netuid: u16 = 1;
    let amounts = vec![Balance::from_tao(0.5), Balance::from_tao(0.5)];

    let results = staking::add_stake_multiple(
        &client,
        &signer,
        vec![hotkey_a, hotkey_b],
        netuid,
        amounts,
    ).await?;

    for (i, result) in results.iter().enumerate() {
        println!("Hotkey {} staked, extrinsic: {}", i, result.extrinsic_hash);
    }

    Ok(())
}
```

---

## remove_stake

Unstake TAO from a hotkey on a specific subnet, returning it to the signer's coldkey balance.

### Signature

```rust
pub async fn remove_stake(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    hotkey: AccountId32,
    netuid: u16,
    amount_unstaked: u64,
) -> Result<TxSuccess>
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected Subtensor client |
| `signer` | `&Keypair` | Coldkey that owns the stake |
| `hotkey` | `AccountId32` | Hotkey to unstake from |
| `netuid` | `u16` | Subnet ID to unstake from |
| `amount_unstaked` | `u64` | Amount in RAO to unstake |

### Returns

`Result<TxSuccess>` containing the block hash and extrinsic hash on success.

### Example

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::extrinsics::staking;

async fn unstake_from_hotkey() -> Result<()> {
    let client = OnlineClient::from_url("wss://entrypoint-finney.opentensor.ai:443").await?;
    let signer = subxt_signer::sr25519::Keypair::from_uri("//Alice")?;

    let hotkey: AccountId32 = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".parse()?;
    let netuid: u16 = 1;
    let amount = Balance::from_tao(0.5);

    let result = staking::remove_stake(&client, &signer, hotkey, netuid, amount).await?;
    println!("Unstaked 0.5 TAO, block: {}", result.block_hash);

    Ok(())
}
```

---

## unstake_all

Remove all stake from a hotkey across every subnet. This iterates through all subnets where the coldkey has active stake under the given hotkey and removes it entirely.

### Signature

```rust
pub async fn unstake_all(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    hotkey: AccountId32,
) -> Result<TxSuccess>
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected Subtensor client |
| `signer` | `&Keypair` | Coldkey that owns the stake |
| `hotkey` | `AccountId32` | Hotkey to fully unstake from |

### Returns

`Result<TxSuccess>` containing the block hash and extrinsic hash on success.

### Example

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::extrinsics::staking;

async fn unstake_everything() -> Result<()> {
    let client = OnlineClient::from_url("wss://entrypoint-finney.opentensor.ai:443").await?;
    let signer = subxt_signer::sr25519::Keypair::from_uri("//Alice")?;

    let hotkey: AccountId32 = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".parse()?;

    let result = staking::unstake_all(&client, &signer, hotkey).await?;
    println!("All stake removed, block: {}", result.block_hash);

    Ok(())
}
```

---

## unstake_multiple

Unstake from multiple hotkeys on the same subnet in a single transaction. Each hotkey has a corresponding amount in the `amounts` vector.

### Signature

```rust
pub async fn unstake_multiple(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    hotkeys: Vec<AccountId32>,
    netuid: u16,
    amounts: Vec<u64>,
) -> Result<Vec<TxSuccess>>
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected Subtensor client |
| `signer` | `&Keypair` | Coldkey that owns the stake |
| `hotkeys` | `Vec<AccountId32>` | List of hotkeys to unstake from |
| `netuid` | `u16` | Subnet ID to unstake from |
| `amounts` | `Vec<u64>` | Amounts in RAO for each hotkey. Must be same length as `hotkeys` |

### Returns

`Result<Vec<TxSuccess>>` with one entry per hotkey. Returns an error if `hotkeys` and `amounts` have different lengths.

### Example

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::extrinsics::staking;

async fn unstake_from_several() -> Result<()> {
    let client = OnlineClient::from_url("wss://entrypoint-finney.opentensor.ai:443").await?;
    let signer = subxt_signer::sr25519::Keypair::from_uri("//Alice")?;

    let hotkey_a: AccountId32 = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".parse()?;
    let hotkey_b: AccountId32 = "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92TjN5Yv7U2Z".parse()?;

    let netuid: u16 = 1;
    let amounts = vec![Balance::from_tao(0.3), Balance::from_tao(0.7)];

    let results = staking::unstake_multiple(
        &client,
        &signer,
        vec![hotkey_a, hotkey_b],
        netuid,
        amounts,
    ).await?;

    println!("Unstaked from {} hotkeys", results.len());

    Ok(())
}
```

---

## move_stake

Move stake from one hotkey to another, possibly across different subnets. The origin coldkey must own stake under the origin hotkey. The destination hotkey must already be registered on the destination subnet.

### Signature

```rust
pub async fn move_stake(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    origin_hotkey: AccountId32,
    destination_hotkey: AccountId32,
    origin_netuid: u16,
    destination_netuid: u16,
    amount: u64,
) -> Result<TxSuccess>
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected Subtensor client |
| `signer` | `&Keypair` | Coldkey that owns the stake |
| `origin_hotkey` | `AccountId32` | Hotkey to move stake from |
| `destination_hotkey` | `AccountId32` | Hotkey to move stake to |
| `origin_netuid` | `u16` | Subnet ID to move stake from |
| `destination_netuid` | `u16` | Subnet ID to move stake to |
| `amount` | `u64` | Amount in RAO to move |

### Returns

`Result<TxSuccess>` containing the block hash and extrinsic hash on success.

### Example

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::extrinsics::staking;

async fn move_stake_between_hotkeys() -> Result<()> {
    let client = OnlineClient::from_url("wss://entrypoint-finney.opentensor.ai:443").await?;
    let signer = subxt_signer::sr25519::Keypair::from_uri("//Alice")?;

    let origin_hotkey: AccountId32 = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".parse()?;
    let dest_hotkey: AccountId32 = "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92TjN5Yv7U2Z".parse()?;

    let result = staking::move_stake(
        &client,
        &signer,
        origin_hotkey,
        dest_hotkey,
        1,   // origin subnet
        3,   // destination subnet
        Balance::from_tao(2.0),
    ).await?;

    println!("Moved 2 TAO, block: {}", result.block_hash);

    Ok(())
}
```

---

## swap_stake

Swap stake between two subnets under the same hotkey. This is a convenience method that moves stake from one subnet to another without changing the hotkey.

### Signature

```rust
pub async fn swap_stake(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    hotkey: AccountId32,
    origin_netuid: u16,
    destination_netuid: u16,
    amount: u64,
) -> Result<TxSuccess>
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected Subtensor client |
| `signer` | `&Keypair` | Coldkey that owns the stake |
| `hotkey` | `AccountId32` | Hotkey holding the stake (same for origin and destination) |
| `origin_netuid` | `u16` | Subnet ID to move stake from |
| `destination_netuid` | `u16` | Subnet ID to move stake to |
| `amount` | `u64` | Amount in RAO to swap |

### Returns

`Result<TxSuccess>` containing the block hash and extrinsic hash on success.

### Example

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::extrinsics::staking;

async fn swap_between_subnets() -> Result<()> {
    let client = OnlineClient::from_url("wss://entrypoint-finney.opentensor.ai:443").await?;
    let signer = subxt_signer::sr25519::Keypair::from_uri("//Alice")?;

    let hotkey: AccountId32 = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".parse()?;

    let result = staking::swap_stake(
        &client,
        &signer,
        hotkey,
        1,   // from subnet 1
        5,   // to subnet 5
        Balance::from_tao(1.5),
    ).await?;

    println!("Swapped 1.5 TAO between subnets, block: {}", result.block_hash);

    Ok(())
}
```

---

## transfer_stake

Transfer stake from the signer's coldkey to another coldkey, while keeping it delegated to the same hotkey. This is used to transfer ownership of delegated stake.

### Signature

```rust
pub async fn transfer_stake(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    destination_coldkey: AccountId32,
    hotkey: AccountId32,
    origin_netuid: u16,
    destination_netuid: u16,
    amount: u64,
) -> Result<TxSuccess>
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected Subtensor client |
| `signer` | `&Keypair` | Coldkey that currently owns the stake |
| `destination_coldkey` | `AccountId32` | Coldkey to transfer ownership to |
| `hotkey` | `AccountId32` | Hotkey the stake is delegated to |
| `origin_netuid` | `u16` | Subnet ID the stake currently sits on |
| `destination_netuid` | `u16` | Subnet ID to transfer the stake to |
| `amount` | `u64` | Amount in RAO to transfer |

### Returns

`Result<TxSuccess>` containing the block hash and extrinsic hash on success.

### Example

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::extrinsics::staking;

async fn transfer_stake_to_another_coldkey() -> Result<()> {
    let client = OnlineClient::from_url("wss://entrypoint-finney.opentensor.ai:443").await?;
    let signer = subxt_signer::sr25519::Keypair::from_uri("//Alice")?;

    let dest_coldkey: AccountId32 = "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92TjN5Yv7U2Z".parse()?;
    let hotkey: AccountId32 = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".parse()?;

    let result = staking::transfer_stake(
        &client,
        &signer,
        dest_coldkey,
        hotkey,
        1,   // origin subnet
        3,   // destination subnet
        Balance::from_tao(0.5),
    ).await?;

    println!("Transferred 0.5 TAO stake, block: {}", result.block_hash);

    Ok(())
}
```

---

## Common Patterns

### Stake to a Validator

The most frequent operation: delegate TAO to a validator's hotkey on a specific subnet.

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::extrinsics::staking;

async fn delegate_to_validator() -> Result<()> {
    let client = OnlineClient::from_url("wss://entrypoint-finney.opentensor.ai:443").await?;

    // Load coldkey from file or URI
    let signer = subxt_signer::sr25519::Keypair::from_uri("//MyColdkey")?;

    // Validator hotkey (obtained from chain query or directory)
    let validator_hotkey: AccountId32 = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
        .parse()?;

    let netuid: u16 = 1;
    let amount = Balance::from_tao(5.0);

    let result = staking::add_stake(&client, &signer, validator_hotkey, netuid, amount).await?;
    println!("Delegated 5 TAO to validator. Extrinsic: {}", result.extrinsic_hash);

    Ok(())
}
```

### Unstake and Transfer

Unstake from a hotkey, then transfer the freed balance to another account.

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::extrinsics::staking;
use bittensor_chain::extrinsics::transfer;

async fn unstake_and_transfer() -> Result<()> {
    let client = OnlineClient::from_url("wss://entrypoint-finney.opentensor.ai:443").await?;
    let signer = subxt_signer::sr25519::Keypair::from_uri("//Alice")?;

    let hotkey: AccountId32 = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".parse()?;
    let recipient: AccountId32 = "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92TjN5Yv7U2Z".parse()?;
    let netuid: u16 = 1;
    let amount = Balance::from_tao(2.0);

    // Step 1: Unstake from the hotkey
    let unstake_result = staking::remove_stake(
        &client, &signer, hotkey.clone(), netuid, amount,
    ).await?;
    println!("Unstaked, block: {}", unstake_result.block_hash);

    // Step 2: Transfer the freed balance
    let transfer_result = transfer::transfer(
        &client, &signer, recipient, amount,
    ).await?;
    println!("Transferred, block: {}", transfer_result.block_hash);

    Ok(())
}
```

### Move Stake Between Subnets

Reallocate stake from one subnet to another without withdrawing to free balance.

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::extrinsics::staking;

async fn reallocate_between_subnets() -> Result<()> {
    let client = OnlineClient::from_url("wss://entrypoint-finney.opentensor.ai:443").await?;
    let signer = subxt_signer::sr25519::Keypair::from_uri("//Alice")?;

    let hotkey: AccountId32 = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".parse()?;

    // Same hotkey, different subnets
    let result = staking::swap_stake(
        &client,
        &signer,
        hotkey,
        1,   // leave subnet 1
        7,   // enter subnet 7
        Balance::from_tao(10.0),
    ).await?;

    println!("Moved 10 TAO from subnet 1 to subnet 7, block: {}", result.block_hash);

    Ok(())
}
```

---

## Important Notes

### Transaction Fees

Every extrinsic requires a transaction fee paid from the signer's free balance. The fee depends on transaction size and current chain congestion. Ensure the signer's coldkey has enough free balance to cover both the stake amount and the fee. Fees are typically a fraction of a TAO but can spike during network congestion.

### Nonce Management

The Subtensor node tracks account nonces. If you submit multiple transactions from the same coldkey in rapid succession, each must use the correct sequential nonce. The `submit_and_watch` pattern handles this automatically by waiting for inclusion before returning. For concurrent submissions from the same account, you must manually manage nonces or use a mortality-aware signer.

### Rate Limits Per Subnet

Each subnet enforces its own rate limits on staking and unstaking operations. Submitting too many stake or unstake calls in a short window for the same subnet can result in the transaction being rejected. The typical cooldown between staking operations on the same subnet is one block (roughly 12 seconds on Finney). Batch operations like `add_stake_multiple` count as a single operation for rate limit purposes.

### Minimum Stake Amounts

Most subnets enforce a minimum stake amount. Attempting to stake or unstake below this threshold will cause the transaction to fail. Check the subnet's `min_stake` value via chain queries before submitting.
