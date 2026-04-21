use crate::types::{AxonInfo, NeuronInfo};
use crate::utils::balance_newtypes::Rao;
use sp_core::crypto::AccountId32;
use std::collections::HashMap;

/// Metagraph containing all neurons and state for a subnet
#[derive(Debug, Clone)]
pub struct Metagraph {
    /// Network UID (subnet ID)
    pub netuid: u16,
    /// Current block number
    pub block: u64,
    /// Total number of neurons
    pub n: u64,
    /// List of neurons indexed by UID
    pub neurons: HashMap<u64, NeuronInfo>,
    /// Axon information indexed by UID
    pub axons: HashMap<u64, AxonInfo>,
    /// Version
    pub version: u64,
    /// Hotkey list indexed by UID
    pub hotkeys: Vec<AccountId32>,
    /// Coldkey list indexed by UID
    pub coldkeys: Vec<AccountId32>,
    /// Validator permit list indexed by UID
    pub validator_permit: Vec<bool>,
    /// Active list indexed by UID
    pub active: Vec<bool>,
}

impl Metagraph {
    pub fn new(netuid: u16) -> Self {
        Self {
            netuid,
            block: 0,
            n: 0,
            neurons: HashMap::new(),
            axons: HashMap::new(),
            version: 0,
            hotkeys: Vec::new(),
            coldkeys: Vec::new(),
            validator_permit: Vec::new(),
            active: Vec::new(),
        }
    }

    /// Get neuron by UID
    pub fn get_neuron(&self, uid: u64) -> Option<&NeuronInfo> {
        self.neurons.get(&uid)
    }

    /// Get axon by UID
    pub fn get_axon(&self, uid: u64) -> Option<&AxonInfo> {
        self.axons.get(&uid)
    }

    /// Get all active neurons
    pub fn active_neurons(&self) -> Vec<&NeuronInfo> {
        self.neurons.values().filter(|n| n.active).collect()
    }

    /// Get all validators (neurons with validator_permit)
    pub fn validators(&self) -> Vec<&NeuronInfo> {
        self.neurons
            .values()
            .filter(|n| n.validator_permit)
            .collect()
    }

    /// Get neuron by hotkey
    pub fn get_neuron_by_hotkey(&self, hotkey: &AccountId32) -> Option<&NeuronInfo> {
        self.neurons.values().find(|n| &n.hotkey == hotkey)
    }

    /// Get total stake in the subnet
    pub fn total_stake(&self) -> Rao {
        self.neurons.values().map(|n| n.stake).sum()
    }
}
