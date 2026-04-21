//! Neuron queries — UID lookups, neuron info, neuron count, and per-UID metric vectors.

use bittensor_core::balance::Balance;
use bittensor_core::config::SubtensorConfig;
use bittensor_core::error::BittensorError;
use bittensor_core::types::{AxonInfo, NeuronInfo, NeuronInfoLite, PrometheusInfo};
use subxt::OnlineClient;

use crate::client::ClientAtBlock;
use crate::generated::subtensor;

type Result<T> = std::result::Result<T, BittensorError>;

async fn at_block(client: &OnlineClient<SubtensorConfig>) -> Result<ClientAtBlock> {
    client.at_current_block().await.map_err(|e| BittensorError::Rpc(e.to_string()))
}

fn decode_val<T>(opt: Option<subxt::storage::StorageValue<'_, T>>) -> T
where
    T: subxt::ext::scale_decode::DecodeAsType + Default,
{
    opt.and_then(|v| v.decode().ok()).unwrap_or_default()
}

macro_rules! fetch_decode {
    ($at:expr, $addr:expr, $method:ident, $keys:expr) => {{
        let opt = $at
            .storage()
            .try_fetch($addr.$method(), $keys)
            .await
            .map_err(|e| BittensorError::Rpc(e.to_string()))?;
        decode_val(opt)
    }};
}

/// Fetch full [`NeuronInfo`] for a UID in a subnet, including weights and bonds.
///
/// Returns `None` if the UID is not active.
pub async fn get_neuron(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    uid: u16,
) -> Result<Option<NeuronInfo>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();

    let active_vec: Vec<bool> = decode_val(
        at.storage()
            .try_fetch(addr.active(), (netuid,))
            .await
            .map_err(|e| BittensorError::Rpc(e.to_string()))?,
    );

    let active = active_vec.get(uid as usize).copied().unwrap_or(false);
    if !active {
        return Ok(None);
    }

    let rank_vec: Vec<u16> = decode_val(
        at.storage()
            .try_fetch(addr.rank(), (netuid,))
            .await
            .map_err(|e| BittensorError::Rpc(e.to_string()))?,
    );
    let trust_vec: Vec<u16> = decode_val(
        at.storage()
            .try_fetch(addr.trust(), (netuid,))
            .await
            .map_err(|e| BittensorError::Rpc(e.to_string()))?,
    );
    let consensus_vec: Vec<u16> = decode_val(
        at.storage()
            .try_fetch(addr.consensus(), (netuid,))
            .await
            .map_err(|e| BittensorError::Rpc(e.to_string()))?,
    );
    let incentive_vec: Vec<u16> = decode_val(
        at.storage()
            .try_fetch(addr.incentive(), (netuid,))
            .await
            .map_err(|e| BittensorError::Rpc(e.to_string()))?,
    );
    let dividends_vec: Vec<u16> = decode_val(
        at.storage()
            .try_fetch(addr.dividends(), (netuid,))
            .await
            .map_err(|e| BittensorError::Rpc(e.to_string()))?,
    );
    let emission_vec: Vec<u64> = decode_val(
        at.storage()
            .try_fetch(addr.emission(), (netuid,))
            .await
            .map_err(|e| BittensorError::Rpc(e.to_string()))?,
    );
    let last_update_vec: Vec<u64> = decode_val(
        at.storage()
            .try_fetch(addr.last_update(), (netuid,))
            .await
            .map_err(|e| BittensorError::Rpc(e.to_string()))?,
    );
    let validator_trust_vec: Vec<u16> = decode_val(
        at.storage()
            .try_fetch(addr.validator_trust(), (netuid,))
            .await
            .map_err(|e| BittensorError::Rpc(e.to_string()))?,
    );

    let rank = rank_vec.get(uid as usize).copied().unwrap_or(0);
    let trust = trust_vec.get(uid as usize).copied().unwrap_or(0);
    let consensus = consensus_vec.get(uid as usize).copied().unwrap_or(0);
    let incentive = incentive_vec.get(uid as usize).copied().unwrap_or(0);
    let dividends = dividends_vec.get(uid as usize).copied().unwrap_or(0);
    let emission = emission_vec.get(uid as usize).copied().unwrap_or(0);
    let last_update = last_update_vec.get(uid as usize).copied().unwrap_or(0);
    let validator_trust = validator_trust_vec.get(uid as usize).copied().unwrap_or(0);

    let hotkey_raw: Option<subxt::utils::AccountId32> = at
        .storage()
        .try_fetch(addr.keys(), (netuid, uid))
        .await
        .map_err(|e| BittensorError::Rpc(e.to_string()))?
        .and_then(|v| v.decode().ok());
    let hotkey = hotkey_raw.as_ref().map(|h| h.to_string()).unwrap_or_default();

    let axon_info = if let Some(ref hk) = hotkey_raw {
        let axon_raw: Option<subtensor::runtime_types::pallet_subtensor::pallet::AxonInfo> = at
            .storage()
            .try_fetch(addr.axons(), (netuid, *hk))
            .await
            .map_err(|e| BittensorError::Rpc(e.to_string()))?
            .and_then(|v| v.decode().ok());

        axon_raw.map(|a| AxonInfo {
            ip: a.ip as u64,
            port: a.port,
            ip_type: a.ip_type,
            protocol: a.protocol,
            version: a.version,
            hotkey: hotkey.clone(),
            coldkey: String::new(),
        })
    } else {
        None
    };

    let prometheus_info = if let Some(ref hk) = hotkey_raw {
        let prom_raw: Option<subtensor::runtime_types::pallet_subtensor::pallet::PrometheusInfo> =
            at.storage()
                .try_fetch(addr.prometheus(), (netuid, *hk))
                .await
                .map_err(|e| BittensorError::Rpc(e.to_string()))?
                .and_then(|v| v.decode().ok());

        prom_raw.map(|p| PrometheusInfo {
            ip: p.ip as u64,
            port: p.port,
            version: p.version,
            block: p.block,
        })
    } else {
        None
    };

    let weights: Vec<(u16, u16)> = decode_val(
        at.storage()
            .try_fetch(addr.weights(), (netuid, uid))
            .await
            .map_err(|e| BittensorError::Rpc(e.to_string()))?,
    );

    let bonds: Vec<(u16, u16)> = decode_val(
        at.storage()
            .try_fetch(addr.bonds(), (netuid, uid))
            .await
            .map_err(|e| BittensorError::Rpc(e.to_string()))?,
    );

    Ok(Some(NeuronInfo {
        uid,
        netuid,
        active,
        stake: Balance::ZERO,
        rank,
        trust,
        consensus,
        incentive,
        dividend: dividends,
        emission,
        prometheus_info,
        axon_info,
        hotkey,
        coldkey: String::new(),
        last_update,
        validator_trust,
        weights: weights.into_iter().flat_map(|(a, b)| vec![a, b]).collect(),
        bonds: bonds.into_iter().flat_map(|(a, b)| vec![a, b]).collect(),
        stake_dict: vec![],
    }))
}

