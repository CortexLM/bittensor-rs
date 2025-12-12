//! DynamicInfo type - comprehensive subnet information
//! Matches Python's bittensor.core.chain_data.DynamicInfo

use serde::{Deserialize, Serialize};

/// Dynamic information about a subnet
/// Contains runtime state and pool information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct DynamicInfo {
    /// Subnet unique identifier
    pub netuid: u16,
    /// Owner coldkey SS58 address
    pub owner_coldkey: String,
    /// Owner hotkey SS58 address  
    pub owner_hotkey: String,
    /// Subnet name/symbol
    pub symbol: String,
    /// Tempo (blocks per epoch)
    pub tempo: u64,
    /// Last step block
    pub last_step: u64,
    /// Blocks since last step
    pub blocks_since_last_step: u64,
    /// Whether subnet is active
    pub is_active: bool,
    /// Block at which subnet was registered
    pub network_registered_at: u64,
    /// Current number of UIDs
    pub subnet_n: u64,
    /// Maximum number of UIDs allowed
    pub max_n: u64,
    /// Emission value per block
    pub emission_value: u128,
    /// Current burn cost for registration
    pub burn: u128,
    /// Pending emission
    pub pending_emission: u128,
    /// Alpha IN amount (in pool)
    pub alpha_in: u128,
    /// Alpha OUT amount
    pub alpha_out: u128,
    /// TAO IN amount (in pool)
    pub tao_in: u128,
    /// Alpha outstanding (total supply)
    pub alpha_out_emission: u128,
    /// TAO in emission
    pub tao_in_emission: u128,
    /// Pending root emission
    pub pending_root_emission: u128,
    /// Network connect (connectivity parameters)
    pub network_connect: Vec<(u16, u16)>,
    /// Subnet volume
    pub subnet_volume: u128,
    /// Moving price average
    pub moving_price: u128,
    /// Current alpha price
    pub price: u128,
}


impl DynamicInfo {
    /// Create new DynamicInfo for a subnet
    pub fn new(netuid: u16) -> Self {
        Self {
            netuid,
            ..Default::default()
        }
    }

    /// Get alpha price as f64 (in TAO)
    pub fn alpha_price_tao(&self) -> f64 {
        self.price as f64 / 1e9
    }

    /// Get emission as TAO per block
    pub fn emission_tao(&self) -> f64 {
        self.emission_value as f64 / 1e9
    }

    /// Get burn cost as TAO
    pub fn burn_tao(&self) -> f64 {
        self.burn as f64 / 1e9
    }

    /// Calculate total alpha supply
    pub fn total_alpha_supply(&self) -> u128 {
        self.alpha_in.saturating_add(self.alpha_out)
    }

    /// Check if subnet has liquidity
    pub fn has_liquidity(&self) -> bool {
        self.alpha_in > 0 && self.tao_in > 0
    }
}

/// Extended subnet state information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubnetState {
    /// Subnet unique identifier
    pub netuid: u16,
    /// Current block number
    pub block: u64,
    /// Number of neurons
    pub n: u64,
    /// Maximum neurons allowed
    pub max_n: u64,
    /// Tempo
    pub tempo: u64,
    /// Immunity period
    pub immunity_period: u64,
    /// Minimum stake required
    pub min_stake: u128,
    /// Maximum weight limit (normalized)
    pub max_weight_limit: f64,
    /// Minimum allowed weights
    pub min_allowed_weights: u64,
    /// Whether registration is allowed
    pub registration_allowed: bool,
    /// Whether POW registration is allowed
    pub pow_registration_allowed: bool,
    /// Current difficulty
    pub difficulty: u128,
    /// Current burn cost
    pub burn: u128,
    /// Weights rate limit (blocks)
    pub weights_rate_limit: u64,
    /// Adjustment alpha
    pub adjustment_alpha: u64,
    /// Bonds moving average
    pub bonds_moving_avg: u64,
    /// Commit reveal enabled
    pub commit_reveal_enabled: bool,
    /// Commit reveal period
    pub commit_reveal_period: u64,
    /// Liquid alpha enabled
    pub liquid_alpha_enabled: bool,
    /// Alpha high value
    pub alpha_high: u64,
    /// Alpha low value
    pub alpha_low: u64,
    /// Kappa value
    pub kappa: u64,
    /// Rho value
    pub rho: u64,
}

impl SubnetState {
    pub fn new(netuid: u16) -> Self {
        Self {
            netuid,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_info_creation() {
        let info = DynamicInfo::new(1);
        assert_eq!(info.netuid, 1);
        assert!(!info.has_liquidity());
    }

    #[test]
    fn test_dynamic_info_with_liquidity() {
        let mut info = DynamicInfo::new(1);
        info.alpha_in = 1_000_000_000;
        info.tao_in = 500_000_000;
        assert!(info.has_liquidity());
    }

    #[test]
    fn test_price_conversion() {
        let mut info = DynamicInfo::new(1);
        info.price = 1_500_000_000; // 1.5 TAO
        assert!((info.alpha_price_tao() - 1.5).abs() < 0.0001);
    }
}
