//! MetagraphInfo type - comprehensive metagraph data structure
//! Matches Python's bittensor.core.chain_data.MetagraphInfo

use serde::{Deserialize, Serialize};


/// Metagraph info parameters
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetagraphInfoParams {
    pub activity_cutoff: u64,
    pub adjustment_alpha: u64,
    pub adjustment_interval: u64,
    pub alpha_high: u64,
    pub alpha_low: u64,
    pub bonds_moving_avg: u64,
    pub burn: f64,
    pub commit_reveal_period: u64,
    pub commit_reveal_weights_enabled: bool,
    pub difficulty: u128,
    pub immunity_period: u64,
    pub kappa: u64,
    pub liquid_alpha_enabled: bool,
    pub max_burn: f64,
    pub max_difficulty: u128,
    pub max_regs_per_block: u64,
    pub max_validators: u64,
    pub max_weights_limit: u64,
    pub min_allowed_weights: u64,
    pub min_burn: f64,
    pub min_difficulty: u128,
    pub pow_registration_allowed: bool,
    pub registration_allowed: bool,
    pub rho: u64,
    pub serving_rate_limit: u64,
    pub target_regs_per_interval: u64,
    pub tempo: u64,
    pub weights_rate_limit: u64,
    pub weights_version: u64,
}

/// Metagraph info pool data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetagraphInfoPool {
    pub alpha_out: f64,
    pub alpha_in: f64,
    pub tao_in: f64,
    pub subnet_volume: f64,
    pub moving_price: f64,
}

/// Metagraph info emissions data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetagraphInfoEmissions {
    pub alpha_out_emission: f64,
    pub alpha_in_emission: f64,
    pub subnet_emission: f64,
    pub tao_in_emission: f64,
    pub pending_alpha_emission: f64,
    pub pending_root_emission: f64,
}

/// Chain identity
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChainIdentity {
    pub name: Option<String>,
    pub url: Option<String>,
    pub github_repo: Option<String>,
    pub image: Option<String>,
    pub discord: Option<String>,
    pub description: Option<String>,
    pub additional: Option<String>,
}

/// Subnet identity
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubnetIdentityInfo {
    pub subnet_name: String,
    pub github_repo: String,
    pub subnet_contact: String,
    pub subnet_url: String,
    pub logo_url: String,
    pub discord: String,
    pub description: String,
    pub additional: String,
}

/// Complete MetagraphInfo structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetagraphInfo {
    // Core identifiers
    pub netuid: u16,
    pub mechid: u16,
    pub name: String,
    pub symbol: String,
    
    // Network state
    pub network_registered_at: u64,
    pub num_uids: u64,
    pub max_uids: u64,
    pub block: u64,
    
    // Ownership
    pub owner_coldkey: String,
    pub owner_hotkey: String,
    
    // Timing
    pub last_step: u64,
    pub tempo: u64,
    pub blocks_since_last_step: u64,
    
    // Neuron data arrays
    pub hotkeys: Vec<String>,
    pub coldkeys: Vec<String>,
    pub active: Vec<bool>,
    pub validator_permit: Vec<bool>,
    pub pruning_score: Vec<f64>,
    pub last_update: Vec<u64>,
    pub block_at_registration: Vec<u64>,
    
    // Stake data
    pub alpha_stake: Vec<f64>,
    pub tao_stake: Vec<f64>,
    pub total_stake: Vec<f64>,
    
    // Performance metrics
    pub emission: Vec<f64>,
    pub incentive: Vec<f64>,
    pub consensus: Vec<f64>,
    pub trust: Vec<f64>,
    pub validator_trust: Vec<f64>,
    pub dividends: Vec<f64>,
    pub rank: Vec<f64>,
    
    // Dividends per hotkey
    pub tao_dividends_per_hotkey: Vec<(String, f64)>,
    pub alpha_dividends_per_hotkey: Vec<(String, f64)>,
    
    // Identities
    pub identities: Vec<Option<ChainIdentity>>,
    pub identity: Option<SubnetIdentityInfo>,
    
    // Parameters, pool, emissions
    pub hparams: MetagraphInfoParams,
    pub pool: MetagraphInfoPool,
    pub emissions: MetagraphInfoEmissions,
    
    // Mechanism data
    pub mechanism_count: u16,
    pub mechanisms_emissions_split: Vec<u64>,
    
    // Weights and bonds (sparse representation)
    pub weights: Vec<Vec<(u16, u16)>>,
    pub bonds: Vec<Vec<(u16, u16)>>,
}

