use crate::chain::BittensorClient;
use crate::types::SubnetInfo;
use crate::utils::value_decode::{
    decode_account_id32, decode_bool, decode_u128, decode_u16, decode_u64,
};
use anyhow::Result;
use parity_scale_codec::Encode;
use subxt::dynamic::Value;

const SUBTENSOR_MODULE: &str = "SubtensorModule";

/// Check if commit-reveal mechanism is enabled for a subnet
pub async fn commit_reveal_enabled(client: &BittensorClient, netuid: u16) -> Result<bool> {
    if let Some(val) = client
        .storage_with_keys(
            SUBTENSOR_MODULE,
            "CommitRevealWeightsEnabled",
            vec![Value::u128(netuid as u128)],
        )
        .await?
    {
        return Ok(decode_bool(&val).unwrap_or(false));
    }
    Ok(false)
}

/// Get the recycle/burn amount for a subnet
pub async fn recycle(client: &BittensorClient, netuid: u16) -> Result<Option<u128>> {
    if let Some(val) = client
        .storage_with_keys(
            SUBTENSOR_MODULE,
            "Burn",
            vec![Value::u128(netuid as u128)],
        )
        .await?
    {
        return Ok(decode_u128(&val).ok());
    }
    Ok(None)
}

/// Get the reveal period epochs for a subnet
pub async fn get_subnet_reveal_period_epochs(
    client: &BittensorClient,
    netuid: u16,
) -> Result<Option<u64>> {
    if let Some(val) = client
        .storage_with_keys(
            SUBTENSOR_MODULE,
            "RevealPeriodEpochs",
            vec![Value::u128(netuid as u128)],
        )
        .await?
    {
        return Ok(decode_u64(&val).ok());
    }
    Ok(None)
}

/// Check if a subnet is active (FirstEmissionBlockNumber > 0)
pub async fn is_subnet_active(client: &BittensorClient, netuid: u16) -> Result<bool> {
    if let Some(val) = client
        .storage_with_keys(
            SUBTENSOR_MODULE,
            "FirstEmissionBlockNumber",
            vec![Value::u128(netuid as u128)],
        )
        .await?
    {
        if let Ok(block_num) = decode_u64(&val) {
            return Ok(block_num > 0);
        }
    }
    Ok(false)
}

/// Get all subnet infos using storage
pub async fn all_subnets(client: &BittensorClient) -> Result<Vec<SubnetInfo>> {
    let total = total_subnets(client).await.unwrap_or(0);
    let mut res = Vec::with_capacity(total as usize);
    for netuid in 0u16..total {
        if let Some(info) = subnet_info(client, netuid).await? {
            res.push(info);
        }
    }
    Ok(res)
}

/// Get subnet information using targeted storage reads
pub async fn subnet_info(client: &BittensorClient, netuid: u16) -> Result<Option<SubnetInfo>> {
    // If subnet does not exist, return None
    if !subnet_exists(client, netuid).await.unwrap_or(false) {
        return Ok(None);
    }

    // neuron_count from SubnetworkN
    let n_val = client
        .storage_with_keys(
            SUBTENSOR_MODULE,
            "SubnetworkN",
            vec![Value::u128(netuid as u128)],
        )
        .await?;
    let neuron_count: u64 = n_val.and_then(|v| decode_u64(&v).ok()).unwrap_or(0);

    // emission: sum Emission[(netuid, uid)] over all uids (convert from rao to TAO)
    let mut emission_rao: u128 = 0;
    for uid in 0..neuron_count {
        if let Some(ev) = client
            .storage_with_keys(
                SUBTENSOR_MODULE,
                "Emission",
                vec![Value::u128(netuid as u128), Value::u128(uid as u128)],
            )
            .await?
        {
            if let Ok(e) = decode_u64(&ev) {
                emission_rao = emission_rao.saturating_add(e as u128);
            }
        }
    }
    let emission = emission_rao as f64 / 1e9;

    // total_stake: sum TotalHotkeyAlpha[(hotkey, netuid)] for each neuron hotkey
    let mut total_stake: u128 = 0;
    for uid in 0..neuron_count {
        if let Some(hotkey_val) = client
            .storage_with_keys(
                SUBTENSOR_MODULE,
                "Keys",
                vec![Value::u128(netuid as u128), Value::u128(uid as u128)],
            )
            .await?
        {
            if let Ok(hk) = decode_account_id32(&hotkey_val) {
                if let Some(alpha_val) = client
                    .storage_with_keys(
                        SUBTENSOR_MODULE,
                        "TotalHotkeyAlpha",
                        vec![Value::from_bytes(&hk.encode()), Value::u128(netuid as u128)],
                    )
                    .await?
                {
                    if let Ok(a) = decode_u128(&alpha_val) {
                        total_stake = total_stake.saturating_add(a);
                    }
                }
            }
        }
    }

    Ok(Some(SubnetInfo {
        netuid,
        neuron_count,
        total_stake,
        emission,
        name: None,
        description: None,
    }))
}

