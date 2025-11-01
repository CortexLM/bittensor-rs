pub mod axon;
pub mod commitment;
pub mod delegate;
pub mod liquidity;
pub mod neuron;
pub mod neuron_lite;
pub mod prometheus;
pub mod proposal_vote;
pub mod subnet;

pub use axon::AxonInfo;
pub use commitment::WeightCommitInfo;
pub use delegate::{DelegateInfo, DelegatedInfo};
pub use liquidity::LiquidityPosition;
pub use neuron::NeuronInfo;
pub use neuron_lite::NeuronInfoLite;
pub use prometheus::PrometheusInfo;
pub use proposal_vote::ProposalVoteData;
pub use subnet::{SubnetHyperparameters, SubnetIdentity, SubnetInfo};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChainIdentity {
    pub fields: std::collections::HashMap<String, String>,
}
