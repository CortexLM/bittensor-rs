use anyhow::Result;
use sp_core::crypto::AccountId32;
use subxt::dynamic::Value;
use subxt::ext::scale_value::{Composite, ValueDef};

use super::primitive;

/// Decode a Vec<T> from a Value
/// Returns empty Vec if value cannot be decoded as a vector
pub fn decode_vec<T, F>(value: &Value, decoder: F) -> Result<Vec<T>>
where
    F: Fn(&Value) -> Result<T>,
{
    let mut results = Vec::new();
    let values = extract_values(value);
    for inner_value in values {
        if let Ok(decoded) = decoder(inner_value) {
            results.push(decoded);
        }
    }
    Ok(results)
}

fn extract_values<'a>(value: &'a Value) -> Vec<&'a Value> {
    match &value.value {
        ValueDef::Composite(composite) => match composite {
            Composite::Named(fields) => fields.iter().map(|(_, v)| v).collect(),
            Composite::Unnamed(vals) => vals.iter().collect(),
        },
        ValueDef::Variant(variant) => match &variant.values {
            Composite::Named(fields) => fields.iter().map(|(_, v)| v).collect(),
            Composite::Unnamed(vals) => vals.iter().collect(),
        },
        _ => Vec::new(),
    }
}

fn extract_tuple2<'a>(value: &'a Value) -> Option<(&'a Value, &'a Value)> {
    match &value.value {
        ValueDef::Composite(composite) => match composite {
            Composite::Named(fields) => {
                if fields.len() >= 2 {
                    Some((&fields[0].1, &fields[1].1))
                } else {
                    None
                }
            }
            Composite::Unnamed(values) => {
                if values.len() >= 2 {
                    Some((&values[0], &values[1]))
                } else {
                    None
                }
            }
        },
        ValueDef::Variant(variant) => match &variant.values {
            Composite::Named(fields) => {
                if fields.len() >= 2 {
                    Some((&fields[0].1, &fields[1].1))
                } else {
                    None
                }
            }
            Composite::Unnamed(values) => {
                if values.len() >= 2 {
                    Some((&values[0], &values[1]))
                } else {
                    None
                }
            }
        },
        _ => None,
    }
}

/// Decode a vector of u16 from Value
pub fn decode_vec_u16(value: &Value) -> Result<Vec<u16>> {
    decode_vec(value, primitive::decode_u16)
}

/// Decode a vector of u128 from Value
pub fn decode_vec_u128(value: &Value) -> Result<Vec<u128>> {
    decode_vec(value, primitive::decode_u128)
}

/// Decode a vector of (u64,u64) pairs from a Value representing Vec<(Compact<u16>, Compact<u16>)>
/// Decode a vector of bool from Value
pub fn decode_vec_bool(value: &Value) -> Result<Vec<bool>> {
    decode_vec(value, primitive::decode_bool)
}

/// Decode a vector of u64 from Value
pub fn decode_vec_u64(value: &Value) -> Result<Vec<u64>> {
    decode_vec(value, primitive::decode_u64)
}

/// Decode a vector of AccountId32 from Value
pub fn decode_vec_account_id32(value: &Value) -> Result<Vec<AccountId32>> {
    decode_vec(value, primitive::decode_account_id32)
}

pub fn decode_vec_u64_u64_pairs(value: &Value) -> Result<Vec<(u64, u64)>> {
    let mut out = Vec::new();
    for entry in extract_values(value) {
        if let Some((first, second)) = extract_tuple2(entry) {
            if let (Ok(a), Ok(b)) = (primitive::decode_u64(first), primitive::decode_u64(second)) {
                out.push((a, b));
            }
        }
    }
    Ok(out)
}

/// Decode a vector of (AccountId32, u128) pairs from Value
/// This is used for Stake[(netuid, uid)] -> Vec<(AccountId32, Compact<u64>)>
pub fn decode_vec_account_u128_pairs(value: &Value) -> Result<Vec<(AccountId32, u128)>> {
    let mut out = Vec::new();
    for entry in extract_values(value) {
        if let Some((first, second)) = extract_tuple2(entry) {
            if let (Ok(account), Ok(amount)) = (
                primitive::decode_account_id32(first),
                primitive::decode_u128(second),
            ) {
                out.push((account, amount));
            }
        }
    }
    Ok(out)
}

/// Decode a vector of tuples (u64, AccountId32) from Value
pub fn decode_vec_tuple_u64_account(value: &Value) -> Result<Vec<(u64, AccountId32)>> {
    let mut res = Vec::new();
    for entry in extract_values(value) {
        if let Some((first, second)) = extract_tuple2(entry) {
            if let (Ok(number), Ok(account)) = (
                primitive::decode_u64(first),
                primitive::decode_account_id32(second),
            ) {
                res.push((number, account));
            }
        }
    }
    Ok(res)
}
