//! Metagraph - the neural graph representing subnet state
//!
//! The Metagraph is a core component of the Bittensor network, representing
//! the state of a subnet including all neurons, their stakes, rankings,
//! and relationships.
//!
//! # Example
//!
//! ```ignore
//! use bittensor_rs::{Subtensor, Metagraph};
//!
//! let subtensor = Subtensor::new("finney").await?;
//! let metagraph = subtensor.metagraph(1).await?;
//!
//! // Access neuron data
//! println!("Total neurons: {}", metagraph.n);
//! println!("Total stake: {}", metagraph.total_stake());
//!
//! // Iterate over neurons
//! for uid in 0..metagraph.n as usize {
//!     println!("UID {}: stake={}", uid, metagraph.stake[uid]);
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::types::{AxonInfo, NeuronInfoLite};
use crate::utils::balance::Balance;

/// Metagraph containing all neurons and state for a subnet
///
/// This structure mirrors the Python SDK's Metagraph class, providing
/// tensor-like arrays for neuron attributes that can be accessed by UID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metagraph {
    /// Subnet unique identifier
    pub netuid: u16,
    /// Network name (e.g., "finney", "test")
    pub network: String,
    /// Metagraph version
    pub version: u64,
    /// Total number of neurons in the subnet
    pub n: u16,
    /// Current block number
    pub block: u64,

    // === Neuron Arrays (indexed by UID) ===
    /// Unique identifiers (0 to n-1)
    pub uids: Vec<u16>,
    /// Stake amounts per neuron (in RAO)
    pub stake: Vec<u64>,
    /// TAO stake per neuron
    pub tao_stake: Vec<u64>,
    /// Alpha stake per neuron
    pub alpha_stake: Vec<u64>,
    /// Rank scores (normalized 0.0-1.0)
    pub ranks: Vec<f64>,
    /// Trust scores (normalized 0.0-1.0)
    pub trust: Vec<f64>,
    /// Consensus scores (normalized 0.0-1.0)
    pub consensus: Vec<f64>,
    /// Validator trust scores (normalized 0.0-1.0)
    pub validator_trust: Vec<f64>,
    /// Incentive scores (normalized 0.0-1.0)
    pub incentive: Vec<f64>,
    /// Emission rates per neuron
    pub emission: Vec<u64>,
    /// Dividend scores (normalized 0.0-1.0)
    pub dividends: Vec<f64>,
    /// Whether each neuron is active
    pub active: Vec<bool>,
    /// Last update block for each neuron
    pub last_update: Vec<u64>,
    /// Whether each neuron has validator permit
    pub validator_permit: Vec<bool>,
    /// Pruning scores
    pub pruning_score: Vec<u16>,

    // === Identity Information ===
    /// Hotkey addresses (SS58)
    pub hotkeys: Vec<String>,
    /// Coldkey addresses (SS58)
    pub coldkeys: Vec<String>,
    /// Axon endpoint information
    pub axons: Vec<AxonInfo>,

    // === Weights and Bonds (optional, can be large) ===
    /// Weight matrix: weights[i] = [(uid, weight), ...] set by neuron i
    #[serde(default)]
    pub weights: Vec<Vec<(u16, u16)>>,
    /// Bond matrix: bonds[i] = [(uid, amount), ...] held by neuron i
    #[serde(default)]
    pub bonds: Vec<Vec<(u16, u64)>>,

    // === Subnet Parameters ===
    /// Subnet tempo (blocks per epoch)
    pub tempo: u16,
    /// Subnet name
    pub name: String,
    /// Subnet symbol
    pub symbol: String,
    /// Block at which each neuron was registered
    pub block_at_registration: Vec<u64>,
}

