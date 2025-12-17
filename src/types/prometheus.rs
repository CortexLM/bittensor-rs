//! Prometheus information type

use serde::{Deserialize, Serialize};

use super::AxonInfo;

/// Prometheus metrics endpoint information
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct PrometheusInfo {
    /// Block number when registered
    pub block: u64,
    /// Protocol version
    pub version: u32,
    /// IP address
    pub ip: String,
    /// Port number
    pub port: u16,
    /// IP type (4 or 6)
    pub ip_type: u8,
}

impl PrometheusInfo {
    /// Create a new PrometheusInfo
    pub fn new(block: u64, version: u32, ip: String, port: u16, ip_type: u8) -> Self {
        Self {
            block,
            version,
            ip,
            port,
            ip_type,
        }
    }

    /// Create from chain data
    pub fn from_chain_data(data: &serde_json::Value) -> Option<Self> {
        Some(Self {
            block: data.get("block")?.as_u64()?,
            version: data.get("version")?.as_u64()? as u32,
            ip: AxonInfo::ip_from_int(
                data.get("ip")?.as_u64()? as u128,
                data.get("ip_type")?.as_u64()? as u8,
            ),
            port: data.get("port")?.as_u64()? as u16,
            ip_type: data.get("ip_type")?.as_u64()? as u8,
        })
    }

    /// Get the full IP string with port
    pub fn ip_str(&self) -> String {
        match self.ip_type {
            4 => format!("{}:{}", self.ip, self.port),
            6 => format!("[{}]:{}", self.ip, self.port),
            _ => format!("{}:{}", self.ip, self.port),
        }
    }
}

impl std::fmt::Display for PrometheusInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PrometheusInfo( {}, version={}, block={} )",
            self.ip_str(),
            self.version,
            self.block
        )
    }
}
