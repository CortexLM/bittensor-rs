use crate::chain::BittensorClient;
use crate::types::{AxonInfo, NeuronInfo, PrometheusInfo};
use crate::utils::value_decode::*;
use anyhow::{Context, Result};
/// Helper functions to query neuron data from storage (production-ready implementation)
use parity_scale_codec::Encode;
use sp_core::crypto::AccountId32;
use std::collections::HashMap;
use subxt::dynamic::Value;

const SUBTENSOR_MODULE: &str = "SubtensorModule";

/// Query neuron information from storage (production-ready)
pub async fn query_neuron_from_storage(
    client: &BittensorClient,
    netuid: u16,
    uid: u64,
    _block: Option<u64>,
) -> Result<Option<NeuronInfo>> {
    let uid_key = vec![Value::u128(netuid as u128), Value::u128(uid as u128)];

    // Query all storage entries needed to build NeuronInfo
    // Keys: (netuid, uid) -> hotkey (StorageDoubleMap)
    let hotkey_val = client
        .storage_with_keys(SUBTENSOR_MODULE, "Keys", uid_key.clone())
        .await?;

    // If no hotkey, neuron doesn't exist
    let hotkey = match hotkey_val {
        Some(val) => decode_account_id32(&val)?,
        None => return Ok(None),
    };

    // Owner: hotkey -> coldkey (StorageMap)
    let owner_key = vec![Value::from_bytes(&hotkey.encode())];
    let coldkey_val = client
        .storage_with_keys(SUBTENSOR_MODULE, "Owner", owner_key)
        .await?
        .ok_or_else(|| {
            anyhow::anyhow!("Coldkey (Owner) not found for hotkey on subnet {}", netuid)
        })?;
    let coldkey = decode_account_id32(&coldkey_val)?;

    // Fetch vector-based storages for the subnet (these are StorageMap<netuid, Vec<T>>)
    let netuid_key = vec![Value::u128(netuid as u128)];
    let idx = uid as usize;

    // Fetch all vector storages
    let rank_vec =
        fetch_vec_u16_storage(client, SUBTENSOR_MODULE, "Rank", netuid_key.clone()).await?;
    let trust_vec =
        fetch_vec_u16_storage(client, SUBTENSOR_MODULE, "Trust", netuid_key.clone()).await?;
    let consensus_vec =
        fetch_vec_u16_storage(client, SUBTENSOR_MODULE, "Consensus", netuid_key.clone()).await?;
    let validator_trust_vec = fetch_vec_u16_storage(
        client,
        SUBTENSOR_MODULE,
        "ValidatorTrust",
        netuid_key.clone(),
    )
    .await?;
    let incentive_vec =
        fetch_vec_u16_storage(client, SUBTENSOR_MODULE, "Incentive", netuid_key.clone()).await?;
    let dividends_vec =
        fetch_vec_u16_storage(client, SUBTENSOR_MODULE, "Dividends", netuid_key.clone()).await?;
    let active_vec =
        fetch_vec_bool_storage(client, SUBTENSOR_MODULE, "Active", netuid_key.clone()).await?;
    let last_update_vec =
        fetch_vec_u64_storage(client, SUBTENSOR_MODULE, "LastUpdate", netuid_key.clone()).await?;
    let emission_vec =
        fetch_vec_u128_storage(client, SUBTENSOR_MODULE, "Emission", netuid_key.clone()).await?;
    let validator_permit_vec = fetch_vec_bool_storage(
        client,
        SUBTENSOR_MODULE,
        "ValidatorPermit",
        netuid_key.clone(),
    )
    .await?;
    let pruning_scores_vec = fetch_vec_u16_storage(
        client,
        SUBTENSOR_MODULE,
        "PruningScores",
        netuid_key.clone(),
    )
    .await?;

    // Extract values from vectors with bounds checking and normalization
    let rank = rank_vec.get(idx).copied().unwrap_or(0) as f64 / 65535.0;
    let trust = trust_vec.get(idx).copied().unwrap_or(0) as f64 / 65535.0;
    let consensus = consensus_vec.get(idx).copied().unwrap_or(0) as f64 / 65535.0;
    let validator_trust = validator_trust_vec.get(idx).copied().unwrap_or(0) as f64 / 65535.0;
    let incentive = incentive_vec.get(idx).copied().unwrap_or(0) as f64 / 65535.0;
    let dividends = dividends_vec.get(idx).copied().unwrap_or(0) as f64 / 65535.0;

    let active = active_vec.get(idx).copied().unwrap_or(false);
    let validator_permit = validator_permit_vec.get(idx).copied().unwrap_or(false);
    let last_update = last_update_vec.get(idx).copied().unwrap_or(0);
    let emission_raw = emission_vec.get(idx).copied().unwrap_or(0);
    let emission = emission_raw as f64 / 1e9; // Convert to TAO
    let pruning_score = pruning_scores_vec.get(idx).copied().unwrap_or(0) as u64;

    // Version: fetch from storage
    let version = fetch_u64_storage_opt(client, SUBTENSOR_MODULE, "Version", uid_key.clone())
        .await
        .unwrap_or(None)
        .unwrap_or(0);

    // Get stake info - query Alpha storage for all coldkeys staking to this hotkey
    // Alpha: (hotkey, coldkey, netuid) -> stake amount
    // TotalHotkeyAlpha: (hotkey, netuid) -> total stake
    let stake_key = vec![
        Value::from_bytes(&hotkey.encode()),
        Value::u128(netuid as u128),
    ];
    let total_stake = if let Some(total_stake_val) = client
        .storage_with_keys(SUBTENSOR_MODULE, "TotalHotkeyAlpha", stake_key)
        .await?
    {
        decode_u128(&total_stake_val)
            .map_err(|e| anyhow::anyhow!("Failed to decode TotalHotkeyAlpha: {}", e))?
    } else {
        0u128 // No stake yet
    };

    // Stake dict from Stake[(netuid, uid)] -> Vec<(AccountId32, Compact<u64>)>
    // This provides the full mapping of coldkey -> amount staked to this neuron
    let stake_entries = match client
        .storage_with_keys(SUBTENSOR_MODULE, "Stake", uid_key.clone())
        .await
    {
        Ok(Some(stake_val)) => {
            decode_vec_account_u128_pairs(&stake_val).unwrap_or_else(|_| Vec::new())
        }
        _ => Vec::new(),
    };
    let mut stake_dict: HashMap<AccountId32, u128> = HashMap::new();
    for (ck, amt) in stake_entries {
        stake_dict.insert(ck, amt);
    }

    // Get axon and prometheus info
    let axon_keys = vec![Value::u128(netuid as u128), Value::u128(uid as u128)];
    let axon_info = fetch_axon_info(client, SUBTENSOR_MODULE, "Axons", axon_keys.clone()).await;
    let prometheus_info =
        fetch_prometheus_info(client, SUBTENSOR_MODULE, "Prometheus", axon_keys).await;

    // Weights and bonds for this neuron using storage index (netuid, mechid=0)
    let storage_index: u64 = ((netuid as u64) << 16) | 0u64; // mechid=0
    let weights_keys = vec![Value::u128(storage_index as u128), Value::u128(uid as u128)];
    let weights = match client
        .storage_with_keys(SUBTENSOR_MODULE, "Weights", weights_keys)
        .await
    {
        Ok(Some(w_val)) => decode_vec_u64_u64_pairs(&w_val).unwrap_or_else(|_| Vec::new()),
        _ => Vec::new(),
    };

    let bonds_keys = vec![Value::u128(storage_index as u128), Value::u128(uid as u128)];
    let bonds_pairs: Vec<(u64, u64)> = match client
        .storage_with_keys(SUBTENSOR_MODULE, "Bonds", bonds_keys)
        .await
    {
        Ok(Some(b_val)) => decode_vec_u64_u64_pairs(&b_val).unwrap_or_else(|_| Vec::new()),
        _ => Vec::new(),
    };
    // Bonds in NeuronInfo type are Vec<Vec<u64>>; convert pairs into two-element vectors
    let bonds: Vec<Vec<u64>> = bonds_pairs.into_iter().map(|(a, b)| vec![a, b]).collect();

    Ok(Some(NeuronInfo::create(
        uid,
        netuid,
        hotkey,
        coldkey,
        total_stake,
        stake_dict,
        total_stake,
        rank,
        trust,
        consensus,
        validator_trust,
        incentive,
        emission,
        dividends,
        active,
        last_update,
        validator_permit,
        version,
        weights,
        bonds,
        pruning_score,
        prometheus_info,
        axon_info,
    )))
}