/// Get all subnets information
pub async fn all_subnets_info(client: &BittensorClient) -> Result<Vec<SubnetInfo>> {
    all_subnets(client).await
}

/// Get neuron count for a subnet
pub async fn subnet_n(client: &BittensorClient, netuid: u16) -> Result<Option<u64>> {
    let result = client
        .storage_with_keys(
            SUBTENSOR_MODULE,
            "SubnetworkN",
            vec![Value::u128(netuid as u128)],
        )
        .await?;

    if let Some(value) = result {
        Ok(decode_u64(&value).ok())
    } else {
        Ok(None)
    }
}

/// Extract if subnet exists
pub async fn subnet_exists(client: &BittensorClient, netuid: u16) -> Result<bool> {
    let keys = vec![Value::u128(netuid as u128)];
    let result = client
        .storage_with_keys(SUBTENSOR_MODULE, "NetworksAdded", keys)
        .await?;
    if let Some(value) = result {
        decode_bool(&value).map_err(|e| {
            anyhow::anyhow!(
                "Failed to decode NetworksAdded for subnet {}: {}",
                netuid,
                e
            )
        })
    } else {
        Ok(false)
    }
}

/// Get total number of subnets
pub async fn total_subnets(client: &BittensorClient) -> Result<u16> {
    let total_val = client
        .storage(SUBTENSOR_MODULE, "TotalNetworks", None)
        .await?
        .ok_or_else(|| anyhow::anyhow!("TotalNetworks storage entry not found"))?;
    decode_u16(&total_val).map_err(|e| anyhow::anyhow!("Failed to decode TotalNetworks: {}", e))
}

