//! Event subscriptions and chain monitoring for the Bittensor Subtensor chain.
//!
//! This module provides:
//! - [`ChainEvent`] — a unified enum of decoded chain events
//! - [`ChainEventHandler`] — a trait with default no-op implementations for each event category
//! - [`ChainMonitor`] — a background task that subscribes to blocks and emits events via broadcast
//! - [`subscribe_events`] / [`subscribe_blocks`] — raw subscription functions
//! - [`subscribe_storage`] — raw storage subscription (feature-gated: `storage-subscriptions`)
//! - [`EventFilter`] — typed filter methods for specific events

pub mod filters;
pub mod monitor;
pub mod subscriptions;

pub use filters::{
    EventFilter, filter_delegate_added, filter_neuron_registered, filter_pallet,
    filter_stake_added, filter_stake_moved, filter_stake_removed, filter_transfer,
    filter_weights_set,
};
pub use monitor::{ChainMonitor, MONITOR_CHANNEL_CAPACITY, MonitorError};
#[cfg(feature = "storage-subscriptions")]
pub use subscriptions::subscribe_storage;
pub use subscriptions::{BlockStream, subscribe_blocks, subscribe_events};

use bittensor_core::config::SubtensorConfig;
use subxt::events::Event;

/// A decoded chain event with metadata about its origin block.
///
/// This enum wraps all events that the Bittensor chain produces,
/// carrying the block number and hash alongside the decoded data.
#[derive(Debug, Clone)]
pub enum ChainEvent {
    // ── SubtensorModule events ─────────────────────────────
    /// A neuron was registered on a subnet.
    NeuronRegistered {
        netuid: u16,
        hotkey: String,
        coldkey: String,
        block_number: u64,
        block_hash: subxt::utils::H256,
    },
    /// Weights were set for a subnet.
    WeightsSet { netuid: u16, hotkey: String, block_number: u64, block_hash: subxt::utils::H256 },
    /// Stake was added to a hotkey.
    StakeAdded {
        hotkey: String,
        coldkey: String,
        amount: u64,
        block_number: u64,
        block_hash: subxt::utils::H256,
    },
    /// Stake was removed from a hotkey.
    StakeRemoved {
        hotkey: String,
        coldkey: String,
        amount: u64,
        block_number: u64,
        block_hash: subxt::utils::H256,
    },
    /// A delegate was added.
    DelegateAdded {
        hotkey: String,
        coldkey: String,
        block_number: u64,
        block_hash: subxt::utils::H256,
    },
    /// Stake was moved between accounts.
    StakeMoved {
        hotkey: String,
        coldkey: String,
        block_number: u64,
        block_hash: subxt::utils::H256,
    },

    // ── Balances events ────────────────────────────────────
    /// A balance transfer occurred.
    Transfer {
        from: String,
        to: String,
        amount: u64,
        block_number: u64,
        block_hash: subxt::utils::H256,
    },

    // ── System events ──────────────────────────────────────
    /// An extrinsic executed successfully.
    ExtrinsicSuccess { block_number: u64, block_hash: subxt::utils::H256 },
    /// An extrinsic failed.
    ExtrinsicFailed { block_number: u64, block_hash: subxt::utils::H256 },

    // ── Catch-all ─────────────────────────────────────────
    /// An event that could not be decoded into a known variant.
    Unknown {
        pallet: String,
        name: String,
        bytes: Vec<u8>,
        block_number: u64,
        block_hash: subxt::utils::H256,
    },
}

impl ChainEvent {
    /// Returns the block number where this event was emitted.
    pub fn block_number(&self) -> u64 {
        match self {
            Self::NeuronRegistered { block_number, .. }
            | Self::WeightsSet { block_number, .. }
            | Self::StakeAdded { block_number, .. }
            | Self::StakeRemoved { block_number, .. }
            | Self::DelegateAdded { block_number, .. }
            | Self::StakeMoved { block_number, .. }
            | Self::Transfer { block_number, .. }
            | Self::ExtrinsicSuccess { block_number, .. }
            | Self::ExtrinsicFailed { block_number, .. }
            | Self::Unknown { block_number, .. } => *block_number,
        }
    }

    /// Returns the block hash where this event was emitted.
    pub fn block_hash(&self) -> subxt::utils::H256 {
        match self {
            Self::NeuronRegistered { block_hash, .. }
            | Self::WeightsSet { block_hash, .. }
            | Self::StakeAdded { block_hash, .. }
            | Self::StakeRemoved { block_hash, .. }
            | Self::DelegateAdded { block_hash, .. }
            | Self::StakeMoved { block_hash, .. }
            | Self::Transfer { block_hash, .. }
            | Self::ExtrinsicSuccess { block_hash, .. }
            | Self::ExtrinsicFailed { block_hash, .. }
            | Self::Unknown { block_hash, .. } => *block_hash,
        }
    }

