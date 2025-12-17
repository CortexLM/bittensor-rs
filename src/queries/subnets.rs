//! Subnet-related queries

use crate::error::Result;
use crate::queries::chain_info::{decode_bool, decode_u16, decode_u64, query_storage_value};
use crate::types::SubnetHyperparameters;
use crate::utils::balance::Balance;
use subxt::dynamic::Value;
use subxt::OnlineClient;
use subxt::PolkadotConfig;

const SUBTENSOR_MODULE: &str = "SubtensorModule";

/// Get list of all subnet netuids
pub async fn get_all_subnet_netuids(client: &OnlineClient<PolkadotConfig>) -> Result<Vec<u16>> {
    // Query NetworksAdded storage map
    use subxt::dynamic::storage;
    
    let storage_query = storage(SUBTENSOR_MODULE, "NetworksAdded", Vec::<Value>::new());
    let mut results = client
        .storage()
        .at_latest()
        .await?
        .iter(storage_query)
        .await?;
    
    let mut netuids = Vec::new();
    while let Some(Ok(kv)) = results.next().await {
        let key = &kv.key_bytes;
        let value = kv.value;
        // Extract netuid from key (last 2 bytes)
        if key.len() >= 2 {
            let netuid_bytes = &key[key.len() - 2..];
            let netuid = u16::from_le_bytes([netuid_bytes[0], netuid_bytes[1]]);
            if decode_bool(&value).unwrap_or(false) {
                netuids.push(netuid);
            }
        }
    }
    
    netuids.sort();
    Ok(netuids)
}

/// Get number of subnets
pub async fn get_num_subnets(client: &OnlineClient<PolkadotConfig>) -> Result<u16> {
    let val = query_storage_value(client, SUBTENSOR_MODULE, "TotalNetworks", vec![]).await?;
    Ok(val.and_then(|v| decode_u16(&v)).unwrap_or(0))
}

/// Get tempo for a subnet
pub async fn get_tempo(client: &OnlineClient<PolkadotConfig>, netuid: u16) -> Result<u16> {
    let val = query_storage_value(
        client,
        SUBTENSOR_MODULE,
        "Tempo",
        vec![Value::u128(netuid as u128)],
    )
    .await?;
    Ok(val.and_then(|v| decode_u16(&v)).unwrap_or(360))
}

/// Get number of neurons in a subnet
pub async fn get_subnetwork_n(client: &OnlineClient<PolkadotConfig>, netuid: u16) -> Result<u16> {
    let val = query_storage_value(
        client,
        SUBTENSOR_MODULE,
        "SubnetworkN",
        vec![Value::u128(netuid as u128)],
    )
    .await?;
    Ok(val.and_then(|v| decode_u16(&v)).unwrap_or(0))
}

/// Get max neurons for a subnet
pub async fn get_max_allowed_uids(
    client: &OnlineClient<PolkadotConfig>,
    netuid: u16,
) -> Result<u16> {
    let val = query_storage_value(
        client,
        SUBTENSOR_MODULE,
        "MaxAllowedUids",
        vec![Value::u128(netuid as u128)],
    )
    .await?;
    Ok(val.and_then(|v| decode_u16(&v)).unwrap_or(256))
}

/// Get immunity period for a subnet
pub async fn get_immunity_period(
    client: &OnlineClient<PolkadotConfig>,
    netuid: u16,
) -> Result<u16> {
    let val = query_storage_value(
        client,
        SUBTENSOR_MODULE,
        "ImmunityPeriod",
        vec![Value::u128(netuid as u128)],
    )
    .await?;
    Ok(val.and_then(|v| decode_u16(&v)).unwrap_or(4096))
}

/// Get activity cutoff for a subnet
pub async fn get_activity_cutoff(
    client: &OnlineClient<PolkadotConfig>,
    netuid: u16,
) -> Result<u16> {
    let val = query_storage_value(
        client,
        SUBTENSOR_MODULE,
        "ActivityCutoff",
        vec![Value::u128(netuid as u128)],
    )
    .await?;
    Ok(val.and_then(|v| decode_u16(&v)).unwrap_or(5000))
}

/// Check if registration is allowed on a subnet
pub async fn get_registration_allowed(
    client: &OnlineClient<PolkadotConfig>,
    netuid: u16,
) -> Result<bool> {
    let val = query_storage_value(
        client,
        SUBTENSOR_MODULE,
        "NetworkRegistrationAllowed",
        vec![Value::u128(netuid as u128)],
    )
    .await?;
    Ok(val.and_then(|v| decode_bool(&v)).unwrap_or(true))
}

