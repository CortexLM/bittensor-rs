/// Bulk neuron queries for fetching all neurons at once
use crate::chain::BittensorClient;
use crate::types::NeuronInfo;
use crate::utils::value_decode::*;
use anyhow::{Context, Result};
use futures::stream::{FuturesUnordered, StreamExt};
use parity_scale_codec::Encode;
use std::collections::HashMap;
use subxt::dynamic::Value;

const SUBTENSOR_MODULE: &str = "SubtensorModule";

/// Get all neurons for a subnet with bulk storage queries
pub async fn neurons_bulk(
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

        neurons.push(NeuronInfo::create(
            uid,
            netuid,
            hotkey,
            coldkey,
            total_stake,
            HashMap::new(), // stake_dict
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
