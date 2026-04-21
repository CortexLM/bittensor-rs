//! Subtensor chain configuration and network presets.

use subxt::config::substrate::SubstrateConfig;

/// Configuration type for the Bittensor Subtensor chain.
///
/// This wraps [`subxt::config::substrate::SubstrateConfig`] to provide a
/// distinct type for Bittensor while inheriting all standard Substrate
/// primitives (Blake2-256 hashing, 32-byte account IDs, etc.).
#[derive(Debug, Clone)]
pub struct SubtensorConfig {
    inner: SubstrateConfig,
}

impl SubtensorConfig {
    /// Create a new SubtensorConfig with default settings.
    pub fn new() -> Self {
        Self { inner: SubstrateConfig::new() }
    }

    /// Create a builder for custom configuration (e.g. historic block support).
    pub fn builder() -> SubtensorConfigBuilder {
        SubtensorConfigBuilder { inner: SubstrateConfig::builder() }
    }
}

impl Default for SubtensorConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for [`SubtensorConfig`].
pub struct SubtensorConfigBuilder {
    inner: subxt::config::substrate::SubstrateConfigBuilder,
}

impl SubtensorConfigBuilder {
    /// Set the genesis hash for this chain.
    pub fn set_genesis_hash(mut self, genesis_hash: subxt::utils::H256) -> Self {
        self.inner = self.inner.set_genesis_hash(genesis_hash);
        self
    }

    /// Build the [`SubtensorConfig`].
    pub fn build(self) -> SubtensorConfig {
        SubtensorConfig { inner: self.inner.build() }
    }
}

impl subxt::Config for SubtensorConfig {
    type AccountId = subxt::utils::AccountId32;
    type Address = subxt::utils::MultiAddress<Self::AccountId, u32>;
    type Signature = subxt::utils::MultiSignature;
    type Hasher = subxt::config::substrate::DynamicHasher256;
    type Header =
        subxt::config::substrate::SubstrateHeader<<Self::Hasher as subxt::config::Hasher>::Hash>;
    type TransactionExtensions = subxt::config::substrate::SubstrateExtrinsicParams<Self>;
    type AssetId = u32;

    fn genesis_hash(&self) -> Option<subxt::config::HashFor<Self>> {
        self.inner.genesis_hash()
    }

    fn spec_and_transaction_version_for_block_number(
        &self,
        block_number: u64,
    ) -> Option<(u32, u32)> {
        self.inner.spec_and_transaction_version_for_block_number(block_number)
    }

    fn metadata_for_spec_version(&self, spec_version: u32) -> Option<subxt::metadata::ArcMetadata> {
        self.inner.metadata_for_spec_version(spec_version)
    }

    fn set_metadata_for_spec_version(
        &self,
        spec_version: u32,
        metadata: subxt::metadata::ArcMetadata,
    ) {
        self.inner.set_metadata_for_spec_version(spec_version, metadata);
    }
}

/// Network configuration for connecting to a Subtensor chain endpoint.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NetworkConfig {
    /// Human-readable network name (e.g. "finney", "test", "local").
    pub name: String,
    /// WebSocket endpoint URL.
    pub ws_endpoint: String,
    /// Optional archive node endpoint URL.
    pub archive_endpoint: Option<String>,
    /// Chain identifier (SS58 prefix or chain name).
    pub chain_id: u16,
}

impl NetworkConfig {
    /// Finney mainnet configuration.
    pub fn finney() -> Self {
        Self {
            name: "finney".into(),
            ws_endpoint: "wss://entrypoint-finney.opentensor.ai:443".into(),
            archive_endpoint: None,
            chain_id: 42,
        }
    }

    /// Testnet configuration.
    pub fn test() -> Self {
        Self {
            name: "test".into(),
            ws_endpoint: "wss://test.finney.opentensor.ai:443".into(),
            archive_endpoint: None,
            chain_id: 42,
        }
    }

    /// Local development node configuration.
    pub fn local() -> Self {
        Self {
            name: "local".into(),
            ws_endpoint: "ws://127.0.0.1:9944".into(),
            archive_endpoint: None,
            chain_id: 42,
        }
    }

    /// Archive node configuration (falls back to Finney if no archive endpoint exists).
    pub fn archive() -> Self {
        Self {
            name: "archive".into(),
            ws_endpoint: "wss://archive.finney.opentensor.ai:443".into(),
            archive_endpoint: Some("wss://archive.finney.opentensor.ai:443".into()),
            chain_id: 42,
        }
    }

    /// Latent-lite endpoint configuration (falls back to Finney if no dedicated endpoint exists).
    pub fn latent_lite() -> Self {
        Self {
            name: "latent-lite".into(),
            ws_endpoint: "wss://lite.finney.opentensor.ai:443".into(),
            archive_endpoint: None,
            chain_id: 42,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subtensor_config_default() {
        let _config = SubtensorConfig::default();
    }

    #[test]
    fn subtensor_config_builder() {
        let _config = SubtensorConfig::builder().build();
    }

    #[test]
    fn finney_network_config() {
        let cfg = NetworkConfig::finney();
        assert_eq!(cfg.name, "finney");
        assert_eq!(cfg.ws_endpoint, "wss://entrypoint-finney.opentensor.ai:443");
        assert!(cfg.archive_endpoint.is_none());
    }

    #[test]
    fn test_network_config() {
        let cfg = NetworkConfig::test();
        assert_eq!(cfg.name, "test");
        assert!(cfg.ws_endpoint.contains("test"));
    }

    #[test]
    fn local_network_config() {
        let cfg = NetworkConfig::local();
        assert_eq!(cfg.name, "local");
        assert_eq!(cfg.ws_endpoint, "ws://127.0.0.1:9944");
    }

    #[test]
    fn archive_network_config() {
        let cfg = NetworkConfig::archive();
        assert_eq!(cfg.name, "archive");
        assert!(cfg.archive_endpoint.is_some());
    }

    #[test]
    fn latent_lite_network_config() {
        let cfg = NetworkConfig::latent_lite();
        assert_eq!(cfg.name, "latent-lite");
    }

    #[test]
    fn network_config_serialization_roundtrip() {
        let cfg = NetworkConfig::finney();
        let json = serde_json::to_string(&cfg).expect("serialize");
        let deserialized: NetworkConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(cfg, deserialized);
    }

    #[test]
    fn network_config_equality() {
        assert_eq!(NetworkConfig::finney(), NetworkConfig::finney());
        assert_ne!(NetworkConfig::finney(), NetworkConfig::test());
    }
}
