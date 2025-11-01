/// Neuron queries for fetching neuron information from the Bittensor network
use crate::chain::BittensorClient;
use crate::types::{AxonInfo, NeuronInfo, PrometheusInfo};
use crate::utils::value_decode::*;
use anyhow::{Context, Result};
use futures::stream::{FuturesUnordered, StreamExt};
use parity_scale_codec::Encode;
use sp_core::crypto::AccountId32;
use std::collections::HashMap;
use subxt::dynamic::Value;

const SUBTENSOR_MODULE: &str = "SubtensorModule";

/// Get all neurons for a subnet with bulk storage queries
pub async fn neurons(
    client: &BittensorClient,
    netuid: u16,
    _block: Option<u64>,
) -> Result<Vec<NeuronInfo>> {
    // First get the count
    let n_key = vec![Value::u128(netuid as u128)];
    let n_value = client
        .storage_with_keys(SUBTENSOR_MODULE, "SubnetworkN", n_key.clone())
        .await?
        .ok_or_else(|| anyhow::anyhow!("Subnet {} not found", netuid))?;
    let n = decode_u64(&n_value).context("Failed to decode SubnetworkN")?;

    if n == 0 {
        return Ok(vec![]);
    }

    // Step 1: Fetch all vector storages for the subnet (these are bulk already)
    let rank_vec = fetch_vec_u16(client, "Rank", &n_key).await?;
    let trust_vec = fetch_vec_u16(client, "Trust", &n_key).await?;
    let consensus_vec = fetch_vec_u16(client, "Consensus", &n_key).await?;
    let validator_trust_vec = fetch_vec_u16(client, "ValidatorTrust", &n_key).await?;
    let incentive_vec = fetch_vec_u16(client, "Incentive", &n_key).await?;
    let dividends_vec = fetch_vec_u16(client, "Dividends", &n_key).await?;
    let active_vec = fetch_vec_bool(client, "Active", &n_key).await?;
    let last_update_vec = fetch_vec_u64(client, "LastUpdate", &n_key).await?;
    let emission_vec = fetch_vec_u128(client, "Emission", &n_key).await?;
    let validator_permit_vec = fetch_vec_bool(client, "ValidatorPermit", &n_key).await?;
    let pruning_scores_vec = fetch_vec_u16(client, "PruningScores", &n_key).await?;

    // Step 2: Batch fetch all hotkeys
    let mut hotkeys = Vec::with_capacity(n as usize);
    let mut futures = FuturesUnordered::new();

    for uid in 0..n {
        let uid_key = vec![Value::u128(netuid as u128), Value::u128(uid as u128)];
        let client_ref = client;
        futures.push(async move {
            let hotkey_val = client_ref
                .storage_with_keys(SUBTENSOR_MODULE, "Keys", uid_key)
                .await
                .ok()
                .flatten();
            (uid, hotkey_val)
        });
    }

    while let Some((uid, hotkey_val)) = futures.next().await {
        if let Some(val) = hotkey_val {
            if let Ok(hotkey) = decode_account_id32(&val) {
                hotkeys.push((uid, hotkey));
            }
        }
    }

    // Step 3: Batch fetch all coldkeys (owners)
    let mut coldkeys = HashMap::new();
    let mut futures = FuturesUnordered::new();

    for (uid, hotkey) in &hotkeys {
        let owner_key = vec![Value::from_bytes(&hotkey.encode())];
        let client_ref = client;
        let hk = hotkey.clone();
        let u = *uid;
        futures.push(async move {
            let coldkey_val = client_ref
                .storage_with_keys(SUBTENSOR_MODULE, "Owner", owner_key)
                .await
                .ok()
                .flatten();
            (u, hk, coldkey_val)
        });
    }

    while let Some((uid, hotkey, coldkey_val)) = futures.next().await {
        if let Some(val) = coldkey_val {
            if let Ok(coldkey) = decode_account_id32(&val) {
                coldkeys.insert(uid, (hotkey, coldkey));
            }
        }
    }

    // Step 4: Batch fetch all stakes
    let mut stakes = HashMap::new();
    let mut futures = FuturesUnordered::new();

    for (uid, (hotkey, _)) in &coldkeys {
        let stake_key = vec![
            Value::from_bytes(&hotkey.encode()),
            Value::u128(netuid as u128),
        ];
        let client_ref = client;
        let u = *uid;
        futures.push(async move {
            let stake_val = client_ref
                .storage_with_keys(SUBTENSOR_MODULE, "TotalHotkeyAlpha", stake_key)
                .await
                .ok()
                .flatten();
            (u, stake_val)
        });
    }

    while let Some((uid, stake_val)) = futures.next().await {
        if let Some(val) = stake_val {
            if let Ok(stake) = decode_u128(&val) {
                stakes.insert(uid, stake);
            }
        }
    }

    // Step 4.5: Batch fetch all root_stakes (TAO stake on root subnet, netuid 0)
    let mut root_stakes = HashMap::new();
    let mut futures = FuturesUnordered::new();

    for (uid, (hotkey, _)) in &coldkeys {
        let root_stake_key = vec![
            Value::from_bytes(&hotkey.encode()),
            Value::u128(0u128), // NetUid::ROOT
        ];
        let client_ref = client;
        let u = *uid;
        futures.push(async move {
            let root_stake_val = client_ref
                .storage_with_keys(SUBTENSOR_MODULE, "TotalHotkeyAlpha", root_stake_key)
                .await
                .ok()
                .flatten();
            (u, root_stake_val)
        });
    }

    while let Some((uid, root_stake_val)) = futures.next().await {
        if let Some(val) = root_stake_val {
            if let Ok(root_stake) = decode_u128(&val) {
                root_stakes.insert(uid, root_stake);
            }
        }
    }

    // Step 5: Build all neurons from the collected data
    let mut neurons = Vec::new();

    for (uid, (hotkey, coldkey)) in coldkeys {
        let idx = uid as usize;

        // Get values from vectors
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
        let emission = emission_raw as f64 / 1e9;
        let pruning_score = pruning_scores_vec.get(idx).copied().unwrap_or(0) as u64;

        let total_stake = stakes.get(&uid).copied().unwrap_or(0);
        let root_stake = root_stakes.get(&uid).copied().unwrap_or(0);

        neurons.push(NeuronInfo::create(
            uid,
            netuid,
            hotkey,
            coldkey,
            total_stake,
            HashMap::new(), // stake_dict
            total_stake,
            root_stake,
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
            0,          // version
            Vec::new(), // weights
            Vec::new(), // bonds
            pruning_score,
            None, // prometheus_info
            None, // axon_info
        ));
    }

    neurons.sort_by_key(|n| n.uid);
    Ok(neurons)
}

