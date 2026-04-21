//! Metagraph sync — fetch all neurons from chain and build a [`Metagraph`].

use bittensor_chain::client::SubtensorClient;
use bittensor_core::error::BittensorError;

use crate::metagraph::Metagraph;

/// Alias for results produced by metagraph operations.
pub type Result<T> = std::result::Result<T, BittensorError>;

/// Populate all metagraph fields from the chain for the given netuid.
///
/// Fetches each neuron individually and builds the full columnar
/// structure via [`Metagraph::from_neurons`].
pub async fn sync(client: &SubtensorClient, netuid: u16) -> Result<Metagraph> {
    let rpc = client.rpc();

    let block_info = bittensor_chain::queries::get_network_block(rpc).await?;
    let block = block_info;

    let neuron_count = bittensor_chain::queries::get_neuron_count(rpc, netuid).await?;
    let n = neuron_count as usize;

    let mut neurons = Vec::with_capacity(n);
    for uid in 0..neuron_count {
        match bittensor_chain::queries::get_neuron(rpc, netuid, uid).await {
            Ok(Some(neuron)) => neurons.push(neuron),
            Ok(None) => {}
            Err(e) => return Err(e),
        }
    }

    Ok(Metagraph::from_neurons(netuid, block, &neurons))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bittensor_core::balance::Balance;
    use bittensor_core::types::{AxonInfo, NeuronInfo, PrometheusInfo};

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
                hotkey: format!("0xhotkey{uid}"),
                coldkey: format!("0xcoldkey{uid}"),
            }),
            hotkey: format!("0xhotkey{uid}"),
            coldkey: format!("0xcoldkey{uid}"),
            last_update: 0,
            validator_trust: uid,
            weights: vec![0, uid, 1, uid],
            bonds: vec![0, uid, 1, uid],
            stake_dict: vec![],
        }
    }

    #[test]
    fn test_from_neurons_builds_consistent_metagraph() {
        let neurons: Vec<NeuronInfo> = (0..3).map(|uid| make_neuron(uid, 1)).collect();
        let mg = Metagraph::from_neurons(1, 500, &neurons);

        assert_eq!(mg.n, 3);
        assert_eq!(mg.uids, vec![0, 1, 2]);
        assert_eq!(mg.block, 500);
        assert_eq!(mg.netuid, 1);
        assert_eq!(mg.hotkeys, vec!["0xhotkey0", "0xhotkey1", "0xhotkey2"]);
        assert_eq!(mg.coldkeys, vec!["0xcoldkey0", "0xcoldkey1", "0xcoldkey2"]);
        assert_eq!(mg.active, vec![true, true, true]);
        assert_eq!(mg.stake.len(), 3);
        assert_eq!(mg.ranks.len(), 3);
        assert_eq!(mg.weights.len(), 9); // 3x3 matrix flattened
        assert_eq!(mg.bonds.len(), 9);
    }

    #[test]
    fn test_from_neurons_empty_subnet() {
        let neurons: Vec<NeuronInfo> = vec![];
        let mg = Metagraph::from_neurons(42, 100, &neurons);

        assert_eq!(mg.n, 0);
        assert_eq!(mg.uids.len(), 0);
        assert_eq!(mg.stake.len(), 0);
        assert_eq!(mg.weights.len(), 0);
        assert_eq!(mg.bonds.len(), 0);
    }

    #[test]
    fn test_from_neurons_single_neuron() {
        let neurons = vec![make_neuron(0, 5)];
        let mg = Metagraph::from_neurons(5, 200, &neurons);

        assert_eq!(mg.n, 1);
        assert_eq!(mg.uids, vec![0]);
        assert_eq!(mg.stake[0], 0.0);
        assert_eq!(mg.weights.len(), 1); // 1x1 matrix
    }

    #[test]
    fn test_from_neurons_weight_expansion() {
        let mut n0 = make_neuron(0, 1);
        n0.weights = vec![0, 5, 2, 10];
        let mut n1 = make_neuron(1, 1);
        n1.weights = vec![];
        let mut n2 = make_neuron(2, 1);
        n2.weights = vec![1, 7];

        let neurons = vec![n0, n1, n2];
        let mg = Metagraph::from_neurons(1, 0, &neurons);

        assert_eq!(mg.weights[0], 5.0);
        assert_eq!(mg.weights[1], 0.0);
        assert_eq!(mg.weights[2], 10.0);
        assert_eq!(mg.weights[3], 0.0);
        assert_eq!(mg.weights[4], 0.0);
        assert_eq!(mg.weights[5], 0.0);
        assert_eq!(mg.weights[6], 0.0);
        assert_eq!(mg.weights[7], 7.0);
        assert_eq!(mg.weights[8], 0.0);
    }
}
