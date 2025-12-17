//! Neuron information types
//!
//! This module contains NeuronInfo and NeuronInfoLite types that represent
//! registered neurons on the Bittensor network.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{AxonInfo, PrometheusInfo};
use crate::utils::balance::Balance;

/// Full neuron information including weights and bonds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuronInfo {
    /// Hotkey SS58 address
    pub hotkey: String,
    /// Coldkey SS58 address
    pub coldkey: String,
    /// Unique identifier within the subnet
    pub uid: u16,
    /// Network/subnet unique identifier
    pub netuid: u16,
    /// Whether the neuron is active
    pub active: bool,
    /// Total stake
    pub stake: Balance,
    /// Stake breakdown by coldkey
    pub stake_dict: HashMap<String, Balance>,
    /// Total stake (alias)
    pub total_stake: Balance,
    /// Rank score (0.0 - 1.0)
    pub rank: f64,
    /// Emission rate
    pub emission: f64,
    /// Incentive score (0.0 - 1.0)
    pub incentive: f64,
    /// Consensus score (0.0 - 1.0)
    pub consensus: f64,
    /// Trust score (0.0 - 1.0)
    pub trust: f64,
    /// Validator trust score (0.0 - 1.0)
    pub validator_trust: f64,
    /// Dividends (0.0 - 1.0)
    pub dividends: f64,
    /// Last update block
    pub last_update: u64,
    /// Whether this neuron can validate
    pub validator_permit: bool,
    /// Weights set by this neuron [(uid, weight), ...]
    pub weights: Vec<(u16, u16)>,
    /// Bonds held by this neuron [[uid, amount], ...]
    pub bonds: Vec<Vec<u64>>,
    /// Pruning score
    pub pruning_score: u16,
    /// Prometheus endpoint info
    pub prometheus_info: Option<PrometheusInfo>,
    /// Axon endpoint info
    pub axon_info: Option<AxonInfo>,
    /// Whether this is a null/empty neuron
    pub is_null: bool,
}

impl Default for NeuronInfo {
    fn default() -> Self {
        Self::null()
    }
}

impl NeuronInfo {
    /// Create a null neuron (placeholder)
    pub fn null() -> Self {
        Self {
            hotkey: "0".repeat(48),
            coldkey: "0".repeat(48),
            uid: 0,
            netuid: 0,
            active: false,
            stake: Balance::from_rao(0),
            stake_dict: HashMap::new(),
            total_stake: Balance::from_rao(0),
            rank: 0.0,
            emission: 0.0,
            incentive: 0.0,
            consensus: 0.0,
            trust: 0.0,
            validator_trust: 0.0,
            dividends: 0.0,
            last_update: 0,
            validator_permit: false,
            weights: Vec::new(),
            bonds: Vec::new(),
            pruning_score: 0,
            prometheus_info: None,
            axon_info: None,
            is_null: true,
        }
    }

    /// Convert to lite version (without weights/bonds)
    pub fn to_lite(&self) -> NeuronInfoLite {
        NeuronInfoLite {
            hotkey: self.hotkey.clone(),
            coldkey: self.coldkey.clone(),
            uid: self.uid,
            netuid: self.netuid,
            active: self.active,
            stake: self.stake,
            stake_dict: self.stake_dict.clone(),
            total_stake: self.total_stake,
            rank: self.rank,
            emission: self.emission,
            incentive: self.incentive,
            consensus: self.consensus,
            trust: self.trust,
            validator_trust: self.validator_trust,
            dividends: self.dividends,
            last_update: self.last_update,
            validator_permit: self.validator_permit,
            pruning_score: self.pruning_score,
            prometheus_info: self.prometheus_info.clone(),
            axon_info: self.axon_info.clone(),
            is_null: self.is_null,
        }
    }
}

impl std::fmt::Display for NeuronInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "NeuronInfo( uid={}, hotkey={}, stake={}, rank={:.4}, incentive={:.4} )",
            self.uid, self.hotkey, self.stake, self.rank, self.incentive
        )
    }
}

