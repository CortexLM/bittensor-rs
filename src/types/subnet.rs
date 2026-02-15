use crate::utils::balance_newtypes::Rao;
use serde::{Deserialize, Serialize};

/// Information about a subnet in the Bittensor network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubnetInfo {
    /// Subnet unique identifier
    pub netuid: u16,
    /// Total number of neurons in the subnet
    pub neuron_count: u64,
    /// Total stake in the subnet (RAO)
    pub total_stake: Rao,
    /// Emission rate (RAO)
    pub emission: Rao,
    /// Subnet name
    pub name: Option<String>,
    /// Subnet description
    pub description: Option<String>,
}

impl SubnetInfo {
    pub fn new(netuid: u16) -> Self {
        Self {
            netuid,
            neuron_count: 0,
            total_stake: Rao::ZERO,
            emission: Rao::ZERO,
            name: None,
            description: None,
        }
    }
}

/// Subnet hyperparameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubnetConfigInfo {
    pub min_allowed_weights: u64,
    pub max_weight_limit: u64,
    pub weights_version: u64,
    pub tempo: u64,
    pub max_allowed_uids: u64,
    pub min_stake: Rao,
    pub immunity_period: u64,
    pub min_burn: Rao,
    pub max_burn: Rao,
    pub adjustment_alpha: u64,
    pub target_regs_per_interval: u64,
}

/// Subnet identity information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubnetIdentity {
    /// Subnet name
    pub subnet_name: String,
    /// GitHub repository URL
    pub github_repo: String,
    /// Subnet contact information
    pub subnet_contact: String,
    /// Subnet URL
    pub subnet_url: String,
    /// Logo URL
    pub logo_url: String,
    /// Discord server
    pub discord: String,
    /// Description
    pub description: String,
    /// Additional information
    pub additional: String,
}
