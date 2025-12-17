//! Subtensor client - main interface for interacting with the Bittensor blockchain
//!
//! This module provides the `Subtensor` and `AsyncSubtensor` types which are the
//! main entry points for interacting with the Bittensor network, similar to
//! the Python SDK's `Subtensor` class.
//!
//! # Example
//!
//! ```ignore
//! use bittensor_rs::Subtensor;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Connect to finney (mainnet)
//!     let subtensor = Subtensor::new("finney").await?;
//!
//!     // Get current block
//!     let block = subtensor.block_number().await?;
//!     println!("Current block: {}", block);
//!
//!     // Get metagraph for subnet 1
//!     let metagraph = subtensor.metagraph(1).await?;
//!     println!("Subnet 1 has {} neurons", metagraph.n);
//!
//!     Ok(())
//! }
//! ```

use subxt::backend::legacy::LegacyRpcMethods;
use subxt::backend::rpc::RpcClient;
use subxt::OnlineClient;
use subxt::PolkadotConfig;
use tracing::{debug, info};

use crate::config::{determine_chain_endpoint_and_network, Config};
use crate::error::{Error, Result};
use crate::metagraph::Metagraph;
use crate::queries::{self, chain_info, subnets};
use crate::types::SubnetHyperparameters;
use crate::utils::balance::Balance;
use crate::utils::ss58::ss58_decode;

/// Async Subtensor client
///
/// This is the main interface for interacting with the Bittensor blockchain.
/// It provides methods for querying chain state, neurons, subnets, and more.
pub struct AsyncSubtensor {
    /// The underlying Substrate client
    client: OnlineClient<PolkadotConfig>,
    /// RPC methods for direct RPC calls
    rpc: LegacyRpcMethods<PolkadotConfig>,
    /// Network name (e.g., "finney", "test")
    pub network: String,
    /// Chain endpoint URL
    pub chain_endpoint: String,
}

impl AsyncSubtensor {
    /// Create a new AsyncSubtensor connection
    ///
    /// # Arguments
    /// * `network` - Network name ("finney", "test", "local") or WebSocket URL
    ///
    /// # Example
    /// ```ignore
    /// let subtensor = AsyncSubtensor::new("finney").await?;
    /// let subtensor = AsyncSubtensor::new("wss://custom-node.example.com").await?;
    /// ```
    pub async fn new(network: &str) -> Result<Self> {
        let (endpoint, network_name) = determine_chain_endpoint_and_network(network);
        
        debug!("Connecting to {} at {}", network_name, endpoint);
        
        // Create RPC client for direct RPC calls
        let rpc_client = RpcClient::from_url(&endpoint)
            .await
            .map_err(|e| Error::connection(format!("Failed to create RPC client: {}", e)))?;
        let rpc = LegacyRpcMethods::new(rpc_client.clone());
        
        // Create OnlineClient from the same RPC client
        let client = OnlineClient::<PolkadotConfig>::from_rpc_client(rpc_client)
            .await
            .map_err(|e| Error::connection(format!("Failed to connect to {}: {}", endpoint, e)))?;
        
        info!("Connected to {} network at {}", network_name, endpoint);
        
        Ok(Self {
            client,
            rpc,
            network: network_name,
            chain_endpoint: endpoint,
        })
    }

    /// Create with custom configuration
    pub async fn with_config(config: Config) -> Result<Self> {
        // Create RPC client for direct RPC calls
        let rpc_client = RpcClient::from_url(&config.chain_endpoint)
            .await
            .map_err(|e| Error::connection(format!("Failed to create RPC client: {}", e)))?;
        let rpc = LegacyRpcMethods::new(rpc_client.clone());
        
        let client = OnlineClient::<PolkadotConfig>::from_rpc_client(rpc_client)
            .await
            .map_err(|e| {
                Error::connection(format!(
                    "Failed to connect to {}: {}",
                    config.chain_endpoint, e
                ))
            })?;
        
        Ok(Self {
            client,
            rpc,
            network: config.network,
            chain_endpoint: config.chain_endpoint,
        })
    }

    /// Get the underlying client
    pub fn client(&self) -> &OnlineClient<PolkadotConfig> {
        &self.client
    }

