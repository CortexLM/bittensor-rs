//! Core Metagraph struct holding per-subnet neural graph state.
//!
//! Field names match the Python SDK exactly:
//! n, uids, hotkeys, coldkeys, stake, ranks, trust, consensus,
//! validator_trust, incentive, dividends, emission, weights, bonds,
//! active, axon_info, prometheus_info, block

use std::ops::Index;

use bittensor_core::types::{AxonInfo, NeuronInfo, PrometheusInfo};
use ndarray::Array1;
use serde::{Deserialize, Serialize};

use crate::iter::NeuronIterator;

/// Neural graph state for a single subnet.
///
/// Each field stores per-UID data. All numeric vector fields use
/// `ndarray::Array1<f32>` as the default tensor backend, matching
/// the Python SDK's tensor-based storage. An optional `ml-backend`
/// feature gate reserves the structure for candle/tch backends.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metagraph {
    /// Subnet identifier.
    pub netuid: u16,
    /// Number of registered neurons in the subnet.
    pub n: usize,
    /// UID of each neuron.
    pub uids: Vec<u16>,
    /// Hotkey (public key) of each neuron.
    pub hotkeys: Vec<String>,
    /// Coldkey of each neuron.
    pub coldkeys: Vec<String>,
    /// Stake (in tao) for each neuron.
    pub stake: Array1<f32>,
    /// Rank score for each neuron.
    pub ranks: Array1<f32>,
    /// Trust score for each neuron.
    pub trust: Array1<f32>,
    /// Consensus score for each neuron.
    pub consensus: Array1<f32>,
    /// Validator trust score for each neuron.
    pub validator_trust: Array1<f32>,
    /// Incentive score for each neuron.
    pub incentive: Array1<f32>,
    /// Dividends for each neuron.
    pub dividends: Array1<f32>,
    /// Emission (in rao) for each neuron.
    pub emission: Array1<f32>,
    /// Weight matrix — flattened `[n * n]` (row-major), weights[i*n + j] = weight from i to j.
    pub weights: Array1<f32>,
    /// Bond matrix — flattened `[n * n]` (row-major), bonds[i*n + j] = bond from i to j.
    pub bonds: Array1<f32>,
    /// Whether each neuron is active.
    pub active: Vec<bool>,
    /// Axon serving info for each neuron.
    pub axon_info: Vec<Option<AxonInfo>>,
    /// Prometheus serving info for each neuron.
    pub prometheus_info: Vec<Option<PrometheusInfo>>,
    /// Block number at which this metagraph was synced.
    pub block: u64,
}

impl Metagraph {
    /// Create an empty metagraph for the given netuid.
    pub fn new(netuid: u16) -> Self {
        Self {
            netuid,
            n: 0,
            uids: Vec::new(),
            hotkeys: Vec::new(),
            coldkeys: Vec::new(),
            stake: Array1::zeros(0),
            ranks: Array1::zeros(0),
            trust: Array1::zeros(0),
            consensus: Array1::zeros(0),
            validator_trust: Array1::zeros(0),
            incentive: Array1::zeros(0),
            dividends: Array1::zeros(0),
            emission: Array1::zeros(0),
            weights: Array1::zeros(0),
            bonds: Array1::zeros(0),
            active: Vec::new(),
            axon_info: Vec::new(),
            prometheus_info: Vec::new(),
            block: 0,
        }
    }

