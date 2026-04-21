//! Subnet queries — subnet metadata, hyperparameters, and existence checks.

use bittensor_core::config::SubtensorConfig;
use bittensor_core::error::BittensorError;
use bittensor_core::types::{ChainIdentity, SubnetHyperparameters, SubnetInfo};
use subxt::OnlineClient;

use crate::client::ClientAtBlock;
use crate::generated::subtensor;

type Result<T> = std::result::Result<T, BittensorError>;

async fn at_block(client: &OnlineClient<SubtensorConfig>) -> Result<ClientAtBlock> {
    client.at_current_block().await.map_err(|e| BittensorError::Rpc(e.to_string()))
}

fn decode_opt<T>(opt: Option<subxt::storage::StorageValue<'_, T>>) -> T
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
        decode_opt(opt)
    }};
}

/// Fetch subnet metadata (owner, tempo, identity, UID count).
///
/// Returns `None` if the subnet does not exist.
pub async fn get_subnet_info(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<Option<SubnetInfo>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();

    let exists: bool = fetch_decode!(at, addr, networks_added, (netuid,));
    if !exists {
        return Ok(None);
    }

    let name = get_subnet_name(client, netuid).await?.unwrap_or_default();

    let owner_hk: Option<subxt::utils::AccountId32> = at
        .storage()
        .try_fetch(addr.subnet_owner(), (netuid,))
        .await
        .map_err(|e| BittensorError::Rpc(e.to_string()))?
        .and_then(|v| v.decode().ok());
    let owner_hotkey = owner_hk.map(|o| o.to_string()).unwrap_or_default();

    let tempo: u16 = fetch_decode!(at, addr, tempo, (netuid,));

    let max_uid: u16 = fetch_decode!(at, addr, subnetwork_n, (netuid,));

    let identity_raw: Option<subtensor::runtime_types::pallet_subtensor::pallet::SubnetIdentityV3> =
        at.storage()
            .try_fetch(addr.subnet_identities_v3(), (netuid,))
            .await
            .map_err(|e| BittensorError::Rpc(e.to_string()))?
            .and_then(|v| v.decode().ok());

    let identity = identity_raw.map(|id| ChainIdentity {
        netuid,
        name: String::from_utf8_lossy(&id.subnet_name).into_owned(),
        symbol: String::from_utf8_lossy(&id.subnet_name).into_owned(),
    });

    Ok(Some(SubnetInfo {
        netuid,
        name,
        owner_hotkey,
        tempo,
        subnet_identity: identity,
        maximum_uid: max_uid,
        modality: 0,
        network_uid: netuid,
    }))
}

/// Fetch the full hyperparameter set for a subnet.
///
/// Returns `None` if the subnet does not exist.
pub async fn get_subnet_hyperparameters(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<Option<SubnetHyperparameters>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();

    let rho: u16 = fetch_decode!(at, addr, rho, (netuid,));
    let kappa: u16 = fetch_decode!(at, addr, kappa, (netuid,));
    let difficulty: u64 = fetch_decode!(at, addr, difficulty, (netuid,));
    let burn: u64 = fetch_decode!(at, addr, burn, (netuid,));
    let immunity: u16 = fetch_decode!(at, addr, immunity_period, (netuid,));
    let min_burn: u64 = fetch_decode!(at, addr, min_burn, (netuid,));
    let max_burn: u64 = fetch_decode!(at, addr, max_burn, (netuid,));
    let weights_rate_limit: u64 = fetch_decode!(at, addr, weights_set_rate_limit, (netuid,));
    let weights_version: u64 = fetch_decode!(at, addr, weights_version_key, (netuid,));
    let max_weight_limit: u16 = fetch_decode!(at, addr, max_weights_limit, (netuid,));
    let scaling_law: u16 = fetch_decode!(at, addr, scaling_law_power, (netuid,));
    let subnetwork_n: u16 = fetch_decode!(at, addr, subnetwork_n, (netuid,));
    let max_n: u16 = fetch_decode!(at, addr, max_allowed_uids, (netuid,));
    let blocks_since: u64 = fetch_decode!(at, addr, blocks_since_last_step, (netuid,));
    let tempo: u16 = fetch_decode!(at, addr, tempo, (netuid,));
    let adj_alpha: u64 = fetch_decode!(at, addr, adjustment_alpha, (netuid,));
    let adj_interval: u16 = fetch_decode!(at, addr, adjustment_interval, (netuid,));
    let bonds_ma: u64 = fetch_decode!(at, addr, bonds_moving_average, (netuid,));
    let (alpha_high, alpha_low): (u16, u16) = fetch_decode!(at, addr, alpha_values, (netuid,));
    let liquid_alpha: bool = fetch_decode!(at, addr, liquid_alpha_on, (netuid,));

    Ok(Some(SubnetHyperparameters {
        rho,
        kappa,
        difficulty: difficulty as u32,
        burn,
        immunity_ratio: immunity,
        min_burn,
        max_burn,
        weights_rate_limit,
        weights_version: weights_version as u16,
        weights_min_stake: 0,
        max_weight_limit,
        scaling_law_power: scaling_law,
        subnetwork_n,
        max_n,
        blocks_since_last_step: blocks_since,
        tempo,
        adjustment_alpha: adj_alpha,
        adjustment_interval: adj_interval,
        bonds_moving_avg: bonds_ma,
        alpha_high,
        alpha_low,
        liquid_alpha_enabled: liquid_alpha,
    }))
}

