//! Core Synapse trait for Bittensor protocol serialization.

use crate::hashing::sha3_256_hex;
use crate::header::keys;
use crate::terminal_info::TerminalInfo;
use serde::de::DeserializeOwned;
use std::collections::HashMap;

/// Error type for synapse operations.
#[derive(Debug, thiserror::Error)]
pub enum SynapseError {
    #[error("missing required header: {0}")]
    MissingHeader(String),
    #[error("invalid header value for {key}: {source}")]
    InvalidHeaderValue {
        key: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    #[error("deserialization failed: {0}")]
    DeserializationFailed(String),
}

/// The core Synapse trait representing a communication schema between neurons.
///
/// Each Synapse type is tailored for a specific task and defines how data is
/// serialized to/from HTTP headers and body for network transmission.
pub trait Synapse: Send + Sync + Sized {
    /// The deserialized output type produced from the body.
    type Output: DeserializeOwned;

    /// The name/route of this synapse type (maps to HTTP route name).
    fn name(&self) -> &str;

    /// The query timeout in seconds.
    fn timeout(&self) -> f64;

    /// Set the timeout.
    fn set_timeout(&mut self, timeout: f64);

    /// The dendrite (requesting) terminal information.
    fn dendrite(&self) -> &TerminalInfo;

    /// Set the dendrite terminal information.
    fn set_dendrite(&mut self, info: TerminalInfo);

    /// The axon (responding) terminal information.
    fn axon(&self) -> &TerminalInfo;

    /// Set the axon terminal information.
    fn set_axon(&mut self, info: TerminalInfo);

    /// The computed body hash string.
    fn computed_body_hash(&self) -> &str;

    /// Set the computed body hash.
    fn set_computed_body_hash(&mut self, hash: String);

    /// The total size of the request body in bytes.
    fn total_size(&self) -> u64;

    /// Set the total size.
    fn set_total_size(&mut self, size: u64);

    /// The size of the request header in bytes.
    fn header_size(&self) -> u64;

    /// Set the header size.
    fn set_header_size(&mut self, size: u64);

    /// Compute the SHA3-256 body hash from the serialized required fields.
    ///
    /// By default, this serializes the body to JSON and hashes it.
    /// Implementations can override this for custom hash field selection.
    fn body_hash(&self) -> String
    where
        Self: serde::Serialize,
    {
        let body = serde_json::to_vec(self).unwrap_or_default();
        sha3_256_hex(&body)
    }

    /// Serialize this synapse into a header map for HTTP transmission.
    ///
    /// Matches the Python SDK's `to_headers()` method:
    /// - Top-level: `name`, `timeout`, `header_size`, `total_size`, `computed_body_hash`
    /// - Axon fields: `bt_header_axon_{field}`
    /// - Dendrite fields: `bt_header_dendrite_{field}`
    fn to_headers(&self) -> HashMap<String, String>
    where
        Self: serde::Serialize,
    {
        let mut headers = HashMap::new();
        headers.insert(keys::NAME.to_string(), self.name().to_string());
        headers.insert(keys::TIMEOUT.to_string(), self.timeout().to_string());

        let axon_headers = self.axon().to_headers_with_prefix(keys::AXON_PREFIX);
        headers.extend(axon_headers);

        let dendrite_headers = self.dendrite().to_headers_with_prefix(keys::DENDRITE_PREFIX);
        headers.extend(dendrite_headers);

        let header_size = headers.keys().map(|k| k.len()).sum::<usize>()
            + headers.values().map(|v| v.len()).sum::<usize>();
        headers.insert(keys::HEADER_SIZE.to_string(), header_size.to_string());
        headers.insert(keys::TOTAL_SIZE.to_string(), self.total_size().to_string());
        headers.insert(keys::COMPUTED_BODY_HASH.to_string(), self.body_hash());

        headers
    }

    /// Deserialize a synapse from a header map.
    ///
    /// Matches the Python SDK's `from_headers()` classmethod.
    fn from_headers(headers: &HashMap<String, String>) -> Result<Self, SynapseError> {
        let _ = headers;
        Err(SynapseError::DeserializationFailed(
            "from_headers must be implemented per-type".to_string(),
        ))
    }

    /// Deserialize the body bytes into the output type.
    fn deserialize_body(body: &[u8]) -> Result<Self::Output, SynapseError> {
        serde_json::from_slice(body).map_err(|e| SynapseError::DeserializationFailed(e.to_string()))
    }
}

/// Helper to parse a required header value as a string.
pub fn parse_header<'a>(
    headers: &'a HashMap<String, String>,
    key: &str,
) -> Result<&'a str, SynapseError> {
    headers.get(key).map(|s| s.as_str()).ok_or_else(|| SynapseError::MissingHeader(key.to_string()))
}

/// Helper to parse a required header as f64.
pub fn parse_header_f64(headers: &HashMap<String, String>, key: &str) -> Result<f64, SynapseError> {
    let val = parse_header(headers, key)?;
    val.parse::<f64>()
        .map_err(|e| SynapseError::InvalidHeaderValue { key: key.to_string(), source: Box::new(e) })
}

