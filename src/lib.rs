#![allow(dead_code)]
//! # Bittensor Rust SDK v2
//!
//! A Rust SDK for interacting with the Bittensor network, designed to match
//! the Python SDK's interface and functionality.
//!
//! ## Features
//!
//! - **Subtensor Client**: Connect to the Bittensor blockchain and query state
//! - **Metagraph**: Access subnet state including neurons, stakes, and rankings
//! - **Chain Data Types**: NeuronInfo, AxonInfo, SubnetInfo, etc.
//! - **Query Functions**: Query neurons, subnets, stakes, and more
//!
//! ## Quick Start
//!
//! ```ignore
//! use bittensor_rs::{Subtensor, Metagraph};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Connect to finney (mainnet)
//!     let subtensor = Subtensor::new("finney").await?;
//!
//!     // Get metagraph for subnet 1
//!     let metagraph = subtensor.metagraph(1).await?;
//!
//!     println!("Subnet 1 has {} neurons", metagraph.n);
//!     println!("Total stake: {}", metagraph.total_stake());
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Modules
//!
//! - [`config`]: Network configuration and settings
//! - [`types`]: Chain data types (NeuronInfo, AxonInfo, etc.)
//! - [`metagraph`]: Metagraph struct and sync functionality
//! - [`subtensor`]: Main Subtensor client interface
//! - [`queries`]: Low-level chain query functions
//! - [`utils`]: Utility functions (ss58, balance, networking)
//! - [`error`]: Error types

pub mod config;
pub mod error;
pub mod metagraph;
pub mod queries;
pub mod subtensor;
pub mod types;
pub mod utils;

// Re-export main types at crate root
pub use config::{Config, Network, DEFAULTS, NETWORKS};
pub use error::{Error, Result};
pub use metagraph::Metagraph;
pub use subtensor::{AsyncSubtensor, Subtensor};

// Re-export commonly used types
pub use types::{
    AxonInfo, DelegateInfo, NeuronInfo, NeuronInfoLite, PrometheusInfo, SubnetHyperparameters,
    SubnetInfo,
};

// Re-export utilities
pub use utils::balance::Balance;
pub use utils::ss58::{is_valid_ss58_address, ss58_decode, ss58_encode};

// Re-export RPC methods for advanced usage
pub use subxt::backend::legacy::LegacyRpcMethods;
pub use subxt::backend::rpc::RpcClient;