impl Metagraph {
    /// Create a new empty metagraph for a subnet
    pub fn new(netuid: u16, network: &str) -> Self {
        Self {
            netuid,
            network: network.to_string(),
            version: 0,
            n: 0,
            block: 0,
            uids: Vec::new(),
            stake: Vec::new(),
            tao_stake: Vec::new(),
            alpha_stake: Vec::new(),
            ranks: Vec::new(),
            trust: Vec::new(),
            consensus: Vec::new(),
            validator_trust: Vec::new(),
            incentive: Vec::new(),
            emission: Vec::new(),
            dividends: Vec::new(),
            active: Vec::new(),
            last_update: Vec::new(),
            validator_permit: Vec::new(),
            pruning_score: Vec::new(),
            hotkeys: Vec::new(),
            coldkeys: Vec::new(),
            axons: Vec::new(),
            weights: Vec::new(),
            bonds: Vec::new(),
            tempo: 360,
            name: String::new(),
            symbol: String::new(),
            block_at_registration: Vec::new(),
        }
    }

    // === Property Accessors (matching Python SDK's property names) ===

    /// Total stake (S) - alias for stake
    pub fn s(&self) -> &[u64] {
        &self.stake
    }

    /// Ranks (R) - alias for ranks
    pub fn r(&self) -> &[f64] {
        &self.ranks
    }

    /// Incentive (I) - alias for incentive
    pub fn i(&self) -> &[f64] {
        &self.incentive
    }

    /// Emission (E) - alias for emission
    pub fn e(&self) -> &[u64] {
        &self.emission
    }

    /// Consensus (C) - alias for consensus
    pub fn c(&self) -> &[f64] {
        &self.consensus
    }

    /// Trust (T) - alias for trust
    pub fn t(&self) -> &[f64] {
        &self.trust
    }

    /// Validator Trust (Tv) - alias for validator_trust
    pub fn tv(&self) -> &[f64] {
        &self.validator_trust
    }

    /// Dividends (D) - alias for dividends
    pub fn d(&self) -> &[f64] {
        &self.dividends
    }

    /// Bonds (B) - alias for bonds
    pub fn b(&self) -> &[Vec<(u16, u64)>] {
        &self.bonds
    }

    /// Weights (W) - alias for weights
    pub fn w(&self) -> &[Vec<(u16, u16)>] {
        &self.weights
    }

    /// TAO stake (TS)
    pub fn ts(&self) -> &[u64] {
        &self.tao_stake
    }

    /// Alpha stake (AS)
    pub fn as_(&self) -> &[u64] {
        &self.alpha_stake
    }

    // === Utility Methods ===

    /// Get total stake in the subnet
    pub fn total_stake(&self) -> Balance {
        Balance::from_rao(self.stake.iter().sum())
    }

    /// Get total TAO stake in the subnet
    pub fn total_tao_stake(&self) -> Balance {
        Balance::from_rao(self.tao_stake.iter().sum())
    }

    /// Get total alpha stake in the subnet
    pub fn total_alpha_stake(&self) -> Balance {
        Balance::from_rao(self.alpha_stake.iter().sum())
    }

    /// Get neuron by UID
    pub fn get_neuron(&self, uid: u16) -> Option<NeuronInfoLite> {
        let idx = uid as usize;
        if idx >= self.n as usize {
            return None;
        }
        
        Some(NeuronInfoLite {
            hotkey: self.hotkeys.get(idx).cloned().unwrap_or_default(),
            coldkey: self.coldkeys.get(idx).cloned().unwrap_or_default(),
            uid,
            netuid: self.netuid,
            active: self.active.get(idx).copied().unwrap_or(false),
            stake: Balance::from_rao(self.stake.get(idx).copied().unwrap_or(0)),
            stake_dict: HashMap::new(),
            total_stake: Balance::from_rao(self.stake.get(idx).copied().unwrap_or(0)),
            rank: self.ranks.get(idx).copied().unwrap_or(0.0),
            emission: self.emission.get(idx).copied().unwrap_or(0) as f64 / 1e9,
            incentive: self.incentive.get(idx).copied().unwrap_or(0.0),
            consensus: self.consensus.get(idx).copied().unwrap_or(0.0),
            trust: self.trust.get(idx).copied().unwrap_or(0.0),
            validator_trust: self.validator_trust.get(idx).copied().unwrap_or(0.0),
            dividends: self.dividends.get(idx).copied().unwrap_or(0.0),
            last_update: self.last_update.get(idx).copied().unwrap_or(0),
            validator_permit: self.validator_permit.get(idx).copied().unwrap_or(false),
            pruning_score: self.pruning_score.get(idx).copied().unwrap_or(0),
            prometheus_info: None,
            axon_info: self.axons.get(idx).cloned(),
            is_null: false,
        })
    }

