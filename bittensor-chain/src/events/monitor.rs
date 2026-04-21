//! Chain monitor — background task that subscribes to finalized blocks and broadcasts events.
//!
//! Use [`ChainMonitor`] when you need multiple consumers to receive events
//! via `tokio::sync::broadcast`. For a simpler single-consumer API, see
//! [`subscribe_events`](super::subscribe_events).

use bittensor_core::config::SubtensorConfig;
use bittensor_core::error::BittensorError;
use subxt::OnlineClient;
use tokio::sync::broadcast;

use super::{ChainEvent, ChainEventHandler, decode_event, dispatch_event};

/// Default broadcast channel capacity for [`ChainMonitor`].
pub const MONITOR_CHANNEL_CAPACITY: usize = 1024;

/// Errors produced by the chain monitor.
#[derive(Debug, thiserror::Error)]
pub enum MonitorError {
    /// Failed to subscribe to the block stream.
    #[error("subscription failed: {0}")]
    SubscriptionFailed(String),

    /// Failed to fetch events at a specific block.
    #[error("event fetch failed at block #{0}: {1}")]
    EventFetchFailed(u64, String),

    /// Broadcast channel send failed (all receivers dropped).
    #[error("broadcast send failed: {0}")]
    BroadcastFailed(String),
}

impl From<MonitorError> for BittensorError {
    fn from(e: MonitorError) -> Self {
        BittensorError::Rpc(e.to_string())
    }
}

/// Background monitor that subscribes to finalized blocks and broadcasts
/// decoded [`ChainEvent`]s via a `tokio::sync::broadcast` channel.
///
/// Multiple consumers can call [`subscribe`](ChainMonitor::subscribe) to
/// receive events independently.
#[derive(Debug)]
pub struct ChainMonitor {
    client: OnlineClient<SubtensorConfig>,
    sender: broadcast::Sender<ChainEvent>,
}

impl ChainMonitor {
    /// Create a new monitor with [`MONITOR_CHANNEL_CAPACITY`].
    pub fn new(client: OnlineClient<SubtensorConfig>) -> Self {
        let (sender, _) = broadcast::channel(MONITOR_CHANNEL_CAPACITY);
        Self { client, sender }
    }

    /// Create a new monitor with a custom broadcast capacity.
    pub fn with_capacity(client: OnlineClient<SubtensorConfig>, capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { client, sender }
    }

    /// Subscribe to the broadcast channel (each subscriber gets its own copy of every event).
    pub fn subscribe(&self) -> broadcast::Receiver<ChainEvent> {
        self.sender.subscribe()
    }

    /// Access the raw broadcast sender (for advanced use cases).
    pub fn sender(&self) -> &broadcast::Sender<ChainEvent> {
        &self.sender
    }

    /// Start the monitor in a background task (auto-reconnects on errors).
    pub fn start(self: &std::sync::Arc<Self>) {
        let this = self.clone();
        tokio::spawn(async move {
            this.run().await;
        });
    }

