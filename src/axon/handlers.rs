//! Request handlers for the Axon HTTP server
//!
//! This module provides the core request handling logic including
//! signature verification, synapse extraction, and response building.

use crate::dendrite::request::header_names;
use crate::errors::{AxonError, SynapseUnauthorized};
use crate::types::{Synapse, TerminalInfo};
use axum::body::Bytes;
use axum::response::{IntoResponse, Response};
use http::{HeaderMap, HeaderValue, StatusCode};
use sha2::{Digest, Sha256};
use sp_core::{sr25519, Pair};
use std::time::Instant;

/// Bittensor protocol version
pub const AXON_VERSION: u64 = 100;

/// Status codes matching the Python SDK
pub mod status_codes {
    pub const SUCCESS: i32 = 200;
    pub const UNAUTHORIZED: i32 = 401;
    pub const FORBIDDEN: i32 = 403;
    pub const NOT_FOUND: i32 = 404;
    pub const TIMEOUT: i32 = 408;
    pub const INTERNAL_ERROR: i32 = 500;
    pub const SERVICE_UNAVAILABLE: i32 = 503;
}

/// Status messages for response headers
pub mod status_messages {
    pub const SUCCESS: &str = "Success";
    pub const UNAUTHORIZED: &str = "Signature verification failed";
    pub const FORBIDDEN: &str = "Blacklisted";
    pub const NOT_FOUND: &str = "Synapse not found";
    pub const TIMEOUT: &str = "Request timeout";
    pub const INTERNAL_ERROR: &str = "Internal server error";
    pub const SERVICE_UNAVAILABLE: &str = "Service unavailable";
}

/// Verified request information extracted from headers
#[derive(Debug, Clone)]
pub struct VerifiedRequest {
    /// The dendrite's hotkey SS58 address
    pub dendrite_hotkey: String,
    /// Request nonce for replay protection
    pub nonce: u64,
    /// Signature from the dendrite
    pub signature: String,
    /// Request UUID
    pub uuid: String,
    /// Computed body hash
    pub body_hash: String,
}

/// Extract and verify a request from headers
///
/// # Arguments
///
/// * `headers` - The HTTP request headers
/// * `body` - The request body bytes
/// * `axon_hotkey` - The axon's hotkey SS58 address for verification
///
/// # Returns
///
/// A VerifiedRequest if verification succeeds, or an error
pub fn verify_request(
    headers: &HeaderMap,
    body: &[u8],
    axon_hotkey: &str,
) -> Result<VerifiedRequest, SynapseUnauthorized> {
    // Extract required headers
    let dendrite_hotkey = get_header_string(headers, header_names::DENDRITE_HOTKEY)
        .ok_or_else(|| SynapseUnauthorized {
            message: "Missing dendrite hotkey header".to_string(),
            hotkey: None,
        })?;

    let nonce_str =
        get_header_string(headers, header_names::DENDRITE_NONCE).ok_or_else(|| {
            SynapseUnauthorized {
                message: "Missing dendrite nonce header".to_string(),
                hotkey: Some(dendrite_hotkey.clone()),
            }
        })?;

    let nonce: u64 = nonce_str.parse().map_err(|_| SynapseUnauthorized {
        message: format!("Invalid nonce format: {}", nonce_str),
        hotkey: Some(dendrite_hotkey.clone()),
    })?;

    let signature =
        get_header_string(headers, header_names::DENDRITE_SIGNATURE).ok_or_else(|| {
            SynapseUnauthorized {
                message: "Missing dendrite signature header".to_string(),
                hotkey: Some(dendrite_hotkey.clone()),
            }
        })?;

    let uuid = get_header_string(headers, header_names::DENDRITE_UUID).unwrap_or_default();

    // Compute body hash
    let body_hash = compute_body_hash(body);

    // Verify signature
    verify_signature(&dendrite_hotkey, nonce, axon_hotkey, &body_hash, &signature).map_err(
        |e| SynapseUnauthorized {
            message: e.to_string(),
            hotkey: Some(dendrite_hotkey.clone()),
        },
    )?;

    Ok(VerifiedRequest {
        dendrite_hotkey,
        nonce,
        signature,
        uuid,
        body_hash,
    })
}