/// Fetch the total number of subnets on chain.
pub async fn get_total_subnets(client: &OnlineClient<SubtensorConfig>) -> Result<u16> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let total: u16 = fetch_decode!(at, addr, total_networks, ());
    Ok(total)
}

/// Check whether a subnet exists on chain.
pub async fn subnet_exists(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<bool> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let exists: bool = fetch_decode!(at, addr, networks_added, (netuid,));
    Ok(exists)
}

/// Fetch the owner coldkey of a subnet.
pub async fn get_subnet_owner(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<Option<String>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let owner: Option<subxt::utils::AccountId32> = at
        .storage()
        .try_fetch(addr.subnet_owner(), (netuid,))
        .await
        .map_err(|e| BittensorError::Rpc(e.to_string()))?
        .and_then(|v| v.decode().ok());
    Ok(owner.map(|o| o.to_string()))
}

/// Fetch the display name of a subnet from its on-chain identity.
pub async fn get_subnet_name(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<Option<String>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let identity: Option<subtensor::runtime_types::pallet_subtensor::pallet::SubnetIdentityV3> = at
        .storage()
        .try_fetch(addr.subnet_identities_v3(), (netuid,))
        .await
        .map_err(|e| BittensorError::Rpc(e.to_string()))?
        .and_then(|v| v.decode().ok());
    Ok(identity.map(|id| String::from_utf8_lossy(&id.subnet_name).into_owned()))
}

/// Fetch the subnet owner hotkey (the hotkey that owns the subnet).
///
/// Returns `None` if the subnet has no owner hotkey set.
pub async fn get_subnet_owner_hotkey(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<Option<subxt::utils::AccountId32>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: Option<subxt::utils::AccountId32> = at
        .storage()
        .try_fetch(addr.subnet_owner_hotkey(), (netuid,))
        .await
        .map_err(|e| BittensorError::Rpc(e.to_string()))?
        .and_then(|val| val.decode().ok());
    Ok(v)
}

/// Fetch the tempo (blocks per epoch) for a subnet.
pub async fn get_tempo(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u16> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u16 = fetch_decode!(at, addr, tempo, (netuid,));
    Ok(v)
}

/// Fetch the current number of UIDs in a subnet.
pub async fn get_subnetwork_n(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u16> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u16 = fetch_decode!(at, addr, subnetwork_n, (netuid,));
    Ok(v)
}

/// Fetch the subnet mechanism type (e.g. Yuma variant).
pub async fn get_subnet_mechanism(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u16> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u16 = fetch_decode!(at, addr, subnet_mechanism, (netuid,));
    Ok(v)
}

/// Check if a hotkey is a member of a subnet.
pub async fn get_is_network_member(
    client: &OnlineClient<SubtensorConfig>,
    hotkey: &subxt::utils::AccountId32,
    netuid: u16,
) -> Result<bool> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: bool = fetch_decode!(at, addr, is_network_member, (*hotkey, netuid));
    Ok(v)
}

/// Check if registration is allowed for a subnet.
pub async fn get_network_registration_allowed(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<bool> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: bool = fetch_decode!(at, addr, network_registration_allowed, (netuid,));
    Ok(v)
}

/// Check if PoW registration is allowed for a subnet.
pub async fn get_network_pow_registration_allowed(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<bool> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: bool = fetch_decode!(at, addr, network_pow_registration_allowed, (netuid,));
    Ok(v)
}

/// Fetch the block at which a subnet was registered.
pub async fn get_network_registered_at(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, network_registered_at, (netuid,));
    Ok(v)
}

