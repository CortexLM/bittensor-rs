//! Chain query functions — read-only storage queries against the Subtensor runtime.
//!
//! Each sub-module groups queries by pallet or domain:
//! - [`metagraph`] — subnet-wide metagraph snapshots
//! - [`neuron`] — individual neuron and UID lookups
//! - [`account`] — balance and stake queries
//! - [`subnet`] — subnet metadata and hyperparameters
//! - [`delegate`] — delegate and delegation info
//! - [`network`] — block, hash rate, issuance
//! - [`weights`] — weight matrix queries
//! - [`commit`] — commit-reveal weight hashes
//! - [`children`] — childkey take and hierarchy
//! - [`proxy`] — proxy account lookups
//! - [`identity`] — on-chain identity

pub mod account;
pub mod children;
pub mod commit;
pub mod delegate;
pub mod identity;
pub mod metagraph;
pub mod network;
pub mod neuron;
pub mod proxy;
pub mod subnet;
pub mod weights;

// Re-export the most commonly used query functions for convenience.
// Other functions are accessible via their module path, e.g. queries::subnet::get_tempo
pub use account::{
    get_balance, get_owned_hotkeys, get_stake, get_stake_info_for_coldkey, get_total_balance,
    get_total_issuance, get_total_network_stake,
};
pub use metagraph::get_metagraph;
pub use network::get_network_block;
pub use neuron::{get_neuron, get_neuron_count};