/// Hyperparameters
pub async fn difficulty(client: &BittensorClient, netuid: u16) -> Result<Option<u64>> {
    if let Some(val) = client
        .storage_with_keys(
            SUBTENSOR_MODULE,
            "Difficulty",
            vec![Value::u128(netuid as u128)],
        )
        .await?
    {
        return Ok(decode_u64(&val).ok());
    }
    Ok(None)
}
pub async fn tempo(client: &BittensorClient, netuid: u16) -> Result<Option<u64>> {
    if let Some(val) = client
        .storage_with_keys(SUBTENSOR_MODULE, "Tempo", vec![Value::u128(netuid as u128)])
        .await?
    {
        return Ok(decode_u64(&val).ok());
    }
    Ok(None)
}
pub async fn min_allowed_weights(client: &BittensorClient, netuid: u16) -> Result<Option<u64>> {
    if let Some(val) = client
        .storage_with_keys(
            SUBTENSOR_MODULE,
            "MinAllowedWeights",
            vec![Value::u128(netuid as u128)],
        )
        .await?
    {
        return Ok(decode_u64(&val).ok());
    }
    Ok(None)
}
pub async fn max_weight_limit(client: &BittensorClient, netuid: u16) -> Result<Option<f64>> {
    if let Some(val) = client
        .storage_with_keys(
            SUBTENSOR_MODULE,
            "MaxWeightsLimit",
            vec![Value::u128(netuid as u128)],
        )
        .await?
    {
        if let Ok(v) = decode_u64(&val) {
            return Ok(Some(v as f64 / u16::MAX as f64));
        }
    }
    Ok(None)
}
pub async fn immunity_period(client: &BittensorClient, netuid: u16) -> Result<Option<u64>> {
    if let Some(val) = client
        .storage_with_keys(
            SUBTENSOR_MODULE,
            "ImmunityPeriod",
            vec![Value::u128(netuid as u128)],
        )
        .await?
    {
        return Ok(decode_u64(&val).ok());
    }
    Ok(None)
}
pub async fn weights_rate_limit(client: &BittensorClient, netuid: u16) -> Result<Option<u64>> {
    if let Some(val) = client
        .storage_with_keys(
            SUBTENSOR_MODULE,
            "WeightsSetRateLimit",
            vec![Value::u128(netuid as u128)],
        )
        .await?
    {
        return Ok(decode_u64(&val).ok());
    }
    Ok(None)
}
pub async fn blocks_since_last_step(client: &BittensorClient, netuid: u16) -> Result<Option<u64>> {
    if let Some(val) = client
        .storage_with_keys(
            SUBTENSOR_MODULE,
            "BlocksSinceLastStep",
            vec![Value::u128(netuid as u128)],
        )
        .await?
    {
        return Ok(decode_u64(&val).ok());
    }
    Ok(None)
}
pub async fn blocks_since_last_update(
    client: &BittensorClient,
    netuid: u16,
) -> Result<Option<u64>> {
    if let Some(val) = client
        .storage_with_keys(
            SUBTENSOR_MODULE,
            "BlocksSinceLastUpdate",
            vec![Value::u128(netuid as u128)],
        )
        .await?
    {
        return Ok(decode_u64(&val).ok());
    }
    Ok(None)
}

/// Subnet owner hotkey
pub async fn subnet_owner_hotkey(
    client: &BittensorClient,
    netuid: u16,
) -> Result<Option<sp_core::crypto::AccountId32>> {
    if let Some(val) = client
        .storage_with_keys(
            SUBTENSOR_MODULE,
            "SubnetOwnerHotkey",
            vec![Value::u128(netuid as u128)],
        )
        .await?
    {
        return Ok(decode_account_id32(&val).ok());
    }
    Ok(None)
}

/// Subnet validator permits
pub async fn subnet_validator_permits(client: &BittensorClient, netuid: u16) -> Result<Vec<bool>> {
    let n_val = client
        .storage_with_keys(
            SUBTENSOR_MODULE,
            "SubnetworkN",
            vec![Value::u128(netuid as u128)],
        )
        .await?;
    let n: u64 = n_val.and_then(|v| decode_u64(&v).ok()).unwrap_or(0);
    let mut permits = Vec::with_capacity(n as usize);
    for uid in 0..n {
        let val = client
            .storage_with_keys(
                SUBTENSOR_MODULE,
                "ValidatorPermit",
                vec![Value::u128(netuid as u128), Value::u128(uid as u128)],
            )
            .await?;
        let is_permit = match val {
            Some(v) => decode_bool(&v).unwrap_or(false),
            None => false,
        };
        permits.push(is_permit);
    }
    Ok(permits)
}

/// Mechanism info
pub async fn mechanism_count(client: &BittensorClient, netuid: u16) -> Result<Option<u64>> {
    if let Some(val) = client
        .storage_with_keys(
            SUBTENSOR_MODULE,
            "MechanismCountCurrent",
            vec![Value::u128(netuid as u128)],
        )
        .await?
    {
        return Ok(decode_u64(&val).ok());
    }
    Ok(None)
}