/// Verify a request signature
///
/// The signature format matches the Python SDK:
/// `sign(message = "{nonce}.{dendrite_hotkey}.{axon_hotkey}.{body_hash}")`
///
/// # Arguments
///
/// * `dendrite_hotkey` - The dendrite's hotkey SS58 address
/// * `nonce` - The request nonce
/// * `axon_hotkey` - The axon's hotkey SS58 address
/// * `body_hash` - The SHA-256 hash of the request body
/// * `signature` - The hex-encoded signature
///
/// # Returns
///
/// Ok(()) if verification succeeds, or an error
pub fn verify_signature(
    dendrite_hotkey: &str,
    nonce: u64,
    axon_hotkey: &str,
    body_hash: &str,
    signature: &str,
) -> Result<(), AxonError> {
    // Decode the signature from hex
    let sig_bytes =
        hex::decode(signature).map_err(|e| AxonError::new(format!("Invalid signature hex: {}", e)))?;

    if sig_bytes.len() != 64 {
        return Err(AxonError::new(format!(
            "Invalid signature length: expected 64 bytes, got {}",
            sig_bytes.len()
        )));
    }

    let mut sig_arr = [0u8; 64];
    sig_arr.copy_from_slice(&sig_bytes);
    let sig = sr25519::Signature::from_raw(sig_arr);

    // Decode the dendrite's public key from SS58
    let public = ss58_to_public(dendrite_hotkey)
        .map_err(|e| AxonError::new(format!("Invalid dendrite hotkey: {}", e)))?;

    // Create the message to verify
    let message = format!("{}.{}.{}.{}", nonce, dendrite_hotkey, axon_hotkey, body_hash);

    // Verify the signature
    if sr25519::Pair::verify(&sig, message.as_bytes(), &public) {
        Ok(())
    } else {
        Err(AxonError::new("Signature verification failed"))
    }
}

/// Decode an SS58 address to a public key
fn ss58_to_public(ss58: &str) -> Result<sr25519::Public, String> {
    use sp_core::crypto::Ss58Codec;
    sr25519::Public::from_ss58check(ss58).map_err(|e| format!("{:?}", e))
}

/// Compute SHA-256 hash of data and return as hex string
pub fn compute_body_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Extract a synapse from request headers and body
///
/// # Arguments
///
/// * `headers` - The HTTP request headers
/// * `body` - The request body bytes
///
/// # Returns
///
/// The extracted Synapse
pub fn extract_synapse(headers: &HeaderMap, body: &[u8]) -> Result<Synapse, AxonError> {
    // Parse body as JSON extra fields
    let extra: std::collections::HashMap<String, serde_json::Value> = if body.is_empty() {
        std::collections::HashMap::new()
    } else {
        serde_json::from_slice(body)
            .map_err(|e| AxonError::new(format!("Invalid JSON body: {}", e)))?
    };

    // Build dendrite terminal info from headers
    let dendrite = TerminalInfo {
        ip: get_header_string(headers, header_names::DENDRITE_IP),
        port: get_header_u16(headers, header_names::DENDRITE_PORT),
        version: get_header_u64(headers, header_names::DENDRITE_VERSION),
        nonce: get_header_u64(headers, header_names::DENDRITE_NONCE),
        uuid: get_header_string(headers, header_names::DENDRITE_UUID),
        hotkey: get_header_string(headers, header_names::DENDRITE_HOTKEY),
        signature: get_header_string(headers, header_names::DENDRITE_SIGNATURE),
        ..Default::default()
    };

    Ok(Synapse {
        name: get_header_string(headers, header_names::NAME),
        timeout: get_header_f64(headers, header_names::TIMEOUT),
        total_size: get_header_u64(headers, header_names::TOTAL_SIZE),
        header_size: get_header_u64(headers, header_names::HEADER_SIZE),
        computed_body_hash: get_header_string(headers, header_names::BODY_HASH),
        dendrite: Some(dendrite),
        axon: Some(TerminalInfo::default()),
        extra,
    })
}

/// Build response headers for a synapse response
///
/// # Arguments
///
/// * `hotkey` - The axon's hotkey SS58 address
/// * `status_code` - The response status code
/// * `status_message` - The response status message
/// * `process_time` - Processing time in seconds
///
/// # Returns
///
/// HeaderMap with all required response headers
pub fn build_response_headers(
    hotkey: &str,
    status_code: i32,
    status_message: &str,
    process_time: f64,
) -> HeaderMap {
    let mut headers = HeaderMap::new();

    // Nonce for response
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);

    // Add axon headers
    if let Ok(hv) = HeaderValue::from_str(&status_code.to_string()) {
        headers.insert(header_names::AXON_STATUS_CODE, hv);
    }
    if let Ok(hv) = HeaderValue::from_str(status_message) {
        headers.insert(header_names::AXON_STATUS_MESSAGE, hv);
    }
    if let Ok(hv) = HeaderValue::from_str(&format!("{:.6}", process_time)) {
        headers.insert(header_names::AXON_PROCESS_TIME, hv);
    }
    if let Ok(hv) = HeaderValue::from_str(hotkey) {
        headers.insert(header_names::AXON_HOTKEY, hv);
    }
    if let Ok(hv) = HeaderValue::from_str(&AXON_VERSION.to_string()) {
        headers.insert(header_names::AXON_VERSION, hv);
    }
    if let Ok(hv) = HeaderValue::from_str(&nonce.to_string()) {
        headers.insert(header_names::AXON_NONCE, hv);
    }

    headers
}

