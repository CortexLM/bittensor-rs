use crate::chain::{BittensorClient, BittensorSigner, ExtrinsicWait};
use anyhow::Result;
use parity_scale_codec::Encode;
use sp_core::crypto::AccountId32;
use subxt::dynamic::Value;

const SUBTENSOR_MODULE: &str = "SubtensorModule";

/// Increase delegate take (commission rate).
///
/// Subtensor extrinsic argument order: `(hotkey, take: u16)`.
/// The `take` value is a u16 proportion (0–65535 maps to 0–100%).
///
/// # Arguments
/// * `client` — The Bittensor RPC client.
/// * `signer` — The signing keypair (coldkey that owns the hotkey).
/// * `hotkey` — The delegate hotkey.
/// * `take` — New take value as u16 proportion.
/// * `wait_for` — How long to wait for on-chain inclusion.
pub async fn increase_take(
    client: &BittensorClient,
    signer: &BittensorSigner,
    hotkey: &AccountId32,
    take: u16,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args = vec![Value::from_bytes(hotkey.encode()), Value::from(take)];

    client
        .submit_extrinsic(
            SUBTENSOR_MODULE,
            "increase_delegate_take",
            args,
            signer,
            wait_for,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to increase take: {}", e))
}

/// Decrease delegate take (commission rate).
///
/// Subtensor extrinsic argument order: `(hotkey, take: u16)`.
/// The `take` value is a u16 proportion (0–65535 maps to 0–100%).
///
/// # Arguments
/// * `client` — The Bittensor RPC client.
/// * `signer` — The signing keypair (coldkey that owns the hotkey).
/// * `hotkey` — The delegate hotkey.
/// * `take` — New take value as u16 proportion.
/// * `wait_for` — How long to wait for on-chain inclusion.
pub async fn decrease_take(
    client: &BittensorClient,
    signer: &BittensorSigner,
    hotkey: &AccountId32,
    take: u16,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args = vec![Value::from_bytes(hotkey.encode()), Value::from(take)];

    client
        .submit_extrinsic(
            SUBTENSOR_MODULE,
            "decrease_delegate_take",
            args,
            signer,
            wait_for,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to decrease take: {}", e))
}

/// Become a delegate (enable delegate take > 0).
///
/// Subtensor extrinsic argument order: `(hotkey, take: u16)`.
/// The `take` value is a u16 proportion (0–65535 maps to 0–100%).
///
/// # Arguments
/// * `client` — The Bittensor RPC client.
/// * `signer` — The signing keypair (coldkey that owns the hotkey).
/// * `hotkey` — The hotkey to promote to delegate.
/// * `take` — Initial take value as u16 proportion.
/// * `wait_for` — How long to wait for on-chain inclusion.
pub async fn become_delegate(
    client: &BittensorClient,
    signer: &BittensorSigner,
    hotkey: &AccountId32,
    take: u16,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args = vec![
        Value::from_bytes(hotkey.encode()),
        Value::u128(take as u128),
    ];

    client
        .submit_extrinsic(SUBTENSOR_MODULE, "become_delegate", args, signer, wait_for)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to become delegate: {}", e))
}
