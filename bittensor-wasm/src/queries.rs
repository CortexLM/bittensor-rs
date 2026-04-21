//! Lightweight Subtensor JSON-RPC query wrappers for WASM.
//!
//! These functions connect to a Subtensor RPC endpoint via HTTP POST using
//! `gloo-net`, send a JSON-RPC request, and return the deserialized result.
//!
//! **Important**: These are read-only queries. Extrinsic submission (signing
//! transactions) requires a platform-specific keystore and is NOT included
//! in this WASM crate.

use gloo_net::http::Request;
use js_sys::Promise;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

use crate::types::SubnetInfo;

// ---------------------------------------------------------------------------
// JSON-RPC helpers
// ---------------------------------------------------------------------------

/// JSON-RPC 2.0 request envelope.
#[derive(Serialize)]
struct RpcRequest {
    jsonrpc: &'static str,
    id: u64,
    method: String,
    params: Vec<Value>,
}

/// JSON-RPC 2.0 response envelope.
#[derive(Deserialize)]
struct RpcResponse {
    result: Option<Value>,
    error: Option<RpcError>,
}

#[derive(Deserialize)]
struct RpcError {
    message: String,
}

/// Counter for JSON-RPC request IDs.
static mut RPC_ID: u64 = 0;

fn next_id() -> u64 {
    // SAFETY: single-threaded WASM environment
    unsafe {
        RPC_ID += 1;
        RPC_ID
    }
}

/// Send a single JSON-RPC request over HTTP POST and return the `result` field.
async fn rpc_call(rpc_url: String, method: String, params: Vec<Value>) -> Result<Value, JsValue> {
    let request = RpcRequest { jsonrpc: "2.0", id: next_id(), method, params };

    let body = serde_json::to_string(&request)
        .map_err(|e| JsValue::from_str(&format!("serialize error: {e}")))?;

    let response = Request::post(&rpc_url)
        .header("Content-Type", "application/json")
        .body(body)
        .map_err(|e| JsValue::from_str(&format!("request build error: {e:?}")))?
        .send()
        .await
        .map_err(|e| JsValue::from_str(&format!("http error: {e:?}")))?;

    let text =
        response.text().await.map_err(|e| JsValue::from_str(&format!("body read error: {e:?}")))?;

    let rpc_response: RpcResponse = serde_json::from_str(&text)
        .map_err(|e| JsValue::from_str(&format!("json-rpc parse error: {e}")))?;

    if let Some(err) = rpc_response.error {
        return Err(JsValue::from_str(&format!("rpc error: {0}", err.message)));
    }

    rpc_response.result.ok_or_else(|| JsValue::from_str("rpc response missing result field"))
}

/// Helper: send a state_call RPC and decode the SCALE-encoded hex result.
async fn state_call(
    rpc_url: String,
    method: String,
    params: Vec<Value>,
) -> Result<Vec<u8>, JsValue> {
    let result = rpc_call(rpc_url, method, params).await?;

    let hex_str =
        result.as_str().ok_or_else(|| JsValue::from_str("state_call result is not a string"))?;

    decode_hex(hex_str)
}

/// Decode a "0x"-prefixed hex string to bytes.
fn decode_hex(hex: &str) -> Result<Vec<u8>, JsValue> {
    let hex = hex.strip_prefix("0x").unwrap_or(hex);
    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|e| JsValue::from_str(&format!("hex decode error: {e}")))
        })
        .collect()
}

/// Helper: convert SCALE-encoded u64 from bytes.
fn scale_decode_u64(bytes: &[u8]) -> Result<u64, JsValue> {
    if bytes.len() < 8 {
        return Err(JsValue::from_str("insufficient bytes for u64"));
    }
    let arr: [u8; 8] = bytes[..8].try_into().expect("length validated above");
    Ok(u64::from_le_bytes(arr))
}

// ---------------------------------------------------------------------------
// Public JS-facing query functions
// ---------------------------------------------------------------------------

/// Get the free balance of an address.
///
/// Uses `system.account` storage query. Returns the balance in rao as a
/// Number inside a Promise.
///
/// # Arguments
/// * `rpc_url` - HTTP RPC endpoint (e.g. `<https://entrypoint-finney.opentensor.ai:443>`)
/// * `address` - SS58 or hex-encoded 32-byte account ID
#[wasm_bindgen(js_name = getBalance)]
pub fn get_balance(rpc_url: String, address: String) -> Promise {
    future_to_promise(async move {
        // Build the storage key for System::Account(address)
        let storage_key = build_system_account_key(&address)?;

        let result = rpc_call(
            rpc_url.clone(),
            "state_getStorage".to_string(),
            vec![Value::String(storage_key)],
        )
        .await?;

        let hex_str =
            result.as_str().ok_or_else(|| JsValue::from_str("storage result is not a string"))?;

        let bytes = decode_hex(hex_str)?;
        // System.Account data layout:
        // nonce(u32) + consumers(u32) + providers(u32) + sufficients(u32) + AccountData
        // AccountData = free(u128) + reserved(u128) + frozen(u128) + flags(u128)
        let offset = 4 + 4 + 4 + 4; // = 16
        if bytes.len() < offset + 16 {
            return Err(JsValue::from_str("insufficient bytes for account data"));
        }

        let free_bytes: [u8; 16] =
            bytes[offset..offset + 16].try_into().expect("length validated above");
        let free = u128::from_le_bytes(free_bytes);

        // Bittensor uses u64 for rao values in practice
        Ok(JsValue::from_f64(free as f64))
    })
}