    // ==========================================================================
    // CHAIN INFO
    // ==========================================================================

    /// Get current block number
    pub async fn block_number(&self) -> Result<u64> {
        chain_info::get_block_number(&self.client).await
    }

    /// Get block hash for a block number
    ///
    /// Uses direct RPC call to support historical blocks on archive nodes.
    pub async fn get_block_hash(&self, block: u64) -> Result<Option<String>> {
        // Use RPC method for reliable block hash retrieval
        chain_info::get_block_hash_with_rpc(&self.rpc, block).await
    }
    
    /// Get RPC methods for advanced operations
    pub fn rpc(&self) -> &LegacyRpcMethods<PolkadotConfig> {
        &self.rpc
    }

    /// Get chain name
    pub fn chain(&self) -> &str {
        &self.network
    }

    // ==========================================================================
    // SUBNET QUERIES
    // ==========================================================================

    /// Get all subnet netuids
    pub async fn get_subnets(&self) -> Result<Vec<u16>> {
        subnets::get_all_subnet_netuids(&self.client).await
    }

    /// Get number of subnets
    pub async fn get_total_subnets(&self) -> Result<u16> {
        subnets::get_num_subnets(&self.client).await
    }

    /// Check if a subnet exists
    pub async fn subnet_exists(&self, netuid: u16) -> Result<bool> {
        subnets::subnet_exists(&self.client, netuid).await
    }

    /// Get tempo for a subnet
    pub async fn tempo(&self, netuid: u16) -> Result<u16> {
        subnets::get_tempo(&self.client, netuid).await
    }

    /// Get number of neurons in a subnet
    pub async fn get_subnetwork_n(&self, netuid: u16) -> Result<u16> {
        subnets::get_subnetwork_n(&self.client, netuid).await
    }

    /// Get max neurons for a subnet
    pub async fn get_max_n(&self, netuid: u16) -> Result<u16> {
        subnets::get_max_allowed_uids(&self.client, netuid).await
    }

    /// Get immunity period for a subnet
    pub async fn immunity_period(&self, netuid: u16) -> Result<u16> {
        subnets::get_immunity_period(&self.client, netuid).await
    }

    /// Get activity cutoff for a subnet
    pub async fn activity_cutoff(&self, netuid: u16) -> Result<u16> {
        subnets::get_activity_cutoff(&self.client, netuid).await
    }

    /// Check if registration is allowed
    pub async fn registration_allowed(&self, netuid: u16) -> Result<bool> {
        subnets::get_registration_allowed(&self.client, netuid).await
    }

    /// Get max validators for a subnet
    pub async fn max_validators(&self, netuid: u16) -> Result<u16> {
        subnets::get_max_validators(&self.client, netuid).await
    }

    /// Get weights rate limit
    pub async fn weights_rate_limit(&self, netuid: u16) -> Result<u64> {
        subnets::get_weights_rate_limit(&self.client, netuid).await
    }

    /// Get weights version key
    pub async fn weights_version_key(&self, netuid: u16) -> Result<u64> {
        subnets::get_weights_version_key(&self.client, netuid).await
    }

    /// Check if commit-reveal is enabled
    pub async fn commit_reveal_enabled(&self, netuid: u16) -> Result<bool> {
        subnets::get_commit_reveal_enabled(&self.client, netuid).await
    }

    /// Get burn cost for registration
    pub async fn burn(&self, netuid: u16) -> Result<Balance> {
        subnets::get_burn(&self.client, netuid).await
    }

    /// Get difficulty for PoW registration
    pub async fn difficulty(&self, netuid: u16) -> Result<u64> {
        subnets::get_difficulty(&self.client, netuid).await
    }

    /// Get subnet hyperparameters
    pub async fn get_subnet_hyperparameters(
        &self,
        netuid: u16,
    ) -> Result<SubnetHyperparameters> {
        subnets::get_subnet_hyperparameters(&self.client, netuid).await
    }

    // ==========================================================================
    // NEURON QUERIES
    // ==========================================================================

    /// Get UID for a hotkey on a subnet
    pub async fn get_uid_for_hotkey_on_subnet(
        &self,
        netuid: u16,
        hotkey: &str,
    ) -> Result<Option<u16>> {
        let account_bytes = ss58_decode(hotkey)?;
        let account = sp_core::crypto::AccountId32::from(account_bytes);
        queries::neurons::get_uid_for_hotkey(&self.client, netuid, &account).await
    }

