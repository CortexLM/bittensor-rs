use super::utils;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use subxt::dynamic::Value;

/// Decode PrometheusInfo from Value
/// Subtensor PrometheusInfo: { block: u64, version: u32, ip: u128, port: u16, ip_type: u8 }
pub fn decode_prometheus_info(value: &Value) -> Result<crate::types::PrometheusInfo> {
    let s = format!("{:?}", value);

    // Extract exactly: U64, U32, U128, U16, U8
    let block =
        utils::extract_u64(&s, 0).ok_or_else(|| anyhow!("PrometheusInfo: missing block (u64)"))?;
    let version = utils::extract_u32(&s, block.1)
        .ok_or_else(|| anyhow!("PrometheusInfo: missing version (u32)"))?;
    let ip_u128 = utils::extract_u128(&s, version.1)
        .ok_or_else(|| anyhow!("PrometheusInfo: missing ip (u128)"))?;
    let port = utils::extract_u16(&s, ip_u128.1)
        .ok_or_else(|| anyhow!("PrometheusInfo: missing port (u16)"))?;
    let ip_type = utils::extract_u8(&s, port.1)
        .ok_or_else(|| anyhow!("PrometheusInfo: missing ip_type (u8)"))?;

    let ip = utils::parse_ip_addr(ip_u128.0, ip_type.0);

    Ok(crate::types::PrometheusInfo::from_chain_data(
        block.0,
        version.0,
        ip.to_string(),
        port.0,
        ip_type.0,
    ))
}

/// Decode AxonInfo from a Value
/// Subtensor AxonInfo: { block: u64, version: u32, ip: u128, port: u16, ip_type: u8, protocol: u8, placeholder1: u8, placeholder2: u8 }
pub fn decode_axon_info(value: &Value) -> Result<crate::types::AxonInfo> {
    let s = format!("{:?}", value);

    // Extract exactly: U64, U32, U128, U16, U8, U8, U8, U8
    let block =
        utils::extract_u64(&s, 0).ok_or_else(|| anyhow!("AxonInfo: missing block (u64)"))?;
    let version = utils::extract_u32(&s, block.1)
        .ok_or_else(|| anyhow!("AxonInfo: missing version (u32)"))?;
    let ip_u128 =
        utils::extract_u128(&s, version.1).ok_or_else(|| anyhow!("AxonInfo: missing ip (u128)"))?;
    let port =
        utils::extract_u16(&s, ip_u128.1).ok_or_else(|| anyhow!("AxonInfo: missing port (u16)"))?;
    let ip_type =
        utils::extract_u8(&s, port.1).ok_or_else(|| anyhow!("AxonInfo: missing ip_type (u8)"))?;
    let protocol = utils::extract_u8(&s, ip_type.1)
        .ok_or_else(|| anyhow!("AxonInfo: missing protocol (u8)"))?;
    let placeholder1 = utils::extract_u8(&s, protocol.1)
        .ok_or_else(|| anyhow!("AxonInfo: missing placeholder1 (u8)"))?;
    let placeholder2 = utils::extract_u8(&s, placeholder1.1)
        .ok_or_else(|| anyhow!("AxonInfo: missing placeholder2 (u8)"))?;

    let ip = utils::parse_ip_addr(ip_u128.0, ip_type.0);

    Ok(crate::types::AxonInfo::from_chain_data(
        block.0,
        version.0,
        ip,
        port.0,
        ip_type.0,
        protocol.0,
        placeholder1.0,
        placeholder2.0,
    ))
}

/// Helper to decode identity data from a map structure
/// TODO: Implement actual field extraction
pub fn decode_identity_map(_value: &Value) -> Result<HashMap<String, String>> {
    Ok(HashMap::new())
}

/// Decode a named composite (struct) from a Value
/// Returns empty HashMap if value is not a named composite
/// TODO: Implement actual field extraction
pub fn decode_named_composite(_value: &Value) -> Result<HashMap<String, Value>> {
    Ok(HashMap::new())
}
