use crate::chain::{BittensorClient, BittensorSigner, ExtrinsicWait};
use crate::core::constants::EXISTENTIAL_DEPOSIT_RAO;
use crate::utils::balance_newtypes::Rao;
use anyhow::Result;
use parity_scale_codec::Encode;
use sp_core::crypto::AccountId32;
use subxt::dynamic::Value;

const BALANCES_MODULE: &str = "Balances";
const SUBTENSOR_MODULE: &str = "SubtensorModule";

/// Transfer TAO to another account.
///
/// # Arguments
/// * `client` — The Bittensor RPC client.
/// * `signer` — The signing keypair (source of funds).
/// * `dest` — Destination account.
/// * `amount` — Amount to transfer **in RAO** (1 TAO = 1e9 RAO).
/// * `keep_alive` — When `true`, uses `transfer_keep_alive` which ensures the
///   sender's account is not reaped (balance stays above the existential
///   deposit).
/// * `wait_for` — How long to wait for on-chain inclusion.
///
/// # Errors
/// Returns an error if the amount is zero, exceeds the safety limit, or the
/// extrinsic submission fails.
pub async fn transfer(
    client: &BittensorClient,
    signer: &BittensorSigner,
    dest: &AccountId32,
    amount: Rao,
    keep_alive: bool,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    if amount.as_u128() == 0 {
        return Err(anyhow::anyhow!("Transfer amount must be greater than zero"));
    }

    if !amount.is_valid_transfer_amount() {
        return Err(anyhow::anyhow!(
            "Transfer amount {} RAO exceeds the safe maximum (u64::MAX). \
             This is almost certainly an error — did you pass TAO instead of RAO?",
            amount.as_u128()
        ));
    }

    if keep_alive && amount.as_u128() < EXISTENTIAL_DEPOSIT_RAO {
        tracing::warn!(
            "Transfer amount ({} RAO) is below the existential deposit ({} RAO). \
             The transaction may fail on-chain.",
            amount.as_u128(),
            EXISTENTIAL_DEPOSIT_RAO,
        );
    }

    let dest_bytes = dest.encode();
    let dest_value = Value::from_bytes(&dest_bytes);

    let function = if keep_alive {
        "transfer_keep_alive"
    } else {
        "transfer"
    };

    let args = vec![dest_value, Value::u128(amount.as_u128())];

    client
        .submit_extrinsic(BALANCES_MODULE, function, args, signer, wait_for)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to transfer: {}", e))
}

/// Transfer stake from one coldkey to another, optionally across subnets.
///
/// Subtensor expects: `(destination_coldkey, hotkey, origin_netuid, destination_netuid, alpha_amount)`.
///
/// # Arguments
/// * `client` — The Bittensor RPC client.
/// * `signer` — The signing keypair (source coldkey).
/// * `destination_coldkey` — Coldkey receiving the stake.
/// * `hotkey` — The hotkey whose stake is being transferred.
/// * `origin_netuid` — Source subnet ID.
/// * `destination_netuid` — Target subnet ID.
/// * `amount` — Amount to transfer **in RAO** (1 TAO = 1e9 RAO).
/// * `wait_for` — How long to wait for on-chain inclusion.
///
/// # Errors
/// Returns an error if the amount is zero, exceeds the safety limit, or the
/// extrinsic submission fails.
#[allow(clippy::too_many_arguments)]
pub async fn transfer_stake(
    client: &BittensorClient,
    signer: &BittensorSigner,
    destination_coldkey: &AccountId32,
    hotkey: &AccountId32,
    origin_netuid: u16,
    destination_netuid: u16,
    amount: Rao,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    if amount.as_u128() == 0 {
        return Err(anyhow::anyhow!(
            "Stake transfer amount must be greater than zero"
        ));
    }

    if !amount.is_valid_transfer_amount() {
        return Err(anyhow::anyhow!(
            "Stake transfer amount {} RAO exceeds the safe maximum (u64::MAX). \
             This is almost certainly an error — did you pass TAO instead of RAO?",
            amount.as_u128()
        ));
    }

    let args = vec![
        Value::from_bytes(destination_coldkey.encode()),
        Value::from_bytes(hotkey.encode()),
        Value::from(origin_netuid),
        Value::from(destination_netuid),
        Value::u128(amount.as_u128()),
    ];

    client
        .submit_extrinsic(SUBTENSOR_MODULE, "transfer_stake", args, signer, wait_for)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to transfer stake: {}", e))
}
