//! Bonds query functions
//! Read-only queries for neuron bonds

use crate::chain::BittensorClient;
use crate::utils::value_decode::decode_u64;
use anyhow::Result;
use subxt::dynamic::Value;

const SUBTENSOR_MODULE: &str = "SubtensorModule";

/// Get bonds for a specific neuron
/// Returns Vec<(uid, bond_value)>
pub async fn get_neuron_bonds(
    client: &BittensorClient,
    netuid: u16,
    uid: u64,
    mechid: u16,
) -> Result<Vec<(u16, u64)>> {
    // Calculate storage index: (netuid << 16) | mechid
    let storage_index = ((netuid as u32) << 16) | (mechid as u32);

    let result = client
        .storage_with_keys(
            SUBTENSOR_MODULE,
            "Bonds",
            vec![
                Value::u128(storage_index as u128),
                Value::u128(uid as u128),
            ],
        )
        .await?;

    if let Some(value) = result {
        parse_bonds_from_value(&value)
    } else {
        Ok(vec![])
    }
}

/// Get all bonds for a subnet
/// Returns Vec<(uid, Vec<(target_uid, bond_value)>)>
pub async fn get_all_bonds(
    client: &BittensorClient,
    netuid: u16,
    mechid: u16,
) -> Result<Vec<(u64, Vec<(u16, u64)>)>> {
    // Get neuron count
    let n = crate::queries::subnets::subnet_n(client, netuid)
        .await?
        .unwrap_or(0);

    let storage_index = ((netuid as u32) << 16) | (mechid as u32);
    let mut all_bonds = Vec::new();

    for uid in 0..n {
        let result = client
            .storage_with_keys(
                SUBTENSOR_MODULE,
                "Bonds",
                vec![
                    Value::u128(storage_index as u128),
                    Value::u128(uid as u128),
                ],
            )
            .await?;

        if let Some(value) = result {
            let bonds = parse_bonds_from_value(&value)?;
            if !bonds.is_empty() {
                all_bonds.push((uid, bonds));
            }
        }
    }

    Ok(all_bonds)
}

/// Get weights for a specific neuron
/// Returns Vec<(uid, weight_value)>
pub async fn get_neuron_weights(
    client: &BittensorClient,
    netuid: u16,
    uid: u64,
    mechid: u16,
) -> Result<Vec<(u16, u16)>> {
    let storage_index = ((netuid as u32) << 16) | (mechid as u32);

    let result = client
        .storage_with_keys(
            SUBTENSOR_MODULE,
            "Weights",
            vec![
                Value::u128(storage_index as u128),
                Value::u128(uid as u128),
            ],
        )
        .await?;

    if let Some(value) = result {
        parse_weights_from_value(&value)
    } else {
        Ok(vec![])
    }
}

/// Get all weights for a subnet
pub async fn get_all_weights(
    client: &BittensorClient,
    netuid: u16,
    mechid: u16,
) -> Result<Vec<(u64, Vec<(u16, u16)>)>> {
    let n = crate::queries::subnets::subnet_n(client, netuid)
        .await?
        .unwrap_or(0);

    let storage_index = ((netuid as u32) << 16) | (mechid as u32);
    let mut all_weights = Vec::new();

    for uid in 0..n {
        let result = client
            .storage_with_keys(
                SUBTENSOR_MODULE,
                "Weights",
                vec![
                    Value::u128(storage_index as u128),
                    Value::u128(uid as u128),
                ],
            )
            .await?;

        if let Some(value) = result {
            let weights = parse_weights_from_value(&value)?;
            if !weights.is_empty() {
                all_weights.push((uid, weights));
            }
        }
    }

    Ok(all_weights)
}

/// Parse bonds from storage value using debug string parsing
fn parse_bonds_from_value(value: &Value) -> Result<Vec<(u16, u64)>> {
    let value_str = format!("{:?}", value);
    parse_pairs_u16_u64(&value_str)
}

/// Parse weights from storage value using debug string parsing
fn parse_weights_from_value(value: &Value) -> Result<Vec<(u16, u16)>> {
    let value_str = format!("{:?}", value);
    parse_pairs_u16_u16(&value_str)
}

/// Parse pairs of (u16, u64) from debug string
fn parse_pairs_u16_u64(s: &str) -> Result<Vec<(u16, u64)>> {
    let mut pairs = Vec::new();
    
    // Look for patterns like "U16(X)" followed by "U64(Y)" or "U128(Y)"
    let mut remaining = s;
    
    while let Some(pos1) = remaining.find("U16(") {
        let after_u16 = &remaining[pos1 + 4..];
        if let Some(end1) = after_u16.find(')') {
            let num1_str = &after_u16[..end1];
            if let Ok(uid) = num1_str.trim().parse::<u16>() {
                let rest = &after_u16[end1..];
                // Find next U64 or U128
                if let Some(pos2) = rest.find("U64(").or_else(|| rest.find("U128(")) {
                    let is_u128 = rest[pos2..].starts_with("U128");
                    let offset = if is_u128 { 5 } else { 4 };
                    let after_u64 = &rest[pos2 + offset..];
                    if let Some(end2) = after_u64.find(')') {
                        let num2_str = &after_u64[..end2];
                        if let Ok(bond) = num2_str.trim().parse::<u64>() {
                            pairs.push((uid, bond));
                        }
                    }
                }
            }
        }
        remaining = &remaining[pos1 + 4..];
    }
    
    Ok(pairs)
}

/// Parse pairs of (u16, u16) from debug string
fn parse_pairs_u16_u16(s: &str) -> Result<Vec<(u16, u16)>> {
    let mut pairs = Vec::new();
    
    // Look for patterns like "U16(X)" followed by another "U16(Y)"
    let mut remaining = s;
    let mut last_u16: Option<u16> = None;
    
    while let Some(pos) = remaining.find("U16(") {
        let after_u16 = &remaining[pos + 4..];
        if let Some(end) = after_u16.find(')') {
            let num_str = &after_u16[..end];
            if let Ok(val) = num_str.trim().parse::<u16>() {
                if let Some(first) = last_u16.take() {
                    pairs.push((first, val));
                } else {
                    last_u16 = Some(val);
                }
            }
        }
        remaining = &after_u16[after_u16.find(')').unwrap_or(0)..];
    }
    
    Ok(pairs)
}

/// Get subnet N (number of neurons)
#[allow(dead_code)]
pub async fn subnet_n(client: &BittensorClient, netuid: u16) -> Result<Option<u64>> {
    let result = client
        .storage_with_keys(
            SUBTENSOR_MODULE,
            "SubnetworkN",
            vec![Value::u128(netuid as u128)],
        )
        .await?;

    if let Some(value) = result {
        Ok(decode_u64(&value).ok())
    } else {
        Ok(None)
    }
}