/// Lite version of neuron info (without weights and bonds)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuronInfoLite {
    /// Hotkey SS58 address
    pub hotkey: String,
    /// Coldkey SS58 address
    pub coldkey: String,
    /// Unique identifier within the subnet
    pub uid: u16,
    /// Network/subnet unique identifier
    pub netuid: u16,
    /// Whether the neuron is active
    pub active: bool,
    /// Total stake
    pub stake: Balance,
    /// Stake breakdown by coldkey
    pub stake_dict: HashMap<String, Balance>,
    /// Total stake (alias)
    pub total_stake: Balance,
    /// Rank score (0.0 - 1.0)
    pub rank: f64,
    /// Emission rate
    pub emission: f64,
    /// Incentive score (0.0 - 1.0)
    pub incentive: f64,
    /// Consensus score (0.0 - 1.0)
    pub consensus: f64,
    /// Trust score (0.0 - 1.0)
    pub trust: f64,
    /// Validator trust score (0.0 - 1.0)
    pub validator_trust: f64,
    /// Dividends (0.0 - 1.0)
    pub dividends: f64,
    /// Last update block
    pub last_update: u64,
    /// Whether this neuron can validate
    pub validator_permit: bool,
    /// Pruning score
    pub pruning_score: u16,
    /// Prometheus endpoint info
    pub prometheus_info: Option<PrometheusInfo>,
    /// Axon endpoint info
    pub axon_info: Option<AxonInfo>,
    /// Whether this is a null/empty neuron
    pub is_null: bool,
}

impl Default for NeuronInfoLite {
    fn default() -> Self {
        Self::null()
    }
}

impl NeuronInfoLite {
    /// Create a null neuron (placeholder)
    pub fn null() -> Self {
        Self {
            hotkey: "0".repeat(48),
            coldkey: "0".repeat(48),
            uid: 0,
            netuid: 0,
            active: false,
            stake: Balance::from_rao(0),
            stake_dict: HashMap::new(),
            total_stake: Balance::from_rao(0),
            rank: 0.0,
            emission: 0.0,
            incentive: 0.0,
            consensus: 0.0,
            trust: 0.0,
            validator_trust: 0.0,
            dividends: 0.0,
            last_update: 0,
            validator_permit: false,
            pruning_score: 0,
            prometheus_info: None,
            axon_info: None,
            is_null: true,
        }
    }

    /// Convert to full neuron with weights and bonds
    pub fn to_full(
        self,
        weights: Vec<(u16, u16)>,
        bonds: Vec<Vec<u64>>,
    ) -> NeuronInfo {
        NeuronInfo {
            hotkey: self.hotkey,
            coldkey: self.coldkey,
            uid: self.uid,
            netuid: self.netuid,
            active: self.active,
            stake: self.stake,
            stake_dict: self.stake_dict,
            total_stake: self.total_stake,
            rank: self.rank,
            emission: self.emission,
            incentive: self.incentive,
            consensus: self.consensus,
            trust: self.trust,
            validator_trust: self.validator_trust,
            dividends: self.dividends,
            last_update: self.last_update,
            validator_permit: self.validator_permit,
            weights,
            bonds,
            pruning_score: self.pruning_score,
            prometheus_info: self.prometheus_info,
            axon_info: self.axon_info,
            is_null: self.is_null,
        }
    }
}

impl std::fmt::Display for NeuronInfoLite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "NeuronInfoLite( uid={}, hotkey={}, stake={}, rank={:.4} )",
            self.uid, self.hotkey, self.stake, self.rank
        )
    }
}

/// Normalize u16 value to f64 (0.0 - 1.0)
pub fn u16_normalized_float(value: u16) -> f64 {
    value as f64 / u16::MAX as f64
}

/// Normalize u64 value to f64 (0.0 - 1.0)
pub fn u64_normalized_float(value: u64) -> f64 {
    value as f64 / u64::MAX as f64
}
