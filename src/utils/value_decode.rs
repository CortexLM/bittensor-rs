/// Utilities for decoding Value from subxt storage results
use anyhow::{Result, anyhow};
use sp_core::crypto::AccountId32;
use subxt::dynamic::Value;

fn parse_numeric(value: &Value, tag: &str) -> Result<u128> {
    let s = format!("{:?}", value);
    if let Some(i) = s.find(&format!("{}(", tag)) {
        let mut j = i + tag.len() + 1;
        let bytes = s.as_bytes();
        let mut num: u128 = 0;
        let mut found = false;
        while j < bytes.len() {
            let b = bytes[j];
            if b.is_ascii_digit() {
                found = true;
                num = num * 10 + (b - b'0') as u128;
                j += 1;
            } else {
                break;
            }
        }
        if found { return Ok(num); }
    }
    Err(anyhow::anyhow!("Failed to parse {} from {:?}", tag, value))
}

/// Decode u128 from Value
pub fn decode_u128(value: &Value) -> Result<u128> {
    parse_numeric(value, "U128")
}

/// Decode u64 from Value
pub fn decode_u64(value: &Value) -> Result<u64> {
    let n = parse_numeric(value, "U64").or_else(|_| parse_numeric(value, "U128"))?;
    u64::try_from(n).map_err(|_| anyhow::anyhow!("u128 does not fit into u64"))
}

/// Decode u16 from Value
pub fn decode_u16(value: &Value) -> Result<u16> {
    let n = parse_numeric(value, "U16").or_else(|_| parse_numeric(value, "U64")).or_else(|_| parse_numeric(value, "U128"))?;
    u16::try_from(n).map_err(|_| anyhow::anyhow!("value does not fit into u16"))
}

/// Decode u8 from Value
pub fn decode_u8(value: &Value) -> Result<u8> {
    let n = parse_numeric(value, "U8").or_else(|_| parse_numeric(value, "U16")).or_else(|_| parse_numeric(value, "U64")).or_else(|_| parse_numeric(value, "U128"))?;
    u8::try_from(n).map_err(|_| anyhow::anyhow!("value does not fit into u8"))
}

