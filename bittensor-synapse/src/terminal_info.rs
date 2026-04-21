//! Terminal information for synapse endpoints.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Header key constants for TerminalInfo fields.
/// These match the Python SDK's `bt_header_axon_` / `bt_header_dendrite_` prefixed headers.
pub mod header_keys {
    pub const STATUS_CODE: &str = "status_code";
    pub const STATUS_MESSAGE: &str = "status_message";
    pub const PROCESS_TIME: &str = "process_time";
    pub const IP: &str = "ip";
    pub const PORT: &str = "port";
    pub const VERSION: &str = "version";
    pub const NONCE: &str = "nonce";
    pub const UUID: &str = "uuid";
    pub const HOTKEY: &str = "hotkey";
    pub const SIGNATURE: &str = "signature";
}

/// Encapsulates detailed information about a network synapse (node)
/// involved in a communication process.
///
/// This mirrors the Python SDK's `TerminalInfo` class, which carries
/// metadata such as HTTP status codes, processing times, IP addresses,
/// ports, version numbers, and cryptographic identifiers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TerminalInfo {
    /// HTTP status code indicating the result of a network request.
    pub status_code: Option<u16>,
    /// Descriptive message associated with the status code.
    pub status_message: Option<String>,
    /// Time taken by the terminal to process the call (seconds).
    pub process_time: Option<f64>,
    /// IP address of the terminal.
    pub ip: Option<String>,
    /// Network port used by the terminal.
    pub port: Option<u16>,
    /// Bittensor version running on the terminal.
    pub version: Option<u32>,
    /// Unique, monotonically increasing number (Unix timestamp) to prevent replay attacks.
    pub nonce: Option<u64>,
    /// Unique identifier for the terminal.
    pub uuid: Option<String>,
    /// Encoded hotkey string of the terminal wallet.
    pub hotkey: Option<String>,
    /// Digital signature verifying the tuple (nonce, axon_hotkey, dendrite_hotkey, uuid).
    pub signature: Option<String>,
}

impl TerminalInfo {
    /// Creates a new `TerminalInfo` with all fields set to `None`.
    pub fn new() -> Self {
        Self {
            status_code: None,
            status_message: None,
            process_time: None,
            ip: None,
            port: None,
            version: None,
            nonce: None,
            uuid: None,
            hotkey: None,
            signature: None,
        }
    }

    /// Serializes non-None fields into a header map using the given prefix.
    ///
    /// The prefix is typically `"bt_header_axon_"` or `"bt_header_dendrite_"`,
    /// matching the Python SDK's `to_headers()` method exactly.
    pub fn to_headers_with_prefix(&self, prefix: &str) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        if let Some(ref v) = self.status_code {
            headers.insert(format!("{prefix}{}", header_keys::STATUS_CODE), v.to_string());
        }
        if let Some(ref v) = self.status_message {
            headers.insert(format!("{prefix}{}", header_keys::STATUS_MESSAGE), v.clone());
        }
        if let Some(ref v) = self.process_time {
            headers.insert(format!("{prefix}{}", header_keys::PROCESS_TIME), v.to_string());
        }
        if let Some(ref v) = self.ip {
            headers.insert(format!("{prefix}{}", header_keys::IP), v.clone());
        }
        if let Some(ref v) = self.port {
            headers.insert(format!("{prefix}{}", header_keys::PORT), v.to_string());
        }
        if let Some(ref v) = self.version {
            headers.insert(format!("{prefix}{}", header_keys::VERSION), v.to_string());
        }
        if let Some(ref v) = self.nonce {
            headers.insert(format!("{prefix}{}", header_keys::NONCE), v.to_string());
        }
        if let Some(ref v) = self.uuid {
            headers.insert(format!("{prefix}{}", header_keys::UUID), v.clone());
        }
        if let Some(ref v) = self.hotkey {
            headers.insert(format!("{prefix}{}", header_keys::HOTKEY), v.clone());
        }
        if let Some(ref v) = self.signature {
            headers.insert(format!("{prefix}{}", header_keys::SIGNATURE), v.clone());
        }
        headers
    }

    /// Deserializes from a header map with the given prefix.
    ///
    /// Fields that are missing or unparseable are left as `None`.
    pub fn from_headers_with_prefix(headers: &HashMap<String, String>, prefix: &str) -> Self {
        let get = |key: &str| -> Option<String> { headers.get(&format!("{prefix}{key}")).cloned() };
        let get_u16 = |key: &str| -> Option<u16> { get(key).and_then(|v| v.parse().ok()) };
        let get_u32 = |key: &str| -> Option<u32> { get(key).and_then(|v| v.parse().ok()) };
        let get_u64 = |key: &str| -> Option<u64> { get(key).and_then(|v| v.parse().ok()) };
        let get_f64 = |key: &str| -> Option<f64> { get(key).and_then(|v| v.parse().ok()) };

        Self {
            status_code: get_u16(header_keys::STATUS_CODE),
            status_message: get(header_keys::STATUS_MESSAGE),
            process_time: get_f64(header_keys::PROCESS_TIME),
            ip: get(header_keys::IP),
            port: get_u16(header_keys::PORT),
            version: get_u32(header_keys::VERSION),
            nonce: get_u64(header_keys::NONCE),
            uuid: get(header_keys::UUID),
            hotkey: get(header_keys::HOTKEY),
            signature: get(header_keys::SIGNATURE),
        }
    }
}