    /// Build a metagraph from a vector of [`NeuronInfo`] snapshots.
    ///
    /// This is the primary construction path used by [`sync`](crate::sync::sync).
    pub fn from_neurons(netuid: u16, block: u64, neurons: &[NeuronInfo]) -> Self {
        let n = neurons.len();
        let uids: Vec<u16> = neurons.iter().map(|n| n.uid).collect();
        let hotkeys: Vec<String> = neurons.iter().map(|n| n.hotkey.clone()).collect();
        let coldkeys: Vec<String> = neurons.iter().map(|n| n.coldkey.clone()).collect();
        let active: Vec<bool> = neurons.iter().map(|n| n.active).collect();
        let axon_info: Vec<Option<AxonInfo>> =
            neurons.iter().map(|n| n.axon_info.clone()).collect();
        let prometheus_info: Vec<Option<PrometheusInfo>> =
            neurons.iter().map(|n| n.prometheus_info.clone()).collect();

        let stake = Array1::from_vec(neurons.iter().map(|n| n.stake.to_tao() as f32).collect());
        let ranks = Array1::from_vec(neurons.iter().map(|n| n.rank as f32).collect());
        let trust = Array1::from_vec(neurons.iter().map(|n| n.trust as f32).collect());
        let consensus = Array1::from_vec(neurons.iter().map(|n| n.consensus as f32).collect());
        let validator_trust =
            Array1::from_vec(neurons.iter().map(|n| n.validator_trust as f32).collect());
        let incentive = Array1::from_vec(neurons.iter().map(|n| n.incentive as f32).collect());
        let dividends = Array1::from_vec(neurons.iter().map(|n| n.dividend as f32).collect());
        let emission = Array1::from_vec(neurons.iter().map(|n| n.emission as f32).collect());

        // Flatten weight matrix: each neuron's weights is a flat list of (uid, weight) pairs
        // encoded as [uid0, weight0, uid1, weight1, ...].
        // We expand into a full n×n matrix, row-major.
        let mut weights_flat = vec![0.0f32; n * n];
        for (i, neuron) in neurons.iter().enumerate() {
            let mut j = 0;
            while j + 1 < neuron.weights.len() {
                let dest_uid = neuron.weights[j] as usize;
                let w = neuron.weights[j + 1] as f32;
                if dest_uid < n {
                    weights_flat[i * n + dest_uid] = w;
                }
                j += 2;
            }
        }

        // Flatten bond matrix: same encoding as weights.
        let mut bonds_flat = vec![0.0f32; n * n];
        for (i, neuron) in neurons.iter().enumerate() {
            let mut j = 0;
            while j + 1 < neuron.bonds.len() {
                let dest_uid = neuron.bonds[j] as usize;
                let b = neuron.bonds[j + 1] as f32;
                if dest_uid < n {
                    bonds_flat[i * n + dest_uid] = b;
                }
                j += 2;
            }
        }

        Self {
            netuid,
            n,
            uids,
            hotkeys,
            coldkeys,
            stake,
            ranks,
            trust,
            consensus,
            validator_trust,
            incentive,
            dividends,
            emission,
            weights: Array1::from_vec(weights_flat),
            bonds: Array1::from_vec(bonds_flat),
            active,
            axon_info,
            prometheus_info,
            block,
        }
    }

    /// Return an iterator over [`NeuronInfo`] for each UID in the metagraph.
    ///
    /// Each iteration reconstructs a [`NeuronInfo`] from the metagraph's
    /// columnar storage, matching the Python SDK's `metagraph.neurons()` pattern.
    pub fn neurons(&self) -> NeuronIterator<'_> {
        NeuronIterator { metagraph: self, index: 0 }
    }

    /// Look up a neuron by UID value (not positional index).
    ///
    /// Returns `None` if the UID is not present in the metagraph.
    pub fn neuron_by_uid(&self, uid: u16) -> Option<NeuronInfo> {
        let pos = self.uids.iter().position(|&u| u == uid)?;
        Some(self.neuron_at(pos))
    }

    /// Reconstruct a [`NeuronInfo`] at the given positional index.
    pub fn neuron_at(&self, pos: usize) -> NeuronInfo {
        if pos >= self.n {
            return NeuronInfo {
                uid: 0,
                netuid: self.netuid,
                active: false,
                stake: bittensor_core::balance::Balance::ZERO,
                rank: 0,
                trust: 0,
                consensus: 0,
                incentive: 0,
                dividend: 0,
                emission: 0,
                prometheus_info: None,
                axon_info: None,
                hotkey: String::new(),
                coldkey: String::new(),
                last_update: 0,
                validator_trust: 0,
                weights: Vec::new(),
                bonds: Vec::new(),
                stake_dict: Vec::new(),
            };
        }

        // Reconstruct weights: extract row from flattened matrix
        let row_start = pos * self.n;
        let row_end = row_start + self.n;
        let weight_row = &self.weights.slice(ndarray::s![row_start..row_end]);
        let weights: Vec<u16> = weight_row
            .iter()
            .enumerate()
            .flat_map(|(j, &w)| if w > 0.0 { vec![j as u16, w as u16] } else { vec![] })
            .collect();

        // Reconstruct bonds: extract row from flattened matrix
        let bond_row = &self.bonds.slice(ndarray::s![row_start..row_end]);
        let bonds: Vec<u16> = bond_row
            .iter()
            .enumerate()
            .flat_map(|(j, &b)| if b > 0.0 { vec![j as u16, b as u16] } else { vec![] })
            .collect();

        NeuronInfo {
            uid: self.uids[pos],
            netuid: self.netuid,
            active: self.active[pos],
            stake: bittensor_core::balance::Balance::from_tao(self.stake[pos] as f64),
            rank: self.ranks[pos] as u16,
            trust: self.trust[pos] as u16,
            consensus: self.consensus[pos] as u16,
            incentive: self.incentive[pos] as u16,
            dividend: self.dividends[pos] as u16,
            emission: self.emission[pos] as u64,
            prometheus_info: self.prometheus_info[pos].clone(),
            axon_info: self.axon_info[pos].clone(),
            hotkey: self.hotkeys[pos].clone(),
            coldkey: self.coldkeys[pos].clone(),
            last_update: 0, // Not stored in metagraph; set to 0
            validator_trust: self.validator_trust[pos] as u16,
            weights,
            bonds,
            stake_dict: Vec::new(),
        }
    }
}

