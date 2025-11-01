pub mod axon;
pub mod neuron;
pub mod neuron_lite;
pub mod prometheus;
pub mod subnet;
pub mod delegate;
pub mod commitment;
pub mod liquidity;
pub mod proposal_vote;

pub use axon::AxonInfo;
pub use neuron::NeuronInfo;
pub use neuron_lite::NeuronInfoLite;
pub use prometheus::PrometheusInfo;
pub use subnet::{SubnetInfo, SubnetHyperparameters, SubnetIdentity};
pub use delegate::{DelegateInfo, DelegatedInfo};
pub use commitment::WeightCommitInfo;
pub use liquidity::LiquidityPosition;
pub use proposal_vote::ProposalVoteData;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChainIdentity {
    pub fields: std::collections::HashMap<String, String>,
}
