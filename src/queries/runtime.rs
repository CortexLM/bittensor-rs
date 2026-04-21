use crate::chain::BittensorClient;
use anyhow::Result;
use parity_scale_codec::Encode;
use sp_core::crypto::AccountId32;
use subxt::dynamic::Value;

/// Get the current weights version key from runtime storage.
pub async fn get_weights_version_key(client: &BittensorClient) -> Result<u64> {
    let value = client
        .storage("SubtensorModule", "WeightsVersion", None)
        .await?
        .ok_or_else(|| anyhow::anyhow!("WeightsVersion not found"))?;
    crate::utils::decoders::decode_u64(&value)
        .map_err(|e| anyhow::anyhow!("Failed to decode WeightsVersion: {}", e))
}

/// Check whether commit-reveal is enabled on chain.
pub async fn commit_reveal_enabled(client: &BittensorClient) -> Result<bool> {
    let value = client
        .storage("SubtensorModule", "CommitRevealEnabled", None)
        .await?;
    if let Some(value) = value {
        return crate::utils::decoders::decode_bool(&value)
            .map_err(|e| anyhow::anyhow!("Failed to decode CommitRevealEnabled: {}", e));
    }
    Ok(true)
}

/// Get the current block step (tempo) for a subnet.
pub async fn get_tempo(client: &BittensorClient, netuid: u16) -> Result<u16> {
    let value = client
        .storage_with_keys(
            "SubtensorModule",
            "Tempo",
            vec![Value::u128(netuid as u128)],
        )
        .await?
        .ok_or_else(|| anyhow::anyhow!("Tempo not found for netuid {}", netuid))?;
    crate::utils::decoders::decode_u16(&value)
        .map_err(|e| anyhow::anyhow!("Failed to decode Tempo: {}", e))
}

/// Get owner coldkey for a hotkey.
pub async fn get_hotkey_owner(
    client: &BittensorClient,
    hotkey: &AccountId32,
) -> Result<Option<AccountId32>> {
    let value = client
        .storage_with_keys(
            "SubtensorModule",
            "Owner",
            vec![Value::from_bytes(hotkey.encode())],
        )
        .await?;
    match value {
        Some(value) => crate::utils::decoders::decode_account_id32(&value)
            .map(Some)
            .map_err(|e| anyhow::anyhow!("Failed to decode owner: {}", e)),
        None => Ok(None),
    }
}
