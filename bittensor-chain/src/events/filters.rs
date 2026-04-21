//! Event filtering — typed predicates and slice-based filter functions for [`ChainEvent`].

use super::ChainEvent;

/// Predicate methods on [`ChainEvent`] for matching specific event categories.
pub trait EventFilter {
    /// Returns `true` for [`ChainEvent::NeuronRegistered`].
    fn is_neuron_registered(&self) -> bool;
    /// Returns `true` for [`ChainEvent::WeightsSet`].
    fn is_weights_set(&self) -> bool;
    /// Returns `true` for [`ChainEvent::StakeAdded`].
    fn is_stake_added(&self) -> bool;
    /// Returns `true` for [`ChainEvent::StakeRemoved`].
    fn is_stake_removed(&self) -> bool;
    /// Returns `true` for [`ChainEvent::DelegateAdded`].
    fn is_delegate_added(&self) -> bool;
    /// Returns `true` for [`ChainEvent::StakeMoved`].
    fn is_stake_moved(&self) -> bool;
    /// Returns `true` for [`ChainEvent::Transfer`].
    fn is_transfer(&self) -> bool;
    /// Returns `true` for [`ChainEvent::ExtrinsicSuccess`].
    fn is_extrinsic_success(&self) -> bool;
    /// Returns `true` for [`ChainEvent::ExtrinsicFailed`].
    fn is_extrinsic_failed(&self) -> bool;
    /// Returns `true` if the event originates from the given pallet name (case-insensitive).
    fn is_pallet(&self, pallet: &str) -> bool;
}

impl EventFilter for ChainEvent {
    fn is_neuron_registered(&self) -> bool {
        matches!(self, ChainEvent::NeuronRegistered { .. })
    }

    fn is_weights_set(&self) -> bool {
        matches!(self, ChainEvent::WeightsSet { .. })
    }

    fn is_stake_added(&self) -> bool {
        matches!(self, ChainEvent::StakeAdded { .. })
    }

    fn is_stake_removed(&self) -> bool {
        matches!(self, ChainEvent::StakeRemoved { .. })
    }

    fn is_delegate_added(&self) -> bool {
        matches!(self, ChainEvent::DelegateAdded { .. })
    }

    fn is_stake_moved(&self) -> bool {
        matches!(self, ChainEvent::StakeMoved { .. })
    }

    fn is_transfer(&self) -> bool {
        matches!(self, ChainEvent::Transfer { .. })
    }

    fn is_extrinsic_success(&self) -> bool {
        matches!(self, ChainEvent::ExtrinsicSuccess { .. })
    }

    fn is_extrinsic_failed(&self) -> bool {
        matches!(self, ChainEvent::ExtrinsicFailed { .. })
    }

    fn is_pallet(&self, pallet: &str) -> bool {
        self.pallet_name().eq_ignore_ascii_case(pallet)
    }
}

/// Filter a slice of events to only [`ChainEvent::NeuronRegistered`].
pub fn filter_neuron_registered(events: &[ChainEvent]) -> Vec<&ChainEvent> {
    events.iter().filter(|e| e.is_neuron_registered()).collect()
}

/// Filter a slice of events to only [`ChainEvent::WeightsSet`].
pub fn filter_weights_set(events: &[ChainEvent]) -> Vec<&ChainEvent> {
    events.iter().filter(|e| e.is_weights_set()).collect()
}

/// Filter a slice of events to only [`ChainEvent::StakeAdded`].
pub fn filter_stake_added(events: &[ChainEvent]) -> Vec<&ChainEvent> {
    events.iter().filter(|e| e.is_stake_added()).collect()
}

/// Filter a slice of events to only [`ChainEvent::StakeRemoved`].
pub fn filter_stake_removed(events: &[ChainEvent]) -> Vec<&ChainEvent> {
    events.iter().filter(|e| e.is_stake_removed()).collect()
}

/// Filter a slice of events to only [`ChainEvent::DelegateAdded`].
pub fn filter_delegate_added(events: &[ChainEvent]) -> Vec<&ChainEvent> {
    events.iter().filter(|e| e.is_delegate_added()).collect()
}

