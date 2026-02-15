use super::utils;
use anyhow::{anyhow, Result};
use sp_core::crypto::AccountId32;
use subxt::dynamic::Value;
use subxt::ext::scale_value::{Composite, ValueDef};

const MAX_DECODE_DEPTH: usize = 32;

fn push_composite_values<'a>(
    stack: &mut Vec<(&'a Value, usize)>,
    composite: &'a Composite<()>,
    depth: usize,
) {
    match composite {
        Composite::Named(fields) => {
            for (_, val) in fields.iter().rev() {
                stack.push((val, depth));
            }
        }
        Composite::Unnamed(vals) => {
            for val in vals.iter().rev() {
                stack.push((val, depth));
            }
        }
    }
}

pub fn extract_u128(value: &Value) -> Option<u128> {
    let mut stack = vec![(value, 0usize)];
    while let Some((current, depth)) = stack.pop() {
        if depth > MAX_DECODE_DEPTH {
            continue;
        }
        match &current.value {
            ValueDef::Primitive(primitive) => {
                if let Some(num) = primitive.as_u128() {
                    return Some(num);
                }
                if let Some(num) = primitive.as_i128() {
                    if num >= 0 {
                        return Some(num as u128);
                    }
                }
            }
            ValueDef::Composite(composite) => {
                push_composite_values(&mut stack, composite, depth + 1);
            }
            ValueDef::Variant(variant) => {
                push_composite_values(&mut stack, &variant.values, depth + 1);
            }
            ValueDef::BitSequence(_) => {}
        }
    }
    None
}

pub fn extract_i128(value: &Value) -> Option<i128> {
    let mut stack = vec![(value, 0usize)];
    while let Some((current, depth)) = stack.pop() {
        if depth > MAX_DECODE_DEPTH {
            continue;
        }
        match &current.value {
            ValueDef::Primitive(primitive) => {
                if let Some(num) = primitive.as_i128() {
                    return Some(num);
                }
                if let Some(num) = primitive.as_u128() {
                    if num <= i128::MAX as u128 {
                        return Some(num as i128);
                    }
                }
            }
            ValueDef::Composite(composite) => {
                push_composite_values(&mut stack, composite, depth + 1);
            }
            ValueDef::Variant(variant) => {
                push_composite_values(&mut stack, &variant.values, depth + 1);
            }
            ValueDef::BitSequence(_) => {}
        }
    }
    None
}

pub fn extract_bool(value: &Value) -> Option<bool> {
    let mut stack = vec![(value, 0usize)];
    while let Some((current, depth)) = stack.pop() {
        if depth > MAX_DECODE_DEPTH {
            continue;
        }
        match &current.value {
            ValueDef::Primitive(primitive) => {
                if let Some(b) = primitive.as_bool() {
                    return Some(b);
                }
                if let Some(num) = primitive.as_u128() {
                    return Some(num != 0);
                }
            }
            ValueDef::Composite(composite) => {
                push_composite_values(&mut stack, composite, depth + 1);
            }
            ValueDef::Variant(variant) => {
                push_composite_values(&mut stack, &variant.values, depth + 1);
            }
            ValueDef::BitSequence(_) => {}
        }
    }
    None
}

/// Decode u128 from Value
pub fn decode_u128(value: &Value) -> Result<u128> {
    extract_u128(value).ok_or_else(|| anyhow!("Failed to decode u128 from value"))
}

/// Decode u64 from Value
pub fn decode_u64(value: &Value) -> Result<u64> {
    let n = extract_u128(value).ok_or_else(|| anyhow!("Failed to decode u64 from value"))?;
    u64::try_from(n).map_err(|_| anyhow!("u128 does not fit into u64"))
}

/// Decode an i32 from a Value
pub fn decode_i32(value: &Value) -> Result<i32> {
    if let Some(num) = extract_i128(value) {
        return i32::try_from(num).map_err(|_| anyhow!("i128 does not fit into i32"));
    }
    Err(anyhow!("Cannot decode i32 from value"))
}

