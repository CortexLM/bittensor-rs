use crate::chain::{BittensorClient, BittensorSigner, ExtrinsicWait};
use anyhow::Result;
use parity_scale_codec::Encode;
use sp_core::crypto::AccountId32;
use subxt::dynamic::Value;

const SUBTENSOR_MODULE: &str = "SubtensorModule";
const ADD_STAKE_FUNCTION: &str = "add_stake";
const UNSTAKE_FUNCTION: &str = "remove_stake";

/// Add stake to a hotkey on a specific subnet
/// Subtensor expects: (hotkey, netuid, amount_staked)
pub async fn add_stake(
    client: &BittensorClient,
    signer: &BittensorSigner,
    hotkey: &AccountId32,
    netuid: u16,
    amount: u128,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args = vec![
        Value::from_bytes(hotkey.encode()),
        Value::u128(netuid as u128),
        Value::u128(amount),
    ];

    let tx_hash = client
        .submit_extrinsic(SUBTENSOR_MODULE, ADD_STAKE_FUNCTION, args, signer, wait_for)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to add stake: {}", e))?;

    Ok(tx_hash)
}

/// Unstake from a hotkey on a specific subnet
/// Subtensor expects: (hotkey, netuid, amount_unstaked)
pub async fn unstake(
    client: &BittensorClient,
    signer: &BittensorSigner,
    hotkey: &AccountId32,
    netuid: u16,
    amount: u128,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args = vec![
        Value::from_bytes(hotkey.encode()),
        Value::u128(netuid as u128),
        Value::u128(amount),
    ];

    let tx_hash = client
        .submit_extrinsic(SUBTENSOR_MODULE, UNSTAKE_FUNCTION, args, signer, wait_for)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to unstake: {}", e))?;

    Ok(tx_hash)
}

/// Unstake all from a hotkey
pub async fn unstake_all(
    client: &BittensorClient,
    signer: &BittensorSigner,
    hotkey: &AccountId32,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args = vec![Value::from_bytes(hotkey.encode())];

    client
        .submit_extrinsic(SUBTENSOR_MODULE, "remove_stake_all", args, signer, wait_for)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to unstake all: {}", e))
}

/// Add stake to multiple hotkeys
pub async fn add_stake_multiple(
    client: &BittensorClient,
    signer: &BittensorSigner,
    hotkeys: &[AccountId32],
    amounts: &[u128],
    wait_for: ExtrinsicWait,
) -> Result<String> {
    if hotkeys.len() != amounts.len() {
        return Err(anyhow::anyhow!(
            "Hotkeys and amounts must have the same length"
        ));
    }

    let hotkey_values: Vec<Value> = hotkeys
        .iter()
        .map(|hk| Value::from_bytes(hk.encode()))
        .collect();

    let amount_values: Vec<Value> = amounts.iter().map(|amt| Value::u128(*amt)).collect();

    let args = vec![
        Value::unnamed_composite(hotkey_values),
        Value::unnamed_composite(amount_values),
    ];

    client
        .submit_extrinsic(
            SUBTENSOR_MODULE,
            "add_stake_multiple",
            args,
            signer,
            wait_for,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to add stake multiple: {}", e))
}

/// Unstake from multiple hotkeys
pub async fn unstake_multiple(
    client: &BittensorClient,
    signer: &BittensorSigner,
    hotkeys: &[AccountId32],
    amounts: &[u128],
    wait_for: ExtrinsicWait,
) -> Result<String> {
    if hotkeys.len() != amounts.len() {
        return Err(anyhow::anyhow!(
            "Hotkeys and amounts must have the same length"
        ));
    }

    let hotkey_values: Vec<Value> = hotkeys
        .iter()
        .map(|hk| Value::from_bytes(hk.encode()))
        .collect();

    let amount_values: Vec<Value> = amounts.iter().map(|amt| Value::u128(*amt)).collect();

    let args = vec![
        Value::unnamed_composite(hotkey_values),
        Value::unnamed_composite(amount_values),
    ];

    client
        .submit_extrinsic(
            SUBTENSOR_MODULE,
            "remove_stake_multiple",
            args,
            signer,
            wait_for,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to unstake multiple: {}", e))
}

/// Set auto-stake for a hotkey
pub async fn set_auto_stake(
    client: &BittensorClient,
    signer: &BittensorSigner,
    hotkey: &AccountId32,
    auto_stake: bool,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args = vec![Value::from_bytes(hotkey.encode()), Value::bool(auto_stake)];

    client
        .submit_extrinsic(SUBTENSOR_MODULE, "set_auto_stake", args, signer, wait_for)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to set auto stake: {}", e))
}

/// Move stake from one hotkey to another across subnets
/// Subtensor expects: (origin_hotkey, destination_hotkey, origin_netuid, destination_netuid, alpha_amount)
#[allow(clippy::too_many_arguments)]
pub async fn move_stake(
    client: &BittensorClient,
    signer: &BittensorSigner,
    from_hotkey: &AccountId32,
    to_hotkey: &AccountId32,
    origin_netuid: u16,
    destination_netuid: u16,
    amount: u128,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args = vec![
        Value::from_bytes(from_hotkey.encode()),
        Value::from_bytes(to_hotkey.encode()),
        Value::u128(origin_netuid as u128),
        Value::u128(destination_netuid as u128),
        Value::u128(amount),
    ];

    client
        .submit_extrinsic(SUBTENSOR_MODULE, "move_stake", args, signer, wait_for)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to move stake: {}", e))
}

/// Swap stake from one subnet to another for a hotkey
/// Subtensor expects: (hotkey, origin_netuid, destination_netuid, alpha_amount)
pub async fn swap_stake(
    client: &BittensorClient,
    signer: &BittensorSigner,
    hotkey: &AccountId32,
    origin_netuid: u16,
    destination_netuid: u16,
    amount: u128,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args = vec![
        Value::from_bytes(hotkey.encode()),
        Value::u128(origin_netuid as u128),
        Value::u128(destination_netuid as u128),
        Value::u128(amount),
    ];

    client
        .submit_extrinsic(SUBTENSOR_MODULE, "swap_stake", args, signer, wait_for)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to swap stake: {}", e))
}
