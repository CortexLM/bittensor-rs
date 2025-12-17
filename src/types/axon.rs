//! Axon information type
//!
//! The AxonInfo struct represents information about an axon endpoint
//! in the Bittensor network, including IP address, ports, and keys.

use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use super::{IP_TYPE_V4, IP_TYPE_V6};

/// Information about an axon endpoint in the Bittensor network
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AxonInfo {
    /// Protocol version
    pub version: u32,
    /// IP address as string
    pub ip: String,
    /// Port number
    pub port: u16,
    /// IP type (4 for IPv4, 6 for IPv6)
    pub ip_type: u8,
    /// Hotkey SS58 address
    pub hotkey: String,
    /// Coldkey SS58 address
    pub coldkey: String,
    /// Protocol version (default 4)
    pub protocol: u8,
    /// Reserved field
    pub placeholder1: u8,
    /// Reserved field
    pub placeholder2: u8,
}

impl AxonInfo {
    /// Create a new AxonInfo with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create AxonInfo from components
    pub fn from_parts(
        version: u32,
        ip: String,
        port: u16,
        ip_type: u8,
        hotkey: String,
        coldkey: String,
    ) -> Self {
        Self {
            version,
            ip,
            port,
            ip_type,
            hotkey,
            coldkey,
            protocol: 4,
            placeholder1: 0,
            placeholder2: 0,
        }
    }

    /// Check if the axon is serving (has a valid IP)
    pub fn is_serving(&self) -> bool {
        self.ip != "0.0.0.0" && !self.ip.is_empty()
    }

    /// Get the full IP string with port
    pub fn ip_str(&self) -> String {
        match self.ip_type {
            IP_TYPE_V4 => format!("{}:{}", self.ip, self.port),
            IP_TYPE_V6 => format!("[{}]:{}", self.ip, self.port),
            _ => format!("{}:{}", self.ip, self.port),
        }
    }

    /// Parse IP from integer representation
    pub fn ip_from_int(ip_int: u128, ip_type: u8) -> String {
        match ip_type {
            IP_TYPE_V4 => {
                let ip = (ip_int as u32).to_be_bytes();
                Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3]).to_string()
            }
            IP_TYPE_V6 => {
                let bytes = ip_int.to_be_bytes();
                Ipv6Addr::from(bytes).to_string()
            }
            _ => "0.0.0.0".to_string(),
        }
    }

    /// Convert IP string to integer
    pub fn ip_to_int(ip: &str) -> Option<u128> {
        if let Ok(addr) = ip.parse::<IpAddr>() {
            match addr {
                IpAddr::V4(v4) => Some(u32::from_be_bytes(v4.octets()) as u128),
                IpAddr::V6(v6) => Some(u128::from_be_bytes(v6.octets())),
            }
        } else {
            None
        }
    }

    /// Create from chain data dictionary
    pub fn from_chain_data(
        data: &serde_json::Value,
        hotkey: &str,
        coldkey: &str,
    ) -> Option<Self> {
        Some(Self {
            version: data.get("version")?.as_u64()? as u32,
            ip: Self::ip_from_int(
                data.get("ip")?.as_u64()? as u128,
                data.get("ip_type")?.as_u64()? as u8,
            ),
            port: data.get("port")?.as_u64()? as u16,
            ip_type: data.get("ip_type")?.as_u64()? as u8,
            hotkey: hotkey.to_string(),
            coldkey: coldkey.to_string(),
            protocol: data.get("protocol").and_then(|v| v.as_u64()).unwrap_or(4) as u8,
            placeholder1: data
                .get("placeholder1")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u8,
            placeholder2: data
                .get("placeholder2")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u8,
        })
    }

    /// Convert to JSON string
    pub fn to_json_string(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// Parse from JSON string
    pub fn from_json_string(json: &str) -> Option<Self> {
        serde_json::from_str(json).ok()
    }
}

impl std::fmt::Display for AxonInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AxonInfo( {}, {}, {}, {} )",
            self.ip_str(),
            self.hotkey,
            self.coldkey,
            self.version
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ip_conversion() {
        // IPv4
        let ip_int: u128 = 0x7F000001; // 127.0.0.1
        let ip_str = AxonInfo::ip_from_int(ip_int, IP_TYPE_V4);
        assert_eq!(ip_str, "127.0.0.1");

        // Round trip
        let back = AxonInfo::ip_to_int(&ip_str).unwrap();
        assert_eq!(back, ip_int);
    }

    #[test]
    fn test_is_serving() {
        let mut axon = AxonInfo::default();
        assert!(!axon.is_serving());

        axon.ip = "192.168.1.1".to_string();
        assert!(axon.is_serving());

        axon.ip = "0.0.0.0".to_string();
        assert!(!axon.is_serving());
    }
}