pub async fn mechanism_emission_split(
    client: &BittensorClient,
    netuid: u16,
) -> Result<Option<u64>> {
    if let Some(val) = client
        .storage_with_keys(
            SUBTENSOR_MODULE,
            "MechanismEmissionSplit",
            vec![Value::u128(netuid as u128)],
        )
        .await?
    {
        return Ok(decode_u64(&val).ok());
    }
    Ok(None)
}

/// Get subnet burn cost (runtime API)
pub async fn subnet_burn_cost(client: &BittensorClient, _netuid: u16) -> Result<u128> {
    let cost_val = client
        .runtime_api(
            "SubnetRegistrationRuntimeApi",
            "get_network_registration_cost",
            vec![],
        )
        .await?
        .ok_or_else(|| {
            anyhow::anyhow!("Failed to retrieve network registration cost from runtime API")
        })?;
    let cost_u64 = decode_u64(&cost_val).map_err(|e| {
        anyhow::anyhow!(
            "Failed to decode network registration cost (TaoCurrency): {}",
            e
        )
    })?;
    Ok(cost_u64 as u128)
}

/// Get subnet Alpha price in RAO via runtime API (SN0 fixed to 1 TAO)
pub async fn get_subnet_price(client: &BittensorClient, netuid: u16) -> Result<u128> {
    if netuid == 0 {
        return Ok(1_000_000_000u128);
    }
    if let Some(val) = client
        .runtime_api(
            "SwapRuntimeApi",
            "current_alpha_price",
            vec![Value::u128(netuid as u128)],
        )
        .await?
    {
        return crate::utils::value_decode::decode_u128(&val)
            .map_err(|e| anyhow::anyhow!("Failed to decode price: {}", e));
    }
    // Fallback: use storage sqrt price squared
    let sqrt_price = super::liquidity::get_current_subnet_price_rao(client, netuid).await?;
    Ok(sqrt_price)
}

/// Get prices for all subnets
pub async fn get_subnet_prices(
    client: &BittensorClient,
) -> Result<std::collections::HashMap<u16, u128>> {
    let total = total_subnets(client).await.unwrap_or(0);
    let mut map = std::collections::HashMap::new();
    for netuid in 0u16..total {
        map.insert(netuid, get_subnet_price(client, netuid).await.unwrap_or(0));
    }
    Ok(map)
}

/// Calculate next epoch start block
pub async fn get_next_epoch_start_block(
    client: &BittensorClient,
    netuid: u16,
    block: Option<u64>,
) -> Result<Option<u64>> {
    let current_block = if let Some(b) = block {
        b
    } else {
        client
            .block_number()
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?
    };
    let blocks_since = super::subnets::blocks_since_last_step(client, netuid)
        .await?
        .unwrap_or(0);
    let tempo = super::subnets::tempo(client, netuid).await?.unwrap_or(0);
    if current_block > 0 && tempo > 0 {
        Ok(Some(
            current_block
                .saturating_sub(blocks_since)
                .saturating_add(tempo)
                .saturating_add(1),
        ))
    } else {
        Ok(None)
    }
}

pub async fn subnet_tao_in_emission(client: &BittensorClient, netuid: u16) -> Result<Option<u64>> {
    if let Some(val) = client
        .storage_with_keys(
            SUBTENSOR_MODULE,
            "SubnetTaoInEmission",
            vec![Value::u128(netuid as u128)],
        )
        .await?
    {
        return Ok(decode_u64(&val).ok());
    }
    Ok(None)
}

pub async fn block_emission(client: &BittensorClient) -> Result<Option<u64>> {
    if let Some(val) = client
        .storage_with_keys(SUBTENSOR_MODULE, "BlockEmission", vec![])
        .await?
    {
        return Ok(decode_u64(&val).ok());
    }
    Ok(None)
}

pub async fn subnet_emission_percent(client: &BittensorClient, netuid: u16) -> Result<Option<f64>> {
    let sub = subnet_tao_in_emission(client, netuid).await?.unwrap_or(0);
    let total = block_emission(client).await?.unwrap_or(0);
    if total == 0 {
        return Ok(None);
    }
    Ok(Some((sub as f64) / (total as f64)))
}
