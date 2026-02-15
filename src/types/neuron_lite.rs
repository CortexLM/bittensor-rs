use crate::types::{AxonInfo, PrometheusInfo};
use serde::{Deserialize, Serialize};
use sp_core::crypto::AccountId32;

/// Lightweight neuron information without weights and bonds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuronInfoLite {
    /// Unique identifier of the neuron within the subnet
    pub uid: u64,
    /// Network unique identifier
    pub netuid: u16,
    /// Hotkey (SS58 address)
    #[serde(with = "crate::utils::ss58::serde_account")]
    pub hotkey: AccountId32,
    /// Coldkey (SS58 address)
    #[serde(with = "crate::utils::ss58::serde_account")]
    pub coldkey: AccountId32,
    /// Neuron's stake amount
    pub stake: u128,
    /// Dictionary mapping coldkey to amount staked to this neuron
    #[serde(with = "crate::utils::ss58::serde_account_map")]
    pub stake_dict: std::collections::HashMap<AccountId32, u128>,
    /// Total stake
    pub total_stake: u128,
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
    /// Pruning score
    pub pruning_score: u64,
    /// Prometheus information
    pub prometheus_info: Option<PrometheusInfo>,
    /// Axon information
    pub axon_info: Option<AxonInfo>,
    /// Whether this is a null neuron
    pub is_null: bool,
}

impl NeuronInfoLite {
    pub fn new(uid: u64, netuid: u16) -> Self {
        Self {
            uid,
            netuid,
            hotkey: AccountId32::from([0u8; 32]),
            coldkey: AccountId32::from([0u8; 32]),
            stake: 0,
            stake_dict: std::collections::HashMap::new(),
            total_stake: 0,
            rank: 0.0,
            trust: 0.0,
            consensus: 0.0,
            validator_trust: 0.0,
            incentive: 0.0,
            emission: 0.0,
            dividends: 0.0,
            active: false,
            last_update: 0,
            validator_permit: false,
            pruning_score: 0,
            prometheus_info: None,
            axon_info: None,
            is_null: false,
        }
    }

    /// Create a null neuron
    pub fn null_neuron() -> Self {
        Self {
            uid: 0,
            netuid: 0,
            hotkey: AccountId32::from([0u8; 32]),
            coldkey: AccountId32::from([0u8; 32]),
            stake: 0,
            stake_dict: std::collections::HashMap::new(),
            total_stake: 0,
            rank: 0.0,
            trust: 0.0,
            consensus: 0.0,
            validator_trust: 0.0,
            incentive: 0.0,
            emission: 0.0,
            dividends: 0.0,
            active: false,
            last_update: 0,
            validator_permit: false,
            pruning_score: 0,
            prometheus_info: None,
            axon_info: None,
            is_null: true,
        }
    }
}