    /// Check if a hotkey is registered on a subnet
    pub async fn is_hotkey_registered(
        &self,
        netuid: u16,
        hotkey: &str,
    ) -> Result<bool> {
        let account_bytes = ss58_decode(hotkey)?;
        let account = sp_core::crypto::AccountId32::from(account_bytes);
        queries::neurons::is_hotkey_registered(&self.client, netuid, &account).await
    }

    /// Get stake for a hotkey from a coldkey
    pub async fn get_stake(
        &self,
        hotkey: &str,
        coldkey: &str,
    ) -> Result<Balance> {
        let hotkey_bytes = ss58_decode(hotkey)?;
        let coldkey_bytes = ss58_decode(coldkey)?;
        let hotkey_account = sp_core::crypto::AccountId32::from(hotkey_bytes);
        let coldkey_account = sp_core::crypto::AccountId32::from(coldkey_bytes);
        let stake = queries::neurons::get_stake(&self.client, &hotkey_account, &coldkey_account).await?;
        Ok(Balance::from_rao(stake))
    }

    /// Get total stake for a hotkey
    pub async fn get_total_stake_for_hotkey(&self, hotkey: &str) -> Result<Balance> {
        let hotkey_bytes = ss58_decode(hotkey)?;
        let hotkey_account = sp_core::crypto::AccountId32::from(hotkey_bytes);
        let stake = queries::neurons::get_total_stake_for_hotkey(&self.client, &hotkey_account).await?;
        Ok(Balance::from_rao(stake))
    }

    /// Get last update block for a neuron
    pub async fn last_update(&self, netuid: u16, uid: u16) -> Result<u64> {
        queries::neurons::get_last_update(&self.client, netuid, uid).await
    }

    /// Get blocks since last update for a neuron
    pub async fn blocks_since_last_update(&self, netuid: u16, uid: u16) -> Result<u64> {
        let last_update = self.last_update(netuid, uid).await?;
        let current_block = self.block_number().await?;
        Ok(current_block.saturating_sub(last_update))
    }

    /// Check if a neuron can set weights (rate limit check)
    pub async fn can_set_weights(&self, netuid: u16, uid: u16) -> Result<bool> {
        let blocks_since = self.blocks_since_last_update(netuid, uid).await?;
        let rate_limit = self.weights_rate_limit(netuid).await?;
        Ok(blocks_since > rate_limit)
    }

    // ==========================================================================
    // METAGRAPH
    // ==========================================================================

    /// Get metagraph for a subnet
    ///
    /// This fetches all neuron data for the subnet and returns a populated Metagraph.
    ///
    /// # Arguments
    /// * `netuid` - Subnet ID
    ///
    /// # Example
    /// ```ignore
    /// let metagraph = subtensor.metagraph(1).await?;
    /// println!("Subnet 1 has {} neurons", metagraph.n);
    /// ```
    pub async fn metagraph(&self, netuid: u16) -> Result<Metagraph> {
        queries::metagraph::sync_metagraph(&self.client, netuid, &self.network, false).await
    }

    /// Get lite metagraph (without axon info)
    pub async fn metagraph_lite(&self, netuid: u16) -> Result<Metagraph> {
        queries::metagraph::sync_metagraph(&self.client, netuid, &self.network, true).await
    }

    /// Sync an existing metagraph
    pub async fn sync_metagraph(&self, metagraph: &mut Metagraph) -> Result<()> {
        let new_mg = queries::metagraph::sync_metagraph(
            &self.client,
            metagraph.netuid,
            &self.network,
            false,
        )
        .await?;
        *metagraph = new_mg;
        Ok(())
    }
}

impl std::fmt::Display for AsyncSubtensor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Subtensor( network={}, endpoint={} )", self.network, self.chain_endpoint)
    }
}

/// Synchronous Subtensor client (wrapper around AsyncSubtensor)
///
/// This provides a blocking interface for environments that don't use async/await.
pub struct Subtensor {
    inner: AsyncSubtensor,
    runtime: tokio::runtime::Runtime,
}

