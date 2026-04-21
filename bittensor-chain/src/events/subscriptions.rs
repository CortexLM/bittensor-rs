//! Raw event and block subscriptions — single-consumer streaming APIs.
//!
//! - [`subscribe_events`] — yields decoded [`ChainEvent`]s via an `mpsc` channel
//! - [`subscribe_blocks`] — yields a stream of finalized blocks
//! - [`subscribe_storage`] — polls a storage key and emits changes (feature-gated)

use bittensor_core::config::SubtensorConfig;
use bittensor_core::error::BittensorError;
use subxt::OnlineClient;

use super::ChainEvent;
use super::decode_event;

/// The type returned by [`subscribe_blocks`].
pub type BlockStream = subxt::client::Blocks<SubtensorConfig>;

/// Subscribe to finalized blocks, yielding a stream of blocks.
///
/// Uses subxt 0.50.0's `stream_blocks()` which subscribes to finalized blocks
/// via the `chainHead_follow` protocol.
pub async fn subscribe_blocks(
    client: &OnlineClient<SubtensorConfig>,
) -> Result<BlockStream, BittensorError> {
    client
        .stream_blocks()
        .await
        .map_err(|e| BittensorError::Rpc(format!("failed to subscribe to blocks: {e}")))
}

/// Subscribe to all chain events from finalized blocks.
///
/// For each finalized block, fetches the event set, decodes each raw event
/// into a [`ChainEvent`], and yields it. Unknown events are emitted as
/// [`ChainEvent::Unknown`].
pub async fn subscribe_events(
    client: &OnlineClient<SubtensorConfig>,
) -> Result<tokio::sync::mpsc::Receiver<ChainEvent>, BittensorError> {
    let mut blocks = subscribe_blocks(client).await?;
    let (tx, rx) = tokio::sync::mpsc::channel(1024);

    tokio::spawn(async move {
        while let Some(result) = blocks.next().await {
            let block = match result {
                Ok(b) => b,
                Err(e) => {
                    tracing::warn!("block stream error: {e}");
                    continue;
                }
            };

            let block_number = block.number();
            let block_hash = block.hash();

            let at_block = match block.at().await {
                Ok(ab) => ab,
                Err(e) => {
                    tracing::warn!("failed to create at-block client for #{block_number}: {e}");
                    continue;
                }
            };

            let events = match at_block.events().fetch().await {
                Ok(evts) => evts,
                Err(e) => {
                    tracing::warn!("failed to fetch events at #{block_number}: {e}");
                    continue;
                }
            };

            for raw_event in events.iter() {
                let raw_event = match raw_event {
                    Ok(e) => e,
                    Err(e) => {
                        tracing::debug!("skipping undecodable event at #{block_number}: {e}");
                        continue;
                    }
                };

                let chain_event = decode_event(&raw_event, block_number, block_hash);
                if tx.send(chain_event).await.is_err() {
                    tracing::debug!("event receiver dropped, stopping subscription");
                    return;
                }
            }
        }
    });

    Ok(rx)
}

/// Subscribe to changes in a specific storage key.
///
/// Since subxt 0.50.0 does not expose a native `subscribe_storage` RPC in its
/// high-level API, this implements storage watching by polling at a configurable
/// interval. Each time the value changes, the new bytes are sent to the channel.
///
/// This functionality is feature-gated behind `storage-subscriptions` because it
/// relies on polling rather than native WebSocket storage subscriptions.
#[cfg(feature = "storage-subscriptions")]
pub async fn subscribe_storage(
    client: &OnlineClient<SubtensorConfig>,
    key: Vec<u8>,
    poll_interval_ms: u64,
) -> Result<tokio::sync::mpsc::Receiver<Vec<u8>>, BittensorError> {
    let client = client.clone();
    let (tx, rx) = tokio::sync::mpsc::channel(256);

    tokio::spawn(async move {
        let mut last_value: Option<Vec<u8>> = None;
        let interval = tokio::time::Duration::from_millis(poll_interval_ms);

        loop {
            tokio::time::sleep(interval).await;

            let at_block = match client.at_current_block().await {
                Ok(ab) => ab,
                Err(e) => {
                    tracing::debug!("storage poll: failed to get current block: {e}");
                    continue;
                }
            };

            let value: Option<Vec<u8>> = at_block.storage().fetch_raw(key.clone()).await.ok();

            let changed = match (&last_value, &value) {
                (None, Some(_)) => true,
                (Some(_), None) => true,
                (Some(prev), Some(curr)) => prev != curr,
                (None, None) => false,
            };

            if changed {
                let new_bytes = value.unwrap_or_default();
                last_value = Some(new_bytes.clone());
                if tx.send(new_bytes).await.is_err() {
                    return;
                }
            }
        }
    });

    Ok(rx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subscribe_blocks_exists() {
        let _fn_ptr = subscribe_blocks;
    }

    #[test]
    fn subscribe_events_exists() {
        let _fn_ptr = subscribe_events;
    }

    #[cfg(feature = "storage-subscriptions")]
    #[test]
    fn subscribe_storage_exists() {
        let _fn_ptr = subscribe_storage;
    }
}
