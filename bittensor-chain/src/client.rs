use bittensor_core::config::{NetworkConfig, SubtensorConfig};
use bittensor_core::error::BittensorError;
use subxt::OnlineClient;

/// Typed client for the Bittensor Subtensor chain.
///
/// Wraps a subxt `OnlineClient` with convenience methods for connection,
/// failover, and block access. Construct via [`SubtensorClient::from_config`]
/// or [`SubtensorClient::from_url`].
///
/// # Examples
///
/// ```ignore
/// use bittensor_chain::SubtensorClient;
/// use bittensor_core::config::NetworkConfig;
///
/// async fn example() -> Result<(), bittensor_core::error::BittensorError> {
///     let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;
///     let block = client.at_current_block().await?;
///     println!("Connected at block {:?}", block.block_hash());
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct SubtensorClient {
    inner: OnlineClient<SubtensorConfig>,
}

impl SubtensorClient {
    /// Connect using a [`NetworkConfig`], with automatic failover to the
    /// archive endpoint if the primary WebSocket fails.
    pub async fn from_config(config: NetworkConfig) -> Result<Self, BittensorError> {
        let endpoints: Vec<&str> = if let Some(ref archive) = config.archive_endpoint {
            vec![archive.as_str(), config.ws_endpoint.as_str()]
        } else {
            vec![config.ws_endpoint.as_str()]
        };
        Self::connect_with_failover(&endpoints).await
    }

    /// Connect to a single WebSocket URL (no failover).
    pub async fn from_url(url: &str) -> Result<Self, BittensorError> {
        Self::connect_with_failover(&[url]).await
    }

    async fn connect_with_failover(endpoints: &[&str]) -> Result<Self, BittensorError> {
        let mut last_err = None;
        for endpoint in endpoints {
            match OnlineClient::<SubtensorConfig>::from_url(*endpoint).await {
                Ok(client) => return Ok(Self { inner: client }),
                Err(e) => {
                    last_err = Some(BittensorError::Rpc(format!(
                        "failed to connect to {}: {e}",
                        endpoint
                    )));
                }
            }
        }
        Err(last_err.unwrap_or_else(|| BittensorError::Config("no endpoints provided".into())))
    }

    /// Access the underlying subxt `OnlineClient` for advanced queries.
    pub fn rpc(&self) -> &OnlineClient<SubtensorConfig> {
        &self.inner
    }

    /// Return a block-specific client at the current best block.
    pub async fn at_current_block(&self) -> Result<ClientAtBlock, BittensorError> {
        self.inner.at_current_block().await.map_err(|e| BittensorError::Rpc(e.to_string()))
    }

    /// Look up the block hash for a given block number.
    ///
    /// Returns `None` if the block is not found (e.g. pruned or not yet produced).
    pub async fn get_block_hash(
        &self,
        block_number: u64,
    ) -> Result<Option<subxt::utils::H256>, BittensorError> {
        match self.inner.at_block(block_number).await {
            Ok(at) => Ok(Some(at.block_hash())),
            Err(_) => Ok(None),
        }
    }
}

/// A subxt client pinned to a specific block, used for historical queries.
pub type ClientAtBlock = subxt::client::ClientAtBlock<
    SubtensorConfig,
    subxt::client::OnlineClientAtBlockImpl<SubtensorConfig>,
>;

impl std::fmt::Debug for SubtensorClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubtensorClient").finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network_config_produces_endpoints() {
        let config = NetworkConfig::finney();
        assert!(!config.ws_endpoint.is_empty());
    }

    #[test]
    fn archive_config_has_two_endpoints() {
        let config = NetworkConfig::archive();
        assert!(config.archive_endpoint.is_some());
    }
}