/// Fetch a lightweight [`NeuronInfoLite`] for a UID in a subnet (no weights/bonds).
///
/// Returns `None` if the UID is not active.
pub async fn get_neuron_lite(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    uid: u16,
) -> Result<Option<NeuronInfoLite>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();

    let active_vec: Vec<bool> = decode_val(
        at.storage()
            .try_fetch(addr.active(), (netuid,))
            .await
            .map_err(|e| BittensorError::Rpc(e.to_string()))?,
    );
    let active = active_vec.get(uid as usize).copied().unwrap_or(false);

    if !active {
        return Ok(None);
    }

    let rank_vec: Vec<u16> = decode_val(
        at.storage()
            .try_fetch(addr.rank(), (netuid,))
            .await
            .map_err(|e| BittensorError::Rpc(e.to_string()))?,
    );
    let trust_vec: Vec<u16> = decode_val(
        at.storage()
            .try_fetch(addr.trust(), (netuid,))
            .await
            .map_err(|e| BittensorError::Rpc(e.to_string()))?,
    );
    let consensus_vec: Vec<u16> = decode_val(
        at.storage()
            .try_fetch(addr.consensus(), (netuid,))
            .await
            .map_err(|e| BittensorError::Rpc(e.to_string()))?,
    );
    let incentive_vec: Vec<u16> = decode_val(
        at.storage()
            .try_fetch(addr.incentive(), (netuid,))
            .await
            .map_err(|e| BittensorError::Rpc(e.to_string()))?,
    );

    let rank = rank_vec.get(uid as usize).copied().unwrap_or(0);
    let trust = trust_vec.get(uid as usize).copied().unwrap_or(0);
    let consensus = consensus_vec.get(uid as usize).copied().unwrap_or(0);
    let incentive = incentive_vec.get(uid as usize).copied().unwrap_or(0);

    let hotkey_raw: Option<subxt::utils::AccountId32> = at
        .storage()
        .try_fetch(addr.keys(), (netuid, uid))
        .await
        .map_err(|e| BittensorError::Rpc(e.to_string()))?
        .and_then(|v| v.decode().ok());
    let hotkey = hotkey_raw.as_ref().map(|h| h.to_string()).unwrap_or_default();

    Ok(Some(NeuronInfoLite {
        uid,
        hotkey,
        coldkey: String::new(),
        active,
        stake: Balance::ZERO,
        rank,
        trust,
        consensus,
        incentive,
    }))
}

