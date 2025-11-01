use crate::chain::{BittensorClient, BittensorSigner, ExtrinsicWait};
use anyhow::Result;
use std::net::IpAddr;
use subxt::dynamic::Value;

const SUBTENSOR_MODULE: &str = "SubtensorModule";

/// Serve axon endpoint on the network
/// Subtensor expects: (netuid, version: u32, ip: u128, port: u16, ip_type: u8, protocol: u8, placeholder1: u8, placeholder2: u8)
pub async fn serve_axon(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    version: u32,
    ip: IpAddr,
    port: u16,
    protocol: u8,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let (ip_type, ip_u128) = match ip {
        IpAddr::V4(ipv4) => {
            // IPv4: convert 4 bytes to u128 (stored as u32 in u128)
            let bytes = ipv4.octets();
            let ip_val = u32::from_be_bytes(bytes) as u128;
            (4u8, ip_val)
        }
        IpAddr::V6(ipv6) => {
            // IPv6: convert 16 bytes to u128 (direct encoding)
            let segments = ipv6.segments();
            let ip_val = ((segments[0] as u128) << 112)
                | ((segments[1] as u128) << 96)
                | ((segments[2] as u128) << 80)
                | ((segments[3] as u128) << 64)
                | ((segments[4] as u128) << 48)
                | ((segments[5] as u128) << 32)
                | ((segments[6] as u128) << 16)
                | (segments[7] as u128);
            (6u8, ip_val)
        }
    };

    let args = vec![
        Value::u128(netuid as u128),
        Value::u128(version as u128),
        Value::u128(ip_u128),
        Value::u128(port as u128),
        Value::u128(ip_type as u128),
        Value::u128(protocol as u128),
        Value::u128(0), // placeholder1
        Value::u128(0), // placeholder2
    ];

    client
        .submit_extrinsic(SUBTENSOR_MODULE, "serve_axon", args, signer, wait_for)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to serve axon: {}", e))
}

/// Serve axon with TLS certificate
/// Subtensor expects: (netuid, version: u32, ip: u128, port: u16, ip_type: u8, protocol: u8, placeholder1: u8, placeholder2: u8, certificate: Vec<u8>)
pub async fn serve_axon_tls(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    version: u32,
    ip: IpAddr,
    port: u16,
    protocol: u8,
    certificate: &[u8],
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let (ip_type, ip_u128) = match ip {
        IpAddr::V4(ipv4) => {
            // IPv4: convert 4 bytes to u128 (stored as u32 in u128)
            let bytes = ipv4.octets();
            let ip_val = u32::from_be_bytes(bytes) as u128;
            (4u8, ip_val)
        }
        IpAddr::V6(ipv6) => {
            // IPv6: convert 16 bytes to u128 (direct encoding)
            let segments = ipv6.segments();
            let ip_val = ((segments[0] as u128) << 112)
                | ((segments[1] as u128) << 96)
                | ((segments[2] as u128) << 80)
                | ((segments[3] as u128) << 64)
                | ((segments[4] as u128) << 48)
                | ((segments[5] as u128) << 32)
                | ((segments[6] as u128) << 16)
                | (segments[7] as u128);
            (6u8, ip_val)
        }
    };

    let cert_value = Value::from_bytes(certificate);

    let args = vec![
        Value::u128(netuid as u128),
        Value::u128(version as u128),
        Value::u128(ip_u128),
        Value::u128(port as u128),
        Value::u128(ip_type as u128),
        Value::u128(protocol as u128),
        Value::u128(0), // placeholder1
        Value::u128(0), // placeholder2
        cert_value,
    ];

    client
        .submit_extrinsic(SUBTENSOR_MODULE, "serve_axon_tls", args, signer, wait_for)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to serve axon with TLS: {}", e))
}
