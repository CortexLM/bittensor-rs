//! Synapse types for Bittensor communication
//! These are read-only type definitions that match the Python Synapse class

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Terminal information about a network endpoint
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TerminalInfo {
    /// HTTP status code
    pub status_code: Option<i32>,
    /// Status message
    pub status_message: Option<String>,
    /// Processing time in seconds
    pub process_time: Option<f64>,
    /// IP address
    pub ip: Option<String>,
    /// Port number
    pub port: Option<u16>,
    /// Bittensor version
    pub version: Option<u64>,
    /// Nonce for replay protection
    pub nonce: Option<u64>,
    /// UUID
    pub uuid: Option<String>,
    /// Hotkey SS58 address
    pub hotkey: Option<String>,
    /// Signature
    pub signature: Option<String>,
}

impl TerminalInfo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_status(mut self, code: i32, message: &str) -> Self {
        self.status_code = Some(code);
        self.status_message = Some(message.to_string());
        self
    }
}

/// Base Synapse structure for network communication
/// This represents the core message format in Bittensor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Synapse {
    /// Name of the synapse (route name)
    pub name: Option<String>,
    /// Request timeout in seconds
    pub timeout: Option<f64>,
    /// Total size of request body in bytes
    pub total_size: Option<u64>,
    /// Size of request header in bytes
    pub header_size: Option<u64>,
    /// Dendrite (sender) terminal information
    pub dendrite: Option<TerminalInfo>,
    /// Axon (receiver) terminal information
    pub axon: Option<TerminalInfo>,
    /// Computed body hash
    pub computed_body_hash: Option<String>,
    /// Additional fields for custom data
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Default for Synapse {
    fn default() -> Self {
        Self {
            name: None,
            timeout: Some(12.0),
            total_size: Some(0),
            header_size: Some(0),
            dendrite: Some(TerminalInfo::default()),
            axon: Some(TerminalInfo::default()),
            computed_body_hash: None,
            extra: HashMap::new(),
        }
    }
}

impl Synapse {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn with_timeout(mut self, timeout: f64) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Check if request was successful
    pub fn is_success(&self) -> bool {
        if let Some(ref dendrite) = self.dendrite {
            if let Some(code) = dendrite.status_code {
                return code == 200;
            }
        }
        false
    }

    /// Check if request failed
    pub fn is_failure(&self) -> bool {
        if let Some(ref dendrite) = self.dendrite {
            if let Some(code) = dendrite.status_code {
                return code != 200;
            }
        }
        true
    }

    /// Check if request timed out
    pub fn is_timeout(&self) -> bool {
        if let Some(ref dendrite) = self.dendrite {
            if let Some(code) = dendrite.status_code {
                return code == 408;
            }
        }
        false
    }

    /// Get total size of the synapse
    pub fn get_total_size(&self) -> u64 {
        self.total_size.unwrap_or(0) + self.header_size.unwrap_or(0)
    }

    /// Set a custom field
    pub fn set_field(&mut self, key: &str, value: serde_json::Value) {
        self.extra.insert(key.to_string(), value);
    }

    /// Get a custom field
    pub fn get_field(&self, key: &str) -> Option<&serde_json::Value> {
        self.extra.get(key)
    }
}

/// HTTP headers for synapse transmission
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SynapseHeaders {
    pub name: Option<String>,
    pub timeout: Option<String>,
    pub total_size: Option<String>,
    pub header_size: Option<String>,
    pub computed_body_hash: Option<String>,
    /// Dendrite fields
    pub dendrite_status_code: Option<String>,
    pub dendrite_status_message: Option<String>,
    pub dendrite_process_time: Option<String>,
    pub dendrite_ip: Option<String>,
    pub dendrite_port: Option<String>,
    pub dendrite_version: Option<String>,
    pub dendrite_nonce: Option<String>,
    pub dendrite_uuid: Option<String>,
    pub dendrite_hotkey: Option<String>,
    pub dendrite_signature: Option<String>,
    /// Axon fields
    pub axon_status_code: Option<String>,
    pub axon_status_message: Option<String>,
    pub axon_process_time: Option<String>,
    pub axon_ip: Option<String>,
    pub axon_port: Option<String>,
    pub axon_version: Option<String>,
    pub axon_nonce: Option<String>,
    pub axon_uuid: Option<String>,
    pub axon_hotkey: Option<String>,
    pub axon_signature: Option<String>,
}

impl Synapse {
    /// Convert synapse to headers for HTTP transmission
    pub fn to_headers(&self) -> SynapseHeaders {
        let mut headers = SynapseHeaders::default();
        
        headers.name = self.name.clone();
        headers.timeout = self.timeout.map(|t| t.to_string());
        headers.total_size = self.total_size.map(|s| s.to_string());
        headers.header_size = self.header_size.map(|s| s.to_string());
        headers.computed_body_hash = self.computed_body_hash.clone();

        if let Some(ref d) = self.dendrite {
            headers.dendrite_status_code = d.status_code.map(|c| c.to_string());
            headers.dendrite_status_message = d.status_message.clone();
            headers.dendrite_process_time = d.process_time.map(|t| t.to_string());
            headers.dendrite_ip = d.ip.clone();
            headers.dendrite_port = d.port.map(|p| p.to_string());
            headers.dendrite_version = d.version.map(|v| v.to_string());
            headers.dendrite_nonce = d.nonce.map(|n| n.to_string());
            headers.dendrite_uuid = d.uuid.clone();
            headers.dendrite_hotkey = d.hotkey.clone();
            headers.dendrite_signature = d.signature.clone();
        }

        if let Some(ref a) = self.axon {
            headers.axon_status_code = a.status_code.map(|c| c.to_string());
            headers.axon_status_message = a.status_message.clone();
            headers.axon_process_time = a.process_time.map(|t| t.to_string());
            headers.axon_ip = a.ip.clone();
            headers.axon_port = a.port.map(|p| p.to_string());
            headers.axon_version = a.version.map(|v| v.to_string());
            headers.axon_nonce = a.nonce.map(|n| n.to_string());
            headers.axon_uuid = a.uuid.clone();
            headers.axon_hotkey = a.hotkey.clone();
            headers.axon_signature = a.signature.clone();
        }

        headers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synapse_creation() {
        let synapse = Synapse::new()
            .with_name("TestSynapse")
            .with_timeout(30.0);
        
        assert_eq!(synapse.name, Some("TestSynapse".to_string()));
        assert_eq!(synapse.timeout, Some(30.0));
    }

    #[test]
    fn test_synapse_status() {
        let mut synapse = Synapse::new();
        synapse.dendrite = Some(TerminalInfo::new().with_status(200, "Success"));
        
        assert!(synapse.is_success());
        assert!(!synapse.is_failure());
        assert!(!synapse.is_timeout());
    }

    #[test]
    fn test_synapse_timeout() {
        let mut synapse = Synapse::new();
        synapse.dendrite = Some(TerminalInfo::new().with_status(408, "Timeout"));
        
        assert!(synapse.is_timeout());
        assert!(synapse.is_failure());
    }

    #[test]
    fn test_custom_fields() {
        let mut synapse = Synapse::new();
        synapse.set_field("input", serde_json::json!(42));
        
        assert_eq!(synapse.get_field("input"), Some(&serde_json::json!(42)));
    }
}