/// Decode u16 from Value
pub fn decode_u16(value: &Value) -> Result<u16> {
    let n = extract_u128(value).ok_or_else(|| anyhow!("Failed to decode u16 from value"))?;
    u16::try_from(n).map_err(|_| anyhow!("value does not fit into u16"))
}

/// Decode u8 from Value
pub fn decode_u8(value: &Value) -> Result<u8> {
    let n = extract_u128(value).ok_or_else(|| anyhow!("Failed to decode u8 from value"))?;
    u8::try_from(n).map_err(|_| anyhow!("value does not fit into u8"))
}

/// Decode bool from Value
pub fn decode_bool(value: &Value) -> Result<bool> {
    extract_bool(value).ok_or_else(|| anyhow!("Failed to decode bool from value"))
}

/// Extract a string from a Value
pub fn decode_string(value: &Value) -> Result<String> {
    if let ValueDef::Primitive(primitive) = &value.value {
        if let Some(s) = primitive.as_str() {
            return Ok(s.to_string());
        }
    }
    let bytes = utils::decode_bytes_from_composite(value)
        .or_else(|_| decode_bytes(value))
        .map_err(|_| anyhow!("Cannot decode string from value"))?;
    String::from_utf8(bytes).map_err(|e| anyhow!("Invalid UTF-8: {}", e))
}

pub fn decode_bytes(value: &Value) -> Result<Vec<u8>> {
    utils::decode_bytes_from_composite(value).map_err(|_| anyhow!("Cannot decode bytes from value"))
}

/// Decode an Option<T> from a Value
/// Returns None if value represents None, otherwise attempts to decode as Some(T)
pub fn decode_option<T, F>(value: &Value, decoder: F) -> Result<Option<T>>
where
    F: FnOnce(&Value) -> Result<T>,
{
    if let ValueDef::Variant(variant) = &value.value {
        if variant.name == "None" {
            return Ok(None);
        }
        if variant.name == "Some" {
            match &variant.values {
                Composite::Unnamed(values) => {
                    if let Some(inner) = values.first() {
                        return decoder(inner).map(Some);
                    }
                }
                Composite::Named(fields) => {
                    if let Some((_, inner)) = fields.first() {
                        return decoder(inner).map(Some);
                    }
                }
            }
        }
    }

    decoder(value).map(Some)
}

pub fn decode_account_id32(value: &Value) -> Result<AccountId32> {
    if let Some(bytes) = utils::extract_bytes_from_composite_sequence(value) {
        return Ok(AccountId32::from(bytes));
    }

    let mut stack = vec![(value, 0usize)];
    while let Some((current, depth)) = stack.pop() {
        if depth > MAX_DECODE_DEPTH {
            continue;
        }
        match &current.value {
            ValueDef::Primitive(primitive) => {
                if let Some(s) = primitive.as_str() {
                    let trimmed = s.strip_prefix("0x").unwrap_or(s);
                    if trimmed.len() >= 64 {
                        let hex_str = &trimmed[..64];
                        if let Ok(bytes) = hex::decode(hex_str) {
                            if bytes.len() == 32 {
                                let mut arr = [0u8; 32];
                                arr.copy_from_slice(&bytes);
                                return Ok(AccountId32::from(arr));
                            }
                        }
                    }
                }
            }
            ValueDef::Composite(composite) => {
                push_composite_values(&mut stack, composite, depth + 1);
            }
            ValueDef::Variant(variant) => {
                if variant.name == "None" {
                    return Err(anyhow!("Failed to decode AccountId32 from value"));
                }
                push_composite_values(&mut stack, &variant.values, depth + 1);
            }
            ValueDef::BitSequence(_) => {}
        }
    }

    Err(anyhow!("Failed to decode AccountId32 from value"))
}