impl Default for MetagraphInfo {
    fn default() -> Self {
        Self {
            netuid: 0,
            mechid: 0,
            name: String::new(),
            symbol: String::new(),
            network_registered_at: 0,
            num_uids: 0,
            max_uids: 0,
            block: 0,
            owner_coldkey: String::new(),
            owner_hotkey: String::new(),
            last_step: 0,
            tempo: 0,
            blocks_since_last_step: 0,
            hotkeys: Vec::new(),
            coldkeys: Vec::new(),
            active: Vec::new(),
            validator_permit: Vec::new(),
            pruning_score: Vec::new(),
            last_update: Vec::new(),
            block_at_registration: Vec::new(),
            alpha_stake: Vec::new(),
            tao_stake: Vec::new(),
            total_stake: Vec::new(),
            emission: Vec::new(),
            incentive: Vec::new(),
            consensus: Vec::new(),
            trust: Vec::new(),
            validator_trust: Vec::new(),
            dividends: Vec::new(),
            rank: Vec::new(),
            tao_dividends_per_hotkey: Vec::new(),
            alpha_dividends_per_hotkey: Vec::new(),
            identities: Vec::new(),
            identity: None,
            hparams: MetagraphInfoParams::default(),
            pool: MetagraphInfoPool::default(),
            emissions: MetagraphInfoEmissions::default(),
            mechanism_count: 1,
            mechanisms_emissions_split: Vec::new(),
            weights: Vec::new(),
            bonds: Vec::new(),
        }
    }
}

impl MetagraphInfo {
    pub fn new(netuid: u16) -> Self {
        Self {
            netuid,
            ..Default::default()
        }
    }

    /// Get neuron count
    pub fn n(&self) -> usize {
        self.hotkeys.len()
    }

    /// Get hotkey by UID
    pub fn get_hotkey(&self, uid: usize) -> Option<&String> {
        self.hotkeys.get(uid)
    }

    /// Get coldkey by UID
    pub fn get_coldkey(&self, uid: usize) -> Option<&String> {
        self.coldkeys.get(uid)
    }

    /// Get stake by UID
    pub fn get_stake(&self, uid: usize) -> Option<f64> {
        self.total_stake.get(uid).copied()
    }

    /// Get all validators (UIDs with validator_permit)
    pub fn get_validators(&self) -> Vec<usize> {
        self.validator_permit
            .iter()
            .enumerate()
            .filter_map(|(i, &permit)| if permit { Some(i) } else { None })
            .collect()
    }

    /// Get all active neurons
    pub fn get_active_neurons(&self) -> Vec<usize> {
        self.active
            .iter()
            .enumerate()
            .filter_map(|(i, &active)| if active { Some(i) } else { None })
            .collect()
    }

    /// Total stake in subnet
    pub fn total_subnet_stake(&self) -> f64 {
        self.total_stake.iter().sum()
    }

    /// Find UID by hotkey
    pub fn find_uid_by_hotkey(&self, hotkey: &str) -> Option<usize> {
        self.hotkeys.iter().position(|h| h == hotkey)
    }
}

/// Selective metagraph index for partial queries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum SelectiveMetagraphIndex {
    Netuid = 0,
    Hotkeys = 1,
    Coldkeys = 2,
    Active = 3,
    ValidatorPermit = 4,
    PruningScore = 5,
    LastUpdate = 6,
    BlockAtRegistration = 7,
    AlphaStake = 8,
    TaoStake = 9,
    TotalStake = 10,
    Emission = 11,
    Incentive = 12,
    Consensus = 13,
    Trust = 14,
    ValidatorTrust = 15,
    Dividends = 16,
    Rank = 17,
    TaoDividendsPerHotkey = 18,
    AlphaDividendsPerHotkey = 19,
    Identities = 20,
    OwnerHotkeys = 21,
    Name = 22,
    Symbol = 23,
    Identity = 24,
    NetworkRegisteredAt = 25,
    NumUids = 26,
    MaxUids = 27,
    Block = 28,
    OwnerColdkey = 29,
    OwnerHotkey = 30,
    LastStep = 31,
    Tempo = 32,
    BlocksSinceLastStep = 33,
    Weights = 34,
    Bonds = 35,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metagraph_info_creation() {
        let info = MetagraphInfo::new(1);
        assert_eq!(info.netuid, 1);
        assert_eq!(info.n(), 0);
    }

    #[test]
    fn test_metagraph_with_neurons() {
        let mut info = MetagraphInfo::new(1);
        info.hotkeys = vec!["hotkey1".to_string(), "hotkey2".to_string()];
        info.coldkeys = vec!["coldkey1".to_string(), "coldkey2".to_string()];
        info.validator_permit = vec![true, false];
        info.active = vec![true, true];
        info.total_stake = vec![100.0, 50.0];

        assert_eq!(info.n(), 2);
        assert_eq!(info.get_validators(), vec![0]);
        assert_eq!(info.get_active_neurons(), vec![0, 1]);
        assert_eq!(info.total_subnet_stake(), 150.0);
        assert_eq!(info.find_uid_by_hotkey("hotkey2"), Some(1));
    }
}
