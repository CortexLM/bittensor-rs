use serde::{Deserialize, Serialize};

/// Prometheus information for a neuron
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrometheusInfo {
    /// Block number when prometheus info was set
    pub block: u64,
    /// Version of prometheus info
    pub version: u64,
    /// IP address as string
    pub ip: String,
    /// Port number
    pub port: u16,
    /// IP type (4 for IPv4, 6 for IPv6)
    pub ip_type: u8,
}

impl PrometheusInfo {
    /// Create PrometheusInfo from chain data - all fields required
    pub fn from_chain_data(
        block: u64,
        version: u64,
        ip: String,
        port: u16,
        ip_type: u8,
    ) -> Self {
        Self {
            block,
            version,
            ip,
            port,
            ip_type,
        }
    }
    
    /// Check if Prometheus is serving (not 0.0.0.0)
    pub fn is_serving(&self) -> bool {
        self.ip != "0.0.0.0" && self.ip != "0:0:0:0:0:0:0:0" && self.port != 0
    }
}

