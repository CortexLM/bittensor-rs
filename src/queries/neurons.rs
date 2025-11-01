use crate::chain::BittensorClient;
use crate::types::{NeuronInfo, NeuronInfoLite};
use anyhow::{Result, Context};
use sp_core::crypto::AccountId32;
use subxt::dynamic::Value;
use parity_scale_codec::Encode;
use crate::utils::value_decode::*;

const SUBTENSOR_MODULE: &str = "SubtensorModule";

/// Get a specific neuron by subnet and UID
pub async fn neuron(
    client: &BittensorClient,
    netuid: u16,
    uid: u64,
    block: Option<u64>,
) -> Result<Option<NeuronInfo>> {
    // Import neurons_storage for storage-based implementation
    use crate::queries::neurons_storage::query_neuron_from_storage;
    
    // Prefer storage-based implementation (robust across runtimes)
    query_neuron_from_storage(client, netuid, uid, block).await
}

/// Get all neurons for a subnet (storage-based)
pub async fn neurons(
    client: &BittensorClient,
    netuid: u16,
    _block: Option<u64>,
) -> Result<Vec<NeuronInfo>> {
    // SubnetworkN for count
    let n_key = vec![Value::u128(netuid as u128)];
    let n_value = client.storage_with_keys(SUBTENSOR_MODULE, "SubnetworkN", n_key).await?
        .ok_or_else(|| anyhow::anyhow!("Subnet {} not found (SubnetworkN storage entry missing)", netuid))?;
    let n = decode_u64(&n_value).context("Failed to decode SubnetworkN")?;

    // Get all the vector storages for the subnet
    let netuid_key = vec![Value::u128(netuid as u128)];
    
    // Fetch all vectors at once for efficiency
    let rank_vec = fetch_normalized_vec(client, "Rank", &netuid_key).await?;
    let trust_vec = fetch_normalized_vec(client, "Trust", &netuid_key).await?;
    let consensus_vec = fetch_normalized_vec(client, "Consensus", &netuid_key).await?;
    let validator_trust_vec = fetch_normalized_vec(client, "ValidatorTrust", &netuid_key).await?;
    let incentive_vec = fetch_normalized_vec(client, "Incentive", &netuid_key).await?;
    let dividends_vec = fetch_normalized_vec(client, "Dividends", &netuid_key).await?;
    let emission_vec = fetch_emission_vec(client, &netuid_key).await?;
    let active_vec = fetch_bool_vec(client, "Active", &netuid_key).await?;
    let last_update_vec = fetch_u64_vec(client, "LastUpdate", &netuid_key).await?;
    let validator_permit_vec = fetch_bool_vec(client, "ValidatorPermit", &netuid_key).await?;
    let pruning_score_vec = fetch_normalized_vec(client, "PruningScores", &netuid_key).await?;

    let mut list = Vec::new();
    for uid in 0..n {
        // Use the helper that fetches individual neuron data
        if let Some(neuron) = query_neuron_from_vectors(
            client, netuid, uid,
            &rank_vec, &trust_vec, &consensus_vec, &validator_trust_vec,
            &incentive_vec, &dividends_vec, &emission_vec, &active_vec,
            &last_update_vec, &validator_permit_vec, &pruning_score_vec,
        ).await? {
            list.push(neuron);
        }
    }
    Ok(list)
}

/// Build a neuron from pre-fetched vectors
async fn query_neuron_from_vectors(
    client: &BittensorClient,
    netuid: u16,
    uid: u64,
    _rank_vec: &[u16],
    _trust_vec: &[u16],
    _consensus_vec: &[u16],
    _validator_trust_vec: &[u16],
    _incentive_vec: &[u16],
    _dividends_vec: &[u16],
    _emission_vec: &[u128],
    _active_vec: &[bool],
    _last_update_vec: &[u64],
    _validator_permit_vec: &[bool],
    _pruning_score_vec: &[u16],
) -> Result<Option<NeuronInfo>> {
    // For full NeuronInfo, delegate to the storage-based helper
    // which handles all the complex data fetching
    use crate::queries::neurons_storage::query_neuron_from_storage;
    query_neuron_from_storage(client, netuid, uid, None).await
}

