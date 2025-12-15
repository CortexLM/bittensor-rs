//! Block listener for Bittensor
//!
//! Subscribes to finalized blocks and emits events for:
//! - New blocks
//! - Epoch transitions
//! - Phase changes (evaluation -> commit -> reveal)

use crate::blocks::epoch_tracker::{EpochInfo, EpochPhase, EpochTracker, EpochTransition};
use crate::chain::{BittensorClient, Error as ChainError};
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// Events emitted by the block listener
#[derive(Debug, Clone)]
pub enum BlockEvent {
    /// New finalized block
    NewBlock {
        block_number: u64,
        epoch_info: EpochInfo,
    },
    /// Epoch transition occurred
    EpochTransition(EpochTransition),
    /// Phase changed within epoch
    PhaseChange {
        block_number: u64,
        old_phase: EpochPhase,
        new_phase: EpochPhase,
        epoch: u64,
    },
    /// Connection error (will retry)
    ConnectionError(String),
    /// Listener stopped
    Stopped,
}

/// Configuration for the block listener
#[derive(Debug, Clone)]
pub struct BlockListenerConfig {
    /// Subnet to track
    pub netuid: u16,
    /// Event channel capacity
    pub channel_capacity: usize,
    /// Auto-reconnect on error
    pub auto_reconnect: bool,
    /// Reconnect delay in milliseconds
    pub reconnect_delay_ms: u64,
}

impl Default for BlockListenerConfig {
    fn default() -> Self {
        Self {
            netuid: 1,
            channel_capacity: 100,
            auto_reconnect: true,
            reconnect_delay_ms: 5000,
        }
    }
}

/// Block listener that subscribes to finalized blocks
pub struct BlockListener {
    config: BlockListenerConfig,
    epoch_tracker: Arc<RwLock<EpochTracker>>,
    event_tx: broadcast::Sender<BlockEvent>,
    running: Arc<RwLock<bool>>,
    last_phase: Arc<RwLock<EpochPhase>>,
}

impl BlockListener {
    /// Create a new block listener
    pub fn new(config: BlockListenerConfig) -> Self {
        let (event_tx, _) = broadcast::channel(config.channel_capacity);
        let epoch_tracker = Arc::new(RwLock::new(EpochTracker::new(config.netuid)));

        Self {
            config,
            epoch_tracker,
            event_tx,
            running: Arc::new(RwLock::new(false)),
            last_phase: Arc::new(RwLock::new(EpochPhase::Evaluation)),
        }
    }

    /// Subscribe to block events
    pub fn subscribe(&self) -> broadcast::Receiver<BlockEvent> {
        self.event_tx.subscribe()
    }

    /// Initialize the epoch tracker with on-chain data
    pub async fn init(&self, client: &BittensorClient) -> anyhow::Result<()> {
        let mut tracker = self.epoch_tracker.write().await;
        tracker.init(client).await?;

        // Get current block to set initial phase
        let current_block = client.block_number().await?;
        let info = tracker.get_epoch_info(current_block);
        *self.last_phase.write().await = info.phase;

        Ok(())
    }

    /// Start listening to blocks (runs in background)
    pub async fn start(&self, client: Arc<BittensorClient>) -> anyhow::Result<()> {
        // Check if already running
        {
            let mut running = self.running.write().await;
            if *running {
                return Ok(());
            }
            *running = true;
        }

        let epoch_tracker = self.epoch_tracker.clone();
        let event_tx = self.event_tx.clone();
        let running = self.running.clone();
        let last_phase = self.last_phase.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            loop {
                // Check if we should stop
                if !*running.read().await {
                    let _ = event_tx.send(BlockEvent::Stopped);
                    break;
                }

                // Subscribe to finalized blocks
                match client.subscribe_finalized_blocks().await {
                    Ok(mut block_stream) => {
                        while let Some(result) = block_stream.next().await {
                            // Check if we should stop
                            if !*running.read().await {
                                break;
                            }

                            match result {
                                Ok(block_number) => {
                                    // Update epoch tracker and get info
                                    let mut tracker = epoch_tracker.write().await;

                                    // Check for epoch transition
                                    if let Some(transition) =
                                        tracker.check_epoch_transition(block_number)
                                    {
                                        let _ =
                                            event_tx.send(BlockEvent::EpochTransition(transition));
                                    }

                                    let epoch_info = tracker.get_epoch_info(block_number);
                                    drop(tracker);

                                    // Check for phase change
                                    let mut last = last_phase.write().await;
                                    if epoch_info.phase != *last {
                                        let _ = event_tx.send(BlockEvent::PhaseChange {
                                            block_number,
                                            old_phase: *last,
                                            new_phase: epoch_info.phase,
                                            epoch: epoch_info.epoch_number,
                                        });
                                        *last = epoch_info.phase;
                                    }
                                    drop(last);

                                    // Send new block event
                                    let _ = event_tx.send(BlockEvent::NewBlock {
                                        block_number,
                                        epoch_info,
                                    });
                                }
                                Err(e) => {
                                    let _ =
                                        event_tx.send(BlockEvent::ConnectionError(e.to_string()));
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = event_tx.send(BlockEvent::ConnectionError(e.to_string()));
                    }
                }

                // Check if we should reconnect
                if !config.auto_reconnect || !*running.read().await {
                    break;
                }

                // Wait before reconnecting
                tokio::time::sleep(tokio::time::Duration::from_millis(
                    config.reconnect_delay_ms,
                ))
                .await;
            }
        });

        Ok(())
    }

    /// Stop the listener
    pub async fn stop(&self) {
        *self.running.write().await = false;
    }

    /// Check if listener is running
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// Get current epoch info without waiting for next block
    pub async fn current_epoch_info(&self, client: &BittensorClient) -> anyhow::Result<EpochInfo> {
        let current_block = client.block_number().await?;
        let tracker = self.epoch_tracker.read().await;
        Ok(tracker.get_epoch_info(current_block))
    }

    /// Get the epoch tracker
    pub fn epoch_tracker(&self) -> Arc<RwLock<EpochTracker>> {
        self.epoch_tracker.clone()
    }
}

/// Convenience function to create and start a block listener
pub async fn start_block_listener(
    client: Arc<BittensorClient>,
    netuid: u16,
) -> anyhow::Result<(BlockListener, broadcast::Receiver<BlockEvent>)> {
    let config = BlockListenerConfig {
        netuid,
        ..Default::default()
    };

    let listener = BlockListener::new(config);
    listener.init(&client).await?;

    let receiver = listener.subscribe();
    listener.start(client).await?;

    Ok((listener, receiver))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_listener_config() {
        let config = BlockListenerConfig::default();
        assert_eq!(config.netuid, 1);
        assert!(config.auto_reconnect);
    }
}
