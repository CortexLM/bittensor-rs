//! Request building and signing for Dendrite HTTP requests
//!
//! This module handles the construction of HTTP requests to Axon servers,
//! including header generation, body hashing, and cryptographic signing.

use crate::types::{AxonInfo, Synapse, SynapseHeaders, TerminalInfo};
use http::header::HeaderMap;
use sha2::{Digest, Sha256};
use sp_core::{sr25519, Pair};
use std::time::Duration;
use thiserror::Error;

/// Errors that can occur during request building
#[derive(Debug, Error)]
pub enum RequestError {
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("Signing error: {0}")]
    Signing(String),
    #[error("Invalid header value: {0}")]
    InvalidHeader(String),
}

/// A prepared Dendrite request ready for transmission
#[derive(Debug, Clone)]
pub struct DendriteRequest {
    /// The target URL for the request
    pub url: String,
    /// Synapse headers for the request
    pub headers: SynapseHeaders,
    /// Serialized request body
    pub body: Vec<u8>,
    /// Request timeout
    pub timeout: Duration,
}

impl DendriteRequest {
    /// Create a new DendriteRequest from an AxonInfo and Synapse
    ///
    /// # Arguments
    ///
    /// * `axon` - The target Axon server information
    /// * `synapse` - The Synapse to send
    /// * `dendrite_info` - Terminal info for the dendrite (sender)
    /// * `timeout` - Request timeout duration
    ///
    /// # Returns
    ///
    /// A prepared request or an error
    pub fn new(
        axon: &AxonInfo,
        synapse: &Synapse,
        dendrite_info: &TerminalInfo,
        timeout: Duration,
    ) -> Result<Self, RequestError> {
        // Build the endpoint URL
        let synapse_name = synapse.name.as_deref().unwrap_or("Synapse");
        let url = format!("{}/{}", axon.to_endpoint(), synapse_name);

        // Serialize the synapse body (just the extra fields, not the headers)
        let body = serde_json::to_vec(&synapse.extra)
            .map_err(|e| RequestError::Serialization(e.to_string()))?;

        // Build initial headers from synapse
        let mut headers = synapse.to_headers();

        // Add dendrite terminal info
        headers.dendrite_ip = dendrite_info.ip.clone();
        headers.dendrite_port = dendrite_info.port.map(|p| p.to_string());
        headers.dendrite_version = dendrite_info.version.map(|v| v.to_string());
        headers.dendrite_nonce = dendrite_info.nonce.map(|n| n.to_string());
        headers.dendrite_uuid = dendrite_info.uuid.clone();
        headers.dendrite_hotkey = dendrite_info.hotkey.clone();

        // Set timeout in headers
        headers.timeout = Some(timeout.as_secs_f64().to_string());

        Ok(Self {
            url,
            headers,
            body,
            timeout,
        })
    }

    /// Compute the SHA-256 hash of the request body
    ///
    /// # Returns
    ///
    /// Hexadecimal string of the body hash
    pub fn compute_body_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(&self.body);
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Sign the request with the given keypair
    ///
    /// The signature format matches the Python SDK:
    /// `sign(message = "{nonce}.{dendrite_hotkey}.{axon_hotkey}.{body_hash}")`
    ///
    /// # Arguments
    ///
    /// * `keypair` - The SR25519 keypair to sign with
    /// * `axon_hotkey` - The axon's hotkey SS58 address
    ///
    /// # Returns
    ///
    /// Ok(()) if signing succeeds, otherwise an error
    pub fn sign(&mut self, keypair: &sr25519::Pair, axon_hotkey: &str) -> Result<(), RequestError> {
        // Compute body hash
        let body_hash = self.compute_body_hash();
        self.headers.computed_body_hash = Some(body_hash.clone());

        // Get nonce from headers
        let nonce = self
            .headers
            .dendrite_nonce
            .as_ref()
            .ok_or_else(|| RequestError::Signing("Missing nonce".to_string()))?;

        // Get dendrite hotkey from headers
        let dendrite_hotkey = self
            .headers
            .dendrite_hotkey
            .as_ref()
            .ok_or_else(|| RequestError::Signing("Missing dendrite hotkey".to_string()))?;

        // Create message to sign: "{nonce}.{dendrite_hotkey}.{axon_hotkey}.{body_hash}"
        let message = format!(
            "{}.{}.{}.{}",
            nonce, dendrite_hotkey, axon_hotkey, body_hash
        );

        // Sign the message
        let signature = keypair.sign(message.as_bytes());
        self.headers.dendrite_signature = Some(hex::encode(signature.0));

        Ok(())
    }
}

