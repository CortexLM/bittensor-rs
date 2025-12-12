use crate::chain::{BittensorClient, BittensorSigner, ExtrinsicWait};
use anyhow::Result;
use parity_scale_codec::Encode;
use sp_core::crypto::AccountId32;
use subxt::dynamic::Value;

const SUBTENSOR_MODULE: &str = "SubtensorModule";

/// Increase delegate take (commission)
/// Subtensor expects: (hotkey, take: u16)
pub async fn increase_take(
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

/// Decrease delegate take (commission)
/// Subtensor expects: (hotkey, take: u16)
pub async fn decrease_take(
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