    async fn run(&self) {
        loop {
            match self.run_once().await {
                Ok(()) => {
                    tracing::info!("chain monitor block stream ended, reconnecting...");
                }
                Err(e) => {
                    tracing::error!("chain monitor error: {e}, reconnecting in 5s...");
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }
    }

    async fn run_once(&self) -> Result<(), MonitorError> {
        let mut blocks = self
            .client
            .stream_blocks()
            .await
            .map_err(|e| MonitorError::SubscriptionFailed(e.to_string()))?;

        while let Some(result) = blocks.next().await {
            let block = result.map_err(|e| MonitorError::SubscriptionFailed(e.to_string()))?;
            let block_number = block.number();
            let block_hash = block.hash();

            let at_block = block
                .at()
                .await
                .map_err(|e| MonitorError::EventFetchFailed(block_number, e.to_string()))?;

            let events = at_block
                .events()
                .fetch()
                .await
                .map_err(|e| MonitorError::EventFetchFailed(block_number, e.to_string()))?;

            for raw_event in events.iter() {
                let raw_event = match raw_event {
                    Ok(e) => e,
                    Err(e) => {
                        tracing::debug!("skipping undecodable event at #{block_number}: {e}");
                        continue;
                    }
                };

                let chain_event = decode_event(&raw_event, block_number, block_hash);
                let _ = self.sender.send(chain_event);
            }
        }

        Ok(())
    }

    /// Start the monitor with an event handler (dispatches each event to the handler).
    pub fn start_with_handler(
        self: &std::sync::Arc<Self>,
        handler: std::sync::Arc<dyn ChainEventHandler>,
    ) {
        let this = self.clone();
        tokio::spawn(async move {
            this.run_with_handler(handler).await;
        });
    }

    async fn run_with_handler(&self, handler: std::sync::Arc<dyn ChainEventHandler>) {
        loop {
            match self.run_once_with_handler(handler.as_ref()).await {
                Ok(()) => {
                    tracing::info!("chain monitor (with handler) stream ended, reconnecting...");
                }
                Err(e) => {
                    tracing::error!(
                        "chain monitor (with handler) error: {e}, reconnecting in 5s..."
                    );
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }
    }

    async fn run_once_with_handler(
        &self,
        handler: &dyn ChainEventHandler,
    ) -> Result<(), MonitorError> {
        let mut blocks = self
            .client
            .stream_blocks()
            .await
            .map_err(|e| MonitorError::SubscriptionFailed(e.to_string()))?;

        while let Some(result) = blocks.next().await {
            let block = result.map_err(|e| MonitorError::SubscriptionFailed(e.to_string()))?;
            let block_number = block.number();
            let block_hash = block.hash();

            let at_block = block
                .at()
                .await
                .map_err(|e| MonitorError::EventFetchFailed(block_number, e.to_string()))?;

            let events = at_block
                .events()
                .fetch()
                .await
                .map_err(|e| MonitorError::EventFetchFailed(block_number, e.to_string()))?;

            for raw_event in events.iter() {
                let raw_event = match raw_event {
                    Ok(e) => e,
                    Err(e) => {
                        tracing::debug!("skipping undecodable event at #{block_number}: {e}");
                        continue;
                    }
                };

                let chain_event = decode_event(&raw_event, block_number, block_hash);
                dispatch_event(handler, &chain_event);
                let _ = self.sender.send(chain_event);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn h256() -> subxt::utils::H256 {
        subxt::utils::H256::zero()
    }

    #[test]
    fn broadcast_sends_to_receiver() {
        let (tx, mut rx) = broadcast::channel::<ChainEvent>(MONITOR_CHANNEL_CAPACITY);

        let event = ChainEvent::NeuronRegistered {
            netuid: 1,
            hotkey: "hk".into(),
            coldkey: "ck".into(),
            block_number: 100,
            block_hash: h256(),
        };

        let _ = tx.send(event.clone());
        let received = rx.try_recv().unwrap();
        assert!(matches!(received, ChainEvent::NeuronRegistered { .. }));
        assert_eq!(received.block_number(), 100);
    }

    #[test]
    fn broadcast_reaches_multiple_receivers() {
        let (tx, _) = broadcast::channel::<ChainEvent>(MONITOR_CHANNEL_CAPACITY);
        let mut rx1 = tx.subscribe();
        let mut rx2 = tx.subscribe();

        let event = ChainEvent::Transfer {
            from: "5A".into(),
            to: "5B".into(),
            amount: 100,
            block_number: 200,
            block_hash: h256(),
        };

        let _ = tx.send(event);
        let r1 = rx1.try_recv().unwrap();
        let r2 = rx2.try_recv().unwrap();
        assert!(matches!(r1, ChainEvent::Transfer { .. }));
        assert!(matches!(r2, ChainEvent::Transfer { .. }));
    }

    #[test]
    fn broadcast_drops_on_laggy_receiver() {
        let (tx, mut rx) = broadcast::channel::<ChainEvent>(2);

        let e1 = ChainEvent::ExtrinsicSuccess { block_number: 1, block_hash: h256() };
        let e2 = ChainEvent::ExtrinsicSuccess { block_number: 2, block_hash: h256() };
        let e3 = ChainEvent::ExtrinsicSuccess { block_number: 3, block_hash: h256() };

        let _ = tx.send(e1);
        let _ = tx.send(e2);
        let _ = tx.send(e3);

        assert!(matches!(rx.try_recv(), Err(broadcast::error::TryRecvError::Lagged(1))));
        assert!(rx.try_recv().is_ok());
        assert!(rx.try_recv().is_ok());
        assert!(matches!(rx.try_recv(), Err(broadcast::error::TryRecvError::Empty)));
    }

    #[test]
    fn monitor_error_converts_to_bittensor_error() {
        let err = MonitorError::SubscriptionFailed("ws closed".into());
        let bt_err: BittensorError = err.into();
        assert!(matches!(bt_err, BittensorError::Rpc(_)));
    }

    #[test]
    fn monitor_error_event_fetch() {
        let err = MonitorError::EventFetchFailed(42, "timeout".into());
        let bt_err: BittensorError = err.into();
        assert!(matches!(bt_err, BittensorError::Rpc(_)));
    }

    #[test]
    fn broadcast_capacity_custom() {
        let (tx, _) = broadcast::channel::<ChainEvent>(64);
        assert_eq!(tx.receiver_count(), 0);
    }

    #[test]
    fn receiver_count_increases_with_subscriptions() {
        let (tx, _) = broadcast::channel::<ChainEvent>(64);
        let _rx1 = tx.subscribe();
        assert_eq!(tx.receiver_count(), 1);
        let _rx2 = tx.subscribe();
        assert_eq!(tx.receiver_count(), 2);
    }

    struct CountingHandler {
        count: std::sync::atomic::AtomicUsize,
    }

    impl ChainEventHandler for CountingHandler {
        fn on_neuron_registered(&self, _netuid: u16, _hotkey: &str, _coldkey: &str) {
            self.count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        fn on_transfer(&self, _from: &str, _to: &str, _amount: u64) {
            self.count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    }

    #[test]
    fn handler_dispatch_with_broadcast() {
        let handler =
            std::sync::Arc::new(CountingHandler { count: std::sync::atomic::AtomicUsize::new(0) });

        let events = vec![
            ChainEvent::NeuronRegistered {
                netuid: 1,
                hotkey: "hk".into(),
                coldkey: "ck".into(),
                block_number: 1,
                block_hash: h256(),
            },
            ChainEvent::Transfer {
                from: "5A".into(),
                to: "5B".into(),
                amount: 100,
                block_number: 2,
                block_hash: h256(),
            },
        ];

        for event in &events {
            dispatch_event(handler.as_ref(), event);
        }

        assert_eq!(handler.count.load(std::sync::atomic::Ordering::Relaxed), 2);
    }
}
