# Proxy Extrinsics

Module path: `bittensor_chain::extrinsics::proxy`

Proxy extrinsics manage proxy relationships on the Bittensor chain. A proxy allows one account (the delegate) to perform certain actions on behalf of another account (the signer). This is useful for coldkey security: the coldkey can authorize a hotkey or another account to sign staking and transfer transactions without exposing the coldkey itself.

## Transaction Result

All proxy functions return `Result<TxSuccess>`. The `TxSuccess` struct:

```rust
pub struct TxSuccess {
    pub block_hash: H256,
    pub extrinsic_hash: H256,
}
```

- **`block_hash`**: The hash of the block that included the transaction.
- **`extrinsic_hash`**: The hash of the extrinsic, used for transaction tracking.

## Proxy Types

The `proxy_type` parameter (u8) defines what actions the delegate can perform. The standard Bittensor proxy types are:

| Value | Name | Allowed Actions |
|---|---|---|
| `0` | Any | All extrinsic calls on behalf of the signer |
| `1` | NonTransfer | All calls except balance transfers |
| `2` | Governance | Voting and governance operations |
| `3` | Staking | Staking and unstaking operations only |
| `4` | NonFungible | NFT operations |
| `5` | Senate | Senate voting operations |
| `6` | Subnet | Subnet management operations |

The `delay` parameter adds a time-lock in blocks. The proxy relationship only becomes active after the specified number of blocks have passed.

---

## add_proxy

Authorize a delegate account to act as a proxy for the signer. The delegate can then submit transactions on the signer's behalf within the scope of the specified proxy type.

### Signature

```rust
pub async fn add_proxy(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    delegate: AccountId32,
    proxy_type: u8,
    delay: u32,
) -> Result<TxSuccess>
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected Subtensor client |
| `signer` | `&Keypair` | Account granting proxy access (typically the coldkey) |
| `delegate` | `AccountId32` | Account receiving proxy authority |
| `proxy_type` | `u8` | Type of proxy relationship (see table above) |
| `delay` | `u32` | Number of blocks before the proxy becomes active. Use `0` for immediate activation |

### Returns

`Result<TxSuccess>` containing the block hash and extrinsic hash on success.

### Example

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::extrinsics::proxy;

async fn authorize_staking_proxy() -> Result<()> {
    let client = OnlineClient::from_url("wss://entrypoint-finney.opentensor.ai:443").await?;
    let signer = subxt_signer::sr25519::Keypair::from_uri("//MyColdkey")?;

    // Hotkey that will act as the staking proxy
    let delegate: AccountId32 = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
        .parse()?;

    // Grant staking-only proxy authority, effective immediately
    let proxy_type: u8 = 3; // Staking
    let delay: u32 = 0;

    let result = proxy::add_proxy(&client, &signer, delegate, proxy_type, delay).await?;
    println!("Staking proxy added, block: {}", result.block_hash);

    Ok(())
}
```

### Adding a Delayed Proxy

A delayed proxy provides a safety window. If the coldkey owner notices an unauthorized proxy addition, they can remove it before it becomes active.

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::extrinsics::proxy;

async fn add_delayed_proxy() -> Result<()> {
    let client = OnlineClient::from_url("wss://entrypoint-finney.opentensor.ai:443").await?;
    let signer = subxt_signer::sr25519::Keypair::from_uri("//MyColdkey")?;

    let delegate: AccountId32 = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
        .parse()?;

    let proxy_type: u8 = 1; // NonTransfer
    let delay: u32 = 100; // Active after 100 blocks (~20 minutes)

    let result = proxy::add_proxy(&client, &signer, delegate, proxy_type, delay).await?;
    println!(
        "Delayed proxy added. Active after block + {} blocks",
        delay
    );

    Ok(())
}
```

---

## remove_proxy

Revoke a proxy relationship. The delegate will no longer be able to submit transactions on behalf of the signer for the specified proxy type.

### Signature

```rust
pub async fn remove_proxy(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    delegate: AccountId32,
    proxy_type: u8,
    delay: u32,
) -> Result<TxSuccess>
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected Subtensor client |
| `signer` | `&Keypair` | Account revoking proxy access |
| `delegate` | `AccountId32` | Account whose proxy authority is being revoked |
| `proxy_type` | `u8` | The proxy type being revoked. Must match the originally added proxy type |
| `delay` | `u32` | The delay value of the proxy being revoked. Must match the originally added delay |

### Returns

`Result<TxSuccess>` containing the block hash and extrinsic hash on success.

### Example

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::extrinsics::proxy;

async fn revoke_staking_proxy() -> Result<()> {
    let client = OnlineClient::from_url("wss://entrypoint-finney.opentensor.ai:443").await?;
    let signer = subxt_signer::sr25519::Keypair::from_uri("//MyColdkey")?;

    let delegate: AccountId32 = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
        .parse()?;

    // Must match the original proxy type and delay
    let proxy_type: u8 = 3; // Staking
    let delay: u32 = 0;

    let result = proxy::remove_proxy(&client, &signer, delegate, proxy_type, delay).await?;
    println!("Staking proxy removed, block: {}", result.block_hash);

    Ok(())
}
```

### Exact Match Requirement

When removing a proxy, the `proxy_type` and `delay` values must exactly match the ones used when the proxy was added. If you attempt to remove a proxy with mismatched parameters, the transaction will fail. This is because a single signer can have multiple proxy relationships with the same delegate but different types or delays.

---

## Important Notes

### Coldkey Security

The primary use case for proxies is coldkey security. By adding a staking proxy, you can keep your coldkey offline and use the delegate (typically a hotkey) to handle day-to-day staking operations. If the delegate key is compromised, you can remove the proxy using the coldkey.

### Proxy Call Pattern

After adding a proxy, the delegate submits transactions using a proxy call wrapper. The chain verifies that the delegate is authorized for the specific call type before executing it. The actual signer of the inner call is the original account (the one that added the proxy), not the delegate.

```rust
// Pseudocode for submitting a proxied transaction:
// 1. Build the inner call (e.g., add_stake)
// 2. Wrap it in a proxy() call specifying the real signer and proxy type
// 3. Sign the outer call with the delegate keypair
```

### Multiple Proxies

A single account can add multiple delegates with different proxy types. For example, you might grant staking proxy authority to one key and governance proxy authority to another. Each relationship is independent.

### Revoking vs. Announcing

The `remove_proxy` function immediately revokes proxy authority. There is no delay on removal. If you want to warn the delegate before revoking, you can announce your intent off-chain before submitting the removal transaction.

### Transaction Fees

Both `add_proxy` and `remove_proxy` are regular extrinsics that require the signer to pay transaction fees. The fees come from the signer's free balance, not from the delegate's balance.