impl Subtensor {
    /// Create a new Subtensor connection
    pub fn new(network: &str) -> Result<Self> {
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| Error::other(format!("Failed to create runtime: {}", e)))?;
        let inner = runtime.block_on(AsyncSubtensor::new(network))?;
        Ok(Self { inner, runtime })
    }

    /// Create with custom configuration
    pub fn with_config(config: Config) -> Result<Self> {
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| Error::other(format!("Failed to create runtime: {}", e)))?;
        let inner = runtime.block_on(AsyncSubtensor::with_config(config))?;
        Ok(Self { inner, runtime })
    }

    /// Get the network name
    pub fn network(&self) -> &str {
        &self.inner.network
    }

    /// Get the chain endpoint
    pub fn chain_endpoint(&self) -> &str {
        &self.inner.chain_endpoint
    }

    // Chain info
    pub fn block_number(&self) -> Result<u64> {
        self.runtime.block_on(self.inner.block_number())
    }

    pub fn get_block_hash(&self, block: u64) -> Result<Option<String>> {
        self.runtime.block_on(self.inner.get_block_hash(block))
    }

    // Subnet queries
    pub fn get_subnets(&self) -> Result<Vec<u16>> {
        self.runtime.block_on(self.inner.get_subnets())
    }

    pub fn get_total_subnets(&self) -> Result<u16> {
        self.runtime.block_on(self.inner.get_total_subnets())
    }

    pub fn subnet_exists(&self, netuid: u16) -> Result<bool> {
        self.runtime.block_on(self.inner.subnet_exists(netuid))
    }

    pub fn tempo(&self, netuid: u16) -> Result<u16> {
        self.runtime.block_on(self.inner.tempo(netuid))
    }

    pub fn get_subnetwork_n(&self, netuid: u16) -> Result<u16> {
        self.runtime.block_on(self.inner.get_subnetwork_n(netuid))
    }

    pub fn immunity_period(&self, netuid: u16) -> Result<u16> {
        self.runtime.block_on(self.inner.immunity_period(netuid))
    }

    pub fn weights_rate_limit(&self, netuid: u16) -> Result<u64> {
        self.runtime.block_on(self.inner.weights_rate_limit(netuid))
    }

    pub fn commit_reveal_enabled(&self, netuid: u16) -> Result<bool> {
        self.runtime.block_on(self.inner.commit_reveal_enabled(netuid))
    }

    pub fn burn(&self, netuid: u16) -> Result<Balance> {
        self.runtime.block_on(self.inner.burn(netuid))
    }

    // Neuron queries
    pub fn get_uid_for_hotkey_on_subnet(
        &self,
        netuid: u16,
        hotkey: &str,
    ) -> Result<Option<u16>> {
        self.runtime.block_on(self.inner.get_uid_for_hotkey_on_subnet(netuid, hotkey))
    }

    pub fn is_hotkey_registered(&self, netuid: u16, hotkey: &str) -> Result<bool> {
        self.runtime.block_on(self.inner.is_hotkey_registered(netuid, hotkey))
    }

    pub fn get_total_stake_for_hotkey(&self, hotkey: &str) -> Result<Balance> {
        self.runtime.block_on(self.inner.get_total_stake_for_hotkey(hotkey))
    }

    pub fn blocks_since_last_update(&self, netuid: u16, uid: u16) -> Result<u64> {
        self.runtime.block_on(self.inner.blocks_since_last_update(netuid, uid))
    }

    pub fn can_set_weights(&self, netuid: u16, uid: u16) -> Result<bool> {
        self.runtime.block_on(self.inner.can_set_weights(netuid, uid))
    }

    // Metagraph
    pub fn metagraph(&self, netuid: u16) -> Result<Metagraph> {
        self.runtime.block_on(self.inner.metagraph(netuid))
    }

    pub fn metagraph_lite(&self, netuid: u16) -> Result<Metagraph> {
        self.runtime.block_on(self.inner.metagraph_lite(netuid))
    }

    pub fn sync_metagraph(&self, metagraph: &mut Metagraph) -> Result<()> {
        self.runtime.block_on(self.inner.sync_metagraph(metagraph))
    }
}

impl std::fmt::Display for Subtensor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}
