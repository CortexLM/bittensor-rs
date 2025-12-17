//! Configuration and network settings for Bittensor
//!
//! This module provides network configuration, default settings, and
//! network endpoint management similar to the Python SDK's `settings.py`.

use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

/// Bittensor network names
pub const NETWORKS: &[&str] = &["finney", "test", "archive", "local", "latent-lite"];

/// Network endpoints (WebSocket URLs)
pub const FINNEY_ENTRYPOINT: &str = "wss://entrypoint-finney.opentensor.ai:443";
pub const FINNEY_TEST_ENTRYPOINT: &str = "wss://test.finney.opentensor.ai:443";
pub const ARCHIVE_ENTRYPOINT: &str = "wss://archive.chain.opentensor.ai:443";
pub const LATENT_LITE_ENTRYPOINT: &str = "wss://lite.sub.latent.to:443";

/// Default local endpoint (can be overridden by BT_SUBTENSOR_CHAIN_ENDPOINT)
pub fn local_entrypoint() -> String {
    env::var("BT_SUBTENSOR_CHAIN_ENDPOINT").unwrap_or_else(|_| "ws://127.0.0.1:9944".to_string())
}

/// Default network
pub const DEFAULT_NETWORK: &str = "finney";

/// Block time in seconds
pub const BLOCKTIME: u64 = 12;

/// SS58 address length
pub const SS58_ADDRESS_LENGTH: usize = 48;

/// SS58 format for Bittensor
pub const SS58_FORMAT: u16 = 42;

/// TAO symbol (τ)
pub const TAO_SYMBOL: char = '\u{03C4}';

/// RAO symbol (ρ)
pub const RAO_SYMBOL: char = '\u{03C1}';

/// One TAO in RAO
pub const RAO_PER_TAO: u64 = 1_000_000_000;

/// Network enum for type-safe network selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Network {
    Finney,
    Test,
    Archive,
    Local,
    LatentLite,
    Custom,
}

impl Network {
    /// Get the WebSocket endpoint for this network
    pub fn endpoint(&self) -> String {
        match self {
            Network::Finney => FINNEY_ENTRYPOINT.to_string(),
            Network::Test => FINNEY_TEST_ENTRYPOINT.to_string(),
            Network::Archive => ARCHIVE_ENTRYPOINT.to_string(),
            Network::Local => local_entrypoint(),
            Network::LatentLite => LATENT_LITE_ENTRYPOINT.to_string(),
            Network::Custom => panic!("Custom network requires explicit endpoint"),
        }
    }

    /// Parse network from string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "finney" => Some(Network::Finney),
            "test" => Some(Network::Test),
            "archive" => Some(Network::Archive),
            "local" => Some(Network::Local),
            "latent-lite" | "latentlite" => Some(Network::LatentLite),
            _ => None,
        }
    }

    /// Get network name
    pub fn name(&self) -> &'static str {
        match self {
            Network::Finney => "finney",
            Network::Test => "test",
            Network::Archive => "archive",
            Network::Local => "local",
            Network::LatentLite => "latent-lite",
            Network::Custom => "custom",
        }
    }
}

impl std::fmt::Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Get network map (name -> endpoint)
pub fn network_map() -> HashMap<&'static str, String> {
    let mut map = HashMap::new();
    map.insert("finney", FINNEY_ENTRYPOINT.to_string());
    map.insert("test", FINNEY_TEST_ENTRYPOINT.to_string());
    map.insert("archive", ARCHIVE_ENTRYPOINT.to_string());
    map.insert("local", local_entrypoint());
    map.insert("latent-lite", LATENT_LITE_ENTRYPOINT.to_string());
    map
}