/// Header name constants matching Python SDK
pub mod header_names {
    pub const DENDRITE_IP: &str = "bt_header_dendrite_ip";
    pub const DENDRITE_PORT: &str = "bt_header_dendrite_port";
    pub const DENDRITE_VERSION: &str = "bt_header_dendrite_version";
    pub const DENDRITE_NONCE: &str = "bt_header_dendrite_nonce";
    pub const DENDRITE_UUID: &str = "bt_header_dendrite_uuid";
    pub const DENDRITE_HOTKEY: &str = "bt_header_dendrite_hotkey";
    pub const DENDRITE_SIGNATURE: &str = "bt_header_dendrite_signature";
    pub const AXON_IP: &str = "bt_header_axon_ip";
    pub const AXON_PORT: &str = "bt_header_axon_port";
    pub const AXON_VERSION: &str = "bt_header_axon_version";
    pub const AXON_NONCE: &str = "bt_header_axon_nonce";
    pub const AXON_UUID: &str = "bt_header_axon_uuid";
    pub const AXON_HOTKEY: &str = "bt_header_axon_hotkey";
    pub const AXON_SIGNATURE: &str = "bt_header_axon_signature";
    pub const AXON_STATUS_CODE: &str = "bt_header_axon_status_code";
    pub const AXON_STATUS_MESSAGE: &str = "bt_header_axon_status_message";
    pub const AXON_PROCESS_TIME: &str = "bt_header_axon_process_time";
    pub const INPUT_OBJ: &str = "bt_header_input_obj";
    pub const OUTPUT_OBJ: &str = "bt_header_output_obj";
    pub const TIMEOUT: &str = "bt_header_timeout";
    pub const BODY_HASH: &str = "computed_body_hash";
    pub const NAME: &str = "name";
    pub const TOTAL_SIZE: &str = "total_size";
    pub const HEADER_SIZE: &str = "header_size";
}

/// Convert a Synapse to HTTP headers for transmission
///
/// # Arguments
///
/// * `synapse` - The Synapse to convert
///
/// # Returns
///
/// An HTTP HeaderMap containing all synapse fields as headers
pub fn synapse_to_headers(synapse: &Synapse) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let synapse_headers = synapse.to_headers();

    // Helper macro to add optional header values
    macro_rules! add_header {
        ($name:expr, $value:expr) => {
            if let Some(ref v) = $value {
                if let Ok(hv) = http::header::HeaderValue::from_str(v) {
                    headers.insert($name, hv);
                }
            }
        };
    }

    // Synapse metadata
    add_header!(header_names::NAME, synapse_headers.name);
    add_header!(header_names::TIMEOUT, synapse_headers.timeout);
    add_header!(header_names::TOTAL_SIZE, synapse_headers.total_size);
    add_header!(header_names::HEADER_SIZE, synapse_headers.header_size);
    add_header!(header_names::BODY_HASH, synapse_headers.computed_body_hash);

    // Dendrite terminal info
    add_header!(header_names::DENDRITE_IP, synapse_headers.dendrite_ip);
    add_header!(header_names::DENDRITE_PORT, synapse_headers.dendrite_port);
    add_header!(
        header_names::DENDRITE_VERSION,
        synapse_headers.dendrite_version
    );
    add_header!(header_names::DENDRITE_NONCE, synapse_headers.dendrite_nonce);
    add_header!(header_names::DENDRITE_UUID, synapse_headers.dendrite_uuid);
    add_header!(
        header_names::DENDRITE_HOTKEY,
        synapse_headers.dendrite_hotkey
    );
    add_header!(
        header_names::DENDRITE_SIGNATURE,
        synapse_headers.dendrite_signature
    );

    // Axon terminal info
    add_header!(header_names::AXON_IP, synapse_headers.axon_ip);
    add_header!(header_names::AXON_PORT, synapse_headers.axon_port);
    add_header!(header_names::AXON_VERSION, synapse_headers.axon_version);
    add_header!(header_names::AXON_NONCE, synapse_headers.axon_nonce);
    add_header!(header_names::AXON_UUID, synapse_headers.axon_uuid);
    add_header!(header_names::AXON_HOTKEY, synapse_headers.axon_hotkey);
    add_header!(header_names::AXON_SIGNATURE, synapse_headers.axon_signature);

    headers
}

