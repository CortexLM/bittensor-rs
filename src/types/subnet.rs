//! Subnet information types

use serde::{Deserialize, Serialize};

use crate::utils::balance::Balance;

/// Basic subnet information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubnetInfo {
    /// Subnet unique identifier
    pub netuid: u16,
    /// Subnet name (symbol)
    pub name: String,
    /// Network modality (0 = text, 1 = image, etc.)
    pub modality: u16,
    /// Number of neurons in subnet
    pub n: u16,
    /// Maximum neurons allowed
    pub max_n: u16,
    /// Emission percentage (0-65535)
    pub emission: u16,
    /// Subnet tempo (blocks per epoch)
    pub tempo: u16,
    /// Burn cost to register
    pub burn: Balance,
    /// Owner hotkey SS58 address
    pub owner: String,
}

impl SubnetInfo {
    /// Get emission as percentage (0.0 - 1.0)
    pub fn emission_percentage(&self) -> f64 {
        self.emission as f64 / u16::MAX as f64
    }

    /// Check if subnet is full
    pub fn is_full(&self) -> bool {
        self.n >= self.max_n
    }
}

impl std::fmt::Display for SubnetInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SubnetInfo( netuid={}, name={}, n={}/{}, emission={:.2}% )",
            self.netuid,
            self.name,
            self.n,
            self.max_n,
            self.emission_percentage() * 100.0
        )
    }
}

/// Subnet identity (on-chain identity)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubnetIdentity {
    pub subnet_name: String,
    pub github_repo: Option<String>,
    pub subnet_contact: Option<String>,
    pub subnet_url: Option<String>,
    pub discord: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
}

impl SubnetIdentity {
    pub fn is_empty(&self) -> bool {
        self.subnet_name.is_empty()
            && self.github_repo.is_none()
            && self.subnet_contact.is_none()
    }
}

/// Subnet hyperparameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubnetHyperparameters {
    /// Rho parameter
    pub rho: u16,
    /// Kappa parameter
    pub kappa: u16,
    /// Immunity period (blocks)
    pub immunity_period: u16,
    /// Minimum allowed weights per validator
    pub min_allowed_weights: u16,
    /// Maximum weight limit
    pub max_weights_limit: u16,
    /// Tempo (blocks per epoch)
    pub tempo: u16,
    /// Minimum difficulty for PoW registration
    pub min_difficulty: u64,
    /// Maximum difficulty for PoW registration
    pub max_difficulty: u64,
    /// Weights version key
    pub weights_version: u64,
    /// Weights rate limit (blocks)
    pub weights_rate_limit: u64,
    /// Adjustment interval (blocks)
    pub adjustment_interval: u64,
    /// Activity cutoff (blocks)
    pub activity_cutoff: u16,
    /// Whether registration is allowed
    pub registration_allowed: bool,
    /// Target registrations per interval
    pub target_regs_per_interval: u16,
    /// Minimum burn to register
    pub min_burn: u64,
    /// Maximum burn to register
    pub max_burn: u64,
    /// Bonds moving average parameter
    pub bonds_moving_avg: u64,
    /// Maximum registrations per block
    pub max_regs_per_block: u16,
    /// Serving rate limit (blocks)
    pub serving_rate_limit: u64,
    /// Maximum validators allowed
    pub max_validators: u16,
    /// Adjustment alpha parameter
    pub adjustment_alpha: u64,
    /// Difficulty for PoW
    pub difficulty: u64,
    /// Whether commit-reveal is enabled
    pub commit_reveal_weights_enabled: bool,
    /// Commit-reveal period (blocks)
    pub commit_reveal_weights_interval: u64,
    /// Alpha high parameter
    pub alpha_high: u16,
    /// Alpha low parameter
    pub alpha_low: u16,
    /// Whether liquid alpha is enabled
    pub liquid_alpha_enabled: bool,
}

impl Default for SubnetHyperparameters {
    fn default() -> Self {
        Self {
            rho: 10,
            kappa: 32767, // 0.5 as u16
            immunity_period: 4096,
            min_allowed_weights: 1,
            max_weights_limit: 1000,
            tempo: 360,
            min_difficulty: 10_000_000,
            max_difficulty: u64::MAX / 4,
            weights_version: 0,
            weights_rate_limit: 100,
            adjustment_interval: 112,
            activity_cutoff: 5000,
            registration_allowed: true,
            target_regs_per_interval: 2,
            min_burn: 1_000_000_000, // 1 TAO
            max_burn: 100_000_000_000, // 100 TAO
            bonds_moving_avg: 900_000,
            max_regs_per_block: 1,
            serving_rate_limit: 10,
            max_validators: 64,
            adjustment_alpha: 0,
            difficulty: 10_000_000,
            commit_reveal_weights_enabled: false,
            commit_reveal_weights_interval: 1000,
            alpha_high: 58982, // 0.9 as u16
            alpha_low: 45875, // 0.7 as u16
            liquid_alpha_enabled: false,
        }
    }
}

/// Subnet state for a specific block
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubnetState {
    pub netuid: u16,
    pub block: u64,
    pub n: u16,
    pub uids: Vec<u16>,
    pub hotkeys: Vec<String>,
    pub coldkeys: Vec<String>,
    pub stakes: Vec<u64>,
    pub ranks: Vec<u16>,
    pub trust: Vec<u16>,
    pub consensus: Vec<u16>,
    pub incentive: Vec<u16>,
    pub dividends: Vec<u16>,
    pub emission: Vec<u64>,
    pub active: Vec<bool>,
    pub validator_permit: Vec<bool>,
    pub last_update: Vec<u64>,
    pub pruning_scores: Vec<u16>,
}