/// Filter a slice of events to only [`ChainEvent::StakeMoved`].
pub fn filter_stake_moved(events: &[ChainEvent]) -> Vec<&ChainEvent> {
    events.iter().filter(|e| e.is_stake_moved()).collect()
}

/// Filter a slice of events to only [`ChainEvent::Transfer`].
pub fn filter_transfer(events: &[ChainEvent]) -> Vec<&ChainEvent> {
    events.iter().filter(|e| e.is_transfer()).collect()
}

/// Filter a slice of events to only those from the named pallet (case-insensitive).
pub fn filter_pallet<'a>(events: &'a [ChainEvent], pallet: &str) -> Vec<&'a ChainEvent> {
    events.iter().filter(|e| e.is_pallet(pallet)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn h256() -> subxt::utils::H256 {
        subxt::utils::H256::zero()
    }

    fn sample_events() -> Vec<ChainEvent> {
        vec![
            ChainEvent::NeuronRegistered {
                netuid: 1,
                hotkey: "hk1".into(),
                coldkey: "ck1".into(),
                block_number: 100,
                block_hash: h256(),
            },
            ChainEvent::WeightsSet {
                netuid: 1,
                hotkey: "hk2".into(),
                block_number: 101,
                block_hash: h256(),
            },
            ChainEvent::StakeAdded {
                hotkey: "hk3".into(),
                coldkey: "ck3".into(),
                amount: 500,
                block_number: 102,
                block_hash: h256(),
            },
            ChainEvent::Transfer {
                from: "5A".into(),
                to: "5B".into(),
                amount: 999,
                block_number: 103,
                block_hash: h256(),
            },
            ChainEvent::ExtrinsicSuccess { block_number: 104, block_hash: h256() },
            ChainEvent::Unknown {
                pallet: "OtherPallet".into(),
                name: "SomeEvent".into(),
                bytes: vec![],
                block_number: 105,
                block_hash: h256(),
            },
        ]
    }

    #[test]
    fn filter_neuron_registered_works() {
        let events = sample_events();
        let result = filter_neuron_registered(&events);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].event_name(), "NeuronRegistered");
    }

    #[test]
    fn filter_weights_set_works() {
        let events = sample_events();
        let result = filter_weights_set(&events);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].event_name(), "WeightsSet");
    }

    #[test]
    fn filter_stake_added_works() {
        let events = sample_events();
        let result = filter_stake_added(&events);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn filter_transfer_works() {
        let events = sample_events();
        let result = filter_transfer(&events);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].event_name(), "Transfer");
    }

    #[test]
    fn filter_stake_removed_returns_empty() {
        let events = sample_events();
        assert!(filter_stake_removed(&events).is_empty());
    }

    #[test]
    fn filter_delegate_added_returns_empty() {
        let events = sample_events();
        assert!(filter_delegate_added(&events).is_empty());
    }

    #[test]
    fn filter_pallet_subtensor() {
        let events = sample_events();
        let result = filter_pallet(&events, "SubtensorModule");
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn filter_pallet_balances() {
        let events = sample_events();
        let result = filter_pallet(&events, "Balances");
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn filter_pallet_case_insensitive() {
        let events = sample_events();
        let result = filter_pallet(&events, "subtensormodule");
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn filter_pallet_system() {
        let events = sample_events();
        let result = filter_pallet(&events, "System");
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn event_filter_trait_methods() {
        let nr = ChainEvent::NeuronRegistered {
            netuid: 1,
            hotkey: "hk".into(),
            coldkey: "ck".into(),
            block_number: 1,
            block_hash: h256(),
        };
        assert!(nr.is_neuron_registered());
        assert!(!nr.is_weights_set());
        assert!(!nr.is_transfer());

        let ws = ChainEvent::WeightsSet {
            netuid: 1,
            hotkey: "hk".into(),
            block_number: 2,
            block_hash: h256(),
        };
        assert!(ws.is_weights_set());
        assert!(!ws.is_neuron_registered());

        let t = ChainEvent::Transfer {
            from: "5A".into(),
            to: "5B".into(),
            amount: 100,
            block_number: 3,
            block_hash: h256(),
        };
        assert!(t.is_transfer());
        assert!(t.is_pallet("Balances"));
        assert!(!t.is_pallet("System"));
    }
}
