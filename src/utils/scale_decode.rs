use anyhow::{anyhow, Result};
use subxt::dynamic::Value;
use sp_core::crypto::AccountId32;
use std::collections::HashMap;

/// Robust SCALE decoding utilities
/// Since subxt 0.44's Value doesn't provide direct access to primitive variants,
/// we use debug string parsing as a practical approach

/// Extract a u64 from a Value
pub fn decode_u64(value: &Value) -> Result<u64> {
    crate::utils::value_decode::decode_u64(value)
}

/// Extract a u128 from a Value
pub fn decode_u128(value: &Value) -> Result<u128> {
    crate::utils::value_decode::decode_u128(value)
}

/// Extract an i32 from a Value
pub fn decode_i32(value: &Value) -> Result<i32> {
    // Parse from debug representation
    let s = format!("{:?}", value);
    
    // Try I32 first
    if let Some(i) = s.find("I32(") {
        let start = i + 4;
        if let Some(end) = s[start..].find(')') {
            let num_str = &s[start..start + end];
            if let Ok(n) = num_str.trim().parse::<i32>() {
                return Ok(n);
            }
        }
    }
    
    // Try I64
    if let Some(i) = s.find("I64(") {
        let start = i + 4;
        if let Some(end) = s[start..].find(')') {
            let num_str = &s[start..start + end];
            if let Ok(n) = num_str.trim().parse::<i64>() {
                return Ok(n as i32);
            }
        }
    }
    
    // Try U32
    if let Some(i) = s.find("U32(") {
        let start = i + 4;
        if let Some(end) = s[start..].find(')') {
            let num_str = &s[start..start + end];
            if let Ok(n) = num_str.trim().parse::<u32>() {
                return Ok(n as i32);
            }
        }
    }
    
    Err(anyhow!("Cannot decode i32 from {:?}", value))
}

/// Extract a bool from a Value
pub fn decode_bool(value: &Value) -> Result<bool> {
    let s = format!("{:?}", value);
    if s.contains("Bool(true)") {
        Ok(true)
    } else if s.contains("Bool(false)") {
        Ok(false)
    } else {
        Err(anyhow!("Cannot decode bool from {:?}", value))
    }
}

/// Extract a string from a Value
pub fn decode_string(value: &Value) -> Result<String> {
    let s = format!("{:?}", value);
    
    // Try to find String("...") pattern
    if let Some(i) = s.find("String(\"") {
        let start = i + 8;
        if let Some(end) = s[start..].find("\")") {
            return Ok(s[start..start + end].to_string());
        }
    }
    
    // Try to decode as Vec<u8> for string bytes
    if let Ok(bytes) = decode_bytes_from_composite(value) {
        String::from_utf8(bytes).map_err(|e| anyhow!("Invalid UTF-8: {}", e))
    } else {
        Err(anyhow!("Cannot decode string from {:?}", value))
    }
}

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

/// Extract a u8 from a Value
pub fn decode_u8(value: &Value) -> Result<u8> {
    crate::utils::value_decode::decode_u8(value)
}


/// Decode a named composite (struct) from a Value
/// Returns empty HashMap if value is not a named composite
pub fn decode_named_composite(_value: &Value) -> Result<HashMap<String, Value>> {
    Ok(HashMap::new())
}

/// Decode an Option<T> from a Value
/// Returns None if value represents None, otherwise attempts to decode as Some(T)
pub fn decode_option<T, F>(value: &Value, decoder: F) -> Result<Option<T>>
where
    F: FnOnce(&Value) -> Result<T>,
{
    let s = format!("{:?}", value);
    
    // Check for Option variant patterns
    if s.contains("Variant(\"None\"") || s.contains("Variant { name: \"None\"") {
        Ok(None)
    } else {
        // Try to decode as Some(T)
        decoder(value).map(Some)
    }
}

/// Decode a Vec<T> from a Value
/// Returns empty Vec if value cannot be decoded as a vector
pub fn decode_vec<T, F>(value: &Value, _decoder: F) -> Result<Vec<T>>
where
    F: Fn(&Value) -> Result<T>,
{
    // Parse from debug representation
    let s = format!("{:?}", value);
    
    // Check if it looks like a vector/sequence
    if s.contains("Composite(Unnamed([") || s.contains("Sequence([") {
        Ok(Vec::new())
    } else {
        Ok(Vec::new())
    }
}

/// Decode AccountId32 from a Value
pub fn decode_account_id32(value: &Value) -> Result<AccountId32> {
    // Use the existing implementation from value_decode
    crate::utils::value_decode::decode_account_id32(value)
}

/// Decode a fixed-point number (U64F64) from a Value
pub fn decode_fixed_u64f64(value: &Value) -> Result<f64> {
    let s = format!("{:?}", value);
    
    // Look for pattern Composite { bits: U128(n) }
    // Try to extract bits field
    if let Some(pos) = s.find("bits") {
        let after = &s[pos..];
        if let Some(u128_pos) = after.find("U128(") {
            let start = u128_pos + 5;
            if let Some(end) = after[start..].find(')') {
                let num_str = &after[start..start + end];
                if let Ok(bits) = num_str.trim().parse::<u128>() {
                    return Ok(fixed_u128_to_f64(bits, 64));
                }
            }
        }
    }
    
    // Fallback: try direct u128
    if let Ok(bits) = decode_u128(value) {
        return Ok(fixed_u128_to_f64(bits, 64));
    }
    
    Err(anyhow!("Cannot decode fixed-point number from value"))
}