/// Parse HTTP headers into a Synapse
///
/// # Arguments
///
/// * `headers` - The HTTP response headers
/// * `body` - The response body bytes
///
/// # Returns
///
/// A reconstructed Synapse with data from headers and body
pub fn headers_to_synapse(headers: &HeaderMap, body: &[u8]) -> Result<Synapse, RequestError> {
    // Helper to get header value as string
    fn get_header(headers: &HeaderMap, name: &str) -> Option<String> {
        headers
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
    }

    // Helper to parse header as number
    fn get_header_u64(headers: &HeaderMap, name: &str) -> Option<u64> {
        get_header(headers, name).and_then(|s| s.parse().ok())
    }

    fn get_header_f64(headers: &HeaderMap, name: &str) -> Option<f64> {
        get_header(headers, name).and_then(|s| s.parse().ok())
    }

    fn get_header_i32(headers: &HeaderMap, name: &str) -> Option<i32> {
        get_header(headers, name).and_then(|s| s.parse().ok())
    }

    fn get_header_u16(headers: &HeaderMap, name: &str) -> Option<u16> {
        get_header(headers, name).and_then(|s| s.parse().ok())
    }

    // Build dendrite terminal info from headers
    let dendrite = TerminalInfo {
        status_code: get_header_i32(headers, "bt_header_dendrite_status_code"),
        status_message: get_header(headers, "bt_header_dendrite_status_message"),
        process_time: get_header_f64(headers, "bt_header_dendrite_process_time"),
        ip: get_header(headers, header_names::DENDRITE_IP),
        port: get_header_u16(headers, header_names::DENDRITE_PORT),
        version: get_header_u64(headers, header_names::DENDRITE_VERSION),
        nonce: get_header_u64(headers, header_names::DENDRITE_NONCE),
        uuid: get_header(headers, header_names::DENDRITE_UUID),
        hotkey: get_header(headers, header_names::DENDRITE_HOTKEY),
        signature: get_header(headers, header_names::DENDRITE_SIGNATURE),
    };

    // Build axon terminal info from headers
    let axon = TerminalInfo {
        status_code: get_header_i32(headers, header_names::AXON_STATUS_CODE),
        status_message: get_header(headers, header_names::AXON_STATUS_MESSAGE),
        process_time: get_header_f64(headers, header_names::AXON_PROCESS_TIME),
        ip: get_header(headers, header_names::AXON_IP),
        port: get_header_u16(headers, header_names::AXON_PORT),
        version: get_header_u64(headers, header_names::AXON_VERSION),
        nonce: get_header_u64(headers, header_names::AXON_NONCE),
        uuid: get_header(headers, header_names::AXON_UUID),
        hotkey: get_header(headers, header_names::AXON_HOTKEY),
        signature: get_header(headers, header_names::AXON_SIGNATURE),
    };

    // Parse body as JSON extra fields
    let extra = if body.is_empty() {
        std::collections::HashMap::new()
    } else {
        serde_json::from_slice(body).map_err(|e| RequestError::Serialization(e.to_string()))?
    };

    Ok(Synapse {
        name: get_header(headers, header_names::NAME),
        timeout: get_header_f64(headers, header_names::TIMEOUT),
        total_size: get_header_u64(headers, header_names::TOTAL_SIZE),
        header_size: get_header_u64(headers, header_names::HEADER_SIZE),
        dendrite: Some(dendrite),
        axon: Some(axon),
        computed_body_hash: get_header(headers, header_names::BODY_HASH),
        extra,
    })
}

/// Create a signature message for request authentication
///
/// The signature format matches the Python SDK:
/// `"{nonce}.{dendrite_hotkey}.{axon_hotkey}.{body_hash}"`
///
/// # Arguments
///
/// * `nonce` - Request nonce
/// * `dendrite_hotkey` - The sender's hotkey SS58 address
/// * `axon_hotkey` - The target axon's hotkey SS58 address
/// * `body_hash` - SHA-256 hash of the request body
///
/// # Returns
///
/// The signature message string
pub fn create_signature_message(
    nonce: u64,
    dendrite_hotkey: &str,
    axon_hotkey: &str,
    body_hash: &str,
) -> String {
    format!(
        "{}.{}.{}.{}",
        nonce, dendrite_hotkey, axon_hotkey, body_hash
    )
}

