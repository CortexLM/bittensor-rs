//! DRAND randomness beacon integration.
//!
//! Fetches round info and randomness from the DRAND HTTP API,
//! verifies BLS12-381 signatures, and caches recent rounds.
//!
//! Feature-gated: `#[cfg(feature = "drand")]`

pub mod beacon;
pub mod timelock;

pub use beacon::{DrandBeacon, DrandBeaconError, DrandRound, MAINNET_CHAIN_HASH};
pub use timelock::{TimelockCommit, TimelockError, TimelockReveal};
