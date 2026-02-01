//! Subnet hyperparameter queries with proper SCALE decoding
//!
//! This module provides functions to query individual subnet hyperparameters
//! from the Bittensor chain, matching the Python SDK SubnetHyperparameters structure.

use crate::chain::BittensorClient;
use crate::errors::{BittensorError, BittensorResult, ChainQueryError};
use crate::utils::decoders::{decode_bool, decode_u16, decode_u64};
use subxt::dynamic::Value;

const SUBTENSOR_MODULE: &str = "SubtensorModule";

/// Complete subnet hyperparameters (matches Python SDK SubnetHyperparameters)
#[derive(Debug, Clone, Default)]
pub struct SubnetHyperparameters {
    pub rho: u16,
    pub kappa: u16,
    pub immunity_period: u16,
    pub min_allowed_weights: u16,
    pub max_weights_limit: u16,
    pub tempo: u16,
    pub min_difficulty: u64,
    pub max_difficulty: u64,
    pub weights_version: u64,
    pub weights_rate_limit: u64,
    pub adjustment_interval: u16,
    pub activity_cutoff: u16,
    pub registration_allowed: bool,
    pub target_regs_per_interval: u16,
    pub min_burn: u64,
    pub max_burn: u64,
    pub bonds_moving_avg: u64,
    pub max_regs_per_block: u16,
    pub serving_rate_limit: u64,
    pub max_validators: u16,
    pub adjustment_alpha: u64,
    pub difficulty: u64,
    pub commit_reveal_weights_interval: u64,
    pub commit_reveal_weights_enabled: bool,
    pub alpha_high: u16,
    pub alpha_low: u16,
    pub liquid_alpha_enabled: bool,
}

/// Helper to fetch a u16 storage value for a subnet
async fn fetch_u16_param(
    client: &BittensorClient,
    entry: &str,
    netuid: u16,
) -> BittensorResult<u16> {
    let keys = vec![Value::u128(netuid as u128)];
    match client.storage_with_keys(SUBTENSOR_MODULE, entry, keys).await {
        Ok(Some(val)) => decode_u16(&val).map_err(|e| {
            BittensorError::ChainQuery(ChainQueryError::with_storage(
                format!("Failed to decode {} as u16: {}", entry, e),
                SUBTENSOR_MODULE,
                entry,
            ))
        }),
        Ok(None) => Ok(0),
        Err(e) => Err(BittensorError::ChainQuery(ChainQueryError::with_storage(
            format!("Failed to query {}: {}", entry, e),
            SUBTENSOR_MODULE,
            entry,
        ))),
    }
}

/// Helper to fetch a u64 storage value for a subnet
async fn fetch_u64_param(
    client: &BittensorClient,
    entry: &str,
    netuid: u16,
) -> BittensorResult<u64> {
    let keys = vec![Value::u128(netuid as u128)];
    match client.storage_with_keys(SUBTENSOR_MODULE, entry, keys).await {
        Ok(Some(val)) => decode_u64(&val).map_err(|e| {
            BittensorError::ChainQuery(ChainQueryError::with_storage(
                format!("Failed to decode {} as u64: {}", entry, e),
                SUBTENSOR_MODULE,
                entry,
            ))
        }),
        Ok(None) => Ok(0),
        Err(e) => Err(BittensorError::ChainQuery(ChainQueryError::with_storage(
            format!("Failed to query {}: {}", entry, e),
            SUBTENSOR_MODULE,
            entry,
        ))),
    }
}

/// Helper to fetch a bool storage value for a subnet
async fn fetch_bool_param(
    client: &BittensorClient,
    entry: &str,
    netuid: u16,
) -> BittensorResult<bool> {
    let keys = vec![Value::u128(netuid as u128)];
    match client.storage_with_keys(SUBTENSOR_MODULE, entry, keys).await {
        Ok(Some(val)) => decode_bool(&val).map_err(|e| {
            BittensorError::ChainQuery(ChainQueryError::with_storage(
                format!("Failed to decode {} as bool: {}", entry, e),
                SUBTENSOR_MODULE,
                entry,
            ))
        }),
        Ok(None) => Ok(false),
        Err(e) => Err(BittensorError::ChainQuery(ChainQueryError::with_storage(
            format!("Failed to query {}: {}", entry, e),
            SUBTENSOR_MODULE,
            entry,
        ))),
    }
}

