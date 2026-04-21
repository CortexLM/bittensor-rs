use crate::chain::BittensorClient;
use anyhow::Result;
use std::time::Duration;

const SUBTENSOR_MODULE: &str = "SubtensorModule";

/// Get on-chain timestamp (ms since epoch)
pub async fn get_timestamp(client: &BittensorClient) -> Result<u64> {
    let val = client
        .storage("Timestamp", "Now", None)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Timestamp.Now not found"))?;
    crate::utils::decoders::decode_u64(&val).map_err(|e| anyhow::anyhow!("{}", e))
}

/// Get last drand round from Drand pallet
pub async fn last_drand_round(client: &BittensorClient) -> Result<Option<u64>> {
    if let Some(val) = client.storage("Drand", "LastStoredRound", None).await? {
        return Ok(crate::utils::decoders::decode_u64(&val).ok());
    }
    Ok(None)
}

/// Get tx rate limit from Subtensor module
pub async fn tx_rate_limit(client: &BittensorClient) -> Result<Option<u64>> {
    if let Some(val) = client
        .storage(SUBTENSOR_MODULE, "TxRateLimit", None)
        .await?
    {
        return Ok(crate::utils::decoders::decode_u64(&val).ok());
    }
    Ok(None)
}

/// Get the admin freeze window (number of blocks where dependent txs are frozen)
pub async fn get_admin_freeze_window(client: &BittensorClient) -> Result<u64> {
    let val = client
        .storage(SUBTENSOR_MODULE, "AdminFreezeWindow", None)
        .await?
        .ok_or_else(|| anyhow::anyhow!("AdminFreezeWindow not found"))?;
    crate::utils::decoders::decode_u64(&val).map_err(|e| anyhow::anyhow!("{}", e))
}

/// Check if current block is within admin freeze window for a subnet
pub async fn is_in_admin_freeze_window(client: &BittensorClient, netuid: u16) -> Result<bool> {
    // SN0 doesn't have admin freeze window
    if netuid == 0 {
        return Ok(false);
    }

    let next_epoch = crate::queries::subnets::get_next_epoch_start_block(client, netuid, None)
        .await?
        .unwrap_or(0);
    let window = get_admin_freeze_window(client).await.unwrap_or(0);
    let current_block = client
        .block_number()
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    if next_epoch > 0 {
        let remaining = next_epoch.saturating_sub(current_block);
        return Ok(remaining < window);
    }
    Ok(false)
}

/// Check if the node is running with fast blocks
pub async fn is_fast_blocks(client: &BittensorClient) -> Result<bool> {
    if let Some(val) = client
        .storage(SUBTENSOR_MODULE, "DurationOfStartCall", None)
        .await?
    {
        if let Ok(duration) = crate::utils::decoders::decode_u64(&val) {
            return Ok(duration == 10);
        }
    }
    Ok(false)
}

/// Get total issuance of the native token (RAO)
/// Reads Balances::TotalIssuance storage
pub async fn get_total_issuance(client: &BittensorClient) -> Result<u128> {
    let val = client
        .storage("Balances", "TotalIssuance", None)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Balances::TotalIssuance not found"))?;
    crate::utils::decoders::decode_u128(&val)
        .map_err(|e| anyhow::anyhow!("Failed to decode TotalIssuance: {}", e))
}

/// Get total stake across all subnets (RAO)
/// Reads SubtensorModule::TotalStake storage
pub async fn get_total_stake(client: &BittensorClient) -> Result<u128> {
    let val = client
        .storage(SUBTENSOR_MODULE, "TotalStake", None)
        .await?
        .ok_or_else(|| anyhow::anyhow!("SubtensorModule::TotalStake not found"))?;
    crate::utils::decoders::decode_u128(&val)
        .map_err(|e| anyhow::anyhow!("Failed to decode TotalStake: {}", e))
}

/// Get block hash for a specific block number
/// Uses the chain RPC to retrieve the hash
pub async fn get_block_hash(client: &BittensorClient, block_number: u64) -> Result<[u8; 32]> {
    let hash = client
        .api()
        .backend()
        .block_header(
            client
                .api()
                .backend()
                .latest_finalized_block_ref()
                .await?
                .hash(),
        )
        .await
        .ok()
        .flatten();

    let _ = hash;

    let params = vec![subxt::dynamic::Value::u128(block_number as u128)];
    let raw_bytes = client
        .runtime_api_call(
            "BlockBuilder",
            "block_hash",
            Some(block_number.to_le_bytes().to_vec()),
        )
        .await;

    match raw_bytes {
        Ok(bytes) if bytes.len() == 32 => {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&bytes);
            Ok(arr)
        }
        _ => {
            let _ = params;
            Err(anyhow::anyhow!(
                "Failed to get block hash for block {}",
                block_number
            ))
        }
    }
}

/// Get current block number with retry logic
/// Retries up to max_retries times with exponential backoff
pub async fn get_current_block_with_retry(
    client: &BittensorClient,
    max_retries: u32,
) -> Result<u64> {
    let mut last_err = None;
    for attempt in 0..=max_retries {
        match client.block_number().await {
            Ok(block) => return Ok(block),
            Err(e) => {
                last_err = Some(anyhow::anyhow!("{}", e));
                if attempt < max_retries {
                    let delay = Duration::from_millis(100 * 2u64.pow(attempt));
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }
    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("Failed to get current block")))
}

/// Get total number of subnets
/// Reads SubtensorModule::TotalNetworks storage
pub async fn get_total_subnets(client: &BittensorClient) -> Result<u16> {
    crate::queries::subnets::total_subnets(client).await
}