/// Fetch AxonInfo from storage (optional)
pub async fn fetch_axon_info(
    client: &BittensorClient,
    module: &str,
    entry: &str,
    keys: Vec<Value>,
) -> Option<AxonInfo> {
    if let Some(value) = client.storage_with_keys(module, entry, keys).await.ok()? {
        crate::utils::scale_decode::decode_axon_info(&value).ok()
    } else {
        None
    }
}

/// Fetch PrometheusInfo from storage (optional)
pub async fn fetch_prometheus_info(
    client: &BittensorClient,
    module: &str,
    entry: &str,
    keys: Vec<Value>,
) -> Option<PrometheusInfo> {
    if let Some(value) = client.storage_with_keys(module, entry, keys).await.ok()? {
        crate::utils::value_decode::decode_prometheus_info(&value).ok()
    } else {
        None
    }
}

// Helper functions for fetching vector storages

async fn fetch_vec_u16_storage(
    client: &BittensorClient,
    module: &str,
    entry: &str,
    keys: Vec<Value>,
) -> Result<Vec<u16>> {
    if let Some(value) = client.storage_with_keys(module, entry, keys).await? {
        decode_vec_u16(&value).context(format!("Failed to decode {} vector", entry))
    } else {
        Ok(Vec::new())
    }
}