/// Fetch the minimum allowed UIDs for a subnet.
pub async fn get_min_allowed_uids(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u16> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u16 = fetch_decode!(at, addr, min_allowed_uids, (netuid,));
    Ok(v)
}

/// Fetch the maximum allowed UIDs for a subnet.
pub async fn get_max_allowed_uids(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u16> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u16 = fetch_decode!(at, addr, max_allowed_uids, (netuid,));
    Ok(v)
}

/// Fetch the maximum allowed validators for a subnet.
pub async fn get_max_allowed_validators(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u16> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u16 = fetch_decode!(at, addr, max_allowed_validators, (netuid,));
    Ok(v)
}

/// Fetch the immunity period for a subnet (in blocks).
pub async fn get_immunity_period(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u16> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u16 = fetch_decode!(at, addr, immunity_period, (netuid,));
    Ok(v)
}

/// Fetch the activity cutoff for a subnet (in blocks).
pub async fn get_activity_cutoff(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u16> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u16 = fetch_decode!(at, addr, activity_cutoff, (netuid,));
    Ok(v)
}

/// Fetch the max weights limit for a subnet.
pub async fn get_max_weights_limit(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u16> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u16 = fetch_decode!(at, addr, max_weights_limit, (netuid,));
    Ok(v)
}

/// Fetch the minimum allowed weights for a subnet.
pub async fn get_min_allowed_weights(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u16> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u16 = fetch_decode!(at, addr, min_allowed_weights, (netuid,));
    Ok(v)
}

/// Fetch the adjustment interval for a subnet.
pub async fn get_adjustment_interval(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u16> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u16 = fetch_decode!(at, addr, adjustment_interval, (netuid,));
    Ok(v)
}

/// Fetch the bonds moving average for a subnet.
pub async fn get_bonds_moving_average(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, bonds_moving_average, (netuid,));
    Ok(v)
}

/// Fetch the bonds penalty for a subnet.
pub async fn get_bonds_penalty(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u16> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u16 = fetch_decode!(at, addr, bonds_penalty, (netuid,));
    Ok(v)
}

/// Fetch whether bonds are reset for a subnet.
pub async fn get_bonds_reset_on(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<bool> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: bool = fetch_decode!(at, addr, bonds_reset_on, (netuid,));
    Ok(v)
}

/// Fetch the scaling law power for a subnet.
pub async fn get_scaling_law_power(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u16> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u16 = fetch_decode!(at, addr, scaling_law_power, (netuid,));
    Ok(v)
}

/// Fetch the target registrations per interval for a subnet.
pub async fn get_target_registrations_per_interval(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u16> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u16 = fetch_decode!(at, addr, target_registrations_per_interval, (netuid,));
    Ok(v)
}

/// Fetch the adjustment alpha for a subnet.
pub async fn get_adjustment_alpha(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, adjustment_alpha, (netuid,));
    Ok(v)
}

/// Check if liquid alpha is enabled for a subnet.
pub async fn get_liquid_alpha_on(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<bool> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: bool = fetch_decode!(at, addr, liquid_alpha_on, (netuid,));
    Ok(v)
}

/// Check if Yuma3 is enabled for a subnet.
pub async fn get_yuma3_on(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<bool> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: bool = fetch_decode!(at, addr, yuma3_on, (netuid,));
    Ok(v)
}

/// Fetch the alpha values (alpha_high, alpha_low) for a subnet.
pub async fn get_alpha_values(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<(u16, u16)> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: (u16, u16) = fetch_decode!(at, addr, alpha_values, (netuid,));
    Ok(v)
}

/// Check if subtoken is enabled for a subnet.
pub async fn get_subtoken_enabled(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<bool> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: bool = fetch_decode!(at, addr, subtoken_enabled, (netuid,));
    Ok(v)
}

/// Fetch the serving rate limit for a subnet (blocks between serve ops).
pub async fn get_serving_rate_limit(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, serving_rate_limit, (netuid,));
    Ok(v)
}

/// Fetch the burn cost for registering in a subnet.
pub async fn get_burn(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, burn, (netuid,));
    Ok(v)
}

/// Fetch the difficulty for PoW registration in a subnet.
pub async fn get_difficulty(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, difficulty, (netuid,));
    Ok(v)
}

/// Fetch the minimum burn for a subnet.
pub async fn get_min_burn(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, min_burn, (netuid,));
    Ok(v)
}

/// Fetch the maximum burn for a subnet.
pub async fn get_max_burn(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, max_burn, (netuid,));
    Ok(v)
}

