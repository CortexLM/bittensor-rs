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

    #[test]
    fn test_network_data_default_uses_balance_zero() {
        let data = NetworkData::default();
        assert_eq!(data.total_stake, Balance::ZERO);
        assert_eq!(data.total_issuance, Balance::ZERO);
    }

    #[test]
    fn test_network_data_default_all_fields() {
        let data = NetworkData::default();
        assert_eq!(data.block_height, 0);
        assert_eq!(data.total_stake, Balance::ZERO);
        assert_eq!(data.total_issuance, Balance::ZERO);
        assert_eq!(data.network_hash_rate, 0);
        assert!(!data.connected);
        assert!(data.subnet_ids.is_empty());
        assert!(data.last_error.is_none());
    }

    #[test]
    fn test_network_fetcher_new() {
        let fetcher = NetworkFetcher::new(NetworkConfig::finney(), 30);
        assert_eq!(fetcher.refresh_secs, 30);
    }

    #[test]
    fn test_network_fetcher_new_with_various_refresh_rates() {
        let f1 = NetworkFetcher::new(NetworkConfig::finney(), 1);
        assert_eq!(f1.refresh_secs, 1);

        let f2 = NetworkFetcher::new(NetworkConfig::test(), 60);
        assert_eq!(f2.refresh_secs, 60);

        let f3 = NetworkFetcher::new(NetworkConfig::local(), 300);
        assert_eq!(f3.refresh_secs, 300);
    }

    #[test]
    fn test_network_data_construction_all_fields() {
        let data = NetworkData {
            block_height: 999,
            total_stake: Balance::from_tao(42.0),
            total_issuance: Balance::from_tao(7.0),
            network_hash_rate: 1234,
            connected: true,
            subnet_ids: vec![0, 1, 2, 18],
            last_error: Some("timeout".into()),
        };
        assert_eq!(data.block_height, 999);
        assert_eq!(data.total_stake, Balance::from_tao(42.0));
        assert_eq!(data.total_issuance, Balance::from_tao(7.0));
        assert_eq!(data.network_hash_rate, 1234);
        assert!(data.connected);
        assert_eq!(data.subnet_ids, vec![0, 1, 2, 18]);
        assert_eq!(data.last_error.as_deref(), Some("timeout"));
    }

    #[test]
    fn test_network_data_clone() {
        let original = NetworkData {
            block_height: 100,
            total_stake: Balance::from_tao(50.0),
            total_issuance: Balance::from_tao(25.0),
            network_hash_rate: 10,
            connected: true,
            subnet_ids: vec![5, 6],
            last_error: Some("err".into()),
        };
        let cloned = original.clone();
        assert_eq!(cloned.block_height, original.block_height);
        assert_eq!(cloned.total_stake, original.total_stake);
        assert_eq!(cloned.total_issuance, original.total_issuance);
        assert_eq!(cloned.network_hash_rate, original.network_hash_rate);
        assert_eq!(cloned.connected, original.connected);
        assert_eq!(cloned.subnet_ids, original.subnet_ids);
        assert_eq!(cloned.last_error, original.last_error);
    }

    #[test]
    fn test_network_data_debug_format() {
        let data = NetworkData {
            block_height: 1,
            total_stake: Balance::ZERO,
            total_issuance: Balance::ZERO,
            network_hash_rate: 0,
            connected: false,
            subnet_ids: vec![],
            last_error: None,
        };
        let debug_str = format!("{data:?}");
        assert!(debug_str.contains("NetworkData"));
    }

    #[test]
    fn test_network_data_field_mutation() {
        let mut data = NetworkData::default();
        data.block_height = 500;
        assert_eq!(data.block_height, 500);

        data.total_stake = Balance::from_tao(100.0);
        assert_eq!(data.total_stake, Balance::from_tao(100.0));

        data.total_issuance = Balance::from_tao(200.0);
        assert_eq!(data.total_issuance, Balance::from_tao(200.0));

        data.network_hash_rate = 999;
        assert_eq!(data.network_hash_rate, 999);

        data.connected = true;
        assert!(data.connected);

        data.subnet_ids = vec![10, 20, 30];
        assert_eq!(data.subnet_ids, vec![10, 20, 30]);

        data.last_error = Some("connection refused".into());
        assert_eq!(data.last_error.as_deref(), Some("connection refused"));
    }
}
