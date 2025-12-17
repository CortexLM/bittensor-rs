//! Chain data types for Bittensor
//!
//! This module contains all the data structures used to represent
//! on-chain data from the Bittensor network, mirroring the Python SDK's
//! `chain_data` module.

mod axon;
mod delegate;
mod neuron;
mod prometheus;
mod subnet;

pub use axon::AxonInfo;
pub use delegate::{DelegateInfo, DelegateInfoLite};
pub use neuron::{NeuronInfo, NeuronInfoLite};
pub use prometheus::PrometheusInfo;
pub use subnet::{SubnetHyperparameters, SubnetIdentity, SubnetInfo};

use crate::utils::balance::Balance;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// IP type constants
pub const IP_TYPE_V4: u8 = 4;
pub const IP_TYPE_V6: u8 = 6;

/// Stake information for a hotkey
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StakeInfo {
    /// Hotkey SS58 address
    pub hotkey: String,
    /// Coldkey SS58 address
    pub coldkey: String,
    /// Stake amount in RAO
    pub stake: Balance,
}

/// Chain identity information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChainIdentity {
    pub name: Option<String>,
    pub url: Option<String>,
    pub image: Option<String>,
    pub discord: Option<String>,
    pub description: Option<String>,
    pub additional: HashMap<String, String>,
}

/// Weight commit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightCommitInfo {
    pub hotkey: String,
    pub commit_hash: Vec<u8>,
    pub block: u64,
}

/// Proposal vote data
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ProposalVoteData {
    pub index: u32,
    pub threshold: u32,
    pub ayes: u64,
    pub nays: u64,
    pub end: u64,
}

/// Metagraph info for a subnet (from runtime API)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetagraphInfo {
    pub netuid: u16,
    pub name: String,
    pub symbol: String,
    pub identity: Option<SubnetIdentity>,
    pub network_registered_at: u64,
    pub owner_hotkey: String,
    pub owner_coldkey: String,
    pub block: u64,
    pub tempo: u16,
    pub last_step: u64,
    pub blocks_since_last_step: u64,
    pub subnet_emission: u64,
    pub alpha_in: u64,
    pub alpha_out: u64,
    pub tao_in: u64,
    pub pending_alpha_emission: u64,
    pub pending_root_emission: u64,
    pub subnet_volume: u64,
    pub moving_price: u64,
    pub rho: u16,
    pub kappa: u16,
    pub min_allowed_weights: u16,
    pub max_weights_limit: u16,
    pub weights_version: u64,
    pub weights_rate_limit: u64,
    pub activity_cutoff: u16,
    pub max_validators: u16,
    pub num_uids: u16,
    pub max_uids: u16,
    pub burn: u64,
    pub difficulty: u64,
    pub registration_allowed: bool,
    pub pow_registration_allowed: bool,
    pub immunity_period: u16,
    pub min_difficulty: u64,
    pub max_difficulty: u64,
    pub min_burn: u64,
    pub max_burn: u64,
    pub adjustment_alpha: u64,
    pub bonds_moving_avg: u64,
    pub commit_reveal_enabled: bool,
    pub commit_reveal_period: u64,
    pub liquid_alpha_enabled: bool,
    pub alpha_high: u16,
    pub alpha_low: u16,
    pub hotkeys: Vec<String>,
    pub coldkeys: Vec<String>,
    pub identities: Vec<Option<ChainIdentity>>,
    pub axons: Vec<AxonInfo>,
    pub active: Vec<bool>,
    pub validator_permit: Vec<bool>,
    pub pruning_score: Vec<u16>,
    pub last_update: Vec<u64>,
    pub emission: Vec<u64>,
    pub dividends: Vec<u16>,
    pub incentive: Vec<u16>,
    pub consensus: Vec<u16>,
    pub trust: Vec<u16>,
    pub rank: Vec<u16>,
    pub block_at_registration: Vec<u64>,
    pub alpha_stake: Vec<u64>,
    pub tao_stake: Vec<u64>,
    pub total_stake: Vec<u64>,
    pub tao_dividends_per_hotkey: Vec<(String, u64)>,
    pub alpha_dividends_per_hotkey: Vec<(String, u64)>,
}

/// Dynamic pool info for a subnet
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DynamicInfo {
    pub netuid: u16,
    pub owner_hotkey: String,
    pub owner_coldkey: String,
    pub tempo: u16,
    pub last_step: u64,
    pub blocks_since_last_step: u64,
    pub emission: u64,
    pub alpha_in: u64,
    pub alpha_out: u64,
    pub tao_in: u64,
    pub price: f64,
    pub k: u128,
}

/// Certificate for TLS/secure communication
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Certificate {
    pub certificate: Vec<u8>,
    pub algorithm: String,
}

impl Certificate {
    pub fn new(certificate: Vec<u8>, algorithm: &str) -> Self {
        Self {
            certificate,
            algorithm: algorithm.to_string(),
        }
    }
}
