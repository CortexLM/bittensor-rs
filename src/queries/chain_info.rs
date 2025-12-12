use crate::chain::BittensorClient;
use anyhow::Result;

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
