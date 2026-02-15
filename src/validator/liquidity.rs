use crate::chain::{BittensorClient, BittensorSigner, ExtrinsicWait};
use crate::utils::balance_newtypes::Rao;
use anyhow::Result;
use subxt::dynamic::Value;

const SUBTENSOR_MODULE: &str = "SubtensorModule";

/// Add liquidity to a pool
pub async fn add_liquidity(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    amount_a: Rao,
    amount_b: Rao,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args = vec![
        Value::from(netuid),
        Value::u128(amount_a.as_u128()),
        Value::u128(amount_b.as_u128()),
    ];

    client
        .submit_extrinsic(SUBTENSOR_MODULE, "add_liquidity", args, signer, wait_for)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to add liquidity: {}", e))
}

/// Remove liquidity from a pool
pub async fn remove_liquidity(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    liquidity_amount: Rao,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args = vec![Value::from(netuid), Value::u128(liquidity_amount.as_u128())];

    client
        .submit_extrinsic(SUBTENSOR_MODULE, "remove_liquidity", args, signer, wait_for)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to remove liquidity: {}", e))
}

/// Modify liquidity position
pub async fn modify_liquidity(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    tick_lower: i32,
    tick_upper: i32,
    liquidity_delta: i128,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args = vec![
        Value::from(netuid),
        Value::i128(tick_lower as i128),
        Value::i128(tick_upper as i128),
        Value::i128(liquidity_delta),
    ];

    client
        .submit_extrinsic(SUBTENSOR_MODULE, "modify_liquidity", args, signer, wait_for)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to modify liquidity: {}", e))
}

/// Toggle user liquidity permission
pub async fn toggle_user_liquidity(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    enabled: bool,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args = vec![Value::from(netuid), Value::bool(enabled)];

    client
        .submit_extrinsic(
            SUBTENSOR_MODULE,
            "toggle_user_liquidity",
            args,
            signer,
            wait_for,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to toggle user liquidity: {}", e))
}