impl Index<u16> for Metagraph {
    type Output = ();

    /// Index access by UID: `metagraph[uid]` validates the UID exists.
    ///
    /// For actual data retrieval, use [`Metagraph::neuron_by_uid`] or
    /// [`Metagraph::neuron_at`]. The `Index` trait can't return allocated
    /// values, so this implementation panics if the UID is not found,
    /// matching the Python SDK's `metagraph[uid]` behavior.
    fn index(&self, uid: u16) -> &Self::Output {
        let pos = self.uids.iter().position(|&u| u == uid);
        match pos {
            Some(_) => &(),
            None => panic!("UID {uid} not found in metagraph for netuid {}", self.netuid),
        }
    }
}

/// ML backend trait for alternative tensor storage (feature-gated).
///
/// When the `ml-backend` feature is enabled, implementations can use
/// candle or tch tensors instead of ndarray. This trait defines the
/// interface that any ML backend must satisfy.
#[cfg(feature = "ml-backend")]
pub trait MlBackend: Clone {
    /// The tensor type used by this backend.
    type Tensor: Clone + Send + Sync;

    /// Create a zero-initialized 1-D tensor of the given length.
    fn zeros(len: usize) -> Self::Tensor;

    /// Create a 1-D tensor from a vec of f32 values.
    fn from_vec(data: Vec<f32>) -> Self::Tensor;

    /// Read the value at the given index.
    fn get(tensor: &Self::Tensor, index: usize) -> f32;

    /// Set the value at the given index.
    fn set(tensor: &mut Self::Tensor, index: usize, value: f32);
}

/// Ndarray ML backend (always available as the default).
#[cfg(feature = "ml-backend")]
#[derive(Clone)]
pub struct NdarrayBackend;

#[cfg(feature = "ml-backend")]
impl MlBackend for NdarrayBackend {
    type Tensor = Array1<f32>;

    fn zeros(len: usize) -> Self::Tensor {
        Array1::zeros(len)
    }

    fn from_vec(data: Vec<f32>) -> Self::Tensor {
        Array1::from_vec(data)
    }

    fn get(tensor: &Self::Tensor, index: usize) -> f32 {
        tensor[index]
    }