/// Decode AccountId32 from Value
/// Handles multiple formats:
/// 1. Sequence of 32 U128/u8 bytes (composite with unnamed values)
/// 2. 0x-prefixed hex string (64 chars)
/// 3. Bytes array
fn extract_bytes_from_composite_sequence(value: &Value) -> Option<[u8; 32]> {
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

/// Decode AccountId32 from Value (from 0x-prefixed hex in debug or byte sequence)
/// Handles wrapped Option and composite structures
pub fn decode_account_id32(value: &Value) -> Result<AccountId32> {
    // First try: AccountId32 stored as sequence of 32 U128 bytes
    if let Some(bytes) = extract_bytes_from_composite_sequence(value) {
        return Ok(AccountId32::from(bytes));
    }
    
    // Second try: 0x hex string format
    let value_str = format!("{:?}", value);
    
    // Try to find all 0x hex strings and check for 64-char AccountId32
    let mut candidates = Vec::new();
    let mut search_start = 0;
    
    while let Some(start) = value_str[search_start..].find("0x") {
        let abs_start = search_start + start;
        let hex_str = &value_str[abs_start + 2..];
        let end = hex_str.find(|c: char| !c.is_ascii_hexdigit()).unwrap_or(hex_str.len());
        let hex = &hex_str[..end.min(64)];
        
        if hex.len() == 64 {
            candidates.push((abs_start, hex.to_string()));
        }
        
        // Move past this hex string
        search_start = abs_start + 2 + hex.len();
    }
    
    // If we found candidates, try to decode the first one
    for (_pos, hex) in candidates {
        if let Ok(bytes) = hex::decode(&hex) {
            if bytes.len() == 32 {
                let array: [u8; 32] = bytes.try_into()
                    .map_err(|_| anyhow::anyhow!("Invalid AccountId32 length"))?;
                return Ok(AccountId32::from(array));
            }
        }
    }
    
    Err(anyhow::anyhow!("Cannot decode AccountId32 from Value: {:?}", value))
}

/// Decode bool from Value
pub fn decode_bool(value: &Value) -> Result<bool> {
    let s = format!("{:?}", value);
    if s.contains("true") { Ok(true) }
    else if s.contains("false") { Ok(false) }
    else {
        // Sometimes booleans appear as U8(0/1)
        Ok(decode_u8(value)? != 0)
    }
}

/// Decode Vec<u8> from Value (from debug representation)
pub fn decode_bytes(value: &Value) -> Result<Vec<u8>> {
    let s = format!("{:?}", value);
    if let Some(l) = s.find('[') { if let Some(r) = s[rfind_index(&s, ']')..].find(']') { let inner=&s[l+1..l+1+r];
        let mut out=Vec::new();
        for part in inner.split(',') { let p=part.trim(); if p.is_empty(){continue;} let v: u8 = p.parse().map_err(|e| anyhow::anyhow!("Failed to parse byte: {}", e))?; out.push(v);} return Ok(out) } }
    Err(anyhow::anyhow!("Cannot decode bytes from Value: {:?}", value))
}

fn rfind_index(s: &str, ch: char) -> usize { s.rfind(ch).unwrap_or(0) }

/// Decode normalized u16 to f64 (0.0-1.0 range)
pub fn decode_normalized_u16(value: &Value) -> Result<f64> {
    let u16_val = decode_u16(value)?;
    Ok(u16_val as f64 / u16::MAX as f64)
}

/// Decode normalized u64 to f64 (0.0-1.0 range)
pub fn decode_normalized_u64(value: &Value) -> Result<f64> {
    let u64_val = decode_u64(value)?;
    Ok(u64_val as f64 / u64::MAX as f64)
}

/// Decode a vector of tuples (u64, AccountId32) from Value
pub fn decode_vec_tuple_u64_account(value: &Value) -> Result<Vec<(u64, AccountId32)>> {
    // Parse by scanning pairs of U64/U128 and following 0x<64-hex>
    let s = format!("{:?}", value);
    let mut res = Vec::new();
    let mut i = 0usize;
    let bytes = s.as_bytes();
    while i < bytes.len() {
        // find number
        if let Some(pos) = s[i..].find("U64(").or_else(|| s[i..].find("U128(")).map(|p| p + i) {
            // Move to position of found pattern
            // extract number
            let tag = if s[pos..].starts_with("U64(") { "U64(" } else { "U128(" };
            let mut j = pos + tag.len();
            let mut num: u128 = 0;
            while j < bytes.len() && bytes[j].is_ascii_digit() { num = num*10 + (bytes[j]-b'0') as u128; j+=1; }
            // find hex
            if let Some(hx) = s[j..].find("0x").map(|p| p + j) {
                let mut k = hx + 2;
                let mut hex = String::new();
                while k < bytes.len() && s.as_bytes()[k].is_ascii_hexdigit() { hex.push(s.as_bytes()[k] as char); k+=1; }
                if hex.len()>=64 { let hex = &hex[..64]; if let Ok(b) = hex::decode(hex) { if b.len()==32 { let mut arr=[0u8;32]; arr.copy_from_slice(&b[..32]); let acct=AccountId32::from(arr); let prop = u64::try_from(num).unwrap_or(u64::MAX); res.push((prop, acct)); } } }
                i = k; continue;
            }
            i = j; continue;
        } else { break }
    }
    Ok(res)
}

/// Decode a vector of AccountId32 from Value
pub fn decode_vec_account_id32(value: &Value) -> Result<Vec<AccountId32>> {
    let s = format!("{:?}", value);
    let mut res = Vec::new();
    let mut i = 0usize;
    while let Some(hx) = s[i..].find("0x").map(|p| p + i) {
        let hex_portion = &s[hx+2..];
        let mut hex = String::new();
        for ch in hex_portion.chars() { if ch.is_ascii_hexdigit() { hex.push(ch); } else { break; } }
        if hex.len() >= 64 { let hex64=&hex[..64]; if let Ok(b)=hex::decode(hex64) { if b.len()==32 { let mut arr=[0u8;32]; arr.copy_from_slice(&b[..32]); res.push(AccountId32::from(arr)); } } }
        i = hx + 2 + hex.len();
    }
    Ok(res)
}

/// Decode a vector of u16 from Value
pub fn decode_vec_u16(value: &Value) -> Result<Vec<u16>> {
    let s = format!("{:?}", value);
    let mut res = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        // Look for U128( pattern (values are stored as U128 even for u16)
        if let Some(pos) = s[i..].find("U128(").map(|p| p + i) {
            let mut j = pos + 5;
            let mut num: u128 = 0;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                num = num * 10 + (bytes[j] - b'0') as u128;
                j += 1;
            }
            // Convert to u16
            res.push(num as u16);
            i = j;
        } else {
            i += 1;
        }
    }
    Ok(res)
}

