//! bittensor-chain — Subtensor chain client, extrinsic submission, and query methods.
//!
//! Connects to a Bittensor Subtensor node via WebSocket using subxt 0.50,
//! provides typed queries against the generated metadata, and submits
//! signed extrinsics (transactions) for staking, transfer, weight-setting,
//! registration, and more.

pub mod client;
pub mod events;
pub mod extrinsics;
pub mod generated;
pub mod queries;

#[cfg(feature = "drand")]
pub mod drand;

#[cfg(feature = "mev-shield")]
pub mod mev_shield;

pub mod prelude {
    pub use crate::client::SubtensorClient;
    #[cfg(feature = "drand")]
    pub use crate::drand::{
        DrandBeacon, DrandBeaconError, DrandRound, MAINNET_CHAIN_HASH, TimelockCommit,
        TimelockError, TimelockReveal,
    };
    #[cfg(feature = "storage-subscriptions")]
    pub use crate::events::subscribe_storage;
    pub use crate::events::{
        BlockStream, ChainEvent, ChainEventHandler, ChainMonitor, EventFilter,
        MONITOR_CHANNEL_CAPACITY, MonitorError, decode_event, dispatch_event,
        filter_delegate_added, filter_neuron_registered, filter_pallet, filter_stake_added,
        filter_stake_moved, filter_stake_removed, filter_transfer, filter_weights_set,
        subscribe_blocks, subscribe_events,
    };
    pub use crate::extrinsics::TxSuccess;
    pub use crate::extrinsics::children::{set_childkey_take, set_children};
    pub use crate::extrinsics::coldkey_swap::*;
    pub use crate::extrinsics::proxy::*;
    pub use crate::extrinsics::registration::*;
    pub use crate::extrinsics::root::*;
    pub use crate::extrinsics::serving::*;
    pub use crate::extrinsics::staking::*;
    pub use crate::extrinsics::sudo::*;
    pub use crate::extrinsics::take::*;
    pub use crate::extrinsics::transfer::*;
    pub use crate::extrinsics::weights::{
        commit_timelocked_weights, commit_weights, reveal_weights, set_weights,
    };
    #[cfg(feature = "mev-shield")]
    pub use crate::mev_shield::{
        EncryptedPayload, MevShieldEncrypt, MevShieldEncryptError, MevShieldSubmit,
        MevShieldSubmitError,
    };
    pub use crate::queries::*;
    pub use bittensor_core::balance::Balance;
    pub use bittensor_core::config::{NetworkConfig, SubtensorConfig};
    pub use bittensor_core::error::BittensorError;
    pub use bittensor_core::types::*;
}