async fn fetch_vec_u64_storage(
    client: &BittensorClient,
    module: &str,
    entry: &str,
    keys: Vec<Value>,
) -> Result<Vec<u64>> {
    if let Some(value) = client.storage_with_keys(module, entry, keys).await? {
        decode_vec_u64(&value).context(format!("Failed to decode {} vector", entry))
    } else {
        Ok(Vec::new())
    }
}

async fn fetch_vec_u128_storage(
    client: &BittensorClient,
    module: &str,
    entry: &str,
    keys: Vec<Value>,
) -> Result<Vec<u128>> {
    if let Some(value) = client.storage_with_keys(module, entry, keys).await? {
        decode_vec_u128(&value).context(format!("Failed to decode {} vector", entry))
    } else {
        Ok(Vec::new())
    }
}

async fn fetch_vec_bool_storage(
    client: &BittensorClient,
    module: &str,
    entry: &str,
    keys: Vec<Value>,
) -> Result<Vec<bool>> {
    if let Some(value) = client.storage_with_keys(module, entry, keys).await? {
        decode_vec_bool(&value).context(format!("Failed to decode {} vector", entry))
    } else {
        Ok(Vec::new())
    }
}

async fn fetch_u64_storage_opt(
    client: &BittensorClient,
    module: &str,
    entry: &str,
    keys: Vec<Value>,
) -> Result<Option<u64>> {
    if let Some(value) = client.storage_with_keys(module, entry, keys).await? {
        Ok(Some(decode_u64(&value).context("Failed to decode u64")?))
    } else {
        Ok(None)
    }
}

// Legacy functions kept for compatibility

#[allow(dead_code)]
async fn fetch_normalized_u16_storage_opt(
    client: &BittensorClient,
    module: &str,
    entry: &str,
    keys: Vec<Value>,
) -> Result<Option<f64>> {
    if let Some(value) = client.storage_with_keys(module, entry, keys).await? {
        Ok(Some(
            decode_normalized_u16(&value).context("Failed to decode normalized u16")?,
        ))
    } else {
        Ok(None)
    }
}

#[allow(dead_code)]
async fn fetch_bool_storage(
    client: &BittensorClient,
    module: &str,
    entry: &str,
    keys: Vec<Value>,
) -> Result<bool> {
    let value = client
        .storage_with_keys(module, entry, keys)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Storage entry not found"))?;
    decode_bool(&value)
}

#[allow(dead_code)]
async fn fetch_bool_storage_opt(
    client: &BittensorClient,
    module: &str,
    entry: &str,
    keys: Vec<Value>,
) -> Result<Option<bool>> {
    if let Some(value) = client.storage_with_keys(module, entry, keys).await? {
        Ok(Some(decode_bool(&value).context("Failed to decode bool")?))
    } else {
        Ok(None)
    }
}
