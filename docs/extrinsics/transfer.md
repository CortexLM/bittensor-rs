# Transfer Extrinsics

Module path: `bittensor_chain::extrinsics::transfer`

Transfer extrinsics handle moving free balance between accounts on the Bittensor chain. Unlike staking operations, transfers move TAO from one coldkey's free balance to another account's free balance, with no delegation or subnet involvement.

## Transaction Result

All transfer functions return `Result<TxSuccess>`. The `TxSuccess` struct:

```rust
pub struct TxSuccess {
    pub block_hash: H256,
    pub extrinsic_hash: H256,
}
```

- **`block_hash`**: The hash of the block that included the transaction.
- **`extrinsic_hash`**: The hash of the extrinsic, used for tracking transaction status.

---

## transfer

Transfer free balance from the signer's coldkey to a destination account.

This is a raw balance transfer. It does not interact with staking, delegation, or subnet state. The transferred amount must be available as free balance (not locked or staked).

### Signature

```rust
pub async fn transfer(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    dest: AccountId32,
    value: u64,
) -> Result<TxSuccess>
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected Subtensor client |
| `signer` | `&Keypair` | Coldkey sending the transfer |
| `dest` | `AccountId32` | Recipient account |
| `value` | `u64` | Amount in RAO to transfer |

### Returns

`Result<TxSuccess>` containing the block hash and extrinsic hash on success.

### Example

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::extrinsics::transfer;

async fn send_tao() -> Result<()> {
    let client = OnlineClient::from_url("wss://entrypoint-finney.opentensor.ai:443").await?;
    let signer = subxt_signer::sr25519::Keypair::from_uri("//Alice")?;

    let recipient: AccountId32 = "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92TjN5Yv7U2Z"
        .parse()?;

    // Transfer 10 TAO
    let amount = Balance::from_tao(10.0);

    let result = transfer::transfer(&client, &signer, recipient, amount).await?;
    println!("Transfer complete, extrinsic: {}", result.extrinsic_hash);

    Ok(())
}
```

### Transfer with Exact RAO Amount

When you need precision at the RAO level, pass the raw u64 value directly.

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::extrinsics::transfer;

async fn send_exact_rao() -> Result<()> {
    let client = OnlineClient::from_url("wss://entrypoint-finney.opentensor.ai:443").await?;
    let signer = subxt_signer::sr25519::Keypair::from_uri("//Alice")?;

    let recipient: AccountId32 = "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92TjN5Yv7U2Z"
        .parse()?;

    // Transfer exactly 500,000,000 RAO (0.5 TAO)
    let amount: u64 = 500_000_000;

    let result = transfer::transfer(&client, &signer, recipient, amount).await?;
    println!("Sent 0.5 TAO, block: {}", result.block_hash);

    Ok(())
}
```

---

## Amount Conversion Reference

All amounts in the transfer module use RAO (u64). 1 TAO = 1,000,000,000 RAO.

```rust
// Using the balance helper
let one_tao: u64 = Balance::from_tao(1.0);      // 1_000_000_000
let half_tao: u64 = Balance::from_tao(0.5);      // 500_000_000
let micro_tao: u64 = Balance::from_tao(0.000001); // 1_000

// Using the conversion function
let two_tao: u64 = tao_to_rao(2.0);  // 2_000_000_000
```

---

## Important Notes

### Transfer vs. Stake Transfer

The `transfer` function moves free balance between accounts. It does not move staked or delegated balance. To move stake between subnets or hotkeys, use the `staking` module functions (`move_stake`, `swap_stake`, `transfer_stake`).

### Existential Deposit

The Bittensor chain enforces an existential deposit. Accounts that fall below this threshold may be reaped (removed from state). Keep this in mind when transferring the entire balance from an account. The existential deposit value is determined by chain constants and can be queried at runtime.

### Transaction Fees

Transfers incur a transaction fee deducted from the signer's balance. The actual fee depends on the transaction size and current network conditions. The fee is paid in addition to the transfer amount. For example, transferring 1 TAO costs 1 TAO plus the fee.

### Transfer to Stake

A common pattern is to transfer TAO to a coldkey, then stake it to a hotkey:

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::extrinsics::transfer;
use bittensor_chain::extrinsics::staking;

async fn transfer_then_stake() -> Result<()> {
    let client = OnlineClient::from_url("wss://entrypoint-finney.opentensor.ai:443").await?;
    let signer = subxt_signer::sr25519::Keypair::from_uri("//Alice")?;

    let coldkey_b: AccountId32 = "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92TjN5Yv7U2Z".parse()?;
    let hotkey: AccountId32 = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".parse()?;

    let amount = Balance::from_tao(3.0);

    // Transfer TAO to coldkey B
    let tx_result = transfer::transfer(&client, &signer, coldkey_b, amount).await?;
    println!("Transferred to coldkey B, block: {}", tx_result.block_hash);

    // Note: coldkey B must sign the staking transaction itself.
    // The original signer cannot stake on behalf of coldkey B.

    Ok(())
}
```

### Nonce Handling for Rapid Transfers

If you need to send multiple transfers from the same account, each must use a unique nonce. The simplest approach is to await each transaction before submitting the next. For parallel transfers, you must track nonces manually or use a transaction queue.