    /// Returns the pallet name that emitted this event.
    pub fn pallet_name(&self) -> &str {
        match self {
            Self::NeuronRegistered { .. } => "SubtensorModule",
            Self::WeightsSet { .. } => "SubtensorModule",
            Self::StakeAdded { .. } => "SubtensorModule",
            Self::StakeRemoved { .. } => "SubtensorModule",
            Self::DelegateAdded { .. } => "SubtensorModule",
            Self::StakeMoved { .. } => "SubtensorModule",
            Self::Transfer { .. } => "Balances",
            Self::ExtrinsicSuccess { .. } => "System",
            Self::ExtrinsicFailed { .. } => "System",
            Self::Unknown { pallet, .. } => pallet,
        }
    }

    /// Returns the specific event name.
    pub fn event_name(&self) -> &str {
        match self {
            Self::NeuronRegistered { .. } => "NeuronRegistered",
            Self::WeightsSet { .. } => "WeightsSet",
            Self::StakeAdded { .. } => "StakeAdded",
            Self::StakeRemoved { .. } => "StakeRemoved",
            Self::DelegateAdded { .. } => "DelegateAdded",
            Self::StakeMoved { .. } => "StakeMoved",
            Self::Transfer { .. } => "Transfer",
            Self::ExtrinsicSuccess { .. } => "ExtrinsicSuccess",
            Self::ExtrinsicFailed { .. } => "ExtrinsicFailed",
            Self::Unknown { name, .. } => name,
        }
    }
}

/// A trait for handling decoded chain events.
///
/// Each method has a default no-op implementation so that implementors
/// can override only the events they care about.
pub trait ChainEventHandler: Send + Sync {
    fn on_neuron_registered(&self, _netuid: u16, _hotkey: &str, _coldkey: &str) {}
    fn on_weights_set(&self, _netuid: u16, _hotkey: &str) {}
    fn on_stake_added(&self, _hotkey: &str, _coldkey: &str, _amount: u64) {}
    fn on_stake_removed(&self, _hotkey: &str, _coldkey: &str, _amount: u64) {}
    fn on_delegate_added(&self, _hotkey: &str, _coldkey: &str) {}
    fn on_stake_moved(&self, _hotkey: &str, _coldkey: &str) {}
    fn on_transfer(&self, _from: &str, _to: &str, _amount: u64) {}
    fn on_extrinsic_success(&self) {}
    fn on_extrinsic_failed(&self) {}
    fn on_unknown_event(&self, _pallet: &str, _name: &str, _bytes: &[u8]) {}
}

/// Dispatches a [`ChainEvent`] to the appropriate handler method.
pub fn dispatch_event(handler: &dyn ChainEventHandler, event: &ChainEvent) {
    match event {
        ChainEvent::NeuronRegistered { netuid, hotkey, coldkey, .. } => {
            handler.on_neuron_registered(*netuid, hotkey, coldkey);
        }
        ChainEvent::WeightsSet { netuid, hotkey, .. } => {
            handler.on_weights_set(*netuid, hotkey);
        }
        ChainEvent::StakeAdded { hotkey, coldkey, amount, .. } => {
            handler.on_stake_added(hotkey, coldkey, *amount);
        }
        ChainEvent::StakeRemoved { hotkey, coldkey, amount, .. } => {
            handler.on_stake_removed(hotkey, coldkey, *amount);
        }
        ChainEvent::DelegateAdded { hotkey, coldkey, .. } => {
            handler.on_delegate_added(hotkey, coldkey);
        }
        ChainEvent::StakeMoved { hotkey, coldkey, .. } => {
            handler.on_stake_moved(hotkey, coldkey);
        }
        ChainEvent::Transfer { from, to, amount, .. } => {
            handler.on_transfer(from, to, *amount);
        }
        ChainEvent::ExtrinsicSuccess { .. } => {
            handler.on_extrinsic_success();
        }
        ChainEvent::ExtrinsicFailed { .. } => {
            handler.on_extrinsic_failed();
        }
        ChainEvent::Unknown { pallet, name, bytes, .. } => {
            handler.on_unknown_event(pallet, name, bytes);
        }
    }
}