// Helper functions
async fn fetch_vec_u16(
    client: &BittensorClient,
    storage: &str,
    keys: &[Value],
) -> Result<Vec<u16>> {
    client
        .storage_with_keys(SUBTENSOR_MODULE, storage, keys.to_vec())
        .await?
        .and_then(|v| decode_vec_u16(&v).ok())
        .ok_or_else(|| anyhow::anyhow!("{} not found", storage))
}

async fn fetch_vec_u64(
    client: &BittensorClient,
    storage: &str,
    keys: &[Value],
) -> Result<Vec<u64>> {
    client
        .storage_with_keys(SUBTENSOR_MODULE, storage, keys.to_vec())
        .await?
        .and_then(|v| decode_vec_u64(&v).ok())
        .ok_or_else(|| anyhow::anyhow!("{} not found", storage))
}

async fn fetch_vec_u128(
    client: &BittensorClient,
    storage: &str,
    keys: &[Value],
) -> Result<Vec<u128>> {
    client
        .storage_with_keys(SUBTENSOR_MODULE, storage, keys.to_vec())
        .await?
        .and_then(|v| decode_vec_u128(&v).ok())
        .ok_or_else(|| anyhow::anyhow!("{} not found", storage))
}

async fn fetch_vec_bool(
    client: &BittensorClient,
    storage: &str,
    keys: &[Value],
) -> Result<Vec<bool>> {
    client
        .storage_with_keys(SUBTENSOR_MODULE, storage, keys.to_vec())
        .await?
        .and_then(|v| decode_vec_bool(&v).ok())
        .ok_or_else(|| anyhow::anyhow!("{} not found", storage))
}

/// Get a specific neuron by subnet and UID
pub async fn neuron(
    client: &BittensorClient,
    netuid: u16,
    uid: u64,
    block: Option<u64>,
) -> Result<Option<NeuronInfo>> {
    query_neuron_from_storage(client, netuid, uid, block).await
}