/// Get lightweight neuron information (without weights and bonds)
pub async fn neurons_lite(
    client: &BittensorClient,
    netuid: u16,
    _block: Option<u64>,
) -> Result<Vec<NeuronInfoLite>> {
    // Build from storage without weights/bonds
    let n_key = vec![Value::u128(netuid as u128)];
    let n_value = client.storage_with_keys(SUBTENSOR_MODULE, "SubnetworkN", n_key).await?
        .ok_or_else(|| anyhow::anyhow!("Subnet {} not found (SubnetworkN storage entry missing)", netuid))?;
    let n = decode_u64(&n_value).context("Failed to decode SubnetworkN")?;

    // Get all the vector storages for the subnet
    let netuid_key = vec![Value::u128(netuid as u128)];
    
    // Fetch all vectors at once for efficiency
    let rank_vec = fetch_normalized_vec(client, "Rank", &netuid_key).await?;
    let trust_vec = fetch_normalized_vec(client, "Trust", &netuid_key).await?;
    let consensus_vec = fetch_normalized_vec(client, "Consensus", &netuid_key).await?;
    let validator_trust_vec = fetch_normalized_vec(client, "ValidatorTrust", &netuid_key).await?;
    let incentive_vec = fetch_normalized_vec(client, "Incentive", &netuid_key).await?;
    let dividends_vec = fetch_normalized_vec(client, "Dividends", &netuid_key).await?;
    let emission_vec = fetch_emission_vec(client, &netuid_key).await?;
    let active_vec = fetch_bool_vec(client, "Active", &netuid_key).await?;
    let last_update_vec = fetch_u64_vec(client, "LastUpdate", &netuid_key).await?;
    let validator_permit_vec = fetch_bool_vec(client, "ValidatorPermit", &netuid_key).await?;
    let pruning_score_vec = fetch_u64_vec(client, "PruningScore", &netuid_key).await?;

    let mut list = Vec::new();
    for uid in 0..n {
        let idx = uid as usize;
        
        // hotkey
        let uid_key = vec![Value::u128(netuid as u128), Value::u128(uid as u128)];
        let hotkey = match client
            .storage_with_keys(SUBTENSOR_MODULE, "Keys", uid_key.clone())
            .await?
        {
            Some(v) => decode_account_id32(&v).ok(),
            None => None,
        };
        if hotkey.is_none() { continue; }
        let hotkey = hotkey.unwrap();

        // owner (coldkey)
        let coldkey = match client
            .storage_with_keys(SUBTENSOR_MODULE, "Owner", vec![Value::from_bytes(&hotkey.encode())])
            .await?
        {
            Some(v) => decode_account_id32(&v).ok(),
            None => None,
        };
        if coldkey.is_none() { continue; }
        let coldkey = coldkey.unwrap();

        // Get values from vectors
        let rank = rank_vec.get(idx).copied().unwrap_or(0) as f64 / 65535.0;
        let trust = trust_vec.get(idx).copied().unwrap_or(0) as f64 / 65535.0;
        let consensus = consensus_vec.get(idx).copied().unwrap_or(0) as f64 / 65535.0;
        let validator_trust = validator_trust_vec.get(idx).copied().unwrap_or(0) as f64 / 65535.0;
        let incentive = incentive_vec.get(idx).copied().unwrap_or(0) as f64 / 65535.0;
        let dividends = dividends_vec.get(idx).copied().unwrap_or(0) as f64 / 65535.0;
        let emission_raw = emission_vec.get(idx).copied().unwrap_or(0);
        let emission = emission_raw as f64 / 1e9;
        let active = active_vec.get(idx).copied().unwrap_or(false);
        let last_update = last_update_vec.get(idx).copied().unwrap_or(0);
        let validator_permit = validator_permit_vec.get(idx).copied().unwrap_or(false);
        let pruning_score = pruning_score_vec.get(idx).copied().unwrap_or(0);

        // Stake: TotalHotkeyAlpha[(hotkey, netuid)]
        let total_stake = if let Some(ts_val) = client
            .storage_with_keys(SUBTENSOR_MODULE, "TotalHotkeyAlpha", vec![
                Value::from_bytes(&hotkey.encode()),
                Value::u128(netuid as u128),
            ])
            .await?
        {
            decode_u128(&ts_val).unwrap_or(0)
        } else { 0 };

        // Build Lite type by converting into NeuronInfoLite
        let lite = NeuronInfoLite {
            uid,
            netuid,
            hotkey,
            coldkey,
            stake: total_stake,
            stake_dict: std::collections::HashMap::new(),
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
            pruning_score,
            prometheus_info: None,
            axon_info: None,
            is_null: false,
        };
        list.push(lite);
    }
    Ok(list)
}