/// Decode a vector of u128 from Value
pub fn decode_vec_u128(value: &Value) -> Result<Vec<u128>> {
    let s = format!("{:?}", value);
    let mut out = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if let Some(pos) = s[i..].find("U128(").map(|p| p + i) {
            let mut j = pos + 5; // len("U128(")
            let mut num: u128 = 0;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                num = num * 10 + (bytes[j] - b'0') as u128;
                j += 1;
            }
            out.push(num);
            i = j;
        } else if let Some(pos) = s[i..].find("U64(").map(|p| p + i) {
            let mut j = pos + 4; // len("U64(")
            let mut num: u128 = 0;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                num = num * 10 + (bytes[j] - b'0') as u128;
                j += 1;
            }
            out.push(num);
            i = j;
        } else {
            break;
        }
    }
    Ok(out)
}

/// Decode a vector of (AccountId32, u128) pairs from Value
/// This is used for Stake[(netuid, uid)] -> Vec<(AccountId32, Compact<u64>)>
pub fn decode_vec_account_u128_pairs(value: &Value) -> Result<Vec<(AccountId32, u128)>> {
    // Strategy: find interleaved 0x<64-hex> (AccountId32) followed by a U64/U128 number
    let s = format!("{:?}", value);
    let mut out = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        // find account hex
        if let Some(hx) = s[i..].find("0x").map(|p| p + i) {
            // extract 64 hex chars
            let mut k = hx + 2;
            let mut hex = String::new();
            while k < bytes.len() && s.as_bytes()[k].is_ascii_hexdigit() {
                hex.push(s.as_bytes()[k] as char);
                k += 1;
            }
            if hex.len() >= 64 {
                let hex64 = &hex[..64];
                if let Ok(b) = hex::decode(hex64) {
                    if b.len() == 32 {
                        let mut arr = [0u8; 32];
                        arr.copy_from_slice(&b[..32]);
                        let acct = AccountId32::from(arr);
                        // after account, seek number tag U64(..) or U128(..)
                        let rest = &s[k..];
                        if let Some(num_pos_rel) = rest.find("U64(").or_else(|| rest.find("U128(")) {
                            let num_pos = k + num_pos_rel;
                            let tag_len = if &s[num_pos..num_pos+4] == "U64(" { 4 } else { 5 };
                            let mut j = num_pos + tag_len;
                            let mut num: u128 = 0;
                            while j < bytes.len() && bytes[j].is_ascii_digit() {
                                num = num * 10 + (bytes[j] - b'0') as u128;
                                j += 1;
                            }
                            out.push((acct, num));
                            i = j;
                            continue;
                        }
                        // if no number found, still advance
                        i = k;
                        continue;
                    }
                }
            }
            i = hx + 2 + hex.len();
        } else {
            break;
        }
    }
    Ok(out)
}

/// Decode a vector of (u64,u64) pairs from a Value representing Vec<(Compact<u16>, Compact<u16>)>
/// Decode a vector of bool from Value
pub fn decode_vec_bool(value: &Value) -> Result<Vec<bool>> {
    let s = format!("{:?}", value);
    let mut res = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        // Try Bool(true/false) first
        if let Some(pos) = s[i..].find("Bool(").map(|p| p + i) {
            let j = pos + 5;
            if j + 4 <= bytes.len() {
                if &s[j..j+4] == "true" {
                    res.push(true);
                    i = j + 4;
                } else if j + 5 <= bytes.len() && &s[j..j+5] == "false" {
                    res.push(false);
                    i = j + 5;
                } else {
                    i = j;
                }
            } else {
                i = j;
            }
        } 
        // Also try U8(0/1) format (bools might be stored as U8)
        else if let Some(pos) = s[i..].find("U8(").map(|p| p + i) {
            let mut j = pos + 3;
            let mut num: u8 = 0;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                num = num * 10 + (bytes[j] - b'0') as u8;
                j += 1;
            }
            res.push(num != 0);
            i = j;
        }
        // Also try U128 format (like with u16 vectors)
        else if let Some(pos) = s[i..].find("U128(").map(|p| p + i) {
            let mut j = pos + 5;
            let mut num: u128 = 0;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                num = num * 10 + (bytes[j] - b'0') as u128;
                j += 1;
            }
            res.push(num != 0);
            i = j;
        } else {
            i += 1;
        }
    }
    Ok(res)
}