/// Fetch the minimum difficulty for a subnet.
pub async fn get_min_difficulty(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, min_difficulty, (netuid,));
    Ok(v)
}

/// Fetch the maximum difficulty for a subnet.
pub async fn get_max_difficulty(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, max_difficulty, (netuid,));
    Ok(v)
}

/// Fetch the last block at which the subnet's burn/difficulty was adjusted.
pub async fn get_last_adjustment_block(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, last_adjustment_block, (netuid,));
    Ok(v)
}

/// Fetch the number of registrations in the current interval for a subnet.
pub async fn get_registrations_this_interval(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u16> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u16 = fetch_decode!(at, addr, registrations_this_interval, (netuid,));
    Ok(v)
}

/// Fetch the number of registrations in the current block for a subnet.
pub async fn get_registrations_this_block(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u16> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u16 = fetch_decode!(at, addr, registrations_this_block, (netuid,));
    Ok(v)
}

/// Fetch the rao recycled for registration in a subnet.
pub async fn get_rao_recycled_for_registration(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, rao_recycled_for_registration, (netuid,));
    Ok(v)
}

/// Fetch the tx rate limit (global, in blocks).
pub async fn get_tx_rate_limit(client: &OnlineClient<SubtensorConfig>) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, tx_rate_limit, ());
    Ok(v)
}

/// Fetch the EMA price halving blocks for a subnet.
pub async fn get_ema_price_halving_blocks(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, ema_price_halving_blocks, (netuid,));
    Ok(v)
}

/// Fetch the rho parameter for a subnet.
pub async fn get_rho(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u16> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u16 = fetch_decode!(at, addr, rho, (netuid,));
    Ok(v)
}

/// Fetch the kappa parameter for a subnet.
pub async fn get_kappa(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u16> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u16 = fetch_decode!(at, addr, kappa, (netuid,));
    Ok(v)
}

/// Fetch the alpha sigmoid steepness for a subnet (signed i16).
pub async fn get_alpha_sigmoid_steepness(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<i16> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: i16 = fetch_decode!(at, addr, alpha_sigmoid_steepness, (netuid,));
    Ok(v)
}

/// Fetch the voting power for a hotkey in a subnet.
pub async fn get_voting_power(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    hotkey: &subxt::utils::AccountId32,
) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, voting_power, (netuid, *hotkey));
    Ok(v)
}

/// Check if voting power tracking is enabled for a subnet.
pub async fn get_voting_power_tracking_enabled(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<bool> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: bool = fetch_decode!(at, addr, voting_power_tracking_enabled, (netuid,));
    Ok(v)
}

/// Fetch the max registrations per block for a subnet.
pub async fn get_max_registrations_per_block(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u16> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u16 = fetch_decode!(at, addr, max_registrations_per_block, (netuid,));
    Ok(v)
}

/// Fetch the validator prune length for a subnet.
pub async fn get_validator_prune_len(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, validator_prune_len, (netuid,));
    Ok(v)
}

/// Fetch the subnet locked amount for a subnet.
pub async fn get_subnet_locked(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, subnet_locked, (netuid,));
    Ok(v)
}

/// Fetch the largest locked amount for a subnet.
pub async fn get_largest_locked(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, largest_locked, (netuid,));
    Ok(v)
}

/// Check if transfers are toggled on for a subnet.
pub async fn get_transfer_toggle(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<bool> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: bool = fetch_decode!(at, addr, transfer_toggle, (netuid,));
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subnet_info_construction() {
        let si = SubnetInfo {
            netuid: 1,
            name: "root".into(),
            owner_hotkey: "0xabc".into(),
            tempo: 100,
            subnet_identity: None,
            maximum_uid: 256,
            modality: 0,
            network_uid: 1,
        };
        assert_eq!(si.netuid, 1);
    }

    #[test]
    fn hyperparams_default_fields() {
        let hp = SubnetHyperparameters {
            rho: 0,
            kappa: 0,
            difficulty: 0,
            burn: 0,
            immunity_ratio: 0,
            min_burn: 0,
            max_burn: 0,
            weights_rate_limit: 0,
            weights_version: 0,
            weights_min_stake: 0,
            max_weight_limit: 0,
            scaling_law_power: 0,
            subnetwork_n: 0,
            max_n: 0,
            blocks_since_last_step: 0,
            tempo: 0,
            adjustment_alpha: 0,
            adjustment_interval: 0,
            bonds_moving_avg: 0,
            alpha_high: 0,
            alpha_low: 0,
            liquid_alpha_enabled: false,
        };
        assert!(!hp.liquid_alpha_enabled);
    }
}