    /// Get neuron by hotkey
    pub fn get_neuron_by_hotkey(&self, hotkey: &str) -> Option<NeuronInfoLite> {
        self.hotkeys
            .iter()
            .position(|h| h == hotkey)
            .and_then(|idx| self.get_neuron(idx as u16))
    }

    /// Get UID for a hotkey
    pub fn get_uid(&self, hotkey: &str) -> Option<u16> {
        self.hotkeys
            .iter()
            .position(|h| h == hotkey)
            .map(|idx| idx as u16)
    }

    /// Get all validators (neurons with validator_permit)
    pub fn validators(&self) -> Vec<u16> {
        self.validator_permit
            .iter()
            .enumerate()
            .filter(|(_, &permit)| permit)
            .map(|(idx, _)| idx as u16)
            .collect()
    }

    /// Get all active neurons
    pub fn active_neurons(&self) -> Vec<u16> {
        self.active
            .iter()
            .enumerate()
            .filter(|(_, &active)| active)
            .map(|(idx, _)| idx as u16)
            .collect()
    }

    /// Get neurons sorted by stake (descending)
    pub fn neurons_by_stake(&self) -> Vec<u16> {
        let mut indices: Vec<u16> = (0..self.n).collect();
        indices.sort_by(|&a, &b| {
            let stake_a = self.stake.get(a as usize).unwrap_or(&0);
            let stake_b = self.stake.get(b as usize).unwrap_or(&0);
            stake_b.cmp(stake_a)
        });
        indices
    }

    /// Get neurons sorted by incentive (descending)
    pub fn neurons_by_incentive(&self) -> Vec<u16> {
        let mut indices: Vec<u16> = (0..self.n).collect();
        indices.sort_by(|&a, &b| {
            let inc_a = self.incentive.get(a as usize).unwrap_or(&0.0);
            let inc_b = self.incentive.get(b as usize).unwrap_or(&0.0);
            inc_b.partial_cmp(inc_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        indices
    }

    /// Check if a hotkey is registered
    pub fn is_registered(&self, hotkey: &str) -> bool {
        self.hotkeys.contains(&hotkey.to_string())
    }

    // === Serialization ===

    /// Save metagraph to file
    pub fn save(&self, path: &PathBuf) -> crate::Result<()> {
        let content = serde_json::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Load metagraph from file
    pub fn load(path: &PathBuf) -> crate::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let metagraph: Self = serde_json::from_str(&content)?;
        Ok(metagraph)
    }

    /// Get save directory path
    pub fn get_save_dir(network: &str, netuid: u16) -> PathBuf {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".bittensor")
            .join("metagraphs")
            .join(format!("network-{}", network))
            .join(format!("netuid-{}", netuid))
    }
}

impl std::fmt::Display for Metagraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Metagraph( netuid={}, network={}, n={}, block={}, total_stake={} )",
            self.netuid,
            self.network,
            self.n,
            self.block,
            self.total_stake()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metagraph_new() {
        let mg = Metagraph::new(1, "finney");
        assert_eq!(mg.netuid, 1);
        assert_eq!(mg.network, "finney");
        assert_eq!(mg.n, 0);
    }

    #[test]
    fn test_metagraph_total_stake() {
        let mut mg = Metagraph::new(1, "finney");
        mg.stake = vec![1_000_000_000, 2_000_000_000, 3_000_000_000];
        mg.n = 3;
        
        let total = mg.total_stake();
        assert!((total.tao() - 6.0).abs() < 1e-9);
    }
}
