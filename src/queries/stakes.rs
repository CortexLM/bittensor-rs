use parity_scale_codec::Encode;
use crate::chain::BittensorClient;
use crate::utils::value_decode::{decode_u128, decode_u64, decode_vec_account_id32};
use anyhow::Result;
use sp_core::crypto::AccountId32;
use subxt::dynamic::Value;

const SUBTENSOR_MODULE: &str = "SubtensorModule";

/// Get stake amount for a coldkey-hotkey pair on a specific subnet
/// Same signature as Bittensor Python
pub async fn get_stake(
    client: &BittensorClient,
    coldkey: &AccountId32,
    hotkey: &AccountId32,
    netuid: u16,
) -> Result<u128> {
    // Use query_module for Alpha storage (same as Bittensor Python: query_module("SubtensorModule", "Alpha", ...))
    // Alpha storage: (hotkey, coldkey, netuid) -> stake amount
    let keys = vec![
        Value::from_bytes(&hotkey.encode()),
        Value::from_bytes(&coldkey.encode()),
        Value::u128(netuid as u128),
    ];
    
    let alpha_val = client
        .storage_with_keys(SUBTENSOR_MODULE, "Alpha", keys)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Alpha not found for hotkey, coldkey, and netuid {}", netuid))?;
    
    decode_u128(&alpha_val)
        .map_err(|e| anyhow::anyhow!("Failed to decode Alpha stake: {}", e))
}

/// Get total stake for a coldkey across all hotkeys
pub async fn get_stake_for_coldkey(
    client: &BittensorClient,
    coldkey: &AccountId32,
) -> Result<Vec<(u16, u128)>> {
    // Strategy without relying on runtime API:
    // 1) Get list of owned hotkeys for this coldkey
    // 2) For each netuid, sum Alpha[(hotkey, coldkey, netuid)] across all owned hotkeys
    // 3) Return non-zero entries as (netuid, total)
    let owned_hotkeys_val = client
        .storage_with_keys("SubtensorModule", "OwnedHotkeys", vec![Value::from_bytes(&coldkey.encode())])
        .await?;
    let owned_hotkeys: Vec<AccountId32> = match owned_hotkeys_val {
        Some(v) => decode_vec_account_id32(&v).unwrap_or_default(),
        None => Vec::new(),
    };

    // Get total networks
    let total_networks_val = client
        .storage("SubtensorModule", "TotalNetworks", None)
        .await?;
    let total_networks: u16 = total_networks_val
        .and_then(|v| decode_u64(&v).ok())
        .and_then(|n| u16::try_from(n).ok())
        .unwrap_or(0u16);

    let mut result: Vec<(u16, u128)> = Vec::new();
    for netuid in 0u16..total_networks {
        let mut total: u128 = 0;
        for hotkey in &owned_hotkeys {
            let alpha_keys = vec![
                Value::from_bytes(&hotkey.encode()),
                Value::from_bytes(&coldkey.encode()),
                Value::u128(netuid as u128),
            ];
            if let Some(alpha_val) = client
                .storage_with_keys(SUBTENSOR_MODULE, "Alpha", alpha_keys)
                .await?
            {
                if let Ok(stake) = decode_u128(&alpha_val) { total = total.saturating_add(stake); }
            }
        }
        if total > 0 { result.push((netuid, total)); }
    }

    Ok(result)
}

/// Get total stake for a hotkey from all coldkeys
pub async fn get_stake_for_hotkey(
    client: &BittensorClient,
    hotkey: &AccountId32,
    netuid: u16,
) -> Result<u128> {
    // Use query_subtensor for TotalHotkeyAlpha (same as Bittensor Python)
    let keys = vec![
        Value::from_bytes(&hotkey.encode()),
        Value::u128(netuid as u128),
    ];
    
    let alpha_val = client
        .storage_with_keys(SUBTENSOR_MODULE, "TotalHotkeyAlpha", keys)
        .await?
        .ok_or_else(|| anyhow::anyhow!("TotalHotkeyAlpha not found for hotkey and netuid {}", netuid))?;
    
    decode_u128(&alpha_val)
        .map_err(|e| anyhow::anyhow!("Failed to decode TotalHotkeyAlpha: {}", e))
}