/// Resolve the UID for a hotkey in a subnet.
///
/// Returns `None` if the hotkey is not a member of the subnet.
pub async fn get_uid_for_hotkey(
    client: &OnlineClient<SubtensorConfig>,
    hotkey: &subxt::utils::AccountId32,
    netuid: u16,
) -> Result<Option<u16>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();

    let is_member: bool = at
        .storage()
        .try_fetch(addr.is_network_member(), (*hotkey, netuid))
        .await
        .map_err(|e| BittensorError::Rpc(e.to_string()))?
        .and_then(|v| v.decode().ok())
        .unwrap_or(false);

    if !is_member {
        return Ok(None);
    }

    let uid: u16 = at
        .storage()
        .try_fetch(addr.uids(), (netuid, *hotkey))
        .await
        .map_err(|e| BittensorError::Rpc(e.to_string()))?
        .and_then(|v| v.decode().ok())
        .unwrap_or(u16::MAX);

    if uid == u16::MAX { Ok(None) } else { Ok(Some(uid)) }
}

/// Fetch full [`NeuronInfo`] for a hotkey in a subnet (resolves UID first).
pub async fn get_neuron_for_pubkey_and_subnet(
    client: &OnlineClient<SubtensorConfig>,
    hotkey: &subxt::utils::AccountId32,
    netuid: u16,
) -> Result<Option<NeuronInfo>> {
    let uid = get_uid_for_hotkey(client, hotkey, netuid).await?;
    match uid {
        Some(u) => get_neuron(client, netuid, u).await,
        None => Ok(None),
    }
}

/// Fetch the current neuron count (number of registered UIDs) in a subnet.
pub async fn get_neuron_count(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u16> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let n: u16 = decode_val(
        at.storage()
            .try_fetch(addr.subnetwork_n(), (netuid,))
            .await
            .map_err(|e| BittensorError::Rpc(e.to_string()))?,
    );
    Ok(n)
}

/// Fetch the maximum allowed UIDs for a subnet.
pub async fn get_max_neurons(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u16> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let max: u16 = decode_val(
        at.storage()
            .try_fetch(addr.max_allowed_uids(), (netuid,))
            .await
            .map_err(|e| BittensorError::Rpc(e.to_string()))?,
    );
    Ok(max)
}

