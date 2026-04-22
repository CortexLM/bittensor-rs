# Registration Extrinsics

Module path: `bittensor_chain::extrinsics::registration`

Registration extrinsics manage how hotkeys join subnets on the Bittensor network. Before a hotkey can stake, set weights, or serve on a subnet, it must be registered. There are three registration mechanisms depending on the subnet and the registrant's role.

## Transaction Result

All registration functions return `Result<TxSuccess>`. The `TxSuccess` struct:

```rust
pub struct TxSuccess {
    pub block_hash: H256,
    pub extrinsic_hash: H256,
}
```

- **`block_hash`**: The hash of the block that included the transaction.
- **`extrinsic_hash`**: The hash of the extrinsic, useful for tracking transaction status.

---

## register

Register the signer's hotkey on a subnet by solving a proof-of-work (PoW) challenge. This is the standard registration path for most subnets. The chain issues a difficulty target, and the client must find a valid nonce before submitting.

### Signature

```rust
pub async fn register(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    netuid: u16,
) -> Result<TxSuccess>
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected Subtensor client |
| `signer` | `&Keypair` | Hotkey to register on the subnet |
| `netuid` | `u16` | Subnet ID to register on |

### Returns

`Result<TxSuccess>` containing the block hash and extrinsic hash on success.

### Example

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::extrinsics::registration;

async fn register_on_subnet() -> Result<()> {
    let client = OnlineClient::from_url("wss://entrypoint-finney.opentensor.ai:443").await?;
    let signer = subxt_signer::sr25519::Keypair::from_uri("//MyHotkey")?;

    let netuid: u16 = 1;

    let result = registration::register(&client, &signer, netuid).await?;
    println!(
        "Registered on subnet {}, block: {}",
        netuid,
        result.block_hash
    );

    Ok(())
}
```

### How PoW Registration Works

The `register` function handles the full PoW cycle internally:

1. Query the current difficulty and block hash from the target subnet.
2. Compute a valid nonce that satisfies the difficulty target.
3. Submit the registration extrinsic with the valid nonce.

If the difficulty is too high for the available compute, the function will continue retrying until it finds a solution or times out. The default timeout is implementation-specific. For subnets with very high difficulty, `burned_register` may be a faster alternative.

---

## burned_register

Register a hotkey on a subnet by burning TAO instead of solving a proof-of-work challenge. The signer pays a burn cost determined by the subnet's current `burn_register_cost`. This is faster than PoW registration but costs TAO.

### Signature

```rust
pub async fn burned_register(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    netuid: u16,
    hotkey: AccountId32,
) -> Result<TxSuccess>
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected Subtensor client |
| `signer` | `&Keypair` | Coldkey paying the burn cost |
| `netuid` | `u16` | Subnet ID to register on |
| `hotkey` | `AccountId32` | Hotkey to register |

### Returns

`Result<TxSuccess>` containing the block hash and extrinsic hash on success.

### Example

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::extrinsics::registration;

async fn burn_register_on_subnet() -> Result<()> {
    let client = OnlineClient::from_url("wss://entrypoint-finney.opentensor.ai:443").await?;
    let signer = subxt_signer::sr25519::Keypair::from_uri("//MyColdkey")?;

    let hotkey: AccountId32 = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
        .parse()?;

    let netuid: u16 = 1;

    let result = registration::burned_register(&client, &signer, netuid, hotkey).await?;
    println!(
        "Burn-registered on subnet {}, block: {}",
        netuid,
        result.block_hash
    );

    Ok(())
}
```

### Burn Cost

The burn cost is not a parameter you pass. It is determined by the subnet's current registration cost, which adjusts based on demand. You can query the current burn cost before committing:

```rust
// Query current burn cost for a subnet (pseudocode, check chain query module for actual API)
let burn_cost = client
    .storage()
    .fetch(&subtensor::storage().subnet_burn_register_cost(netuid), None)
    .await?
    .unwrap_or_default();

println!("Current burn cost: {} RAO", burn_cost);
```

The signer's coldkey must have enough free balance to cover the burn cost plus the transaction fee.

---

## root_register

Register the signer on the root subnet (netuid 0). The root subnet is special: only senators and trusted accounts can register here. Root registrants can set root-level weights that influence global emission distribution across subnets.

### Signature

```rust
pub async fn root_register(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
) -> Result<TxSuccess>
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected Subtensor client |
| `signer` | `&Keypair` | Hotkey to register on the root subnet |

### Returns

`Result<TxSuccess>` containing the block hash and extrinsic hash on success.

### Example

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::extrinsics::registration;

async fn register_on_root() -> Result<()> {
    let client = OnlineClient::from_url("wss://entrypoint-finney.opentensor.ai:443").await?;
    let signer = subxt_signer::sr25519::Keypair::from_uri("//MyHotkey")?;

    let result = registration::root_register(&client, &signer).await?;
    println!("Registered on root subnet, block: {}", result.block_hash);

    Ok(())
}
```

### Access Control

Root registration is restricted. Attempting to register on the root subnet without authorization will result in a transaction failure. Only accounts that have been granted permission by the senate or meet the chain's root registration criteria can succeed.

---

## Important Notes

### Single Registration Per Subnet

A hotkey can only be registered once per subnet. Attempting to register a hotkey that is already registered on the target subnet will fail. Check registration status before calling:

```rust
// Query whether a hotkey is already registered on a subnet
let is_registered = client
    .storage()
    .fetch(&subtensor::storage().is_registered(netuid, &hotkey), None)
    .await?
    .unwrap_or(false);

if is_registered {
    println!("Already registered on subnet {}", netuid);
} else {
    let result = registration::register(&client, &signer, netuid).await?;
}
```

### Max Registrations Per Block

Each subnet has a maximum number of registrations allowed per block. During high-demand periods, PoW registration can become competitive, and you may need to retry across multiple blocks. Burned registration is not subject to the same per-block cap, making it more reliable under load.

### PoW Difficulty

PoW difficulty varies by subnet and adjusts over time based on registration demand. Subnets with high validator counts tend to have higher difficulty. If your hardware cannot solve the challenge in a reasonable time, consider using `burned_register` instead.

### Coldkey vs. Hotkey Signers

- `register`: The signer is the hotkey being registered. The hotkey signs the transaction.
- `burned_register`: The signer is the coldkey paying the burn cost. The hotkey is passed as a parameter.
- `root_register`: The signer is the hotkey being registered on the root subnet.

Make sure you use the correct keypair for each registration type.
