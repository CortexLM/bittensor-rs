//! Axon configuration and info types
//!
//! This module provides configuration structures for the Axon HTTP server
//! and re-exports the on-chain AxonInfo type.

use serde::{Deserialize, Serialize};

/// Re-export the on-chain AxonInfo type
pub use crate::types::axon::AxonInfo;

/// Axon server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxonConfig {
    /// The port to listen on
    pub port: u16,
    /// The IP address to bind to (default: "0.0.0.0")
    pub ip: String,
    /// External IP address for registration (if different from binding IP)
    pub external_ip: Option<String>,
    /// External port for registration (if different from listening port)
    pub external_port: Option<u16>,
    /// Maximum number of worker threads
    pub max_workers: usize,
    /// Maximum concurrent requests to process
    pub max_concurrent_requests: usize,
    /// Default request timeout in seconds
    pub default_timeout_secs: u64,
    /// Whether to verify request signatures
    pub verify_signatures: bool,
    /// Whether to trust X-Forwarded-For and X-Real-IP headers.
    /// Only enable this when running behind a trusted reverse proxy.
    /// When disabled (default), only the direct connection IP is used for IP blacklisting.
    pub trust_proxy_headers: bool,
}

impl Default for AxonConfig {
    fn default() -> Self {
        Self {
            port: 8091,
            ip: "0.0.0.0".to_string(),
            external_ip: None,
            external_port: None,
            max_workers: 10,
            max_concurrent_requests: 256,
            default_timeout_secs: 12,
            verify_signatures: true,
            trust_proxy_headers: false,
        }
    }
}

impl AxonConfig {
    /// Create a new AxonConfig with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the port
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Set the binding IP address
    pub fn with_ip(mut self, ip: impl Into<String>) -> Self {
        self.ip = ip.into();
        self
    }

    /// Set the external IP for chain registration
    pub fn with_external_ip(mut self, ip: impl Into<String>) -> Self {
        self.external_ip = Some(ip.into());
        self
    }

    /// Set the external port for chain registration
    pub fn with_external_port(mut self, port: u16) -> Self {
        self.external_port = Some(port);
        self
    }

    /// Set the maximum worker threads
    pub fn with_max_workers(mut self, workers: usize) -> Self {
        self.max_workers = workers;
        self
    }

    /// Set the maximum concurrent requests
    pub fn with_max_concurrent_requests(mut self, max: usize) -> Self {
        self.max_concurrent_requests = max;
        self
    }

    /// Set the default timeout
    pub fn with_default_timeout(mut self, timeout_secs: u64) -> Self {
        self.default_timeout_secs = timeout_secs;
        self
    }

    /// Enable or disable signature verification
    pub fn with_signature_verification(mut self, enabled: bool) -> Self {
        self.verify_signatures = enabled;
        self
    }

    /// Enable or disable trusting proxy headers (X-Forwarded-For, X-Real-IP).
    /// Only enable this when running behind a trusted reverse proxy.
    /// When disabled (default), only the direct connection IP is used.
    pub fn with_trust_proxy_headers(mut self, enabled: bool) -> Self {
        self.trust_proxy_headers = enabled;
        self
    }

    /// Get the socket address string for binding
    pub fn socket_addr(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }

    /// Get the external IP to use for chain registration
    pub fn get_external_ip(&self) -> &str {
        self.external_ip.as_deref().unwrap_or(&self.ip)
    }

    /// Get the external port to use for chain registration
    pub fn get_external_port(&self) -> u16 {
        self.external_port.unwrap_or(self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AxonConfig::default();
        assert_eq!(config.port, 8091);
        assert_eq!(config.ip, "0.0.0.0");
        assert_eq!(config.max_concurrent_requests, 256);
    }

    #[test]
    fn test_builder_pattern() {
        let config = AxonConfig::new()
            .with_port(9000)
            .with_ip("127.0.0.1")
            .with_external_ip("1.2.3.4")
            .with_external_port(9001)
            .with_max_workers(20);

        assert_eq!(config.port, 9000);
        assert_eq!(config.ip, "127.0.0.1");
        assert_eq!(config.external_ip, Some("1.2.3.4".to_string()));
        assert_eq!(config.external_port, Some(9001));
        assert_eq!(config.max_workers, 20);
    }

    #[test]
    fn test_socket_addr() {
        let config = AxonConfig::new().with_ip("192.168.1.1").with_port(8080);
        assert_eq!(config.socket_addr(), "192.168.1.1:8080");
    }

    #[test]
    fn test_external_ip_fallback() {
        let config = AxonConfig::new().with_ip("127.0.0.1");
        assert_eq!(config.get_external_ip(), "127.0.0.1");

        let config_with_external = config.with_external_ip("1.2.3.4");
        assert_eq!(config_with_external.get_external_ip(), "1.2.3.4");
    }
}