/// Determine chain endpoint and network from a network string or URL
///
/// If the input looks like a URL (starts with ws:// or wss://), it's used directly.
/// Otherwise, it's treated as a network name and looked up in the network map.
pub fn determine_chain_endpoint_and_network(network: &str) -> (String, String) {
    if network.starts_with("ws://") || network.starts_with("wss://") {
        // It's a URL, determine network name from it
        let network_name = network_map()
            .iter()
            .find(|(_, v)| v == &network)
            .map(|(k, _)| k.to_string())
            .unwrap_or_else(|| "custom".to_string());
        (network.to_string(), network_name)
    } else {
        // It's a network name
        let endpoint = network_map()
            .get(network)
            .cloned()
            .unwrap_or_else(|| FINNEY_ENTRYPOINT.to_string());
        (endpoint, network.to_string())
    }
}

/// Default configuration values
pub struct Defaults {
    pub axon: AxonDefaults,
    pub subtensor: SubtensorDefaults,
    pub wallet: WalletDefaults,
    pub logging: LoggingDefaults,
}

pub struct AxonDefaults {
    pub port: u16,
    pub ip: String,
    pub external_port: Option<u16>,
    pub external_ip: Option<String>,
    pub max_workers: usize,
}

pub struct SubtensorDefaults {
    pub chain_endpoint: String,
    pub network: String,
}

pub struct WalletDefaults {
    pub name: String,
    pub hotkey: String,
    pub path: PathBuf,
}

pub struct LoggingDefaults {
    pub debug: bool,
    pub trace: bool,
    pub info: bool,
}

impl Default for Defaults {
    fn default() -> Self {
        Self {
            axon: AxonDefaults {
                port: env::var("BT_AXON_PORT")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(8091),
                ip: env::var("BT_AXON_IP").unwrap_or_else(|_| "[::]".to_string()),
                external_port: env::var("BT_AXON_EXTERNAL_PORT")
                    .ok()
                    .and_then(|s| s.parse().ok()),
                external_ip: env::var("BT_AXON_EXTERNAL_IP").ok(),
                max_workers: env::var("BT_AXON_MAX_WORKERS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(10),
            },
            subtensor: SubtensorDefaults {
                chain_endpoint: env::var("BT_SUBTENSOR_CHAIN_ENDPOINT")
                    .unwrap_or_else(|_| FINNEY_ENTRYPOINT.to_string()),
                network: env::var("BT_SUBTENSOR_NETWORK")
                    .unwrap_or_else(|_| DEFAULT_NETWORK.to_string()),
            },
            wallet: WalletDefaults {
                name: env::var("BT_WALLET_NAME").unwrap_or_else(|_| "default".to_string()),
                hotkey: env::var("BT_WALLET_HOTKEY").unwrap_or_else(|_| "default".to_string()),
                path: env::var("BT_WALLET_PATH")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| {
                        dirs::home_dir()
                            .unwrap_or_default()
                            .join(".bittensor")
                            .join("wallets")
                    }),
            },
            logging: LoggingDefaults {
                debug: env::var("BT_LOGGING_DEBUG")
                    .map(|v| v == "1" || v.to_lowercase() == "true")
                    .unwrap_or(false),
                trace: env::var("BT_LOGGING_TRACE")
                    .map(|v| v == "1" || v.to_lowercase() == "true")
                    .unwrap_or(false),
                info: env::var("BT_LOGGING_INFO")
                    .map(|v| v == "1" || v.to_lowercase() == "true")
                    .unwrap_or(false),
            },
        }
    }
}

lazy_static::lazy_static! {
    /// Global defaults
    pub static ref DEFAULTS: Defaults = Defaults::default();
}

/// Configuration struct for Subtensor client
#[derive(Debug, Clone)]
pub struct Config {
    pub network: String,
    pub chain_endpoint: String,
}

impl Config {
    pub fn new(network: &str) -> Self {
        let (endpoint, network_name) = determine_chain_endpoint_and_network(network);
        Self {
            network: network_name,
            chain_endpoint: endpoint,
        }
    }

    pub fn with_endpoint(endpoint: &str) -> Self {
        let (endpoint, network_name) = determine_chain_endpoint_and_network(endpoint);
        Self {
            network: network_name,
            chain_endpoint: endpoint,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new(DEFAULT_NETWORK)
    }
}