/// Try to decode a raw subxt [`Event`] into a [`ChainEvent`].
///
/// Returns a [`ChainEvent`] for known events, or [`ChainEvent::Unknown`]
/// for events that cannot be decoded.
pub fn decode_event(
    raw: &Event<'_, SubtensorConfig>,
    block_number: u64,
    block_hash: subxt::utils::H256,
) -> ChainEvent {
    let pallet = raw.pallet_name();
    let name = raw.event_name();

    match (pallet, name) {
        // ── SubtensorModule ──────────────────────
        ("SubtensorModule", "NeuronRegistered") => {
            let fields = decode_subtensor_neuron_registered(raw);
            ChainEvent::NeuronRegistered {
                netuid: fields.0,
                hotkey: fields.1,
                coldkey: fields.2,
                block_number,
                block_hash,
            }
        }
        ("SubtensorModule", "WeightsSet") => {
            let (netuid, hotkey) = decode_subtensor_weights_set(raw);
            ChainEvent::WeightsSet { netuid, hotkey, block_number, block_hash }
        }
        ("SubtensorModule", "StakeAdded") => {
            let (hotkey, coldkey, amount) = decode_subtensor_stake_added(raw);
            ChainEvent::StakeAdded { hotkey, coldkey, amount, block_number, block_hash }
        }
        ("SubtensorModule", "StakeRemoved") => {
            let (hotkey, coldkey, amount) = decode_subtensor_stake_removed(raw);
            ChainEvent::StakeRemoved { hotkey, coldkey, amount, block_number, block_hash }
        }
        ("SubtensorModule", "DelegateAdded") => {
            let (hotkey, coldkey) = decode_subtensor_delegate_added(raw);
            ChainEvent::DelegateAdded { hotkey, coldkey, block_number, block_hash }
        }
        ("SubtensorModule", "StakeMoved") => {
            let (hotkey, coldkey) = decode_subtensor_stake_moved(raw);
            ChainEvent::StakeMoved { hotkey, coldkey, block_number, block_hash }
        }

        // ── Balances ────────────────────────────
        ("Balances", "Transfer") => {
            let (from, to, amount) = decode_balances_transfer(raw);
            ChainEvent::Transfer { from, to, amount, block_number, block_hash }
        }

        // ── System ──────────────────────────────
        ("System", "ExtrinsicSuccess") => ChainEvent::ExtrinsicSuccess { block_number, block_hash },
        ("System", "ExtrinsicFailed") => ChainEvent::ExtrinsicFailed { block_number, block_hash },

        // ── Unknown ─────────────────────────────
        _ => ChainEvent::Unknown {
            pallet: pallet.to_string(),
            name: name.to_string(),
            bytes: raw.field_bytes().to_vec(),
            block_number,
            block_hash,
        },
    }
}

// ── Decode helpers ────────────────────────────────────────
// These use dynamic decoding via subxt's Event::decode_fields_as
// which returns Option<Result<E, EventsError>>. We flatten to Option<E>,
// then map the fields.

use crate::generated::subtensor;

fn decode_subtensor_neuron_registered(raw: &Event<'_, SubtensorConfig>) -> (u16, String, String) {
    raw.decode_fields_as::<subtensor::subtensor_module::events::NeuronRegistered>()
        .and_then(|r| r.ok())
        .map(|e| (e.0, format!("{:?}", e.1), format!("{:?}", e.2)))
        .unwrap_or((0, String::new(), String::new()))
}

fn decode_subtensor_weights_set(raw: &Event<'_, SubtensorConfig>) -> (u16, String) {
    raw.decode_fields_as::<subtensor::subtensor_module::events::WeightsSet>()
        .and_then(|r| r.ok())
        .map(|e| (e.0, format!("{:?}", e.1)))
        .unwrap_or((0, String::new()))
}

fn decode_subtensor_stake_added(raw: &Event<'_, SubtensorConfig>) -> (String, String, u64) {
    raw.decode_fields_as::<subtensor::subtensor_module::events::StakeAdded>()
        .and_then(|r| r.ok())
        .map(|e| (format!("{:?}", e.0), format!("{:?}", e.1), e.2))
        .unwrap_or((String::new(), String::new(), 0))
}

fn decode_subtensor_stake_removed(raw: &Event<'_, SubtensorConfig>) -> (String, String, u64) {
    raw.decode_fields_as::<subtensor::subtensor_module::events::StakeRemoved>()
        .and_then(|r| r.ok())
        .map(|e| (format!("{:?}", e.0), format!("{:?}", e.1), e.2))
        .unwrap_or((String::new(), String::new(), 0))
}

