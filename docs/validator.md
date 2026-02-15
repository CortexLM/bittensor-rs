# Validator Operations

This document describes validator-specific operations for interacting with the Bittensor network, including weight setting, staking, registration, and network serving.

## Weight Operations

### set_weights

Set validator weights for a subnet. This is the final step after commit-reveal.

```rust
pub async fn set_weights(
    client: &BittensorClient,
    signer: &PairSigner<DefaultConfig, Pair>,
    netuid: u16,
    uids: Vec<u16>,
    weights: Vec<u16>,
    version_key: u64
) -> Result<()>
```

**Parameters:**
- `client`: Bittensor client instance
- `signer`: Key pair signer
- `netuid`: Subnet ID
- `uids`: Vector of u16 UIDs
- `weights`: Vector of u16 weights (scaled by u16::MAX)

**Returns:**
- `Result<()>`: Success or error

**Example:**
```rust
use bittensor_rs::validator::set_weights;
use bittensor_rs::utils::normalize_weights;

let (uid_vals, weight_vals) = normalize_weights(&uids, &weights)?;
set_weights(&client, &signer, netuid, uid_vals, weight_vals, version_key).await?;
```

### commit_weights

Commit weights using a commit-reveal scheme.

```rust
pub async fn commit_weights(
    client: &BittensorClient,
    signer: &PairSigner<DefaultConfig, Pair>,
    netuid: u16,
    commitment: [u8; 32]
) -> Result<()>
```

**Parameters:**
- `client`: Bittensor client instance
- `signer`: Key pair signer
- `netuid`: Subnet ID
- `commitment`: 32-byte commitment hash

**Returns:**
- `Result<()>`: Success or error

**Example:**
```rust
use bittensor_rs::validator::commit_weights;
use bittensor_rs::utils::commit_weights_hash;

let commitment = commit_weights_hash(&uids, &weights, &salt)?;
commit_weights(&client, &signer, netuid, commitment).await?;
```

### reveal_weights

Reveal weights after committing.

```rust
pub async fn reveal_weights(
    client: &BittensorClient,
    signer: &PairSigner<DefaultConfig, Pair>,
    netuid: u16,
    uids: Vec<u16>,
    weights: Vec<u16>,
    salt: Vec<u16>
) -> Result<()>
```

**Parameters:**
- `client`: Bittensor client instance
- `signer`: Key pair signer
- `netuid`: Subnet ID
- `uids`: Vector of u16 UIDs
- `weights`: Vector of u16 weights
- `salt`: Salt used in commitment

**Returns:**
- `Result<()>`: Success or error

**Example:**
```rust
use bittensor_rs::validator::reveal_weights;

reveal_weights(&client, &signer, netuid, uid_vals, weight_vals, salt).await?;
```

## Mechanism Weights

### set_mechanism_weights

Set mechanism-specific weights.

```rust
pub async fn set_mechanism_weights(
    client: &BittensorClient,
    signer: &PairSigner<DefaultConfig, Pair>,
    netuid: u16,
    mechanism_id: u8,
    uids: Vec<u16>,
    weights: Vec<u16>
) -> Result<()>
```

### commit_mechanism_weights

Commit mechanism weights.

```rust
pub async fn commit_mechanism_weights(
    client: &BittensorClient,
    signer: &PairSigner<DefaultConfig, Pair>,
    netuid: u16,
    mechanism_id: u8,
    commitment: [u8; 32]
) -> Result<()>
```

### reveal_mechanism_weights

Reveal mechanism weights.

```rust
pub async fn reveal_mechanism_weights(
    client: &BittensorClient,
    signer: &PairSigner<DefaultConfig, Pair>,
    netuid: u16,
    mechanism_id: u8,
    uids: Vec<u16>,
    weights: Vec<u16>,
    salt: Vec<u16>
) -> Result<()>
```

## Staking Operations

### add_stake

Add stake to a neuron on a specific subnet.

```rust
pub async fn add_stake(
    client: &BittensorClient,
    signer: &PairSigner<DefaultConfig, Pair>,
    hotkey: &AccountId32,
    netuid: u16,
    amount: u128
) -> Result<()>
```

**Parameters:**
- `client`: Bittensor client instance
- `signer`: Key pair signer
- `hotkey`: Hotkey to stake to
- `netuid`: Subnet ID
- `amount`: Amount to stake in RAO

**Example:**
```rust
use bittensor_rs::validator::add_stake;
use bittensor_rs::utils::tao_to_rao;

let amount_rao = tao_to_rao(100.0); // 100 TAO
add_stake(&client, &signer, &hotkey, netuid, amount_rao).await?;
```

### unstake

Unstake from a neuron on a specific subnet.