/// Query neuron information from storage
pub async fn query_neuron_from_storage(
    client: &BittensorClient,
    netuid: u16,
    uid: u64,
    _block: Option<u64>,
) -> Result<Option<NeuronInfo>> {
    let uid_key = vec![Value::u128(netuid as u128), Value::u128(uid as u128)];

    let hotkey_val = client
        .storage_with_keys(SUBTENSOR_MODULE, "Keys", uid_key.clone())
        .await?;

    let hotkey = match hotkey_val {
        Some(val) => decode_account_id32(&val)?,
        None => return Ok(None),
    };

    let owner_key = vec![Value::from_bytes(&hotkey.encode())];
    let coldkey_val = client
        .storage_with_keys(SUBTENSOR_MODULE, "Owner", owner_key)
        .await?
        .ok_or_else(|| {
            anyhow::anyhow!("Coldkey (Owner) not found for hotkey on subnet {}", netuid)
        })?;
    let coldkey = decode_account_id32(&coldkey_val)?;

    let netuid_key = vec![Value::u128(netuid as u128)];
    let idx = uid as usize;

    let rank_vec = fetch_vec_u16_storage(client, SUBTENSOR_MODULE, "Rank", netuid_key.clone()).await?;
    let trust_vec = fetch_vec_u16_storage(client, SUBTENSOR_MODULE, "Trust", netuid_key.clone()).await?;
    let consensus_vec = fetch_vec_u16_storage(client, SUBTENSOR_MODULE, "Consensus", netuid_key.clone()).await?;
    let validator_trust_vec = fetch_vec_u16_storage(client, SUBTENSOR_MODULE, "ValidatorTrust", netuid_key.clone()).await?;
    let incentive_vec = fetch_vec_u16_storage(client, SUBTENSOR_MODULE, "Incentive", netuid_key.clone()).await?;
    let dividends_vec = fetch_vec_u16_storage(client, SUBTENSOR_MODULE, "Dividends", netuid_key.clone()).await?;
    let active_vec = fetch_vec_bool_storage(client, SUBTENSOR_MODULE, "Active", netuid_key.clone()).await?;
    let last_update_vec = fetch_vec_u64_storage(client, SUBTENSOR_MODULE, "LastUpdate", netuid_key.clone()).await?;
    let emission_vec = fetch_vec_u128_storage(client, SUBTENSOR_MODULE, "Emission", netuid_key.clone()).await?;
    let validator_permit_vec = fetch_vec_bool_storage(client, SUBTENSOR_MODULE, "ValidatorPermit", netuid_key.clone()).await?;
    let pruning_scores_vec = fetch_vec_u16_storage(client, SUBTENSOR_MODULE, "PruningScores", netuid_key.clone()).await?;

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
    let emission = emission_raw as f64 / 1e9;
    let pruning_score = pruning_scores_vec.get(idx).copied().unwrap_or(0) as u64;

    let version = fetch_u64_storage_opt(client, SUBTENSOR_MODULE, "Version", uid_key.clone())
        .await
        .unwrap_or(None)
        .unwrap_or(0);

    let stake_key = vec![
        Value::from_bytes(&hotkey.encode()),
        Value::u128(netuid as u128),
    ];
    let total_stake = if let Some(total_stake_val) = client
        .storage_with_keys(SUBTENSOR_MODULE, "TotalHotkeyAlpha", stake_key.clone())
        .await?
    {
        decode_u128(&total_stake_val)
            .map_err(|e| anyhow::anyhow!("Failed to decode TotalHotkeyAlpha: {}", e))?
    } else {
        0u128
    };

    let root_stake_key = vec![
        Value::from_bytes(&hotkey.encode()),
        Value::u128(0u128),
    ];
    let root_stake = if let Some(root_stake_val) = client
        .storage_with_keys(SUBTENSOR_MODULE, "TotalHotkeyAlpha", root_stake_key)
        .await?
    {
        decode_u128(&root_stake_val)
            .map_err(|e| anyhow::anyhow!("Failed to decode TotalHotkeyAlpha (root): {}", e))
            .unwrap_or(0u128)
    } else {
        0u128
    };

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

    let axon_keys = vec![Value::u128(netuid as u128), Value::u128(uid as u128)];
    let axon_info = fetch_axon_info(client, SUBTENSOR_MODULE, "Axons", axon_keys.clone()).await;
    let prometheus_info =
        fetch_prometheus_info(client, SUBTENSOR_MODULE, "Prometheus", axon_keys).await;

    let storage_index: u64 = ((netuid as u64) << 16) | 0u64;
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
    let bonds: Vec<Vec<u64>> = bonds_pairs.into_iter().map(|(a, b)| vec![a, b]).collect();

    Ok(Some(NeuronInfo::create(
        uid,
        netuid,
        hotkey,
        coldkey,
        total_stake,
        stake_dict,
        total_stake,
        root_stake,
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

/// Fetch AxonInfo from storage
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

/// Fetch PrometheusInfo from storage
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

// Helper functions for storage queries
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