/// Fetch all neurons in a subnet as a list of [`NeuronInfoLite`].
pub async fn get_neurons(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<Vec<NeuronInfoLite>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();

    let n: u16 = decode_val(
        at.storage()
            .try_fetch(addr.subnetwork_n(), (netuid,))
            .await
            .map_err(|e| BittensorError::Rpc(e.to_string()))?,
    );

    let active_vec: Vec<bool> = decode_val(
        at.storage()
            .try_fetch(addr.active(), (netuid,))
            .await
            .map_err(|e| BittensorError::Rpc(e.to_string()))?,
    );
    let rank_vec: Vec<u16> = decode_val(
        at.storage()
            .try_fetch(addr.rank(), (netuid,))
            .await
            .map_err(|e| BittensorError::Rpc(e.to_string()))?,
    );
    let trust_vec: Vec<u16> = decode_val(
        at.storage()
            .try_fetch(addr.trust(), (netuid,))
            .await
            .map_err(|e| BittensorError::Rpc(e.to_string()))?,
    );
    let consensus_vec: Vec<u16> = decode_val(
        at.storage()
            .try_fetch(addr.consensus(), (netuid,))
            .await
            .map_err(|e| BittensorError::Rpc(e.to_string()))?,
    );
    let incentive_vec: Vec<u16> = decode_val(
        at.storage()
            .try_fetch(addr.incentive(), (netuid,))
            .await
            .map_err(|e| BittensorError::Rpc(e.to_string()))?,
    );

    let mut neurons = Vec::with_capacity(n as usize);
    for uid in 0..n {
        let active = active_vec.get(uid as usize).copied().unwrap_or(false);
        let rank = rank_vec.get(uid as usize).copied().unwrap_or(0);
        let trust = trust_vec.get(uid as usize).copied().unwrap_or(0);
        let consensus = consensus_vec.get(uid as usize).copied().unwrap_or(0);
        let incentive = incentive_vec.get(uid as usize).copied().unwrap_or(0);

        let hotkey_raw: Option<subxt::utils::AccountId32> = at
            .storage()
            .try_fetch(addr.keys(), (netuid, uid))
            .await
            .map_err(|e| BittensorError::Rpc(e.to_string()))?
            .and_then(|v| v.decode().ok());
        let hotkey = hotkey_raw.as_ref().map(|h| h.to_string()).unwrap_or_default();

        neurons.push(NeuronInfoLite {
            uid,
            hotkey,
            coldkey: String::new(),
            active,
            stake: Balance::ZERO,
            rank,
            trust,
            consensus,
            incentive,
        });
    }

    Ok(neurons)
}

/// Fetch the rank vector for a subnet (one entry per UID).
pub async fn get_rank(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<Vec<u16>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: Vec<u16> = fetch_decode!(at, addr, rank, (netuid,));
    Ok(v)
}

/// Fetch the trust vector for a subnet (one entry per UID).
pub async fn get_trust(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<Vec<u16>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: Vec<u16> = fetch_decode!(at, addr, trust, (netuid,));
    Ok(v)
}

/// Fetch the consensus vector for a subnet (one entry per UID).
pub async fn get_consensus(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<Vec<u16>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: Vec<u16> = fetch_decode!(at, addr, consensus, (netuid,));
    Ok(v)
}

/// Fetch the incentive vector for a subnet (one entry per UID).
pub async fn get_incentive(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<Vec<u16>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: Vec<u16> = fetch_decode!(at, addr, incentive, (netuid,));
    Ok(v)
}

/// Fetch the dividends vector for a subnet (one entry per UID).
pub async fn get_dividends(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<Vec<u16>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: Vec<u16> = fetch_decode!(at, addr, dividends, (netuid,));
    Ok(v)
}

/// Fetch the emission vector for a subnet (one entry per UID).
pub async fn get_emission(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<Vec<u64>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: Vec<u64> = fetch_decode!(at, addr, emission, (netuid,));
    Ok(v)
}

/// Fetch the active vector for a subnet (one entry per UID).
pub async fn get_active(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<Vec<bool>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: Vec<bool> = fetch_decode!(at, addr, active, (netuid,));
    Ok(v)
}

