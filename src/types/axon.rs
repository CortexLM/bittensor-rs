use serde::{Deserialize, Serialize};
use std::net::IpAddr;

/// Complete axon information for a neuron endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxonInfo {
    /// Version of the axon
    pub version: u64,
    /// IP address (can be IPv4 or IPv6)
    pub ip: IpAddr,
    /// Port number
    pub port: u16,
    /// IP protocol version (4 or 6)
    pub ip_type: u8,
    /// Protocol identifier
    pub protocol: u8,
    /// Reserved field for future use
    pub placeholder1: u64,
    /// Reserved field for future use
    pub placeholder2: u64,
}

impl AxonInfo {
    /// Create AxonInfo from chain data - all fields required
    pub fn from_chain_data(
        version: u64,
        ip: IpAddr,
        port: u16,
        ip_type: u8,
        protocol: u8,
        placeholder1: u64,
        placeholder2: u64,
    ) -> Self {
        Self {
            version,
            ip,
            port,
            ip_type,
            protocol,
            placeholder1,
            placeholder2,
        }
    }

    /// Check if the axon is serving (not 0.0.0.0)
    pub fn is_serving(&self) -> bool {
        match self.ip {
            IpAddr::V4(ipv4) => ipv4.octets() != [0, 0, 0, 0],
            IpAddr::V6(ipv6) => ipv6.segments() != [0, 0, 0, 0, 0, 0, 0, 0],
        }
    }

    /// Get endpoint string
    pub fn to_endpoint(&self) -> String {
        match self.ip {
            IpAddr::V4(ip) => format!("http://{}:{}", ip, self.port),
            IpAddr::V6(ip) => format!("http://[{}]:{}", ip, self.port),
        }
    }

    /// Get IP as string
    pub fn ip_str(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}
