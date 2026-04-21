//! Configuration for the Dendrite HTTP client.

use subxt_signer::sr25519::Keypair;

/// Configuration for constructing a [`Dendrite`](crate::dendrite::Dendrite) instance.
#[derive(Debug, Clone)]
pub struct DendriteConfig {
    /// Request timeout in seconds. Default: 12.
    pub timeout_secs: u64,
    /// Maximum number of idle connections per host in the connection pool. Default: 100.
    pub max_connections: usize,
    /// Optional Sr25519 keypair used for request signing.
    /// If `None`, requests are sent without `bt-signature` / `bt-dendrite-hotkey` headers.
    pub hotkey: Option<Keypair>,
}

impl Default for DendriteConfig {
    fn default() -> Self {
        Self { timeout_secs: 12, max_connections: 100, hotkey: None }
    }
}

impl DendriteConfig {
    /// Create a new config with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the request timeout in seconds.
    pub fn with_timeout_secs(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    /// Set the maximum number of idle connections per host.
    pub fn with_max_connections(mut self, max: usize) -> Self {
        self.max_connections = max;
        self
    }

    /// Set the hotkey keypair for request signing.
    pub fn with_hotkey(mut self, keypair: Keypair) -> Self {
        self.hotkey = Some(keypair);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = DendriteConfig::default();
        assert_eq!(cfg.timeout_secs, 12);
        assert_eq!(cfg.max_connections, 100);
        assert!(cfg.hotkey.is_none());
    }

    #[test]
    fn builder_pattern() {
        let keypair = subxt_signer::sr25519::dev::alice();
        let cfg = DendriteConfig::new()
            .with_timeout_secs(30)
            .with_max_connections(50)
            .with_hotkey(keypair);
        assert_eq!(cfg.timeout_secs, 30);
        assert_eq!(cfg.max_connections, 50);
        assert!(cfg.hotkey.is_some());
    }
}