/// Decode a vector of u64 from Value
pub fn decode_vec_u64(value: &Value) -> Result<Vec<u64>> {
    let s = format!("{:?}", value);
    let mut res = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        // Try U64 first
        if let Some(pos) = s[i..].find("U64(").map(|p| p + i) {
            let mut j = pos + 4;
            let mut num: u64 = 0;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                num = num * 10 + (bytes[j] - b'0') as u64;
                j += 1;
            }
            res.push(num);
            i = j;
        } 
        // Also try U128 (values might be stored as U128 even for u64)
        else if let Some(pos) = s[i..].find("U128(").map(|p| p + i) {
            let mut j = pos + 5;
            let mut num: u128 = 0;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                num = num * 10 + (bytes[j] - b'0') as u128;
                j += 1;
            }
            // Convert to u64 (will truncate if too large)
            res.push(num as u64);
            i = j;
        } else {
            i += 1;
        }
    }
    Ok(res)
}

pub fn decode_vec_u64_u64_pairs(value: &Value) -> Result<Vec<(u64, u64)>> {
    // Parse by scanning pairs of numbers inside composite tuples
    let s = format!("{:?}", value);
    let bytes = s.as_bytes();
    let mut i = 0usize;
    let mut res = Vec::new();
    loop {
        // find first number
        let pos1_opt = s[i..].find("U16(")
            .or_else(|| s[i..].find("U32("))
            .or_else(|| s[i..].find("U64("))
            .or_else(|| s[i..].find("U128("))
            .map(|p| p + i);
        let Some(pos1) = pos1_opt else { break };
        let tag1 = if s[pos1..].starts_with("U16(") { 4 } else if s[pos1..].starts_with("U32(") { 4 } else if s[pos1..].starts_with("U64(") { 4 } else { 5 };
        let mut j = pos1 + tag1;
        let mut a: u128 = 0;
        while j < bytes.len() && bytes[j].is_ascii_digit() { a = a*10 + (bytes[j]-b'0') as u128; j+=1; }
        // find second number
        let pos2_opt = s[j..].find("U16(")
            .or_else(|| s[j..].find("U32("))
            .or_else(|| s[j..].find("U64("))
            .or_else(|| s[j..].find("U128("))
            .map(|p| p + j);
        let Some(pos2) = pos2_opt else { i = j; continue };
        let tag2 = if s[pos2..].starts_with("U16(") { 4 } else if s[pos2..].starts_with("U32(") { 4 } else if s[pos2..].starts_with("U64(") { 4 } else { 5 };
        let mut k = pos2 + tag2;
        let mut b: u128 = 0;
        while k < bytes.len() && bytes[k].is_ascii_digit() { b = b*10 + (bytes[k]-b'0') as u128; k+=1; }
        res.push((u64::try_from(a).unwrap_or(u64::MAX), u64::try_from(b).unwrap_or(u64::MAX)));
        i = k;
    }
    Ok(res)
}

/// Decode PrometheusInfo from Value
pub fn decode_prometheus_info(value: &Value) -> Result<crate::types::PrometheusInfo> {
    let s = format!("{:?}", value);
    
    // PrometheusInfo is a tuple with 5 elements: (block, version, ip, port, ip_type)
    let mut values: Vec<u128> = Vec::new();
    let mut remaining = &s[..];
    
    // Extract all numeric values in order
    while values.len() < 5 {
        // Try U64 first (block, version)
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
        
        // Try U128
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
        
        // Try U16 (port)
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
        
        // Try U8 (ip_type)
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
    
    if values.len() < 5 {
        return Err(anyhow!("PrometheusInfo requires 5 elements, got {}", values.len()));
    }
    
    let block = values[0] as u64;
    let version = values[1] as u64;
    let ip_u128 = values[2];
    let port = values[3] as u16;
    let ip_type = values[4] as u8;
    
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
    
    Ok(crate::types::PrometheusInfo::from_chain_data(
        block,
        version,
        ip.to_string(),
        port,
        ip_type,
    ))
}

