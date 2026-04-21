//! bittensor-core — shared types, errors, config, balance arithmetic, weight utils, and POW for the bittensor-rs SDK

#![deny(missing_docs)]

/// Balance type and arithmetic (1 TAO = 10^9 rao).
pub mod balance;
/// Network and chain configuration types.
pub mod config;
/// Error types and retry classification.
pub mod error;
/// Proof-of-work registration solver.
pub mod pow;
/// Convenience re-exports for common types.
pub mod prelude;
/// Shared chain data types (AxonInfo, NeuronInfo, SubnetHyperparameters, etc.).
pub mod types;
/// Weight normalization and validation utilities.
pub mod weight_utils;
