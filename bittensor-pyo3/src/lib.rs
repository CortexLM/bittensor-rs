//! bittensor-pyo3 — Python bindings for the bittensor-rs SDK via PyO3.
//!
//! Package name in Python: `bittensor_rs`

mod axon;
mod chain_client;
mod core_types;
mod dendrite;
mod metagraph;
mod synapse;
mod wallet;

#[cfg(feature = "drand")]
mod drand_beacon;

#[cfg(feature = "mev-shield")]
mod mev_shield;

use pyo3::prelude::*;

use axon::{Axon, AxonConfig};
use chain_client::{SubtensorClient, TxSuccessPy};
use core_types::{
    AxonInfo, Balance, BittensorError, DelegateInfo, MetagraphInfo, NetworkConfig,
    NeuronCertificate, NeuronInfo, NeuronInfoLite, PrometheusInfo, StakeInfo,
    SubnetHyperparameters, SubnetInfo,
};
use dendrite::{Dendrite, DendriteConfig};
use metagraph::Metagraph;
use synapse::{StreamingSynapse, Synapse, TerminalInfo};
use wallet::Wallet;

#[cfg(feature = "drand")]
use drand_beacon::DrandBeacon;

#[cfg(feature = "mev-shield")]
use mev_shield::MevShield;

/// The bittensor_rs Python module.
#[pymodule]
fn bittensor_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Core types
    m.add_class::<Balance>()?;
    m.add_class::<NetworkConfig>()?;
    m.add("BittensorError", m.py().get_type::<BittensorError>())?;
    m.add_class::<AxonInfo>()?;
    m.add_class::<PrometheusInfo>()?;
    m.add_class::<StakeInfo>()?;
    m.add_class::<DelegateInfo>()?;
    m.add_class::<NeuronInfo>()?;
    m.add_class::<NeuronInfoLite>()?;
    m.add_class::<SubnetInfo>()?;
    m.add_class::<SubnetHyperparameters>()?;
    m.add_class::<MetagraphInfo>()?;
    m.add_class::<NeuronCertificate>()?;

    // Wallet
    m.add_class::<Wallet>()?;

    // Chain client
    m.add_class::<SubtensorClient>()?;
    m.add_class::<TxSuccessPy>()?;

    // Synapse types
    m.add_class::<TerminalInfo>()?;
    m.add_class::<Synapse>()?;
    m.add_class::<StreamingSynapse>()?;

    // Axon
    m.add_class::<AxonConfig>()?;
    m.add_class::<Axon>()?;

    // Dendrite
    m.add_class::<DendriteConfig>()?;
    m.add_class::<Dendrite>()?;

    // Metagraph
    m.add_class::<Metagraph>()?;

    // Feature-gated classes
    #[cfg(feature = "drand")]
    m.add_class::<DrandBeacon>()?;

    #[cfg(feature = "mev-shield")]
    m.add_class::<MevShield>()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn module_types_import_correctly() {
        use crate::axon::{Axon, AxonConfig};
        use crate::chain_client::{SubtensorClient, TxSuccessPy};
        use crate::core_types::{
            AxonInfo, Balance, BittensorError, DelegateInfo, MetagraphInfo, NetworkConfig,
            NeuronCertificate, NeuronInfo, NeuronInfoLite, PrometheusInfo, StakeInfo,
            SubnetHyperparameters, SubnetInfo,
        };
        use crate::dendrite::{Dendrite, DendriteConfig};
        use crate::metagraph::Metagraph;
        use crate::synapse::{StreamingSynapse, Synapse, TerminalInfo};
        use crate::wallet::Wallet;

        let _ = std::mem::size_of::<Balance>();
        let _ = std::mem::size_of::<NetworkConfig>();
        let _ = std::mem::size_of::<AxonInfo>();
        let _ = std::mem::size_of::<Wallet>();
        let _ = std::mem::size_of::<SubtensorClient>();
        let _ = std::mem::size_of::<TxSuccessPy>();
        let _ = std::mem::size_of::<TerminalInfo>();
        let _ = std::mem::size_of::<Synapse>();
        let _ = std::mem::size_of::<StreamingSynapse>();
        let _ = std::mem::size_of::<AxonConfig>();
        let _ = std::mem::size_of::<Axon>();
        let _ = std::mem::size_of::<DendriteConfig>();
        let _ = std::mem::size_of::<Dendrite>();
        let _ = std::mem::size_of::<Metagraph>();
        let _ = std::mem::size_of::<StakeInfo>();
        let _ = std::mem::size_of::<DelegateInfo>();
        let _ = std::mem::size_of::<NeuronInfo>();
        let _ = std::mem::size_of::<NeuronInfoLite>();
        let _ = std::mem::size_of::<SubnetInfo>();
        let _ = std::mem::size_of::<SubnetHyperparameters>();
        let _ = std::mem::size_of::<MetagraphInfo>();
        let _ = std::mem::size_of::<NeuronCertificate>();
        let _ = std::mem::size_of::<PrometheusInfo>();
        let _ = std::mem::size_of::<BittensorError>();
    }
}
