use crate::chain::{BittensorClient, BittensorSigner, ExtrinsicWait};
use anyhow::Result;
use subxt::dynamic::Value;

const SUBTENSOR_MODULE: &str = "SubtensorModule";

/// Increase delegate take (commission)
pub async fn increase_take(
    client: &BittensorClient,
    signer: &BittensorSigner,
    take: u16, // Normalized u16 (0-65535 representing 0.0-1.0)
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args = vec![Value::u128(take as u128)];
    
    client
        .submit_extrinsic(SUBTENSOR_MODULE, "increase_delegate_take", args, signer, wait_for)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to increase take: {}", e))
}

/// Decrease delegate take (commission)
pub async fn decrease_take(
    client: &BittensorClient,
    signer: &BittensorSigner,
    take: u16, // Normalized u16 (0-65535 representing 0.0-1.0)
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args = vec![Value::u128(take as u128)];
    
    client
        .submit_extrinsic(SUBTENSOR_MODULE, "decrease_delegate_take", args, signer, wait_for)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to decrease take: {}", e))
}