/// Get all hyperparameters for a subnet
pub async fn get_subnet_hyperparameters(
    client: &BittensorClient,
    netuid: u16,
) -> BittensorResult<SubnetHyperparameters> {
    // Fetch all hyperparameters in parallel for efficiency
    let (
        rho,
        kappa,
        immunity_period,
        min_allowed_weights,
        max_weights_limit,
        tempo,
        min_difficulty,
        max_difficulty,
        weights_version,
        weights_rate_limit,
        adjustment_interval,
        activity_cutoff,
        registration_allowed,
        target_regs_per_interval,
        min_burn,
        max_burn,
        bonds_moving_avg,
        max_regs_per_block,
        serving_rate_limit,
        max_validators,
        adjustment_alpha,
        difficulty,
        commit_reveal_weights_interval,
        commit_reveal_weights_enabled,
        alpha_high,
        alpha_low,
        liquid_alpha_enabled,
    ) = tokio::join!(
        get_rho(client, netuid),
        get_kappa(client, netuid),
        get_immunity_period(client, netuid),
        get_min_allowed_weights(client, netuid),
        get_max_weights_limit(client, netuid),
        get_tempo(client, netuid),
        get_min_difficulty(client, netuid),
        get_max_difficulty(client, netuid),
        get_weights_version_key(client, netuid),
        get_weights_rate_limit(client, netuid),
        get_adjustment_interval(client, netuid),
        get_activity_cutoff(client, netuid),
        get_registration_allowed(client, netuid),
        get_target_regs_per_interval(client, netuid),
        get_min_burn(client, netuid),
        get_max_burn(client, netuid),
        get_bonds_moving_average(client, netuid),
        get_max_regs_per_block(client, netuid),
        get_serving_rate_limit(client, netuid),
        get_max_validators(client, netuid),
        get_adjustment_alpha(client, netuid),
        get_difficulty(client, netuid),
        get_commit_reveal_weights_interval(client, netuid),
        get_commit_reveal_weights_enabled(client, netuid),
        get_alpha_high(client, netuid),
        get_alpha_low(client, netuid),
        get_liquid_alpha_enabled(client, netuid),
    );

    Ok(SubnetHyperparameters {
        rho: rho.unwrap_or(0),
        kappa: kappa.unwrap_or(0),
        immunity_period: immunity_period.unwrap_or(0),
        min_allowed_weights: min_allowed_weights.unwrap_or(0),
        max_weights_limit: max_weights_limit.unwrap_or(0),
        tempo: tempo.unwrap_or(0),
        min_difficulty: min_difficulty.unwrap_or(0),
        max_difficulty: max_difficulty.unwrap_or(0),
        weights_version: weights_version.unwrap_or(0),
        weights_rate_limit: weights_rate_limit.unwrap_or(0),
        adjustment_interval: adjustment_interval.unwrap_or(0),
        activity_cutoff: activity_cutoff.unwrap_or(0),
        registration_allowed: registration_allowed.unwrap_or(false),
        target_regs_per_interval: target_regs_per_interval.unwrap_or(0),
        min_burn: min_burn.unwrap_or(0),
        max_burn: max_burn.unwrap_or(0),
        bonds_moving_avg: bonds_moving_avg.unwrap_or(0),
        max_regs_per_block: max_regs_per_block.unwrap_or(0),
        serving_rate_limit: serving_rate_limit.unwrap_or(0),
        max_validators: max_validators.unwrap_or(0),
        adjustment_alpha: adjustment_alpha.unwrap_or(0),
        difficulty: difficulty.unwrap_or(0),
        commit_reveal_weights_interval: commit_reveal_weights_interval.unwrap_or(0),
        commit_reveal_weights_enabled: commit_reveal_weights_enabled.unwrap_or(false),
        alpha_high: alpha_high.unwrap_or(0),
        alpha_low: alpha_low.unwrap_or(0),
        liquid_alpha_enabled: liquid_alpha_enabled.unwrap_or(false),
    })
}

/// Get Rho parameter for a subnet
/// Rho is the ratio for calculating the weights to set
pub async fn get_rho(client: &BittensorClient, netuid: u16) -> BittensorResult<u16> {
    fetch_u16_param(client, "Rho", netuid).await
}

