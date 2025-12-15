#![allow(dead_code, unused_variables, unused_imports)]

pub mod blocks;
pub mod chain;
pub mod config;
pub mod core;
pub mod metagraph;
pub mod queries;
pub mod types;
pub mod utils;
pub mod validator;

pub use chain::{BittensorClient, Error as ChainError};
pub use config::{AxonConfig, Config, LoggingConfig, SubtensorConfig};
pub use metagraph::Metagraph;

// Re-export types first (includes liquidity types)
pub use types::*;

// Re-export queries with specific naming to avoid conflicts
pub use queries::{
    balances::*, chain_info::*, delegates::*, identity::*, metagraph_queries::*, stakes::*,
    subnets::*,
};

// Re-export neurons module (use prefix for children/parents to avoid conflict with validator)
pub use queries::neurons::{
    fetch_axon_info, fetch_prometheus_info, get_all_neuron_certificates, get_neuron_certificate,
    get_neuron_for_pubkey_and_subnet, neuron, neurons, query_neuron_from_storage, Certificate,
};

// Children/parents queries accessible via module path
pub use queries::neurons as neuron_queries;

// Re-export liquidity queries with module prefix to avoid conflict
pub use queries::liquidity as liquidity_queries;

// Re-export validator functions (except weights to avoid conflict)
pub use validator::{
    liquidity::*, mechanism::*, registration::*, root::*, serving::*, staking::*, take::*,
    transfer::*, utility::*,
};

// Children module accessible via module path to avoid conflict with query children
pub use validator::children as validator_children;

// Re-export weights module with prefix to avoid conflict
pub use validator::weights as validator_weights;

// Re-export utils with specific modules to avoid conflicts
pub use utils::{balance, crypto, encode, scale, ss58};

// Re-export decoders module
pub use utils::decoders;

// Re-export weights utils with prefix to avoid conflict
pub use utils::weights as utils_weights;

// Re-export key crypto functions for commit-reveal
pub use utils::crypto::{
    commit_hash_to_hex, generate_mechanism_commit_hash, generate_salt,
    generate_subtensor_commit_hash, hex_to_commit_hash_32, salt_u8_to_u16, verify_commit_hash,
};