    fn set(tensor: &mut Self::Tensor, index: usize, value: f32) {
        tensor[index] = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bittensor_core::balance::Balance;
    use bittensor_core::types::{AxonInfo, PrometheusInfo};

    fn make_neuron(uid: u16, netuid: u16) -> NeuronInfo {
        NeuronInfo {
            uid,
            netuid,
            active: true,
            stake: Balance::from_tao((uid as f64) * 10.0),
            rank: uid,
            trust: uid,
            consensus: uid,
            incentive: uid,
            dividend: uid,
            emission: (uid as u64) * 1000,
            prometheus_info: Some(PrometheusInfo {
                ip: 16777343,
                port: 9100,
                version: 1,
                block: 100,
            }),
            axon_info: Some(AxonInfo {
                ip: 2130706433,
                port: 8090,
                ip_type: 4,
                protocol: 0,
                version: 1,
                hotkey: format!("0xhk{uid}"),
                coldkey: format!("0xck{uid}"),
            }),
            hotkey: format!("0xhk{uid}"),
            coldkey: format!("0xck{uid}"),
            last_update: 0,
            validator_trust: uid,
            weights: vec![0, uid, 1, uid],
            bonds: vec![0, uid, 1, uid],
            stake_dict: vec![],
        }
    }

    #[test]
    fn test_new_creates_empty_metagraph() {
        let mg = Metagraph::new(7);
        assert_eq!(mg.netuid, 7);
        assert_eq!(mg.n, 0);
        assert!(mg.uids.is_empty());
        assert!(mg.hotkeys.is_empty());
        assert!(mg.coldkeys.is_empty());
        assert!(mg.active.is_empty());
        assert_eq!(mg.stake.len(), 0);
        assert_eq!(mg.ranks.len(), 0);
        assert_eq!(mg.trust.len(), 0);
        assert_eq!(mg.consensus.len(), 0);
        assert_eq!(mg.validator_trust.len(), 0);
        assert_eq!(mg.incentive.len(), 0);
        assert_eq!(mg.dividends.len(), 0);
        assert_eq!(mg.emission.len(), 0);
        assert_eq!(mg.weights.len(), 0);
        assert_eq!(mg.bonds.len(), 0);
        assert_eq!(mg.block, 0);
    }

    #[test]
    fn test_index_valid_uid() {
        let neurons: Vec<NeuronInfo> = (0..3).map(|uid| make_neuron(uid, 1)).collect();
        let mg = Metagraph::from_neurons(1, 100, &neurons);
        let _ = &mg[0];
        let _ = &mg[1];
        let _ = &mg[2];
    }

    #[test]
    #[should_panic(expected = "UID 99 not found")]
    fn test_index_invalid_uid_panics() {
        let neurons: Vec<NeuronInfo> = (0..3).map(|uid| make_neuron(uid, 1)).collect();
        let mg = Metagraph::from_neurons(1, 100, &neurons);
        let _ = &mg[99];
    }

    #[test]
    fn test_neuron_by_uid_found() {
        let neurons: Vec<NeuronInfo> = (0..5).map(|uid| make_neuron(uid, 3)).collect();
        let mg = Metagraph::from_neurons(3, 500, &neurons);
        let neuron = mg.neuron_by_uid(3).expect("should find uid 3");
        assert_eq!(neuron.uid, 3);
        assert_eq!(neuron.netuid, 3);
        assert_eq!(neuron.hotkey, "0xhk3");
        assert_eq!(neuron.coldkey, "0xck3");
        assert!(neuron.active);
    }

    #[test]
    fn test_neuron_by_uid_not_found() {
        let neurons: Vec<NeuronInfo> = (0..3).map(|uid| make_neuron(uid, 1)).collect();
        let mg = Metagraph::from_neurons(1, 100, &neurons);
        assert!(mg.neuron_by_uid(10).is_none());
    }

    #[test]
    fn test_neuron_at_valid_position() {
        let neurons: Vec<NeuronInfo> = (0..3).map(|uid| make_neuron(uid, 1)).collect();
        let mg = Metagraph::from_neurons(1, 100, &neurons);
        let neuron = mg.neuron_at(1);
        assert_eq!(neuron.uid, 1);
        assert_eq!(neuron.hotkey, "0xhk1");
    }

    #[test]
    fn test_neuron_at_out_of_bounds_returns_default() {
        let mg = Metagraph::new(1);
        let neuron = mg.neuron_at(0);
        assert_eq!(neuron.uid, 0);
        assert!(!neuron.active);
        assert_eq!(neuron.stake, Balance::ZERO);
    }

    #[test]
    fn test_neurons_iterator_yields_all() {
        let neurons: Vec<NeuronInfo> = (0..4).map(|uid| make_neuron(uid, 2)).collect();
        let mg = Metagraph::from_neurons(2, 300, &neurons);
        let collected: Vec<NeuronInfo> = mg.neurons().collect();
        assert_eq!(collected.len(), 4);
        assert_eq!(collected[0].uid, 0);
        assert_eq!(collected[3].uid, 3);
    }

    #[test]
    fn test_neurons_iterator_empty() {
        let mg = Metagraph::new(1);
        let collected: Vec<NeuronInfo> = mg.neurons().collect();
        assert!(collected.is_empty());
    }

    #[test]
    fn test_into_iter_via_reference() {
        let neurons: Vec<NeuronInfo> = (0..2).map(|uid| make_neuron(uid, 1)).collect();
        let mg = Metagraph::from_neurons(1, 100, &neurons);
        let uids: Vec<u16> = (&mg).into_iter().map(|n| n.uid).collect();
        assert_eq!(uids, vec![0, 1]);
    }

    #[test]
    fn test_iterator_exact_size() {
        let neurons: Vec<NeuronInfo> = (0..5).map(|uid| make_neuron(uid, 1)).collect();
        let mg = Metagraph::from_neurons(1, 100, &neurons);
        let mut iter = mg.neurons();
        assert_eq!(iter.len(), 5);
        iter.next();
        assert_eq!(iter.len(), 4);
    }

    #[test]
    fn test_field_lengths_consistency() {
        let neurons: Vec<NeuronInfo> = (0..7).map(|uid| make_neuron(uid, 1)).collect();
        let mg = Metagraph::from_neurons(1, 100, &neurons);
        assert_eq!(mg.uids.len(), mg.n);
        assert_eq!(mg.hotkeys.len(), mg.n);
        assert_eq!(mg.coldkeys.len(), mg.n);
        assert_eq!(mg.active.len(), mg.n);
        assert_eq!(mg.axon_info.len(), mg.n);
        assert_eq!(mg.prometheus_info.len(), mg.n);
        assert_eq!(mg.stake.len(), mg.n);
        assert_eq!(mg.ranks.len(), mg.n);
        assert_eq!(mg.trust.len(), mg.n);
        assert_eq!(mg.consensus.len(), mg.n);
        assert_eq!(mg.validator_trust.len(), mg.n);
        assert_eq!(mg.incentive.len(), mg.n);
        assert_eq!(mg.dividends.len(), mg.n);
        assert_eq!(mg.emission.len(), mg.n);
        assert_eq!(mg.weights.len(), mg.n * mg.n);
        assert_eq!(mg.bonds.len(), mg.n * mg.n);
    }

    #[test]
    fn test_stake_values_from_neurons() {
        let neurons: Vec<NeuronInfo> = (0..3).map(|uid| make_neuron(uid, 1)).collect();
        let mg = Metagraph::from_neurons(1, 100, &neurons);
        assert_eq!(mg.stake[0], 0.0);
        assert_eq!(mg.stake[1], 10.0);
        assert_eq!(mg.stake[2], 20.0);
    }

    #[test]
    fn test_rank_values_from_neurons() {
        let neurons: Vec<NeuronInfo> = (0..3).map(|uid| make_neuron(uid, 1)).collect();
        let mg = Metagraph::from_neurons(1, 100, &neurons);
        assert_eq!(mg.ranks[0], 0.0);
        assert_eq!(mg.ranks[1], 1.0);
        assert_eq!(mg.ranks[2], 2.0);
    }

    #[test]
    fn test_bond_expansion() {
        let mut n0 = make_neuron(0, 1);
        n0.bonds = vec![0, 5, 1, 10];
        let mut n1 = make_neuron(1, 1);
        n1.bonds = vec![];
        let neurons = vec![n0, n1];
        let mg = Metagraph::from_neurons(1, 0, &neurons);

        assert_eq!(mg.bonds[0], 5.0);
        assert_eq!(mg.bonds[1], 10.0);
        assert_eq!(mg.bonds[2], 0.0);
        assert_eq!(mg.bonds[3], 0.0);
    }

    #[test]
    fn test_neuron_at_reconstructs_weights() {
        let mut n0 = make_neuron(0, 1);
        n0.weights = vec![0, 3, 1, 7];
        let mut n1 = make_neuron(1, 1);
        n1.weights = vec![0, 1];
        let neurons = vec![n0, n1];
        let mg = Metagraph::from_neurons(1, 0, &neurons);

        let reconstructed = mg.neuron_at(0);
        assert_eq!(reconstructed.weights, vec![0, 3, 1, 7]);

        let reconstructed1 = mg.neuron_at(1);
        assert_eq!(reconstructed1.weights, vec![0, 1]);
    }

    #[test]
    fn test_neuron_at_preserves_axon_info() {
        let neurons: Vec<NeuronInfo> = (0..2).map(|uid| make_neuron(uid, 1)).collect();
        let mg = Metagraph::from_neurons(1, 0, &neurons);
        let neuron = mg.neuron_at(0);
        assert!(neuron.axon_info.is_some());
        assert_eq!(neuron.axon_info.as_ref().map(|a| a.port.clone()), Some(8090));
    }

    #[test]
    fn test_neuron_at_preserves_prometheus_info() {
        let neurons: Vec<NeuronInfo> = (0..2).map(|uid| make_neuron(uid, 1)).collect();
        let mg = Metagraph::from_neurons(1, 0, &neurons);
        let neuron = mg.neuron_at(0);
        assert!(neuron.prometheus_info.is_some());
        assert_eq!(neuron.prometheus_info.as_ref().map(|p| p.port), Some(9100));
    }

    #[test]
    fn test_clone_is_independent() {
        let neurons: Vec<NeuronInfo> = (0..3).map(|uid| make_neuron(uid, 1)).collect();
        let mg = Metagraph::from_neurons(1, 100, &neurons);
        let mut clone = mg.clone();
        clone.stake[0] = 999.0;
        assert_ne!(mg.stake[0], clone.stake[0]);
    }

    #[test]
    fn test_empty_metagraph_iterator() {
        let mg = Metagraph::new(1);
        assert_eq!(mg.neurons().count(), 0);
    }
}