/// Get Kappa parameter for a subnet
/// Kappa is used in the Yuma Consensus algorithm
pub async fn get_kappa(client: &BittensorClient, netuid: u16) -> BittensorResult<u16> {
    fetch_u16_param(client, "Kappa", netuid).await
}

/// Get immunity period for a subnet
/// Number of blocks a neuron is protected from deregistration after registration
pub async fn get_immunity_period(client: &BittensorClient, netuid: u16) -> BittensorResult<u16> {
    fetch_u16_param(client, "ImmunityPeriod", netuid).await
}

/// Get minimum allowed weights for a subnet
/// Minimum number of weights each validator must set
pub async fn get_min_allowed_weights(client: &BittensorClient, netuid: u16) -> BittensorResult<u16> {
    fetch_u16_param(client, "MinAllowedWeights", netuid).await
}

/// Get maximum weights limit for a subnet
/// Maximum weight value that can be assigned (normalized to u16 range)
pub async fn get_max_weights_limit(client: &BittensorClient, netuid: u16) -> BittensorResult<u16> {
    fetch_u16_param(client, "MaxWeightsLimit", netuid).await
}

/// Get tempo for a subnet
/// Number of blocks between weight setting epochs
pub async fn get_tempo(client: &BittensorClient, netuid: u16) -> BittensorResult<u16> {
    fetch_u16_param(client, "Tempo", netuid).await
}

/// Get minimum difficulty for a subnet
/// Minimum PoW difficulty for registration
pub async fn get_min_difficulty(client: &BittensorClient, netuid: u16) -> BittensorResult<u64> {
    fetch_u64_param(client, "MinDifficulty", netuid).await
}

/// Get maximum difficulty for a subnet
/// Maximum PoW difficulty for registration
pub async fn get_max_difficulty(client: &BittensorClient, netuid: u16) -> BittensorResult<u64> {
    fetch_u64_param(client, "MaxDifficulty", netuid).await
}

/// Get current difficulty for a subnet
/// Current PoW difficulty for registration
pub async fn get_difficulty(client: &BittensorClient, netuid: u16) -> BittensorResult<u64> {
    fetch_u64_param(client, "Difficulty", netuid).await
}

/// Get weights version key for a subnet
/// Version number for weight format compatibility
pub async fn get_weights_version_key(client: &BittensorClient, netuid: u16) -> BittensorResult<u64> {
    fetch_u64_param(client, "WeightsVersionKey", netuid).await
}

/// Get weights rate limit for a subnet
/// Minimum blocks between weight setting transactions
pub async fn get_weights_rate_limit(client: &BittensorClient, netuid: u16) -> BittensorResult<u64> {
    fetch_u64_param(client, "WeightsSetRateLimit", netuid).await
}

/// Get adjustment interval for a subnet
/// Number of blocks between difficulty adjustments
pub async fn get_adjustment_interval(client: &BittensorClient, netuid: u16) -> BittensorResult<u16> {
    fetch_u16_param(client, "AdjustmentInterval", netuid).await
}

/// Get activity cutoff for a subnet
/// Number of blocks of inactivity before a neuron becomes inactive
pub async fn get_activity_cutoff(client: &BittensorClient, netuid: u16) -> BittensorResult<u16> {
    fetch_u16_param(client, "ActivityCutoff", netuid).await
}

/// Check if registration is allowed for a subnet
/// Whether new neurons can register on this subnet
pub async fn get_registration_allowed(client: &BittensorClient, netuid: u16) -> BittensorResult<bool> {
    fetch_bool_param(client, "NetworkRegistrationAllowed", netuid).await
}

/// Get target registrations per interval for a subnet
/// Target number of registrations per adjustment interval
pub async fn get_target_regs_per_interval(
    client: &BittensorClient,
    netuid: u16,
) -> BittensorResult<u16> {
    fetch_u16_param(client, "TargetRegistrationsPerInterval", netuid).await
}

/// Get minimum burn amount for a subnet (in RAO)
/// Minimum amount of TAO to burn for registration
pub async fn get_min_burn(client: &BittensorClient, netuid: u16) -> BittensorResult<u64> {
    fetch_u64_param(client, "MinBurn", netuid).await
}