impl Default for TerminalInfo {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_terminal_info_has_all_none() {
        let info = TerminalInfo::default();
        assert!(info.status_code.is_none());
        assert!(info.status_message.is_none());
        assert!(info.process_time.is_none());
        assert!(info.ip.is_none());
        assert!(info.port.is_none());
        assert!(info.version.is_none());
        assert!(info.nonce.is_none());
        assert!(info.uuid.is_none());
        assert!(info.hotkey.is_none());
        assert!(info.signature.is_none());
    }

    #[test]
    fn to_headers_omits_none_values() {
        let info = TerminalInfo {
            status_code: Some(200),
            nonce: Some(12345),
            hotkey: Some("5EnjDGNqqWnuL2HCAdxeEtN2oqtXZw6BMBe936Kfy2PFz1J1".to_string()),
            ..Default::default()
        };
        let headers = info.to_headers_with_prefix("bt_header_axon_");
        assert_eq!(headers.len(), 3);
        assert_eq!(headers.get("bt_header_axon_status_code"), Some(&"200".to_string()));
        assert_eq!(headers.get("bt_header_axon_nonce"), Some(&"12345".to_string()));
        assert_eq!(
            headers.get("bt_header_axon_hotkey"),
            Some(&"5EnjDGNqqWnuL2HCAdxeEtN2oqtXZw6BMBe936Kfy2PFz1J1".to_string())
        );
        // None fields should not appear
        assert!(headers.get("bt_header_axon_ip").is_none());
        assert!(headers.get("bt_header_axon_uuid").is_none());
    }

    #[test]
    fn round_trip_axon_prefix() {
        let info = TerminalInfo {
            status_code: Some(200),
            status_message: Some("Success".to_string()),
            process_time: Some(0.1),
            ip: Some("198.123.23.1".to_string()),
            port: Some(9282),
            version: Some(111),
            nonce: Some(111111),
            uuid: Some("5ecbd69c-1cec-11ee-b0dc-e29ce36fec1a".to_string()),
            hotkey: Some("5EnjDGNqqWnuL2HCAdxeEtN2oqtXZw6BMBe936Kfy2PFz1J1".to_string()),
            signature: Some("0xsig".to_string()),
        };
        let headers = info.to_headers_with_prefix("bt_header_axon_");
        let restored = TerminalInfo::from_headers_with_prefix(&headers, "bt_header_axon_");

        assert_eq!(restored.status_code, info.status_code);
        assert_eq!(restored.status_message, info.status_message);
        assert_eq!(restored.process_time, info.process_time);
        assert_eq!(restored.ip, info.ip);
        assert_eq!(restored.port, info.port);
        assert_eq!(restored.version, info.version);
        assert_eq!(restored.nonce, info.nonce);
        assert_eq!(restored.uuid, info.uuid);
        assert_eq!(restored.hotkey, info.hotkey);
        assert_eq!(restored.signature, info.signature);
    }

    #[test]
    fn round_trip_dendrite_prefix() {
        let info = TerminalInfo {
            status_code: Some(408),
            status_message: Some("Timeout".to_string()),
            nonce: Some(99999),
            ..Default::default()
        };
        let headers = info.to_headers_with_prefix("bt_header_dendrite_");
        let restored = TerminalInfo::from_headers_with_prefix(&headers, "bt_header_dendrite_");

        assert_eq!(restored.status_code, Some(408));
        assert_eq!(restored.status_message, Some("Timeout".to_string()));
        assert_eq!(restored.nonce, Some(99999));
    }
}
