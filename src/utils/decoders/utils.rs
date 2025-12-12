use anyhow::{anyhow, Result};
use subxt::dynamic::Value;

/// Extract bytes from a composite Value (for Vec<u8> representations)
pub fn decode_bytes_from_composite(value: &Value) -> Result<Vec<u8>> {
    let s = format!("{:?}", value);

    // Look for Composite(Unnamed([...])) pattern
    if s.contains("Composite(Unnamed([") {
        let mut bytes = Vec::new();
        let mut remaining = &s[..];

        // Extract all U8/U128 values that represent bytes
        while let Some(pos) = remaining.find("U8(") {
            let after = &remaining[pos + 3..];
            if let Some(end) = after.find(')') {
                let num_str = &after[..end];
                if let Ok(num) = num_str.trim().parse::<u8>() {
                    bytes.push(num);
                }
            }
            remaining = &remaining[pos + 3..];
        }

        // Also try U128 values that might be bytes
        remaining = &s[..];
        while let Some(pos) = remaining.find("U128(") {
            let after = &remaining[pos + 5..];
            if let Some(end) = after.find(')') {
                let num_str = &after[..end];
                if let Ok(num) = num_str.trim().parse::<u128>() {
                    if num <= 255 {
                        bytes.push(num as u8);
                    }
                }
            }
            remaining = &remaining[pos + 5..];
        }

        if !bytes.is_empty() {
            Ok(bytes)
        } else {
            Err(anyhow!("No bytes found in composite"))
        }
    } else {
        Err(anyhow!("Not a composite value"))
    }
}

/// Handles multiple formats:
/// 1. Sequence of 32 U128/u8 bytes (composite with unnamed values)
/// 2. 0x-prefixed hex string (64 chars)
/// 3. Bytes array
pub fn extract_bytes_from_composite_sequence(value: &Value) -> Option<[u8; 32]> {
    let value_str = format!("{:?}", value);

    // Look for pattern: Composite(Unnamed([Value { value: Primitive(U128(XXX)), ... }, ...]))
    // Extract all U128 values that represent bytes
    let mut bytes = Vec::new();
    let mut remaining = &value_str[..];

    while let Some(pos) = remaining.find("U128(") {
        let after_u128 = &remaining[pos + 5..];
        if let Some(end) = after_u128.find(')') {
            let num_str = &after_u128[..end];
            if let Ok(num) = num_str.trim().parse::<u128>() {
                if num <= 255 {
                    bytes.push(num as u8);
                    if bytes.len() == 32 {
                        let mut arr = [0u8; 32];
                        arr.copy_from_slice(&bytes);
                        return Some(arr);
                    }
                }
            }
        }
        remaining = &remaining[pos + 5..];
    }

    None
}

pub fn rfind_index(s: &str, ch: char) -> usize {
    s.rfind(ch).unwrap_or(0)
}

// Strict extraction helpers - return (value, end_position)

// Extract u8 from string starting at 'from' position
// Returns (u8, end_position) on success
pub fn extract_u8(s: &str, from: usize) -> Option<(u8, usize)> {
    let pos = s[from..].find("U8(")? + from;
    let start = pos + 3;
    let end = s[start..].find(')')? + start;
    let num = s[start..end].trim().parse::<u8>().ok()?;
    Some((num, end))
}

// Extract u16 from string starting at 'from' position
// Returns (u16, end_position) on success
pub fn extract_u16(s: &str, from: usize) -> Option<(u16, usize)> {
    let pos = s[from..].find("U16(")? + from;
    let start = pos + 4;
    let end = s[start..].find(')')? + start;
    let num = s[start..end].trim().parse::<u16>().ok()?;
    Some((num, end))
}

// Extract u32 from string starting at 'from' position
// Returns (u32, end_position) on success
pub fn extract_u32(s: &str, from: usize) -> Option<(u32, usize)> {
    let pos = s[from..].find("U32(")? + from;
    let start = pos + 4;
    let end = s[start..].find(')')? + start;
    let num = s[start..end].trim().parse::<u32>().ok()?;
    Some((num, end))
}

// Extract u64 from string starting at 'from' position
// Returns (u64, end_position) on success
pub fn extract_u64(s: &str, from: usize) -> Option<(u64, usize)> {
    let pos = s[from..].find("U64(")? + from;
    let start = pos + 4;
    let end = s[start..].find(')')? + start;
    let num = s[start..end].trim().parse::<u64>().ok()?;
    Some((num, end))
}

// Extract u128 from string starting at 'from' position
// Returns (u128, end_position) on success
pub fn extract_u128(s: &str, from: usize) -> Option<(u128, usize)> {
    let pos = s[from..].find("U128(")? + from;
    let start = pos + 5;
    let end = s[start..].find(')')? + start;
    let num = s[start..end].trim().parse::<u128>().ok()?;
    Some((num, end))
}

pub fn parse_ip_addr(ip_u128: u128, ip_type: u8) -> std::net::IpAddr {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    if ip_type == 4 {
        let ip_bytes = (ip_u128 as u32).to_be_bytes();
        IpAddr::V4(Ipv4Addr::new(
            ip_bytes[0],
            ip_bytes[1],
            ip_bytes[2],
            ip_bytes[3],
        ))
    } else {
        let ip_bytes = ip_u128.to_be_bytes();
        IpAddr::V6(Ipv6Addr::new(
            u16::from_be_bytes([ip_bytes[0], ip_bytes[1]]),
            u16::from_be_bytes([ip_bytes[2], ip_bytes[3]]),
            u16::from_be_bytes([ip_bytes[4], ip_bytes[5]]),
            u16::from_be_bytes([ip_bytes[6], ip_bytes[7]]),
            u16::from_be_bytes([ip_bytes[8], ip_bytes[9]]),
            u16::from_be_bytes([ip_bytes[10], ip_bytes[11]]),
            u16::from_be_bytes([ip_bytes[12], ip_bytes[13]]),
            u16::from_be_bytes([ip_bytes[14], ip_bytes[15]]),
        ))
    }
}