/// Sign a message with the given keypair and return hex-encoded signature
///
/// # Arguments
///
/// * `keypair` - The SR25519 keypair to sign with
/// * `message` - The message bytes to sign
///
/// # Returns
///
/// Hex-encoded signature string
pub fn sign_message(keypair: &sr25519::Pair, message: &[u8]) -> String {
    let signature = keypair.sign(message);
    hex::encode(signature.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::IpAddr;

    fn create_test_axon() -> AxonInfo {
        AxonInfo {
            hotkey: Some("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".to_string()),
            block: 1000,
            version: 100,
            ip: IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
            port: 8091,
            ip_type: 4,
            protocol: 0,
            placeholder1: 0,
            placeholder2: 0,
        }
    }

    fn create_test_synapse() -> Synapse {
        Synapse::new().with_name("TestSynapse").with_timeout(12.0)
    }

    fn create_test_dendrite_info() -> TerminalInfo {
        TerminalInfo {
            ip: Some("192.168.1.1".to_string()),
            port: Some(8080),
            version: Some(100),
            nonce: Some(12345678),
            uuid: Some("test-uuid-1234".to_string()),
            hotkey: Some("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn test_dendrite_request_new() {
        let axon = create_test_axon();
        let synapse = create_test_synapse();
        let dendrite_info = create_test_dendrite_info();

        let request =
            DendriteRequest::new(&axon, &synapse, &dendrite_info, Duration::from_secs(12)).unwrap();

        assert_eq!(request.url, "http://127.0.0.1:8091/TestSynapse");
        assert_eq!(
            request.headers.dendrite_hotkey.as_deref(),
            Some("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY")
        );
        assert_eq!(request.headers.dendrite_nonce.as_deref(), Some("12345678"));
    }

    #[test]
    fn test_compute_body_hash() {
        let axon = create_test_axon();
        let synapse = create_test_synapse();
        let dendrite_info = create_test_dendrite_info();

        let request =
            DendriteRequest::new(&axon, &synapse, &dendrite_info, Duration::from_secs(12)).unwrap();

        let hash = request.compute_body_hash();
        // SHA-256 hash should be 64 hex characters
        assert_eq!(hash.len(), 64);
        // Hash should be deterministic
        assert_eq!(hash, request.compute_body_hash());
    }

    #[test]
    fn test_create_signature_message() {
        let message = create_signature_message(
            12345,
            "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
            "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
            "abc123def456",
        );

        assert_eq!(
            message,
            "12345.5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY.5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty.abc123def456"
        );
    }

    #[test]
    fn test_synapse_to_headers_and_back() {
        let mut synapse = Synapse::new().with_name("TestSynapse").with_timeout(15.0);

        synapse.dendrite = Some(TerminalInfo {
            ip: Some("10.0.0.1".to_string()),
            port: Some(8080),
            version: Some(200),
            nonce: Some(99999),
            ..Default::default()
        });

        let headers = synapse_to_headers(&synapse);

        // Verify some headers exist
        assert!(headers.contains_key(header_names::NAME));
        assert!(headers.contains_key(header_names::TIMEOUT));
    }

    #[test]
    fn test_headers_to_synapse_empty_body() {
        let mut headers = HeaderMap::new();
        headers.insert(header_names::NAME, "ParsedSynapse".parse().unwrap());
        headers.insert(header_names::TIMEOUT, "20.0".parse().unwrap());

        let synapse = headers_to_synapse(&headers, &[]).unwrap();

        assert_eq!(synapse.name, Some("ParsedSynapse".to_string()));
        assert_eq!(synapse.timeout, Some(20.0));
        assert!(synapse.extra.is_empty());
    }

    #[test]
    fn test_sign_message() {
        // Create a test keypair from seed
        let keypair =
            sr25519::Pair::from_string("//Alice", None).expect("Failed to create test keypair");

        let message = b"test message";
        let signature = sign_message(&keypair, message);

        // Signature should be 128 hex characters (64 bytes)
        assert_eq!(signature.len(), 128);
    }
}
