# Children Extrinsics

Module path: `bittensor_chain::extrinsics::children`

Children extrinsics manage the parent-child hotkey hierarchy within a subnet. A parent hotkey can designate child hotkeys and allocate a portion of its stake or emission to them. This enables subnet validators to distribute work across multiple miner nodes under a single parent identity.

## Transaction Result

All children functions return `Result<TxSuccess>`. The `TxSuccess` struct:

```rust
pub struct TxSuccess {
    pub block_hash: H256,
    pub extrinsic_hash: H256,
}
```

- **`block_hash`**: The hash of the block that included the transaction.
- **`extrinsic_hash`**: The hash of the extrinsic, used for transaction tracking.

---

## set_children

Assign child hotkeys to a parent hotkey on a specific subnet, along with proportional weight allocations. Each child is paired with a weight (u64) that determines its share relative to other children. The parent retains the remaining share.

### Signature

```rust
pub async fn set_children(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    hotkey: AccountId32,
    netuid: u16,
    children: Vec<(u64, AccountId32)>,
) -> Result<TxSuccess>
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected Subtensor client |
| `signer` | `&Keypair` | Coldkey that owns the parent hotkey |
| `hotkey` | `AccountId32` | Parent hotkey that will have children assigned |
| `netuid` | `u16` | Subnet ID where the parent-child relationship is established |
| `children` | `Vec<(u64, AccountId32)>` | List of (weight, child_hotkey) tuples. The weight determines the child's proportional share |

### Returns

`Result<TxSuccess>` containing the block hash and extrinsic hash on success.

### Example

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::extrinsics::children;

async fn assign_child_miners() -> Result<()> {
    let client = OnlineClient::from_url("wss://entrypoint-finney.opentensor.ai:443").await?;
    let signer = subxt_signer::sr25519::Keypair::from_uri("//MyColdkey")?;

    let parent_hotkey: AccountId32 = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
        .parse()?;

    let child_a: AccountId32 = "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92TjN5Yv7U2Z".parse()?;
    let child_b: AccountId32 = "5DAAnrj7VHTznn2AWBemMuyBwZWsQFhQnj2qF1nL7dM5LP2a".parse()?;

    let netuid: u16 = 1;

    // Assign children with proportional weights.
    // child_a gets weight 60, child_b gets weight 40.
    // The parent keeps the remainder of its own weight.
    let children = vec![
        (60u64, child_a),
        (40u64, child_b),
    ];

    let result = children::set_children(
        &client,
        &signer,
        parent_hotkey,
        netuid,
        children,
    ).await?;

    println!("Children assigned on subnet {}, block: {}", netuid, result.block_hash);

    Ok(())
}
```

### Weight Distribution

Children weights are relative. If you assign weights `[60, 40]` to two children, child A receives 60% of the child allocation and child B receives 40%. The parent's own weight is separate and controlled by the parent's own stake and emission.

### Overwriting Children

Calling `set_children` replaces the entire children list for that parent on the given subnet. There is no append or remove operation. To add a child, include all existing children plus the new one in the vector. To remove a child, omit it from the vector.

```rust
// To add a third child while keeping the first two:
let updated_children = vec![
    (60u64, child_a),
    (40u64, child_b),
    (20u64, child_c),  // new child
];
children::set_children(&client, &signer, parent_hotkey, netuid, updated_children).await?;

// To remove child_b:
let remaining_children = vec![
    (60u64, child_a),
    (20u64, child_c),
];
children::set_children(&client, &signer, parent_hotkey, netuid, remaining_children).await?;
```

---

## set_childkey_take

Set the take rate for a child hotkey on a specific subnet. This controls what percentage of the child's emission the child keeps versus what flows up to the parent.

### Signature

```rust
pub async fn set_childkey_take(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    hotkey: AccountId32,
    netuid: u16,
    take: u16,
) -> Result<TxSuccess>
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected Subtensor client |
| `signer` | `&Keypair` | Coldkey that owns the parent hotkey |
| `hotkey` | `AccountId32` | Child hotkey whose take is being set |
| `netuid` | `u16` | Subnet ID where the child's take applies |
| `take` | `u16` | Take rate in basis points (0-10000). A take of 1000 means the child keeps 10% and 90% flows to the parent |

### Returns

`Result<TxSuccess>` containing the block hash and extrinsic hash on success.

### Example

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::extrinsics::children;

async fn set_child_take_rate() -> Result<()> {
    let client = OnlineClient::from_url("wss://entrypoint-finney.opentensor.ai:443").await?;
    let signer = subxt_signer::sr25519::Keypair::from_uri("//MyColdkey")?;

    let child_hotkey: AccountId32 = "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92TjN5Yv7U2Z"
        .parse()?;

    let netuid: u16 = 1;

    // Set child take to 20% (2000 basis points).
    // The child keeps 20% of its emission, 80% goes to the parent.
    let take: u16 = 2000;

    let result = children::set_childkey_take(
        &client,
        &signer,
        child_hotkey,
        netuid,
        take,
    ).await?;

    println!("Child take set to 20%, block: {}", result.block_hash);

    Ok(())
}
```

### Child Take vs. Delegate Take

Child take and delegate take serve different purposes:
- **Delegate take** (see the `take` module) controls what a delegate keeps from delegator emission.
- **Child take** controls what a child hotkey keeps from its own emission, with the remainder flowing to the parent.

A typical pattern is to set a lower child take so that most emission flows up to the parent, who then distributes it according to the parent's own delegate take structure.

---

## Important Notes

### Parent Ownership

Only the coldkey that owns the parent hotkey can set children or modify childkey take. The child hotkey's owner cannot change these settings.

### Child Registration

A child hotkey must be registered on the same subnet as the parent before it can be assigned as a child. Attempting to assign an unregistered hotkey will cause the transaction to fail.

### Children Limit

Each subnet may impose a maximum number of children per parent. The exact limit depends on the subnet configuration. Exceeding this limit will cause the transaction to fail. Check the subnet's `max_children` parameter before adding children.

### Nonce and Ordering

When setting children and then immediately setting childkey take for the same child, ensure transactions are submitted sequentially or with correct nonces. The parent coldkey signs both transactions, so rapid sequential calls require proper nonce handling.

### Child Weights Are Not Stake

The weight parameter in `set_children` determines emission share, not stake. Children do not automatically receive stake from the parent. You must separately stake to child hotkeys using `staking::add_stake` if you want them to have stake on the subnet.
