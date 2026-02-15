//! Associated IP address queries
//!
//! This module provides functions to query IP addresses associated with hotkeys
//! on the Bittensor network.

use crate::chain::BittensorClient;
use crate::errors::{BittensorError, BittensorResult, ChainQueryError};
use crate::utils::decoders::primitive;
use crate::utils::decoders::utils::parse_ip_addr;
use crate::utils::decoders::vec::decode_vec;
use anyhow::anyhow;
use parity_scale_codec::Encode;
use sp_core::crypto::AccountId32;
use std::net::IpAddr;
use subxt::dynamic::Value;
use subxt::ext::scale_value::{Composite, ValueDef};

const SUBTENSOR_MODULE: &str = "SubtensorModule";

/// IP information associated with a hotkey
#[derive(Debug, Clone)]
pub struct IpInfo {
    /// The IP address
    pub ip: IpAddr,
    /// IP type: 4 for IPv4, 6 for IPv6
    pub ip_type: u8,
    /// Protocol identifier
    pub protocol: u8,
}

impl IpInfo {
    /// Create a new IpInfo instance
    pub fn new(ip: IpAddr, ip_type: u8, protocol: u8) -> Self {
        Self {
            ip,
            ip_type,
            protocol,
        }
    }

    /// Create IpInfo from raw chain data
    pub fn from_chain_data(ip_u128: u128, ip_type: u8, protocol: u8) -> Self {
        let ip = parse_ip_addr(ip_u128, ip_type);
        Self {
            ip,
            ip_type,
            protocol,
        }
    }

    /// Check if this is an IPv4 address
    pub fn is_ipv4(&self) -> bool {
        self.ip_type == 4
    }

    /// Check if this is an IPv6 address
    pub fn is_ipv6(&self) -> bool {
        self.ip_type == 6
    }
}

/// Decode IpInfo from a Value
/// Chain stores: { ip: u128, ip_type: u8, protocol: u8 }
#[allow(dead_code)]
fn decode_ip_info(value: &Value) -> Option<IpInfo> {
    let values = match &value.value {
        ValueDef::Composite(Composite::Named(fields)) => fields.iter().map(|(_, v)| v).collect(),
        ValueDef::Composite(Composite::Unnamed(vals)) => vals.iter().collect(),
        ValueDef::Variant(variant) => match &variant.values {
            Composite::Named(fields) => fields.iter().map(|(_, v)| v).collect(),
            Composite::Unnamed(vals) => vals.iter().collect(),
        },
        _ => Vec::new(),
    };
    if values.len() < 3 {
        return None;
    }
    let ip_u128 = primitive::extract_u128(values[0])?;
    let ip_type = primitive::extract_u128(values[1])? as u8;
    let protocol = primitive::extract_u128(values[2])? as u8;
    Some(IpInfo::from_chain_data(ip_u128, ip_type, protocol))
}

/// Get associated IPs for a hotkey
///
/// Queries the AssociatedIps storage map which stores a list of IP addresses
/// associated with a given hotkey.
///
/// # Arguments
/// * `client` - The Bittensor client
/// * `hotkey` - The hotkey account to query IPs for
///
/// # Returns
/// A vector of IpInfo structures containing the associated IP addresses
pub async fn get_associated_ips(
    client: &BittensorClient,
    hotkey: &AccountId32,
) -> BittensorResult<Vec<IpInfo>> {
    let keys = vec![Value::from_bytes(hotkey.encode())];

    match client
        .storage_with_keys(SUBTENSOR_MODULE, "AssociatedIps", keys)
        .await
    {
        Ok(Some(val)) => {
            let ips = decode_ip_info_vec(&val);
            Ok(ips)
        }
        Ok(None) => Ok(Vec::new()),
        Err(e) => Err(BittensorError::ChainQuery(ChainQueryError::with_storage(
            format!("Failed to query AssociatedIps: {}", e),
            SUBTENSOR_MODULE,
            "AssociatedIps",
        ))),
    }
}

/// Decode a vector of IpInfo from a Value
fn decode_ip_info_vec(value: &Value) -> Vec<IpInfo> {
    decode_vec(value, |entry| {
        decode_ip_info(entry).ok_or_else(|| anyhow!("invalid ip info"))
    })
    .unwrap_or_default()
}

/// Get the number of associated IPs for a hotkey
///
/// # Arguments
/// * `client` - The Bittensor client
/// * `hotkey` - The hotkey account to query
///
/// # Returns
/// The count of associated IP addresses
pub async fn get_associated_ip_count(
    client: &BittensorClient,
    hotkey: &AccountId32,
) -> BittensorResult<usize> {
    let ips = get_associated_ips(client, hotkey).await?;
    Ok(ips.len())
}

/// Check if a hotkey has any associated IPs
///
/// # Arguments
/// * `client` - The Bittensor client
/// * `hotkey` - The hotkey account to check
///
/// # Returns
/// true if the hotkey has at least one associated IP
pub async fn has_associated_ips(
    client: &BittensorClient,
    hotkey: &AccountId32,
) -> BittensorResult<bool> {
    let ips = get_associated_ips(client, hotkey).await?;
    Ok(!ips.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[test]
    fn test_ip_info_from_chain_data_ipv4() {
        // IPv4 address 192.168.1.1 in u128 representation
        let ip_u128: u128 = (192u128 << 24) | (168u128 << 16) | (1u128 << 8) | 1u128;
        let ip_info = IpInfo::from_chain_data(ip_u128, 4, 0);

        assert!(ip_info.is_ipv4());
        assert!(!ip_info.is_ipv6());
        assert_eq!(ip_info.ip_type, 4);
        assert_eq!(ip_info.protocol, 0);
        assert!(matches!(ip_info.ip, IpAddr::V4(_)));
    }

    #[test]
    fn test_ip_info_from_chain_data_ipv6() {
        // IPv6 ::1 in u128
        let ip_u128: u128 = 1;
        let ip_info = IpInfo::from_chain_data(ip_u128, 6, 0);

        assert!(!ip_info.is_ipv4());
        assert!(ip_info.is_ipv6());
        assert_eq!(ip_info.ip_type, 6);
        assert_eq!(ip_info.protocol, 0);
        assert!(matches!(ip_info.ip, IpAddr::V6(_)));
    }

    #[test]
    fn test_ip_info_new() {
        let ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
        let ip_info = IpInfo::new(ip, 4, 1);

        assert_eq!(ip_info.ip, IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)));
        assert_eq!(ip_info.ip_type, 4);
        assert_eq!(ip_info.protocol, 1);
    }

    #[test]
    fn test_ip_info_clone() {
        let ip_info = IpInfo::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 4, 0);
        let cloned = ip_info.clone();

        assert_eq!(cloned.ip, ip_info.ip);
        assert_eq!(cloned.ip_type, ip_info.ip_type);
        assert_eq!(cloned.protocol, ip_info.protocol);
    }

    #[test]
    fn test_ip_info_debug() {
        let ip_info = IpInfo::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 6, 0);
        let debug_str = format!("{:?}", ip_info);

        assert!(debug_str.contains("IpInfo"));
        assert!(debug_str.contains("::1"));
    }
}
