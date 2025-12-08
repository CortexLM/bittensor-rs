pub mod axon;
pub mod commitment;
pub mod delegate;
pub mod dynamic_info;
pub mod liquidity;
pub mod metagraph_info;
pub mod neuron;
pub mod neuron_lite;
pub mod prometheus;
pub mod proposal_vote;
pub mod subnet;
pub mod synapse;

pub use axon::AxonInfo;
pub use commitment::WeightCommitInfo;
pub use delegate::{DelegateInfo, DelegatedInfo};
pub use dynamic_info::{DynamicInfo, SubnetState};
pub use liquidity::LiquidityPosition;
pub use metagraph_info::{
    ChainIdentity as MetagraphChainIdentity, MetagraphInfo, MetagraphInfoEmissions,
    MetagraphInfoParams, MetagraphInfoPool, SelectiveMetagraphIndex, SubnetIdentityInfo,
};
pub use neuron::NeuronInfo;
pub use neuron_lite::NeuronInfoLite;
pub use prometheus::PrometheusInfo;
pub use proposal_vote::ProposalVoteData;
pub use subnet::{SubnetHyperparameters, SubnetIdentity, SubnetInfo};
pub use synapse::{Synapse, SynapseHeaders, TerminalInfo};

/// Chain identity for delegates
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct ChainIdentity {
    pub fields: std::collections::HashMap<String, String>,
}

impl ChainIdentity {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.fields.get(key)
    }

    pub fn name(&self) -> Option<&String> {
        self.fields.get("name")
    }

    pub fn url(&self) -> Option<&String> {
        self.fields.get("url")
    }

    pub fn description(&self) -> Option<&String> {
        self.fields.get("description")
    }
}
