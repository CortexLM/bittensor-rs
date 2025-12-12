use super::utils;
use anyhow::{anyhow, Result};
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
        if found {
            return Ok(num);
        }
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

/// Decode an i32 from a Value
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

/// Decode u16 from Value
pub fn decode_u16(value: &Value) -> Result<u16> {
    let n = parse_numeric(value, "U16")
        .or_else(|_| parse_numeric(value, "U64"))
        .or_else(|_| parse_numeric(value, "U128"))?;
    u16::try_from(n).map_err(|_| anyhow::anyhow!("value does not fit into u16"))
}

/// Decode u8 from Value
pub fn decode_u8(value: &Value) -> Result<u8> {
    let n = parse_numeric(value, "U8")
        .or_else(|_| parse_numeric(value, "U16"))
        .or_else(|_| parse_numeric(value, "U64"))
        .or_else(|_| parse_numeric(value, "U128"))?;
    u8::try_from(n).map_err(|_| anyhow::anyhow!("value does not fit into u8"))
}

/// Decode bool from Value
pub fn decode_bool(value: &Value) -> Result<bool> {
    let s = format!("{:?}", value);
    if s.contains("true") {
        Ok(true)
    } else if s.contains("false") {
        Ok(false)
    } else {
        // Sometimes booleans appear as U8(0/1)
        Ok(decode_u8(value)? != 0)
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
    if let Ok(bytes) = utils::decode_bytes_from_composite(value) {
        String::from_utf8(bytes).map_err(|e| anyhow!("Invalid UTF-8: {}", e))
    } else {
        Err(anyhow!("Cannot decode string from {:?}", value))
    }
}

pub fn decode_bytes(value: &Value) -> Result<Vec<u8>> {
    let s = format!("{:?}", value);
    if let Some(l) = s.find('[') {
        if let Some(r) = s[utils::rfind_index(&s, ']')..].find(']') {
            let inner = &s[l + 1..l + 1 + r];
            let mut out = Vec::new();
            for part in inner.split(',') {
                let p = part.trim();
                if p.is_empty() {
                    continue;
                }
                let v: u8 = p
                    .parse()
                    .map_err(|e| anyhow::anyhow!("Failed to parse byte: {}", e))?;
                out.push(v);
            }
            return Ok(out);
        }
    }
    Err(anyhow::anyhow!(
        "Cannot decode bytes from Value: {:?}",
        value
    ))
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

/// Decode AccountId32 from Value (from 0x-prefixed hex in debug or byte sequence)
/// Handles wrapped Option and composite structures
pub fn decode_account_id32(value: &Value) -> Result<AccountId32> {
    // First try: AccountId32 stored as sequence of 32 U128 bytes
    if let Some(bytes) = utils::extract_bytes_from_composite_sequence(value) {
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
        let end = hex_str
            .find(|c: char| !c.is_ascii_hexdigit())
            .unwrap_or(hex_str.len());
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
                let array: [u8; 32] = bytes
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("Invalid AccountId32 length"))?;
                return Ok(AccountId32::from(array));
            }
        }
    }

    Err(anyhow::anyhow!(
        "Cannot decode AccountId32 from Value: {:?}",
        value
    ))
}