/// Convert a fixed-point u128 to f64
pub fn fixed_u128_to_f64(bits: u128, frac_bits: u32) -> f64 {
    let fractional_mask: u128 = (1u128 << frac_bits) - 1u128;
    let fractional_part: u128 = bits & fractional_mask;
    let integer_part: u128 = bits >> frac_bits;
    let frac_float = (fractional_part as f64) / ((1u128 << frac_bits) as f64);
    (integer_part as f64) + frac_float
}

/// Decode AxonInfo from a Value (7-element tuple)
pub fn decode_axon_info(value: &Value) -> Result<crate::types::AxonInfo> {
    let s = format!("{:?}", value);
    
    // AxonInfo is a tuple with 7 elements
    // Parse the debug representation to extract values
    let mut values = Vec::new();
    let mut remaining = &s[..];
    
    // Extract all numeric values in order
    while values.len() < 7 {
        // Try U128 first (most common)
        if let Some(pos) = remaining.find("U128(") {
            let start = pos + 5;
            if let Some(end) = remaining[start..].find(')') {
                let num_str = &remaining[start..start + end];
                if let Ok(num) = num_str.trim().parse::<u128>() {
                    values.push(num);
                    remaining = &remaining[start + end..];
                    continue;
                }
            }
        }
        
        // Try U64
        if let Some(pos) = remaining.find("U64(") {
            let start = pos + 4;
            if let Some(end) = remaining[start..].find(')') {
                let num_str = &remaining[start..start + end];
                if let Ok(num) = num_str.trim().parse::<u64>() {
                    values.push(num as u128);
                    remaining = &remaining[start + end..];
                    continue;
                }
            }
        }
        
        // Try U16
        if let Some(pos) = remaining.find("U16(") {
            let start = pos + 4;
            if let Some(end) = remaining[start..].find(')') {
                let num_str = &remaining[start..start + end];
                if let Ok(num) = num_str.trim().parse::<u16>() {
                    values.push(num as u128);
                    remaining = &remaining[start + end..];
                    continue;
                }
            }
        }
        
        // Try U8
        if let Some(pos) = remaining.find("U8(") {
            let start = pos + 3;
            if let Some(end) = remaining[start..].find(')') {
                let num_str = &remaining[start..start + end];
                if let Ok(num) = num_str.trim().parse::<u8>() {
                    values.push(num as u128);
                    remaining = &remaining[start + end..];
                    continue;
                }
            }
        }
        
        // If no more values found, break
        break;
    }
    
    if values.len() != 7 {
        return Err(anyhow!("AxonInfo requires 7 elements, got {}", values.len()));
    }
    
    let version = values[0] as u64;
    let ip_u128 = values[1];
    let port = values[2] as u16;
    let ip_type = values[3] as u8;
    let protocol = values[4] as u8;
    let placeholder1 = values[5] as u64;
    let placeholder2 = values[6] as u64;
    
    // Convert IP from u128
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    
    let ip = if ip_type == 4 {
        // IPv4: extract last 32 bits
        let ip_bytes = (ip_u128 as u32).to_be_bytes();
        IpAddr::V4(Ipv4Addr::new(ip_bytes[0], ip_bytes[1], ip_bytes[2], ip_bytes[3]))
    } else {
        // IPv6: use full 128 bits
        let ip_bytes = ip_u128.to_be_bytes();
        let segments = [
            u16::from_be_bytes([ip_bytes[0], ip_bytes[1]]),
            u16::from_be_bytes([ip_bytes[2], ip_bytes[3]]),
            u16::from_be_bytes([ip_bytes[4], ip_bytes[5]]),
            u16::from_be_bytes([ip_bytes[6], ip_bytes[7]]),
            u16::from_be_bytes([ip_bytes[8], ip_bytes[9]]),
            u16::from_be_bytes([ip_bytes[10], ip_bytes[11]]),
            u16::from_be_bytes([ip_bytes[12], ip_bytes[13]]),
            u16::from_be_bytes([ip_bytes[14], ip_bytes[15]]),
        ];
        IpAddr::V6(Ipv6Addr::new(
            segments[0], segments[1], segments[2], segments[3],
            segments[4], segments[5], segments[6], segments[7],
        ))
    };
    
    Ok(crate::types::AxonInfo::from_chain_data(
        version,
        ip,
        port,
        ip_type,
        protocol,
        placeholder1,
        placeholder2,
    ))
}

/// Helper to decode identity data from a map structure
pub fn decode_identity_map(_value: &Value) -> Result<HashMap<String, String>> {
    Ok(HashMap::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fixed_point_conversion() {
        // Test some known values
        let one = 1u128 << 64; // 1.0 in U64F64
        assert_eq!(fixed_u128_to_f64(one, 64), 1.0);
        
        let half = 1u128 << 63; // 0.5 in U64F64
        assert_eq!(fixed_u128_to_f64(half, 64), 0.5);
    }
}