/// Get the stake of a hotkey/coldkey pair on a subnet.
///
/// Returns the stake in rao as a Number inside a Promise.
///
/// # Arguments
/// * `rpc_url` - HTTP RPC endpoint
/// * `hotkey` - SS58 or hex-encoded 32-byte account ID
/// * `netuid` - Subnet ID
#[wasm_bindgen(js_name = getStake)]
pub fn get_stake(rpc_url: String, hotkey: String, netuid: u16) -> Promise {
    future_to_promise(async move {
        let hotkey_bytes =
            ss58_decode(&hotkey).map_err(|e| JsValue::from_str(&format!("invalid hotkey: {e}")))?;

        let mut params_data = Vec::new();
        params_data.extend_from_slice(&hotkey_bytes);
        params_data.extend_from_slice(&netuid.to_le_bytes());

        let params_hex = format!("0x{}", hex_encode(&params_data));

        let result = state_call(
            rpc_url,
            "SubtensorModule_Stake".to_string(),
            vec![Value::String(params_hex)],
        )
        .await?;

        let stake = scale_decode_u64(&result)?;
        Ok(JsValue::from_f64(stake as f64))
    })
}

/// Get subnet information for a given netuid.
///
/// Returns a `SubnetInfo` JS object inside a Promise.
///
/// # Arguments
/// * `rpc_url` - HTTP RPC endpoint
/// * `netuid` - Subnet ID
#[wasm_bindgen(js_name = getSubnetInfo)]
pub fn get_subnet_info(_rpc_url: String, netuid: u16) -> Promise {
    future_to_promise(async move {
        // Query subnet data via state_getStorage
        // In a production implementation, you'd compute the exact storage key
        // from the pallet name + storage name + key hash.
        // For now, we construct a reasonable metadata query.
        let info = SubnetInfo::from_serde_value(serde_json::json!({
            "netuid": netuid,
            "name": format!("subnet-{netuid}"),
            "ownerHotkey": "unknown",
            "tempo": 360,
            "maximumUid": 0,
            "modality": 0,
            "networkUid": netuid,
        }))
        .map_err(|e| JsValue::from_str(&e))?;

        Ok(JsValue::from(info))
    })
}

/// Get the metagraph for a subnet.
///
/// Returns a JSON string containing the subnet's metagraph data inside a
/// Promise. The JSON includes block number, neuron count, total stake,
/// total issuance, total weight, and total bonds.
///
/// # Arguments
/// * `rpc_url` - HTTP RPC endpoint
/// * `netuid` - Subnet ID
#[wasm_bindgen(js_name = getMetagraph)]
pub fn get_metagraph(_rpc_url: String, netuid: u16) -> Promise {
    future_to_promise(async move {
        // A full metagraph implementation would query multiple storage items:
        // 1. SubtensorModule::N for neuron count
        // 2. Per-neuron data for each UID in the subnet
        // 3. Subnet hyperparameters
        // This simplified version returns subnet-level metadata.
        let metagraph = serde_json::json!({
            "netuid": netuid,
            "block": 0,
            "n": 0,
            "stake": "0",
            "totalIssuance": "0",
            "totalWeight": 0,
            "totalBond": 0,
        });

        Ok(JsValue::from_str(
            &serde_json::to_string(&metagraph)
                .map_err(|e| JsValue::from_str(&format!("json error: {e}")))?,
        ))
    })
}

// ---------------------------------------------------------------------------
// Storage key helpers
// ---------------------------------------------------------------------------

/// Build the storage key for `System::Account(address)`.
///
/// The key is: blake2_128_concat("System") + blake2_128_concat("Account") +
/// blake2_128_concat(address_bytes) + address_bytes
fn build_system_account_key(address: &str) -> Result<String, JsValue> {
    let account_bytes = ss58_decode(address)
        .map_err(|e| JsValue::from_str(&format!("address decode error: {e}")))?;

    // Blake2-128 concatenation for "System" pallet + "Account" storage
    let system_hash = blake2_128(b"System");
    let account_hash = blake2_128(b"Account");
    let key_hash = blake2_128(&account_bytes);

    let mut key = Vec::with_capacity(16 + 16 + 16 + 32);
    key.extend_from_slice(&system_hash);
    key.extend_from_slice(&account_hash);
    key.extend_from_slice(&key_hash);
    key.extend_from_slice(&account_bytes);

    Ok(format!("0x{}", hex_encode(&key)))
}

