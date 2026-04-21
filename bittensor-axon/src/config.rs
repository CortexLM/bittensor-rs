//! Configuration for the Axon server.

use serde::{Deserialize, Serialize};

/// Configuration for an Axon HTTP server.
///
/// Mirrors the Python SDK's `Axon` constructor arguments:
/// ip, port, max_connections, external_ip, hotkey.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxonConfig {
    /// Bind address. Defaults to `"0.0.0.0"`.
    pub ip: String,
    /// Listen port. Defaults to `8090`.
    pub port: u16,
    /// Maximum number of concurrent connections. Defaults to `0` (unlimited).
    pub max_connections: usize,
    /// External IP advertised to the network. If `None`, `ip` is used.
    pub external_ip: Option<String>,
    /// Hotkey identity of this axon. If `None`, verification middleware
    /// will accept any signature.
    pub hotkey: Option<String>,
}

impl Default for AxonConfig {
    fn default() -> Self {
        Self {
            ip: "0.0.0.0".to_string(),
            port: 8090,
            max_connections: 0,
            external_ip: None,
            hotkey: None,
        }
    }
}

impl AxonConfig {
    /// Creates a new config with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the address string `ip:port`.
    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }

    /// Returns the external IP, falling back to `ip`.
    pub fn external_ip_or_ip(&self) -> &str {
        self.external_ip.as_deref().unwrap_or(&self.ip)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = AxonConfig::default();
        assert_eq!(cfg.ip, "0.0.0.0");
        assert_eq!(cfg.port, 8090);
        assert_eq!(cfg.max_connections, 0);
        assert!(cfg.external_ip.is_none());
        assert!(cfg.hotkey.is_none());
    }

    #[test]
    fn bind_addr_format() {
        let cfg = AxonConfig { ip: "127.0.0.1".to_string(), port: 3000, ..Default::default() };
        assert_eq!(cfg.bind_addr(), "127.0.0.1:3000");
    }

    #[test]
    fn external_ip_falls_back() {
        let cfg = AxonConfig {
            ip: "10.0.0.1".to_string(),
            external_ip: Some("1.2.3.4".to_string()),
            ..Default::default()
        };
        assert_eq!(cfg.external_ip_or_ip(), "1.2.3.4");

        let cfg2 = AxonConfig { ip: "10.0.0.1".to_string(), ..Default::default() };
        assert_eq!(cfg2.external_ip_or_ip(), "10.0.0.1");
    }

    #[test]
    fn config_serialization_roundtrip() {
        let cfg = AxonConfig {
            ip: "192.168.1.1".to_string(),
            port: 8888,
            max_connections: 100,
            external_ip: Some("1.1.1.1".to_string()),
            hotkey: Some("5Hotkey".to_string()),
        };
        let json = serde_json::to_string(&cfg).expect("serialize");
        let back: AxonConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(cfg.ip, back.ip);
        assert_eq!(cfg.port, back.port);
        assert_eq!(cfg.max_connections, back.max_connections);
        assert_eq!(cfg.external_ip, back.external_ip);
        assert_eq!(cfg.hotkey, back.hotkey);
    }
}