/// Helper functions for fetching storage vectors

async fn fetch_normalized_vec(
    client: &BittensorClient,
    storage_name: &str,
    keys: &[Value],
) -> Result<Vec<u16>> {
    client
        .storage_with_keys(SUBTENSOR_MODULE, storage_name, keys.to_vec())
        .await?
        .and_then(|v| decode_vec_u16(&v).ok())
        .ok_or_else(|| anyhow::anyhow!("{} not found", storage_name))
}

async fn fetch_emission_vec(
    client: &BittensorClient,
    keys: &[Value],
) -> Result<Vec<u128>> {
    client
        .storage_with_keys(SUBTENSOR_MODULE, "Emission", keys.to_vec())
        .await?
        .and_then(|v| decode_vec_u128(&v).ok())
        .ok_or_else(|| anyhow::anyhow!("Emission not found"))
}

async fn fetch_bool_vec(
    client: &BittensorClient,
    storage_name: &str,
    keys: &[Value],
) -> Result<Vec<bool>> {
    client
        .storage_with_keys(SUBTENSOR_MODULE, storage_name, keys.to_vec())
        .await?
        .and_then(|v| decode_vec_bool(&v).ok())
        .ok_or_else(|| anyhow::anyhow!("{} not found", storage_name))
}

async fn fetch_u64_vec(
    client: &BittensorClient,
    storage_name: &str,
    keys: &[Value],
) -> Result<Vec<u64>> {
    client
        .storage_with_keys(SUBTENSOR_MODULE, storage_name, keys.to_vec())
        .await?
        .and_then(|v| decode_vec_u64(&v).ok())
        .ok_or_else(|| anyhow::anyhow!("{} not found", storage_name))
}

/// Get neuron for a specific pubkey on a subnet
pub async fn neuron_for_pubkey_and_subnet(
    client: &BittensorClient,
    hotkey: &AccountId32,
    netuid: u16,
    block: Option<u64>,
) -> Result<Option<NeuronInfo>> {
    // First get UID for the hotkey
    if let Some(uid) = uid_for_hotkey_on_subnet(client, netuid, hotkey, block).await? {
        query_neuron_from_storage(client, netuid, uid, block).await
    } else {
        Ok(None)
    }
}

/// Get UID for a hotkey on a subnet
pub async fn uid_for_hotkey_on_subnet(
    client: &BittensorClient,
    netuid: u16,
    hotkey: &AccountId32,
    _block: Option<u64>,
) -> Result<Option<u64>> {
    let keys = vec![
        Value::u128(netuid as u128),
        Value::from_bytes(&hotkey.encode()),
    ];
    
    if let Some(uid_value) = client
        .storage_with_keys(SUBTENSOR_MODULE, "Uids", keys)
        .await?
    {
        // Decode u64 from Value
        let uid = decode_u64(&uid_value)
            .context("Failed to decode UID from storage")?;
        return Ok(Some(uid));
    }
    
    Ok(None)
}

/// Check if hotkey is registered on any subnet
pub async fn is_hotkey_registered_any(
    client: &BittensorClient,
    hotkey: &AccountId32,
) -> Result<bool> {
    use crate::queries::subnets::all_subnets;
    
    // Get all subnets
    let subnets = all_subnets(client).await?;
    
    // Check each subnet
    for subnet in subnets {
        if is_hotkey_registered_on_subnet(client, subnet.netuid, hotkey).await? {
            return Ok(true);
        }
    }
    
    Ok(false)
}

/// Check if hotkey is registered on specific subnet
pub async fn is_hotkey_registered_on_subnet(
    client: &BittensorClient,
    netuid: u16,
    hotkey: &AccountId32,
) -> Result<bool> {
    if let Some(uid) = uid_for_hotkey_on_subnet(client, netuid, hotkey, None).await? {
        Ok(uid > 0)
    } else {
        Ok(false)
    }
}

// Import the storage-based helper
use crate::queries::neurons_storage::query_neuron_from_storage;