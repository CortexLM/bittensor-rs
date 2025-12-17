//! Chain information queries

use crate::error::{Error, Result};
use scale_value::{Composite, Primitive, Value, ValueDef};
use subxt::backend::legacy::LegacyRpcMethods;
use subxt::backend::rpc::RpcClient;
use subxt::dynamic::Value as DynValue;
use subxt::OnlineClient;
use subxt::PolkadotConfig;

/// Get current block number
pub async fn get_block_number(client: &OnlineClient<PolkadotConfig>) -> Result<u64> {
    let block = client.blocks().at_latest().await?;
    Ok(block.number() as u64)
}

/// Get block hash for a block number using the OnlineClient
///
/// This function queries the chain for the block hash at a specific block number.
/// Returns `None` if the block doesn't exist.
pub async fn get_block_hash(
    client: &OnlineClient<PolkadotConfig>,
    block_number: u64,
) -> Result<Option<String>> {
    // Get latest block first to check bounds
    let latest = client.blocks().at_latest().await?;
    let latest_number = latest.number() as u64;
    
    if block_number > latest_number {
        return Ok(None);
    }
    
    // If requesting the latest block, return its hash directly
    if block_number == latest_number {
        return Ok(Some(format!("0x{}", hex::encode(latest.hash().0))));
    }
    
    // For historical blocks, return None - this requires an archive node
    // The caller should use get_block_hash_with_rpc for historical blocks
    Ok(None)
}

/// Get block hash for a block number using direct RPC call
///
/// This function uses the LegacyRpcMethods to make a direct RPC call,
/// which works with archive nodes for historical blocks.
pub async fn get_block_hash_with_rpc(
    rpc: &LegacyRpcMethods<PolkadotConfig>,
    block_number: u64,
) -> Result<Option<String>> {
    let hash = rpc
        .chain_get_block_hash(Some(block_number.into()))
        .await
        .map_err(|e| Error::query(format!("RPC call failed: {}", e)))?;
    
    Ok(hash.map(|h| format!("0x{}", hex::encode(h.0))))
}

/// Create LegacyRpcMethods from an endpoint URL
pub async fn create_rpc_methods(endpoint: &str) -> Result<LegacyRpcMethods<PolkadotConfig>> {
    let rpc_client = RpcClient::from_url(endpoint)
        .await
        .map_err(|e| Error::connection(format!("Failed to create RPC client: {}", e)))?;
    Ok(LegacyRpcMethods::new(rpc_client))
}

/// Query a storage map value
pub async fn query_storage_value(
    client: &OnlineClient<PolkadotConfig>,
    pallet: &str,
    entry: &str,
    keys: Vec<DynValue>,
) -> Result<Option<subxt::dynamic::DecodedValueThunk>> {
    use subxt::dynamic::storage;
    
    let storage_query = storage(pallet, entry, keys);
    let result = client
        .storage()
        .at_latest()
        .await?
        .fetch(&storage_query)
        .await?;
    
    Ok(result)
}

/// Decode a u64 from a dynamic value
pub fn decode_u64(value: &subxt::dynamic::DecodedValueThunk) -> Option<u64> {
    let val = value.to_value().ok()?;
    extract_u64_generic(&val)
}

/// Extract u64 from scale_value::Value (generic over type param)
fn extract_u64_generic<T>(val: &Value<T>) -> Option<u64> {
    match &val.value {
        ValueDef::Primitive(Primitive::U128(n)) => Some(*n as u64),
        ValueDef::Primitive(Primitive::U256(n)) => {
            // U256 is [u8; 32], take last 8 bytes as u64 (little endian interpretation)
            let bytes: [u8; 8] = n[0..8].try_into().ok()?;
            Some(u64::from_le_bytes(bytes))
        }
        _ => None,
    }
}

/// Decode a u16 from a dynamic value
pub fn decode_u16(value: &subxt::dynamic::DecodedValueThunk) -> Option<u16> {
    let val = value.to_value().ok()?;
    extract_u16_generic(&val)
}

/// Extract u16 from scale_value::Value (generic over type param)
fn extract_u16_generic<T>(val: &Value<T>) -> Option<u16> {
    match &val.value {
        ValueDef::Primitive(Primitive::U128(n)) => Some(*n as u16),
        ValueDef::Primitive(Primitive::U256(n)) => {
            let bytes: [u8; 2] = n[0..2].try_into().ok()?;
            Some(u16::from_le_bytes(bytes))
        }
        _ => None,
    }
}

/// Decode a bool from a dynamic value
pub fn decode_bool(value: &subxt::dynamic::DecodedValueThunk) -> Option<bool> {
    let val = value.to_value().ok()?;
    extract_bool_generic(&val)
}

/// Extract bool from scale_value::Value (generic over type param)
fn extract_bool_generic<T>(val: &Value<T>) -> Option<bool> {
    match &val.value {
        ValueDef::Primitive(Primitive::Bool(b)) => Some(*b),
        _ => None,
    }
}

/// Extract account bytes from scale_value::Value (generic over type param)
pub fn extract_account_bytes<T>(val: &Value<T>) -> Option<[u8; 32]> {
    match &val.value {
        ValueDef::Composite(Composite::Unnamed(values)) => {
            if let Some(first) = values.first() {
                if let ValueDef::Primitive(Primitive::U256(bytes)) = &first.value {
                    return Some(*bytes);
                }
            }
            None
        }
        ValueDef::Primitive(Primitive::U256(bytes)) => Some(*bytes),
        _ => None,
    }
}
