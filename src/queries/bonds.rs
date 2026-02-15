//! Bonds query functions
//! Read-only queries for neuron bonds

use crate::chain::BittensorClient;
use crate::utils::decoders::vec::decode_vec;
use crate::utils::decoders::{decode_u16, decode_u64};
use anyhow::Result;
use subxt::dynamic::Value;
use subxt::ext::scale_value::{Composite, ValueDef};

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
            vec![Value::u128(storage_index as u128), Value::u128(uid as u128)],
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
                vec![Value::u128(storage_index as u128), Value::u128(uid as u128)],
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
            vec![Value::u128(storage_index as u128), Value::u128(uid as u128)],
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
                vec![Value::u128(storage_index as u128), Value::u128(uid as u128)],
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
    decode_vec(value, |entry| {
        if let Some((uid_val, bond_val)) = extract_pair(entry) {
            let uid = decode_u16(uid_val)?;
            let bond = decode_u64(bond_val)?;
            return Ok((uid, bond));
        }
        Err(anyhow::anyhow!("invalid bond pair"))
    })
}

/// Parse weights from storage value using debug string parsing
fn parse_weights_from_value(value: &Value) -> Result<Vec<(u16, u16)>> {
    decode_vec(value, |entry| {
        if let Some((uid_val, weight_val)) = extract_pair(entry) {
            let uid = decode_u16(uid_val)?;
            let weight = decode_u16(weight_val)?;
            return Ok((uid, weight));
        }
        Err(anyhow::anyhow!("invalid weight pair"))
    })
}
fn extract_pair(value: &Value) -> Option<(&Value, &Value)> {
    match &value.value {
        ValueDef::Composite(Composite::Named(fields)) => {
            if fields.len() >= 2 {
                Some((&fields[0].1, &fields[1].1))
            } else {
                None
            }
        }
        ValueDef::Composite(Composite::Unnamed(values)) => {
            if values.len() >= 2 {
                Some((&values[0], &values[1]))
            } else {
                None
            }
        }
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
