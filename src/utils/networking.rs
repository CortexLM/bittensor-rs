//! Networking utilities

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Convert IP integer to string
pub fn int_to_ip(ip_int: u128, ip_type: u8) -> String {
    match ip_type {
        4 => {
            let ip = (ip_int as u32).to_be_bytes();
            Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3]).to_string()
        }
        6 => {
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

/// Get IP type (4 or 6) from address string
pub fn get_ip_type(ip: &str) -> u8 {
    if ip.parse::<Ipv4Addr>().is_ok() {
        4
    } else if ip.parse::<Ipv6Addr>().is_ok() {
        6
    } else {
        4 // default
    }
}

/// Format IP with port
pub fn ip_str(ip: &str, port: u16, ip_type: u8) -> String {
    match ip_type {
        6 => format!("[{}]:{}", ip, port),
        _ => format!("{}:{}", ip, port),
    }
}

/// Format WebSocket endpoint URL
pub fn get_formatted_ws_endpoint_url(endpoint: &str) -> String {
    let endpoint = endpoint.trim();
    
    // Already has protocol
    if endpoint.starts_with("ws://") || endpoint.starts_with("wss://") {
        return endpoint.to_string();
    }
    
    // Add appropriate protocol
    if endpoint.contains(":443") || endpoint.contains("finney") || endpoint.contains("opentensor") {
        format!("wss://{}", endpoint)
    } else {
        format!("ws://{}", endpoint)
    }
}

/// Check if endpoint is local
pub fn is_local_endpoint(endpoint: &str) -> bool {
    endpoint.contains("127.0.0.1")
        || endpoint.contains("localhost")
        || endpoint.contains("0.0.0.0")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ip_conversion() {
        let ip_int = 0x7F000001u128; // 127.0.0.1
        let ip_str = int_to_ip(ip_int, 4);
        assert_eq!(ip_str, "127.0.0.1");

        let back = ip_to_int(&ip_str).unwrap();
        assert_eq!(back, ip_int);
    }

    #[test]
    fn test_ws_endpoint_format() {
        assert_eq!(
            get_formatted_ws_endpoint_url("127.0.0.1:9944"),
            "ws://127.0.0.1:9944"
        );
        assert_eq!(
            get_formatted_ws_endpoint_url("entrypoint-finney.opentensor.ai:443"),
            "wss://entrypoint-finney.opentensor.ai:443"
        );
        assert_eq!(
            get_formatted_ws_endpoint_url("wss://example.com"),
            "wss://example.com"
        );
    }
}
