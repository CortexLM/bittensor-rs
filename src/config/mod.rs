//! Configuration module for Bittensor SDK
//! Provides configuration management similar to Python's bittensor.core.config

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::core::constants;

/// Default network configuration
pub const DEFAULT_NETWORK: &str = "finney";
pub const DEFAULT_CHAIN_ENDPOINT: &str = constants::FINNEY_ENDPOINT;

/// Network endpoints mapping
pub fn get_network_endpoint(network: &str) -> &'static str {
    match network {
        "finney" => constants::FINNEY_ENDPOINT,
        "test" | "testnet" => constants::FINNEY_TEST_ENDPOINT,
        "archive" => constants::ARCHIVE_ENDPOINT,
        "local" => constants::LOCAL_ENDPOINT,
        _ => DEFAULT_CHAIN_ENDPOINT,
    }
}

/// Axon configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxonConfig {
    pub port: u16,
    pub ip: String,
    pub external_ip: Option<String>,
    pub external_port: Option<u16>,
    pub max_workers: usize,
}

impl Default for AxonConfig {
    fn default() -> Self {
        Self {
            port: 8091,
            ip: "0.0.0.0".to_string(),
            external_ip: None,
            external_port: None,
            max_workers: 10,
        }
    }
}

/// Subtensor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtensorConfig {
    pub network: String,
    pub chain_endpoint: String,
    pub retry_forever: bool,
}

impl Default for SubtensorConfig {
    fn default() -> Self {
        Self {
            network: DEFAULT_NETWORK.to_string(),
            chain_endpoint: DEFAULT_CHAIN_ENDPOINT.to_string(),
            retry_forever: false,
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub debug: bool,
    pub trace: bool,
    pub record_log: bool,
    pub logging_dir: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            debug: false,
            trace: false,
            record_log: false,
            logging_dir: "~/.bittensor/logs".to_string(),
        }
    }
}

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub axon: AxonConfig,
    pub subtensor: SubtensorConfig,
    pub logging: LoggingConfig,
    /// Additional custom configuration
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Config {
    /// Create new config with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Create config for a specific network
    pub fn for_network(network: &str) -> Self {
        let endpoint = get_network_endpoint(network);
        Self {
            subtensor: SubtensorConfig {
                network: network.to_string(),
                chain_endpoint: endpoint.to_string(),
                retry_forever: false,
            },
            ..Default::default()
        }
    }

    /// Set network
    pub fn with_network(mut self, network: &str) -> Self {
        self.subtensor.network = network.to_string();
        self.subtensor.chain_endpoint = get_network_endpoint(network).to_string();
        self
    }

    /// Set chain endpoint directly
    pub fn with_endpoint(mut self, endpoint: &str) -> Self {
        self.subtensor.chain_endpoint = endpoint.to_string();
        self
    }

    /// Set axon port
    pub fn with_axon_port(mut self, port: u16) -> Self {
        self.axon.port = port;
        self
    }

    /// Set debug mode
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.logging.debug = debug;
        self
    }

    /// Load config from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(network) = std::env::var("BITTENSOR_NETWORK") {
            config.subtensor.network = network.clone();
            config.subtensor.chain_endpoint = get_network_endpoint(&network).to_string();
        }

        if let Ok(endpoint) = std::env::var("BITTENSOR_RPC") {
            config.subtensor.chain_endpoint = endpoint;
        }

        if let Ok(port) = std::env::var("BITTENSOR_AXON_PORT") {
            if let Ok(p) = port.parse() {
                config.axon.port = p;
            }
        }

        if std::env::var("BITTENSOR_DEBUG").is_ok() {
            config.logging.debug = true;
        }

        config
    }

    /// Merge with another config (other takes precedence)
    pub fn merge(mut self, other: Config) -> Self {
        if other.subtensor.network != DEFAULT_NETWORK {
            self.subtensor = other.subtensor;
        }
        if other.axon.port != 8091 {
            self.axon = other.axon;
        }
        if other.logging.debug {
            self.logging = other.logging;
        }
        self.extra.extend(other.extra);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::new();
        assert_eq!(config.subtensor.network, "finney");
        assert_eq!(config.axon.port, 8091);
    }

    #[test]
    fn test_network_config() {
        let config = Config::for_network("test");
        assert_eq!(config.subtensor.network, "test");
        assert!(config.subtensor.chain_endpoint.contains("test"));
    }

    #[test]
    fn test_builder_pattern() {
        let config = Config::new()
            .with_network("local")
            .with_axon_port(9000)
            .with_debug(true);
        
        assert_eq!(config.subtensor.network, "local");
        assert_eq!(config.axon.port, 9000);
        assert!(config.logging.debug);
    }
}