```rust
pub async fn unstake(
    client: &BittensorClient,
    signer: &PairSigner<DefaultConfig, Pair>,
    hotkey: &AccountId32,
    netuid: u16,
    amount: u128
) -> Result<()>
```

### move_stake

Move stake between subnets.

```rust
pub async fn move_stake(
    client: &BittensorClient,
    signer: &PairSigner<DefaultConfig, Pair>,
    hotkey: &AccountId32,
    origin_netuid: u16,
    destination_netuid: u16,
    amount: u128
) -> Result<()>
```

### swap_stake

Swap stake between hotkeys.

```rust
pub async fn swap_stake(
    client: &BittensorClient,
    signer: &PairSigner<DefaultConfig, Pair>,
    hotkey: &AccountId32,
    origin_netuid: u16,
    destination_netuid: u16,
    amount: u128
) -> Result<()>
```

### transfer_stake

Transfer stake to another coldkey.

```rust
pub async fn transfer_stake(
    client: &BittensorClient,
    signer: &PairSigner<DefaultConfig, Pair>,
    destination_coldkey: &AccountId32,
    hotkey: &AccountId32,
    origin_netuid: u16,
    destination_netuid: u16,
    amount: u128
) -> Result<()>
```

### get_stake

Query stake amount for a coldkey-hotkey pair on a subnet.

```rust
pub async fn get_stake(
    client: &BittensorClient,
    coldkey: &AccountId32,
    hotkey: &AccountId32,
    netuid: u16
) -> Result<u128>
```

## Registration

### register

Register a neuron on a subnet.

```rust
pub async fn register(
    client: &BittensorClient,
    signer: &PairSigner<DefaultConfig, Pair>,
    netuid: u16
) -> Result<()>
```

**Example:**
```rust
use bittensor_rs::validator::register;

register(&client, &signer, 1).await?;
```

### is_registered

Check if a hotkey is registered on a subnet.

```rust
pub async fn is_registered(
    client: &BittensorClient,
    hotkey: &AccountId32,
    netuid: u16
) -> Result<bool>
```

## Network Serving

### serve_axon

Serve axon information (TCP).

```rust
pub async fn serve_axon(
    client: &BittensorClient,
    signer: &PairSigner<DefaultConfig, Pair>,
    netuid: u16,
    version: u32,
    ip: u128,
    port: u16,
    ip_type: u8
) -> Result<()>
```

**Parameters:**
- `version`: Protocol version
- `ip`: IP address encoded as u128
- `port`: Port number
- `ip_type`: 4 for IPv4, 6 for IPv6

### serve_axon_tls

Serve axon information with TLS.

```rust
pub async fn serve_axon_tls(
    client: &BittensorClient,
    signer: &PairSigner<DefaultConfig, Pair>,
    netuid: u16,
    version: u32,
    ip: u128,
    port: u16,
    ip_type: u8
) -> Result<()>
```

## Delegate Operations

### increase_take

Increase delegate take percentage.

```rust
pub async fn increase_take(
    client: &BittensorClient,
    signer: &PairSigner<DefaultConfig, Pair>,
    hotkey: &AccountId32,
    take: u16
) -> Result<()>
```

### decrease_take

Decrease delegate take percentage.

```rust
pub async fn decrease_take(
    client: &BittensorClient,
    signer: &PairSigner<DefaultConfig, Pair>,
    hotkey: &AccountId32,
    take: u16
) -> Result<()>
```

## Root Operations

Root subnet operations for managing validator permissions and network governance.

See the `root` module for detailed root operations.

## Child Operations

Operations for managing child subnets and relationships.

See the `children` module for detailed child subnet operations.

## Error Handling

All validator operations return `Result<()>` types. Handle errors appropriately:

```rust
match set_weights(&client, &signer, netuid, uid_vals, weight_vals, version_key).await {
    Ok(()) => println!("Weights set successfully"),
    Err(e) => eprintln!("Error setting weights: {}", e),
}
```

## Transaction Fees

All validator operations require transaction fees. Ensure sufficient balance before executing operations.

## Usage

Import validator operations from the main crate:

```rust
use bittensor_rs::validator::{
    set_weights, commit_weights, reveal_weights,
    add_stake, unstake, move_stake,
    register, is_registered,
    serve_axon, serve_axon_tls,
    increase_take, decrease_take
};
```

## Best Practices

1. **Weight Setting**: Use commit-reveal scheme to hide weights until reveal phase
2. **Staking**: Monitor stake distributions and ensure sufficient liquidity
3. **Registration**: Verify registration status before attempting operations
4. **Network Serving**: Keep axon information up-to-date for network connectivity
5. **Error Handling**: Always handle errors and check transaction status
6. **Transaction Fees**: Maintain sufficient balance for operations