/// Helper to parse a required header as u64.
pub fn parse_header_u64(headers: &HashMap<String, String>, key: &str) -> Result<u64, SynapseError> {
    let val = parse_header(headers, key)?;
    val.parse::<u64>()
        .map_err(|e| SynapseError::InvalidHeaderValue { key: key.to_string(), source: Box::new(e) })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(serde::Serialize)]
    struct TestSynapse {
        name_val: String,
        timeout_val: f64,
        dendrite_info: TerminalInfo,
        axon_info: TerminalInfo,
        computed_hash: String,
        total: u64,
        header: u64,
    }

    impl Synapse for TestSynapse {
        type Output = serde_json::Value;

        fn name(&self) -> &str {
            &self.name_val
        }
        fn timeout(&self) -> f64 {
            self.timeout_val
        }
        fn set_timeout(&mut self, t: f64) {
            self.timeout_val = t;
        }
        fn dendrite(&self) -> &TerminalInfo {
            &self.dendrite_info
        }
        fn set_dendrite(&mut self, info: TerminalInfo) {
            self.dendrite_info = info;
        }
        fn axon(&self) -> &TerminalInfo {
            &self.axon_info
        }
        fn set_axon(&mut self, info: TerminalInfo) {
            self.axon_info = info;
        }
        fn computed_body_hash(&self) -> &str {
            &self.computed_hash
        }
        fn set_computed_body_hash(&mut self, h: String) {
            self.computed_hash = h;
        }
        fn total_size(&self) -> u64 {
            self.total
        }
        fn set_total_size(&mut self, s: u64) {
            self.total = s;
        }
        fn header_size(&self) -> u64 {
            self.header
        }
        fn set_header_size(&mut self, s: u64) {
            self.header = s;
        }
    }

    fn make_test_synapse() -> TestSynapse {
        TestSynapse {
            name_val: "TestSynapse".to_string(),
            timeout_val: 12.0,
            dendrite_info: TerminalInfo {
                hotkey: Some("5DendriteKey".to_string()),
                nonce: Some(100),
                uuid: Some("test-uuid-d".to_string()),
                status_code: Some(200),
                status_message: Some("OK".to_string()),
                ..Default::default()
            },
            axon_info: TerminalInfo {
                hotkey: Some("5AxonKey".to_string()),
                nonce: Some(200),
                uuid: Some("test-uuid-a".to_string()),
                status_code: Some(200),
                status_message: Some("OK".to_string()),
                ..Default::default()
            },
            computed_hash: String::new(),
            total: 0,
            header: 0,
        }
    }

    #[test]
    fn to_headers_contains_top_level_keys() {
        let syn = make_test_synapse();
        let headers = syn.to_headers();
        assert!(headers.contains_key(keys::NAME));
        assert!(headers.contains_key(keys::TIMEOUT));
        assert!(headers.contains_key(keys::HEADER_SIZE));
        assert!(headers.contains_key(keys::TOTAL_SIZE));
        assert!(headers.contains_key(keys::COMPUTED_BODY_HASH));
    }

    #[test]
    fn to_headers_contains_axon_prefix_fields() {
        let syn = make_test_synapse();
        let headers = syn.to_headers();
        assert!(headers.contains_key("bt_header_axon_hotkey"));
        assert!(headers.contains_key("bt_header_axon_nonce"));
        assert!(headers.contains_key("bt_header_axon_uuid"));
    }

    #[test]
    fn to_headers_contains_dendrite_prefix_fields() {
        let syn = make_test_synapse();
        let headers = syn.to_headers();
        assert!(headers.contains_key("bt_header_dendrite_hotkey"));
        assert!(headers.contains_key("bt_header_dendrite_nonce"));
        assert!(headers.contains_key("bt_header_dendrite_uuid"));
    }

    #[test]
    fn header_round_trip_preserves_terminal_info() {
        let syn = make_test_synapse();
        let headers = syn.to_headers();
        let restored_axon = TerminalInfo::from_headers_with_prefix(&headers, keys::AXON_PREFIX);
        let restored_dendrite =
            TerminalInfo::from_headers_with_prefix(&headers, keys::DENDRITE_PREFIX);

        assert_eq!(restored_axon.hotkey, syn.axon_info.hotkey);
        assert_eq!(restored_axon.nonce, syn.axon_info.nonce);
        assert_eq!(restored_axon.uuid, syn.axon_info.uuid);
        assert_eq!(restored_dendrite.hotkey, syn.dendrite_info.hotkey);
        assert_eq!(restored_dendrite.nonce, syn.dendrite_info.nonce);
        assert_eq!(restored_dendrite.uuid, syn.dendrite_info.uuid);
    }

    #[test]
    fn deserialize_body_valid_json() {
        let body = br#"{"result":"hello"}"#;
        let result = TestSynapse::deserialize_body(body);
        assert!(result.is_ok());
    }

    #[test]
    fn deserialize_body_invalid_json() {
        let body = b"not json";
        let result = TestSynapse::deserialize_body(body);
        assert!(result.is_err());
    }

    #[test]
    fn parse_header_missing_key() {
        let headers = HashMap::new();
        let result = parse_header(&headers, "nonexistent");
        assert!(result.is_err());
    }
}