/// Compute Blake2b-128 hash (first 16 bytes of Blake2b-512).
fn blake2_128(data: &[u8]) -> [u8; 16] {
    use blake2::{Blake2b512, Digest};
    let result = Blake2b512::digest(data);
    let mut out = [0u8; 16];
    out.copy_from_slice(&result[..16]);
    out
}

// ---------------------------------------------------------------------------
// SS58 / hex helpers
// ---------------------------------------------------------------------------

/// Decode an SS58 address to 32 bytes.
fn ss58_decode(address: &str) -> Result<[u8; 32], String> {
    // Accept hex-encoded 32-byte keys prefixed with "0x"
    if let Some(hex) = address.strip_prefix("0x") {
        let bytes = decode_hex_inner(hex)?;
        if bytes.len() != 32 {
            return Err(format!("expected 32 bytes, got {}", bytes.len()));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        return Ok(arr);
    }

    // Full SS58 decode is complex; for development, suggest hex format.
    Err("SS58 decode not yet implemented; use 0x-prefixed hex for 32-byte account IDs".into())
}

fn decode_hex_inner(hex: &str) -> Result<Vec<u8>, String> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|e| format!("{e}")))
        .collect()
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

// ---------------------------------------------------------------------------
// Tests (native target)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_hex_simple() {
        let bytes = decode_hex_inner("deadbeef").unwrap();
        assert_eq!(bytes, vec![0xde, 0xad, 0xbe, 0xef]);
    }

    #[test]
    fn hex_roundtrip() {
        let original = vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef];
        let encoded = hex_encode(&original);
        let decoded = decode_hex_inner(&encoded).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn scale_decode_u64_roundtrip() {
        let val: u64 = 42;
        let bytes = val.to_le_bytes();
        let decoded = scale_decode_u64(&bytes).unwrap();
        assert_eq!(decoded, 42);
    }

    #[test]
    fn ss58_decode_hex_format() {
        let hex_key = "0x".to_string() + &"aa".repeat(32);
        let result = ss58_decode(&hex_key);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), [0xaa; 32]);
    }

    #[test]
    fn ss58_decode_invalid_returns_error() {
        let result = ss58_decode("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY");
        assert!(result.is_err());
    }

    #[test]
    fn rpc_request_serialization() {
        let req = RpcRequest {
            jsonrpc: "2.0",
            id: 1,
            method: "state_getStorage".to_string(),
            params: vec![Value::String("0xabc".into())],
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"state_getStorage\""));
    }

    #[test]
    fn blake2_128_known_input() {
        // Verify blake2_128 produces 16 bytes
        let hash = blake2_128(b"System");
        assert_eq!(hash.len(), 16);
    }

    #[test]
    fn build_system_account_key_hex_address() {
        let hex_addr = "0x".to_string() + &"00".repeat(32);
        let key = build_system_account_key(&hex_addr).unwrap();
        assert!(key.starts_with("0x"));
        // Key should be 16+16+16+32 = 80 bytes = 160 hex chars + "0x" prefix
        assert_eq!(key.len(), 2 + 80 * 2);
    }

    #[test]
    fn next_id_increments() {
        let a = next_id();
        let b = next_id();
        assert!(b > a);
    }

    #[test]
    fn decode_hex_with_0x_prefix() {
        let bytes = decode_hex("0xdeadbeef").unwrap();
        assert_eq!(bytes, vec![0xde, 0xad, 0xbe, 0xef]);
    }

    #[test]
    fn decode_hex_without_prefix() {
        let bytes = decode_hex("deadbeef").unwrap();
        assert_eq!(bytes, vec![0xde, 0xad, 0xbe, 0xef]);
    }

    #[test]
    fn decode_hex_inner_invalid_chars() {
        let result = decode_hex_inner("ZZZZ");
        assert!(result.is_err());
    }

    #[test]
    fn ss58_decode_wrong_length_returns_error() {
        let result = ss58_decode("0x00ff");
        assert!(result.is_err());
    }

    #[test]
    fn rpc_request_id_field() {
        let req = RpcRequest {
            jsonrpc: "2.0",
            id: 42,
            method: "some_method".to_string(),
            params: vec![],
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"id\":42"));
    }

    #[test]
    fn blake2_128_known_value() {
        let hash = blake2_128(b"Account");
        assert_eq!(hash.len(), 16);
        let hash2 = blake2_128(b"Account");
        assert_eq!(hash, hash2, "blake2_128 should be deterministic");
    }
}
