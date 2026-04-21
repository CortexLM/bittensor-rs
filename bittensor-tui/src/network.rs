//! Async data fetcher that polls the Bittensor chain at regular intervals.

use bittensor_chain::client::SubtensorClient;
use bittensor_core::balance::Balance;
use bittensor_core::config::NetworkConfig;
use std::sync::mpsc;

/// Snapshot of data fetched from the chain.
#[derive(Debug, Clone, Default)]
pub struct NetworkData {
    /// Current block height.
    pub block_height: u64,
    /// Total stake across the network.
    pub total_stake: Balance,
    /// Total issuance.
    pub total_issuance: Balance,
    /// Network hash rate (placeholder — chain doesn't expose this directly).
    pub network_hash_rate: u64,
    /// Whether the last fetch succeeded.
    pub connected: bool,
    /// List of subnet netuids (fetched on demand).
    pub subnet_ids: Vec<u16>,
    /// Last error message, if any.
    pub last_error: Option<String>,
}

/// Async fetcher that polls the Subtensor chain and sends data to the UI.
pub struct NetworkFetcher {
    config: NetworkConfig,
    refresh_secs: u64,
}

impl NetworkFetcher {
    /// Create a new fetcher with the given network config and refresh interval.
    pub fn new(config: NetworkConfig, refresh_secs: u64) -> Self {
        Self { config, refresh_secs }
    }

    /// Run the fetch loop, sending updates to the provided channel.
    pub async fn run(self, tx: mpsc::Sender<NetworkData>) {
        let interval = tokio::time::interval(std::time::Duration::from_secs(self.refresh_secs));
        tokio::pin!(interval);

        // Try to connect
        let client = match SubtensorClient::from_config(self.config.clone()).await {
            Ok(c) => {
                let data = NetworkData { connected: true, last_error: None, ..Default::default() };
                let _ = tx.send(data);
                Some(c)
            }
            Err(e) => {
                let data = NetworkData {
                    connected: false,
                    last_error: Some(e.to_string()),
                    ..Default::default()
                };
                let _ = tx.send(data);
                None
            }
        };

        loop {
            interval.tick().await;

            let mut data = NetworkData::default();

            if let Some(ref client) = client {
                match Self::fetch(client).await {
                    Ok(d) => {
                        data = d;
                        data.connected = true;
                        data.last_error = None;
                    }
                    Err(e) => {
                        data.connected = false;
                        data.last_error = Some(e.to_string());
                    }
                }
            } else {
                data.connected = false;
                data.last_error = Some("not connected".into());
            }

            if tx.send(data).is_err() {
                // Receiver dropped — exit
                break;
            }
        }
    }

    /// Fetch all data from the chain in one pass.
    async fn fetch(
        client: &SubtensorClient,
    ) -> Result<NetworkData, bittensor_core::error::BittensorError> {
        let rpc = client.rpc();

        let block_height = bittensor_chain::queries::get_network_block(rpc).await?;
        let total_stake = bittensor_chain::queries::get_total_network_stake(rpc).await?;
        let total_issuance =
            bittensor_chain::queries::get_total_issuance(rpc).await.unwrap_or(Balance::ZERO);

        Ok(NetworkData {
            block_height,
            total_stake,
            total_issuance,
            network_hash_rate: 0,
            connected: true,
            subnet_ids: Vec::new(),
            last_error: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_data_default() {
        let data = NetworkData::default();
        assert_eq!(data.block_height, 0);
        assert!(!data.connected);
        assert!(data.last_error.is_none());
        assert!(data.subnet_ids.is_empty());
    }

    #[test]
    fn test_network_data_fields() {
        let data = NetworkData {
            block_height: 12345,
            total_stake: Balance::from_tao(1000.0),
            total_issuance: Balance::from_tao(500.0),
            network_hash_rate: 42,
            connected: true,
            subnet_ids: vec![1, 2, 3],
            last_error: None,
        };
        assert_eq!(data.block_height, 12345);
        assert_eq!(data.subnet_ids.len(), 3);
        assert!(data.connected);
    }
}