/// Get maximum burn amount for a subnet (in RAO)
/// Maximum amount of TAO to burn for registration
pub async fn get_max_burn(client: &BittensorClient, netuid: u16) -> BittensorResult<u64> {
    fetch_u64_param(client, "MaxBurn", netuid).await
}

/// Get bonds moving average for a subnet
/// Rate at which bonds update (higher = faster updates)
pub async fn get_bonds_moving_average(client: &BittensorClient, netuid: u16) -> BittensorResult<u64> {
    fetch_u64_param(client, "BondsMovingAverage", netuid).await
}

/// Get maximum registrations per block for a subnet
/// Maximum number of neurons that can register in a single block
pub async fn get_max_regs_per_block(client: &BittensorClient, netuid: u16) -> BittensorResult<u16> {
    fetch_u16_param(client, "MaxRegistrationsPerBlock", netuid).await
}

/// Get serving rate limit for a subnet
/// Minimum blocks between axon serving info updates
pub async fn get_serving_rate_limit(client: &BittensorClient, netuid: u16) -> BittensorResult<u64> {
    fetch_u64_param(client, "ServingRateLimit", netuid).await
}

/// Get maximum validators for a subnet
/// Maximum number of validators allowed on the subnet
pub async fn get_max_validators(client: &BittensorClient, netuid: u16) -> BittensorResult<u16> {
    fetch_u16_param(client, "MaxAllowedValidators", netuid).await
}

/// Get adjustment alpha for a subnet
/// Alpha parameter for difficulty adjustment algorithm
pub async fn get_adjustment_alpha(client: &BittensorClient, netuid: u16) -> BittensorResult<u64> {
    fetch_u64_param(client, "AdjustmentAlpha", netuid).await
}

/// Get commit reveal weights interval for a subnet
/// Number of blocks for commit-reveal weight setting cycle
pub async fn get_commit_reveal_weights_interval(
    client: &BittensorClient,
    netuid: u16,
) -> BittensorResult<u64> {
    fetch_u64_param(client, "CommitRevealWeightsInterval", netuid).await
}

/// Check if commit-reveal weights mechanism is enabled for a subnet
pub async fn get_commit_reveal_weights_enabled(
    client: &BittensorClient,
    netuid: u16,
) -> BittensorResult<bool> {
    fetch_bool_param(client, "CommitRevealWeightsEnabled", netuid).await
}

/// Get alpha high parameter for liquid alpha
/// Upper bound for alpha in liquid alpha mechanism
pub async fn get_alpha_high(client: &BittensorClient, netuid: u16) -> BittensorResult<u16> {
    fetch_u16_param(client, "AlphaHigh", netuid).await
}

/// Get alpha low parameter for liquid alpha
/// Lower bound for alpha in liquid alpha mechanism
pub async fn get_alpha_low(client: &BittensorClient, netuid: u16) -> BittensorResult<u16> {
    fetch_u16_param(client, "AlphaLow", netuid).await
}

/// Check if liquid alpha mechanism is enabled for a subnet
pub async fn get_liquid_alpha_enabled(client: &BittensorClient, netuid: u16) -> BittensorResult<bool> {
    fetch_bool_param(client, "LiquidAlphaOn", netuid).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subnet_hyperparameters_default() {
        let params = SubnetHyperparameters::default();
        assert_eq!(params.rho, 0);
        assert_eq!(params.kappa, 0);
        assert_eq!(params.tempo, 0);
        assert!(!params.registration_allowed);
        assert!(!params.commit_reveal_weights_enabled);
        assert!(!params.liquid_alpha_enabled);
    }

    #[test]
    fn test_subnet_hyperparameters_clone() {
        let params = SubnetHyperparameters {
            rho: 10,
            kappa: 32767,
            tempo: 360,
            registration_allowed: true,
            ..Default::default()
        };
        let cloned = params.clone();
        assert_eq!(cloned.rho, 10);
        assert_eq!(cloned.kappa, 32767);
        assert_eq!(cloned.tempo, 360);
        assert!(cloned.registration_allowed);
    }

    #[test]
    fn test_subnet_hyperparameters_debug() {
        let params = SubnetHyperparameters::default();
        let debug_str = format!("{:?}", params);
        assert!(debug_str.contains("SubnetHyperparameters"));
        assert!(debug_str.contains("rho"));
        assert!(debug_str.contains("tempo"));
    }
}
