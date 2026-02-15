use serde::{Deserialize, Serialize};
use std::net::IpAddr;

/// Complete axon information stored on-chain for a neuron endpoint
/// Includes optional hotkey metadata when available.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxonInfo {
    /// Hotkey SS58 address (optional when read from chain)
    pub hotkey: Option<String>,
    /// Axon serving block
    pub block: u64,
    /// Axon version
    pub version: u32,
    /// IP address (can be IPv4 or IPv6)
    pub ip: IpAddr,
    /// Port number
    pub port: u16,
    /// IP type: 4 for IPv4, 6 for IPv6
    pub ip_type: u8,
    /// Protocol: TCP, UDP, other
    pub protocol: u8,
    /// Reserved field for future use
    pub placeholder1: u8,
    /// Reserved field for future use
    pub placeholder2: u8,
}

impl AxonInfo {
    /// Create AxonInfo from chain data - all fields required
    #[allow(clippy::too_many_arguments)]
    pub fn from_chain_data(
        block: u64,
        version: u32,
        ip: IpAddr,
        port: u16,
        ip_type: u8,
        protocol: u8,
        placeholder1: u8,
        placeholder2: u8,
    ) -> Self {
        Self {
            hotkey: None,
            block,
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