/// Get stake for specific coldkey-hotkey pair across multiple subnets
/// Returns HashMap<netuid, StakeInfo> (same as Bittensor Python)
pub async fn get_stake_for_coldkey_and_hotkey(
    client: &BittensorClient,
    coldkey: &AccountId32,
    hotkey: &AccountId32,
    netuids: Option<Vec<u16>>,
) -> Result<std::collections::HashMap<u16, u128>> {
    let all_netuids = if let Some(nets) = netuids { nets } else {
        // Build from TotalNetworks
        let total_val = client
            .storage(SUBTENSOR_MODULE, "TotalNetworks", None)
            .await?
            .ok_or_else(|| anyhow::anyhow!("TotalNetworks not found"))?;
        let total = u16::try_from(decode_u64(&total_val).unwrap_or(0)).unwrap_or(0);
        (0..total).collect()
    };

    let mut stakes = std::collections::HashMap::new();
    for netuid in all_netuids {
        match get_stake(client, coldkey, hotkey, netuid).await {
            Ok(stake) => {
                if stake > 0 { stakes.insert(netuid, stake); }
            }
            Err(_) => { /* no stake for this pair on this netuid */ }
        }
    }

    Ok(stakes)
}

/// Get auto-stake settings for a coldkey
/// Returns HashMap<netuid, hotkey> (same as Bittensor Python)
pub async fn get_auto_stakes(
    client: &BittensorClient,
    coldkey: &AccountId32,
) -> Result<std::collections::HashMap<u16, AccountId32>> {
    let mut map = std::collections::HashMap::new();
    // Iterate across all subnets and query (coldkey, netuid) -> destination
    let total_networks_val = client
        .storage(SUBTENSOR_MODULE, "TotalNetworks", None)
        .await?;
    let total_networks: u16 = total_networks_val
        .and_then(|v| decode_u64(&v).ok())
        .and_then(|n| u16::try_from(n).ok())
        .unwrap_or(0u16);

    for netuid in 0u16..total_networks {
        let keys = vec![
            Value::from_bytes(&coldkey.encode()),
            Value::u128(netuid as u128),
        ];
        if let Some(dest_val) = client
            .storage_with_keys(SUBTENSOR_MODULE, "AutoStakeDestination", keys)
            .await?
        {
            if let Ok(hotkey) = crate::utils::value_decode::decode_account_id32(&dest_val) {
                map.insert(netuid, hotkey);
            }
        }
    }
    Ok(map)
}

/// Get stake weight (normalized)
pub async fn get_stake_weight(
    client: &BittensorClient,
    netuid: u16,
    hotkey: &AccountId32,
) -> Result<f64> {
    // Query stake weights from storage
    let keys = vec![
        Value::u128(netuid as u128),
        Value::from_bytes(&hotkey.encode()),
    ];
    
    let weight_val = client
        .storage_with_keys(SUBTENSOR_MODULE, "StakeWeight", keys)
        .await?
        .ok_or_else(|| anyhow::anyhow!("StakeWeight not found for hotkey and netuid {}", netuid))?;
    
    let weight = decode_u64(&weight_val)
        .map_err(|e| anyhow::anyhow!("Failed to decode StakeWeight: {}", e))?;
    
    // Normalize from u64 to 0.0-1.0 range
    Ok(weight as f64 / u64::MAX as f64)
}

/// Get minimum required stake
pub async fn get_minimum_required_stake(client: &BittensorClient) -> Result<u128> {
    let min_stake_val = client
        .storage(SUBTENSOR_MODULE, "NominatorMinRequiredStake", None)
        .await?
        .ok_or_else(|| anyhow::anyhow!("NominatorMinRequiredStake storage entry not found"))?;
    
    decode_u128(&min_stake_val)
        .map_err(|e| anyhow::anyhow!("Failed to decode NominatorMinRequiredStake: {}", e))
}

