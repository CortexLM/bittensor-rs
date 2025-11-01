use crate::chain::{BittensorClient, BittensorSigner, ExtrinsicWait};
use anyhow::Result;
use std::net::IpAddr;
use subxt::dynamic::Value;

const SUBTENSOR_MODULE: &str = "SubtensorModule";

/// Serve axon endpoint on the network
pub async fn serve_axon(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    ip: IpAddr,
    port: u16,
    protocol: u8,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let (ip_type, ip_bytes) = match ip {
        IpAddr::V4(ipv4) => (4u8, ipv4.octets().to_vec()),
        IpAddr::V6(ipv6) => (6u8, ipv6.octets().to_vec()),
    };
    
    // Convert IP bytes to u128 representation (or appropriate type)
    let ip_value = Value::from_bytes(&ip_bytes);
    
    let args = vec![
        Value::u128(netuid as u128),
        ip_value,
        Value::u128(port as u128),
        Value::u128(ip_type as u128),
        Value::u128(protocol as u128),
        Value::u128(0), // reserved field 1
        Value::u128(0), // reserved field 2
    ];
    
    client
        .submit_extrinsic(SUBTENSOR_MODULE, "serve_axon", args, signer, wait_for)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to serve axon: {}", e))
}

/// Serve axon with TLS certificate
pub async fn serve_axon_tls(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    ip: IpAddr,
    port: u16,
    protocol: u8,
    certificate: &[u8],
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let (ip_type, ip_bytes) = match ip {
        IpAddr::V4(ipv4) => (4u8, ipv4.octets().to_vec()),
        IpAddr::V6(ipv6) => (6u8, ipv6.octets().to_vec()),
    };
    
    let ip_value = Value::from_bytes(&ip_bytes);
    let cert_value = Value::from_bytes(certificate);
    
    let args = vec![
        Value::u128(netuid as u128),
        ip_value,
        Value::u128(port as u128),
        Value::u128(ip_type as u128),
        Value::u128(protocol as u128),
        Value::u128(0), // reserved field 1
        Value::u128(0), // reserved field 2
        cert_value,
    ];
    
    client
        .submit_extrinsic(SUBTENSOR_MODULE, "serve_axon_tls", args, signer, wait_for)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to serve axon with TLS: {}", e))
}

