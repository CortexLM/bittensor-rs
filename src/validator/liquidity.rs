use crate::chain::{BittensorClient, BittensorSigner, ExtrinsicWait};
use crate::utils::balance_newtypes::Rao;
use anyhow::Result;
use subxt::dynamic::Value;

const SUBTENSOR_MODULE: &str = "SubtensorModule";

/// Add liquidity to a subnet pool.
///
/// # Arguments
/// * `client` — The Bittensor RPC client.
/// * `signer` — The signing keypair.
/// * `netuid` — The subnet ID.
/// * `amount_a` — First token amount **in RAO** (1 TAO = 1e9 RAO).
/// * `amount_b` — Second token amount **in RAO** (1 TAO = 1e9 RAO).
/// * `wait_for` — How long to wait for on-chain inclusion.
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

/// Remove liquidity from a subnet pool.
///
/// # Arguments
/// * `client` — The Bittensor RPC client.
/// * `signer` — The signing keypair.
/// * `netuid` — The subnet ID.
/// * `liquidity_amount` — Amount of liquidity tokens to remove **in RAO** (1 TAO = 1e9 RAO).
/// * `wait_for` — How long to wait for on-chain inclusion.
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

/// Modify a concentrated-liquidity position.
///
/// The `liquidity_delta` is a signed value (`i128`) representing the change
/// in liquidity — positive to add, negative to remove.  This is NOT an
/// amount in RAO; it is the raw liquidity-unit delta used by the AMM.
///
/// # Arguments
/// * `client` — The Bittensor RPC client.
/// * `signer` — The signing keypair.
/// * `netuid` — The subnet ID.
/// * `tick_lower` — Lower tick boundary.
/// * `tick_upper` — Upper tick boundary.
/// * `liquidity_delta` — Signed change in liquidity units.
/// * `wait_for` — How long to wait for on-chain inclusion.
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

/// Toggle user liquidity permission for a subnet.
///
/// # Arguments
/// * `client` — The Bittensor RPC client.
/// * `signer` — The signing keypair (must be subnet owner).
/// * `netuid` — The subnet ID.
/// * `enabled` — `true` to allow user liquidity operations, `false` to disable.
/// * `wait_for` — How long to wait for on-chain inclusion.
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
