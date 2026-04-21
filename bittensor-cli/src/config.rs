//! CLI configuration: loads settings from `~/.bittensor/config.yml` with
//! command-line overrides for network, wallet name, and wallet path.

use std::path::PathBuf;

use anyhow::{Context, Result};
use bittensor_core::config::NetworkConfig;
use serde::Deserialize;

/// On-disk config (optional — every field has a sensible default).
#[derive(Debug, Deserialize, Default)]
pub struct ConfigFile {
    pub network: Option<String>,
    pub wallet_name: Option<String>,
    pub wallet_path: Option<String>,
}

/// Resolved configuration after merging disk config + CLI flags.
#[derive(Debug, Clone)]
pub struct Config {
    pub network: NetworkConfig,
    pub wallet_name: String,
    pub wallet_path: PathBuf,
}

impl Config {
    /// Build a `Config` by:
    /// 1. Loading `~/.bittensor/config.yml` if present
    /// 2. Applying CLI `--network` flag
    /// 3. Applying CLI `--wallet.name` flag
    /// 4. Applying CLI `--wallet.path` flag
    pub fn resolve(
        network_flag: Option<&str>,
        wallet_name_flag: Option<&str>,
        wallet_path_flag: Option<&str>,
    ) -> Result<Self> {
        let disk = load_config_file()?;

        let network_name = network_flag.or(disk.network.as_deref()).unwrap_or("finney");
        let network = resolve_network(network_name)?;

        let wallet_name =
            wallet_name_flag.or(disk.wallet_name.as_deref()).unwrap_or("default").to_string();

        let wallet_path = wallet_path_flag
            .or(disk.wallet_path.as_deref())
            .map(PathBuf::from)
            .unwrap_or_else(default_wallet_base);

        Ok(Self { network, wallet_name, wallet_path })
    }

    /// Full path to the wallet directory: `<wallet_path>/<wallet_name>`.
    pub fn wallet_dir(&self) -> PathBuf {
        self.wallet_path.join(&self.wallet_name)
    }
}

fn resolve_network(name: &str) -> Result<NetworkConfig> {
    match name {
        "finney" | "mainnet" => Ok(NetworkConfig::finney()),
        "test" | "testnet" => Ok(NetworkConfig::test()),
        "local" => Ok(NetworkConfig::local()),
        "archive" => Ok(NetworkConfig::archive()),
        "latent-lite" => Ok(NetworkConfig::latent_lite()),
        other => Err(anyhow::anyhow!(
            "unknown network '{other}'; choose finney, test, local, archive, or latent-lite"
        )),
    }
}

fn default_wallet_base() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")).join(".bittensor").join("wallets")
}

fn config_file_path() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")).join(".bittensor").join("config.yml")
}

fn load_config_file() -> Result<ConfigFile> {
    let path = config_file_path();
    if !path.exists() {
        return Ok(ConfigFile::default());
    }
    let contents = std::fs::read_to_string(&path)
        .with_context(|| format!("reading config from {}", path.display()))?;
    let cfg: ConfigFile = serde_yaml::from_str(&contents)
        .with_context(|| format!("parsing config from {}", path.display()))?;
    Ok(cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_finney_network() {
        let cfg = Config::resolve(Some("finney"), None, None).unwrap();
        assert_eq!(cfg.network.name, "finney");
    }

    #[test]
    fn resolve_test_network() {
        let cfg = Config::resolve(Some("test"), None, None).unwrap();
        assert_eq!(cfg.network.name, "test");
    }

    #[test]
    fn resolve_local_network() {
        let cfg = Config::resolve(Some("local"), None, None).unwrap();
        assert_eq!(cfg.network.name, "local");
    }

    #[test]
    fn resolve_mainnet_alias() {
        let cfg = Config::resolve(Some("mainnet"), None, None).unwrap();
        assert_eq!(cfg.network.name, "finney");
    }

    #[test]
    fn resolve_testnet_alias() {
        let cfg = Config::resolve(Some("testnet"), None, None).unwrap();
        assert_eq!(cfg.network.name, "test");
    }

    #[test]
    fn resolve_unknown_network_fails() {
        assert!(Config::resolve(Some("invalid"), None, None).is_err());
    }

    #[test]
    fn default_wallet_name() {
        let cfg = Config::resolve(None, None, None).unwrap();
        assert_eq!(cfg.wallet_name, "default");
    }

    #[test]
    fn wallet_name_override() {
        let cfg = Config::resolve(None, Some("my-wallet"), None).unwrap();
        assert_eq!(cfg.wallet_name, "my-wallet");
    }

    #[test]
    fn wallet_path_override() {
        let cfg = Config::resolve(None, None, Some("/tmp/wallets")).unwrap();
        assert_eq!(cfg.wallet_path, PathBuf::from("/tmp/wallets"));
    }

    #[test]
    fn wallet_dir_combines_path_and_name() {
        let cfg = Config::resolve(None, Some("test-wallet"), Some("/tmp/wallets")).unwrap();
        assert_eq!(cfg.wallet_dir(), PathBuf::from("/tmp/wallets/test-wallet"));
    }
}