/// Build an error response
///
/// # Arguments
///
/// * `hotkey` - The axon's hotkey SS58 address
/// * `status_code` - The HTTP status code
/// * `bt_status_code` - The Bittensor status code
/// * `message` - The error message
/// * `process_time` - Processing time in seconds
///
/// # Returns
///
/// An axum Response
pub fn build_error_response(
    hotkey: &str,
    status_code: StatusCode,
    bt_status_code: i32,
    message: &str,
    process_time: f64,
) -> Response {
    let headers = build_response_headers(hotkey, bt_status_code, message, process_time);
    (status_code, headers, message.to_string()).into_response()
}

/// Build a success response with JSON body
///
/// # Arguments
///
/// * `hotkey` - The axon's hotkey SS58 address
/// * `body` - The response body
/// * `process_time` - Processing time in seconds
///
/// # Returns
///
/// An axum Response
pub fn build_success_response(hotkey: &str, body: Bytes, process_time: f64) -> Response {
    let headers = build_response_headers(
        hotkey,
        status_codes::SUCCESS,
        status_messages::SUCCESS,
        process_time,
    );
    (StatusCode::OK, headers, body).into_response()
}

// Helper functions for header extraction

fn get_header_string(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

fn get_header_u64(headers: &HeaderMap, name: &str) -> Option<u64> {
    get_header_string(headers, name).and_then(|s| s.parse().ok())
}

fn get_header_u16(headers: &HeaderMap, name: &str) -> Option<u16> {
    get_header_string(headers, name).and_then(|s| s.parse().ok())
}

fn get_header_f64(headers: &HeaderMap, name: &str) -> Option<f64> {
    get_header_string(headers, name).and_then(|s| s.parse().ok())
}

/// Handler context for processing requests
#[derive(Clone)]
pub struct HandlerContext {
    /// The axon's hotkey
    pub hotkey: String,
    /// Request start time
    pub start_time: Instant,
}

impl HandlerContext {
    /// Create a new handler context
    pub fn new(hotkey: impl Into<String>) -> Self {
        Self {
            hotkey: hotkey.into(),
            start_time: Instant::now(),
        }
    }

    /// Get elapsed time in seconds
    pub fn elapsed_secs(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_body_hash() {
        let data = b"test data";
        let hash = compute_body_hash(data);
        // SHA-256 hash should be 64 hex characters
        assert_eq!(hash.len(), 64);
        // Should be deterministic
        assert_eq!(hash, compute_body_hash(data));
    }

    #[test]
    fn test_build_response_headers() {
        let headers = build_response_headers(
            "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
            200,
            "Success",
            0.123456,
        );

        assert!(headers.contains_key(header_names::AXON_STATUS_CODE));
        assert!(headers.contains_key(header_names::AXON_STATUS_MESSAGE));
        assert!(headers.contains_key(header_names::AXON_PROCESS_TIME));
        assert!(headers.contains_key(header_names::AXON_HOTKEY));
        assert!(headers.contains_key(header_names::AXON_VERSION));
        assert!(headers.contains_key(header_names::AXON_NONCE));
    }

    #[test]
    fn test_extract_synapse_empty_body() {
        let mut headers = HeaderMap::new();
        headers.insert(header_names::NAME, "TestSynapse".parse().unwrap());
        headers.insert(header_names::TIMEOUT, "12.0".parse().unwrap());

        let synapse = extract_synapse(&headers, &[]).unwrap();

        assert_eq!(synapse.name, Some("TestSynapse".to_string()));
        assert_eq!(synapse.timeout, Some(12.0));
        assert!(synapse.extra.is_empty());
    }

    #[test]
    fn test_extract_synapse_with_body() {
        let headers = HeaderMap::new();
        let body = br#"{"key": "value"}"#;

        let synapse = extract_synapse(&headers, body).unwrap();

        assert!(synapse.extra.contains_key("key"));
        assert_eq!(
            synapse.extra.get("key"),
            Some(&serde_json::json!("value"))
        );
    }

    #[test]
    fn test_handler_context() {
        let ctx = HandlerContext::new("test_hotkey");
        assert_eq!(ctx.hotkey, "test_hotkey");
        // Small sleep to ensure elapsed time is > 0
        std::thread::sleep(std::time::Duration::from_millis(1));
        assert!(ctx.elapsed_secs() > 0.0);
    }

    #[test]
    fn test_status_codes() {
        assert_eq!(status_codes::SUCCESS, 200);
        assert_eq!(status_codes::UNAUTHORIZED, 401);
        assert_eq!(status_codes::FORBIDDEN, 403);
        assert_eq!(status_codes::TIMEOUT, 408);
        assert_eq!(status_codes::INTERNAL_ERROR, 500);
    }
}