/// Fetch the last-update block vector for a subnet (one entry per UID).
pub async fn get_last_update(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<Vec<u64>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: Vec<u64> = fetch_decode!(at, addr, last_update, (netuid,));
    Ok(v)
}

/// Fetch the validator-permit vector for a subnet (one entry per UID).
pub async fn get_validator_permit(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<Vec<bool>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: Vec<bool> = fetch_decode!(at, addr, validator_permit, (netuid,));
    Ok(v)
}

/// Fetch the validator-trust vector for a subnet (one entry per UID).
pub async fn get_validator_trust(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<Vec<u16>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: Vec<u16> = fetch_decode!(at, addr, validator_trust, (netuid,));
    Ok(v)
}

/// Fetch the bonds for a given UID in a subnet.
pub async fn get_bonds(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    uid: u16,
) -> Result<Vec<(u16, u16)>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: Vec<(u16, u16)> = fetch_decode!(at, addr, bonds, (netuid, uid));
    Ok(v)
}

/// Fetch the block at which a UID was registered in a subnet.
pub async fn get_block_at_registration(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    uid: u16,
) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, block_at_registration, (netuid, uid));
    Ok(v)
}

/// Fetch the UID for a hotkey in a subnet.
pub async fn get_uids(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    hotkey: &subxt::utils::AccountId32,
) -> Result<u16> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u16 = fetch_decode!(at, addr, uids, (netuid, *hotkey));
    Ok(v)
}

/// Fetch the hotkey for a UID in a subnet.
///
/// Returns `None` if the UID is not assigned.
pub async fn get_keys(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    uid: u16,
) -> Result<Option<subxt::utils::AccountId32>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: Option<subxt::utils::AccountId32> = at
        .storage()
        .try_fetch(addr.keys(), (netuid, uid))
        .await
        .map_err(|e| BittensorError::Rpc(e.to_string()))?
        .and_then(|val| val.decode().ok());
    Ok(v)
}

/// Fetch the loaded emission vector for a subnet.
///
/// Each entry is `(hotkey, server_emission, validator_emission)`.
pub async fn get_loaded_emission(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<Vec<(subxt::utils::AccountId32, u64, u64)>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: Vec<(subxt::utils::AccountId32, u64, u64)> =
        fetch_decode!(at, addr, loaded_emission, (netuid,));
    Ok(v)
}

/// Fetch the pruning scores vector for a subnet (one entry per UID).
pub async fn get_pruning_scores(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<Vec<u16>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: Vec<u16> = fetch_decode!(at, addr, pruning_scores, (netuid,));
    Ok(v)
}

/// Fetch the stake-weight vector for a subnet.
pub async fn get_stake_weight(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<Vec<u16>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: Vec<u16> = fetch_decode!(at, addr, stake_weight, (netuid,));
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neuron_info_lite_construction() {
        let info = NeuronInfoLite {
            uid: 42,
            hotkey: "0xabc".into(),
            coldkey: "0xdef".into(),
            active: true,
            stake: Balance::from_tao(100.0),
            rank: 10,
            trust: 5,
            consensus: 3,
            incentive: 7,
        };
        assert_eq!(info.uid, 42);
        assert!(info.active);
    }

    #[test]
    fn neuron_info_construction() {
        let info = NeuronInfo {
            uid: 1,
            netuid: 1,
            active: true,
            stake: Balance::from_tao(50.0),
            rank: 5,
            trust: 3,
            consensus: 2,
            incentive: 1,
            dividend: 4,
            emission: 1000,
            prometheus_info: None,
            axon_info: None,
            hotkey: "0xabc".into(),
            coldkey: "0xdef".into(),
            last_update: 500,
            validator_trust: 8,
            weights: vec![10, 20],
            bonds: vec![5, 10],
            stake_dict: vec![],
        };
        assert_eq!(info.netuid, 1);
    }

    #[test]
    fn rank_vector_default() {
        let v: Vec<u16> = Vec::new();
        assert!(v.is_empty());
    }
}
