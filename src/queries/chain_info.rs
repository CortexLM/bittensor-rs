use crate::chain::BittensorClient;
use anyhow::Result;

/// Get on-chain timestamp (ms since epoch)
pub async fn get_timestamp(client: &BittensorClient) -> Result<u64> {
    let val = client
        .storage("Timestamp", "Now", None)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Timestamp.Now not found"))?;
    crate::utils::value_decode::decode_u64(&val).map_err(|e| anyhow::anyhow!("{}", e))
}

/// Get last drand round from Drand pallet
pub async fn last_drand_round(client: &BittensorClient) -> Result<Option<u64>> {
    if let Some(val) = client.storage("Drand", "LastStoredRound", None).await? {
        return Ok(crate::utils::value_decode::decode_u64(&val).ok());
    }
    Ok(None)
}

/// Get tx rate limit from Subtensor module
pub async fn tx_rate_limit(client: &BittensorClient) -> Result<Option<u64>> {
    if let Some(val) = client.storage("SubtensorModule", "TxRateLimit", None).await? {
        return Ok(crate::utils::value_decode::decode_u64(&val).ok());
    }
    Ok(None)
}