/// Get max validators for a subnet
pub async fn get_max_validators(
    client: &OnlineClient<PolkadotConfig>,
    netuid: u16,
) -> Result<u16> {
    let val = query_storage_value(
        client,
        SUBTENSOR_MODULE,
        "MaxAllowedValidators",
        vec![Value::u128(netuid as u128)],
    )
    .await?;
    Ok(val.and_then(|v| decode_u16(&v)).unwrap_or(64))
}

/// Get weights rate limit for a subnet
pub async fn get_weights_rate_limit(
    client: &OnlineClient<PolkadotConfig>,
    netuid: u16,
) -> Result<u64> {
    let val = query_storage_value(
        client,
        SUBTENSOR_MODULE,
        "WeightsSetRateLimit",
        vec![Value::u128(netuid as u128)],
    )
    .await?;
    Ok(val.and_then(|v| decode_u64(&v)).unwrap_or(100))
}

/// Get weights version key for a subnet
pub async fn get_weights_version_key(
    client: &OnlineClient<PolkadotConfig>,
    netuid: u16,
) -> Result<u64> {
    let val = query_storage_value(
        client,
        SUBTENSOR_MODULE,
        "WeightsVersionKey",
        vec![Value::u128(netuid as u128)],
    )
    .await?;
    Ok(val.and_then(|v| decode_u64(&v)).unwrap_or(0))
}

/// Check if commit-reveal is enabled for a subnet
pub async fn get_commit_reveal_enabled(
    client: &OnlineClient<PolkadotConfig>,
    netuid: u16,
) -> Result<bool> {
    let val = query_storage_value(
        client,
        SUBTENSOR_MODULE,
        "CommitRevealWeightsEnabled",
        vec![Value::u128(netuid as u128)],
    )
    .await?;
    Ok(val.and_then(|v| decode_bool(&v)).unwrap_or(false))
}

/// Get burn cost for registering on a subnet
pub async fn get_burn(client: &OnlineClient<PolkadotConfig>, netuid: u16) -> Result<Balance> {
    let val = query_storage_value(
        client,
        SUBTENSOR_MODULE,
        "Burn",
        vec![Value::u128(netuid as u128)],
    )
    .await?;
    Ok(Balance::from_rao(
        val.and_then(|v| decode_u64(&v)).unwrap_or(1_000_000_000),
    ))
}

/// Get difficulty for PoW registration on a subnet
pub async fn get_difficulty(client: &OnlineClient<PolkadotConfig>, netuid: u16) -> Result<u64> {
    let val = query_storage_value(
        client,
        SUBTENSOR_MODULE,
        "Difficulty",
        vec![Value::u128(netuid as u128)],
    )
    .await?;
    Ok(val.and_then(|v| decode_u64(&v)).unwrap_or(10_000_000))
}

/// Get all subnet hyperparameters
pub async fn get_subnet_hyperparameters(
    client: &OnlineClient<PolkadotConfig>,
    netuid: u16,
) -> Result<SubnetHyperparameters> {
    // Query multiple parameters
    let tempo = get_tempo(client, netuid).await?;
    let immunity_period = get_immunity_period(client, netuid).await?;
    let activity_cutoff = get_activity_cutoff(client, netuid).await?;
    let max_validators = get_max_validators(client, netuid).await?;
    let weights_rate_limit = get_weights_rate_limit(client, netuid).await?;
    let weights_version = get_weights_version_key(client, netuid).await?;
    let registration_allowed = get_registration_allowed(client, netuid).await?;
    let commit_reveal_enabled = get_commit_reveal_enabled(client, netuid).await?;
    let difficulty = get_difficulty(client, netuid).await?;
    let burn = get_burn(client, netuid).await?;

    Ok(SubnetHyperparameters {
        tempo,
        immunity_period,
        activity_cutoff,
        max_validators,
        weights_rate_limit,
        weights_version,
        registration_allowed,
        commit_reveal_weights_enabled: commit_reveal_enabled,
        difficulty,
        min_burn: burn.rao(),
        max_burn: burn.rao() * 100,
        ..Default::default()
    })
}

/// Check if a subnet exists
pub async fn subnet_exists(client: &OnlineClient<PolkadotConfig>, netuid: u16) -> Result<bool> {
    let val = query_storage_value(
        client,
        SUBTENSOR_MODULE,
        "NetworksAdded",
        vec![Value::u128(netuid as u128)],
    )
    .await?;
    Ok(val.and_then(|v| decode_bool(&v)).unwrap_or(false))
}