fn decode_subtensor_delegate_added(raw: &Event<'_, SubtensorConfig>) -> (String, String) {
    raw.decode_fields_as::<subtensor::subtensor_module::events::DelegateAdded>()
        .and_then(|r| r.ok())
        .map(|e| (format!("{:?}", e.0), format!("{:?}", e.1)))
        .unwrap_or((String::new(), String::new()))
}

fn decode_subtensor_stake_moved(raw: &Event<'_, SubtensorConfig>) -> (String, String) {
    raw.decode_fields_as::<subtensor::subtensor_module::events::StakeMoved>()
        .and_then(|r| r.ok())
        .map(|e| (format!("{:?}", e.0), format!("{:?}", e.1)))
        .unwrap_or((String::new(), String::new()))
}

fn decode_balances_transfer(raw: &Event<'_, SubtensorConfig>) -> (String, String, u64) {
    raw.decode_fields_as::<subtensor::balances::events::Transfer>()
        .and_then(|r| r.ok())
        .map(|e| (format!("{:?}", e.from), format!("{:?}", e.to), e.amount))
        .unwrap_or((String::new(), String::new(), 0))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── ChainEvent tests ──────────────────────

    #[test]
    fn chain_event_neuron_registered_accessors() {
        let event = ChainEvent::NeuronRegistered {
            netuid: 1,
            hotkey: "5Test".into(),
            coldkey: "5Cold".into(),
            block_number: 100,
            block_hash: subxt::utils::H256::zero(),
        };
        assert_eq!(event.block_number(), 100);
        assert_eq!(event.block_hash(), subxt::utils::H256::zero());
        assert_eq!(event.pallet_name(), "SubtensorModule");
        assert_eq!(event.event_name(), "NeuronRegistered");
    }

    #[test]
    fn chain_event_weights_set_accessors() {
        let event = ChainEvent::WeightsSet {
            netuid: 3,
            hotkey: "5W".into(),
            block_number: 200,
            block_hash: subxt::utils::H256::repeat_byte(0xab),
        };
        assert_eq!(event.pallet_name(), "SubtensorModule");
        assert_eq!(event.event_name(), "WeightsSet");
    }

    #[test]
    fn chain_event_stake_added_accessors() {
        let event = ChainEvent::StakeAdded {
            hotkey: "hk".into(),
            coldkey: "ck".into(),
            amount: 5000,
            block_number: 300,
            block_hash: subxt::utils::H256::zero(),
        };
        assert_eq!(event.block_number(), 300);
        assert_eq!(event.event_name(), "StakeAdded");
    }

    #[test]
    fn chain_event_stake_removed_accessors() {
        let event = ChainEvent::StakeRemoved {
            hotkey: "hk".into(),
            coldkey: "ck".into(),
            amount: 1000,
            block_number: 400,
            block_hash: subxt::utils::H256::zero(),
        };
        assert_eq!(event.event_name(), "StakeRemoved");
    }

    #[test]
    fn chain_event_delegate_added_accessors() {
        let event = ChainEvent::DelegateAdded {
            hotkey: "hk".into(),
            coldkey: "ck".into(),
            block_number: 500,
            block_hash: subxt::utils::H256::zero(),
        };
        assert_eq!(event.pallet_name(), "SubtensorModule");
        assert_eq!(event.event_name(), "DelegateAdded");
    }

    #[test]
    fn chain_event_stake_moved_accessors() {
        let event = ChainEvent::StakeMoved {
            hotkey: "hk".into(),
            coldkey: "ck".into(),
            block_number: 600,
            block_hash: subxt::utils::H256::zero(),
        };
        assert_eq!(event.pallet_name(), "SubtensorModule");
        assert_eq!(event.event_name(), "StakeMoved");
    }

    #[test]
    fn chain_event_transfer_accessors() {
        let event = ChainEvent::Transfer {
            from: "5A".into(),
            to: "5B".into(),
            amount: 999,
            block_number: 700,
            block_hash: subxt::utils::H256::zero(),
        };
        assert_eq!(event.pallet_name(), "Balances");
        assert_eq!(event.event_name(), "Transfer");
    }

    #[test]
    fn chain_event_system_accessors() {
        let success = ChainEvent::ExtrinsicSuccess {
            block_number: 800,
            block_hash: subxt::utils::H256::zero(),
        };
        assert_eq!(success.pallet_name(), "System");
        assert_eq!(success.event_name(), "ExtrinsicSuccess");

        let failed = ChainEvent::ExtrinsicFailed {
            block_number: 801,
            block_hash: subxt::utils::H256::zero(),
        };
        assert_eq!(failed.event_name(), "ExtrinsicFailed");
    }

    #[test]
    fn chain_event_unknown_accessors() {
        let event = ChainEvent::Unknown {
            pallet: "CustomPallet".into(),
            name: "SomethingHappened".into(),
            bytes: vec![1, 2, 3],
            block_number: 900,
            block_hash: subxt::utils::H256::zero(),
        };
        assert_eq!(event.pallet_name(), "CustomPallet");
        assert_eq!(event.event_name(), "SomethingHappened");
        assert_eq!(event.block_number(), 900);
    }

    // ── ChainEventHandler tests ─────────────────

    struct TestHandler {
        stake_adds: std::sync::Mutex<Vec<(String, String, u64)>>,
        transfers: std::sync::Mutex<Vec<(String, String, u64)>>,
        neuron_regs: std::sync::Mutex<Vec<(u16, String, String)>>,
    }

    impl TestHandler {
        fn new() -> Self {
            Self {
                stake_adds: std::sync::Mutex::new(Vec::new()),
                transfers: std::sync::Mutex::new(Vec::new()),
                neuron_regs: std::sync::Mutex::new(Vec::new()),
            }
        }
    }

    impl ChainEventHandler for TestHandler {
        fn on_stake_added(&self, hotkey: &str, coldkey: &str, amount: u64) {
            self.stake_adds.lock().unwrap().push((hotkey.into(), coldkey.into(), amount));
        }
        fn on_transfer(&self, from: &str, to: &str, amount: u64) {
            self.transfers.lock().unwrap().push((from.into(), to.into(), amount));
        }
        fn on_neuron_registered(&self, netuid: u16, hotkey: &str, coldkey: &str) {
            self.neuron_regs.lock().unwrap().push((netuid, hotkey.into(), coldkey.into()));
        }
    }

    #[test]
    fn handler_dispatch_stake_added() {
        let handler = TestHandler::new();
        let event = ChainEvent::StakeAdded {
            hotkey: "hk".into(),
            coldkey: "ck".into(),
            amount: 100,
            block_number: 1,
            block_hash: subxt::utils::H256::zero(),
        };
        dispatch_event(&handler, &event);
        let adds = handler.stake_adds.lock().unwrap();
        assert_eq!(adds.len(), 1);
        assert_eq!(adds[0], ("hk".into(), "ck".into(), 100));
    }

    #[test]
    fn handler_dispatch_transfer() {
        let handler = TestHandler::new();
        let event = ChainEvent::Transfer {
            from: "5A".into(),
            to: "5B".into(),
            amount: 500,
            block_number: 2,
            block_hash: subxt::utils::H256::zero(),
        };
        dispatch_event(&handler, &event);
        let xfers = handler.transfers.lock().unwrap();
        assert_eq!(xfers.len(), 1);
        assert_eq!(xfers[0], ("5A".into(), "5B".into(), 500));
    }

    #[test]
    fn handler_dispatch_neuron_registered() {
        let handler = TestHandler::new();
        let event = ChainEvent::NeuronRegistered {
            netuid: 1,
            hotkey: "5Test".into(),
            coldkey: "5Cold".into(),
            block_number: 3,
            block_hash: subxt::utils::H256::zero(),
        };
        dispatch_event(&handler, &event);
        let regs = handler.neuron_regs.lock().unwrap();
        assert_eq!(regs.len(), 1);
        assert_eq!(regs[0], (1, "5Test".into(), "5Cold".into()));
    }

    #[test]
    fn handler_dispatch_unknown_is_noop() {
        struct NoopHandler;
        impl ChainEventHandler for NoopHandler {}
        let handler = NoopHandler;
        let event = ChainEvent::Unknown {
            pallet: "Foo".into(),
            name: "Bar".into(),
            bytes: vec![],
            block_number: 0,
            block_hash: subxt::utils::H256::zero(),
        };
        // Should not panic — just calls the default no-op
        dispatch_event(&handler, &event);
    }

    #[test]
    fn default_handler_is_noop() {
        struct NopHandler;
        impl ChainEventHandler for NopHandler {}
        let handler = NopHandler;
        // All no-op methods should be callable without panic
        handler.on_neuron_registered(1, "hk", "ck");
        handler.on_weights_set(1, "hk");
        handler.on_stake_added("hk", "ck", 0);
        handler.on_stake_removed("hk", "ck", 0);
        handler.on_delegate_added("hk", "ck");
        handler.on_stake_moved("hk", "ck");
        handler.on_transfer("a", "b", 0);
        handler.on_extrinsic_success();
        handler.on_extrinsic_failed();
        handler.on_unknown_event("p", "n", &[]);
    }
}
