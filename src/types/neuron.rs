use crate::types::{AxonInfo, PrometheusInfo};
use serde::{Deserialize, Serialize};
use sp_core::crypto::AccountId32;

/// Information about a neuron in the Bittensor network (complete version with all fields)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuronInfo {
    /// Unique identifier of the neuron within the subnet
    pub uid: u64,
    /// Network unique identifier
    pub netuid: u16,
    /// Hotkey (SS58 address)
    pub hotkey: AccountId32,
    /// Coldkey (SS58 address)
    pub coldkey: AccountId32,
    /// Neuron's stake amount
    pub stake: u128,
    /// Dictionary mapping coldkey to amount staked to this neuron
    pub stake_dict: std::collections::HashMap<AccountId32, u128>,
    /// Total stake on this subnet (alpha)
    pub total_stake: u128,
    /// Total stake on root subnet (TAO) - used for dividend calculations
    pub root_stake: u128,
    /// Neuron's rank score (normalized)
    pub rank: f64,
    /// Neuron's trust score (normalized)
    pub trust: f64,
    /// Neuron's consensus score (normalized)
    pub consensus: f64,
    /// Validator trust score (normalized)
    pub validator_trust: f64,
    /// Incentive score (normalized)
    pub incentive: f64,
    /// Emission amount
    pub emission: f64,
    /// Dividends received (normalized)
    pub dividends: f64,
    /// Whether the neuron is active
    pub active: bool,
    /// Last update block
    pub last_update: u64,
    /// Whether the neuron has validator permit
    pub validator_permit: bool,
    /// Version key
    pub version: u64,
    /// Weights list [(uid, weight), ...]
    pub weights: Vec<(u64, u64)>,
    /// Bonds list [(uid, bond), ...]
    pub bonds: Vec<Vec<u64>>,
    /// Pruning score
    pub pruning_score: u64,
    /// Prometheus information
    pub prometheus_info: Option<PrometheusInfo>,
    /// Axon information
    pub axon_info: Option<AxonInfo>,
    /// Whether this is a null neuron
    pub is_null: bool,
}

impl NeuronInfo {
    /// Create NeuronInfo with all required fields - no defaults
    pub fn create(
        uid: u64,
        netuid: u16,
        hotkey: AccountId32,
        coldkey: AccountId32,
        stake: u128,
        stake_dict: std::collections::HashMap<AccountId32, u128>,
        total_stake: u128,
        root_stake: u128,
        rank: f64,
        trust: f64,
        consensus: f64,
        validator_trust: f64,
        incentive: f64,
        emission: f64,
        dividends: f64,
        active: bool,
        last_update: u64,
        validator_permit: bool,
        version: u64,
        weights: Vec<(u64, u64)>,
        bonds: Vec<Vec<u64>>,
        pruning_score: u64,
        prometheus_info: Option<PrometheusInfo>,
        axon_info: Option<AxonInfo>,
    ) -> Self {
        Self {
            uid,
            netuid,
            hotkey,
            coldkey,
            stake,
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
            is_null: false,
        }
    }
}
