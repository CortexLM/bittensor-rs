//! Synapse types for Bittensor communication
//!
//! This module provides synapse types that match the Python SDK's Synapse class,
//! supporting both standard request/response patterns and streaming communication.
//!
//! # Features
//!
//! - `Synapse` - Base synapse structure with body hash computation
//! - `SynapseType` - Trait for custom synapse implementations
//! - `StreamingSynapse` - Trait for streaming synapse implementations
//! - `TextPromptSynapse` - Built-in text prompt synapse
//! - `headers` module - Header constants matching Python SDK
//!
//! # Example
//!
//! ```ignore
//! use bittensor_rs::types::synapse::{Synapse, SynapseType, TextPromptSynapse, Message};
//!
//! // Create a basic synapse
//! let synapse = Synapse::new().with_name("MyQuery").with_timeout(30.0);
//!
//! // Create a text prompt synapse
//! let prompt = TextPromptSynapse::new(vec![
//!     Message::user("Hello, how are you?"),
//! ]);
//! ```

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::Duration;

use crate::errors::{BittensorError, SerializationError};

/// Header constants matching the Python SDK exactly
///
/// These constants define the HTTP header names used for Bittensor network
/// communication between Dendrite clients and Axon servers.
pub mod headers {
    // Dendrite (client) headers
    /// Dendrite IP address header
    pub const BT_HEADER_DENDRITE_IP: &str = "bt_header_dendrite_ip";
    /// Dendrite port header
    pub const BT_HEADER_DENDRITE_PORT: &str = "bt_header_dendrite_port";
    /// Dendrite version header
    pub const BT_HEADER_DENDRITE_VERSION: &str = "bt_header_dendrite_version";
    /// Dendrite nonce for replay protection
    pub const BT_HEADER_DENDRITE_NONCE: &str = "bt_header_dendrite_nonce";
    /// Dendrite UUID header
    pub const BT_HEADER_DENDRITE_UUID: &str = "bt_header_dendrite_uuid";
    /// Dendrite hotkey SS58 address
    pub const BT_HEADER_DENDRITE_HOTKEY: &str = "bt_header_dendrite_hotkey";
    /// Dendrite signature for authentication
    pub const BT_HEADER_DENDRITE_SIGNATURE: &str = "bt_header_dendrite_signature";

    // Axon (server) headers
    /// Axon IP address header
    pub const BT_HEADER_AXON_IP: &str = "bt_header_axon_ip";
    /// Axon port header
    pub const BT_HEADER_AXON_PORT: &str = "bt_header_axon_port";
    /// Axon version header
    pub const BT_HEADER_AXON_VERSION: &str = "bt_header_axon_version";
    /// Axon nonce for replay protection
    pub const BT_HEADER_AXON_NONCE: &str = "bt_header_axon_nonce";
    /// Axon UUID header
    pub const BT_HEADER_AXON_UUID: &str = "bt_header_axon_uuid";
    /// Axon hotkey SS58 address
    pub const BT_HEADER_AXON_HOTKEY: &str = "bt_header_axon_hotkey";
    /// Axon signature for authentication
    pub const BT_HEADER_AXON_SIGNATURE: &str = "bt_header_axon_signature";
    /// Axon status code from response
    pub const BT_HEADER_AXON_STATUS_CODE: &str = "bt_header_axon_status_code";
    /// Axon status message from response
    pub const BT_HEADER_AXON_STATUS_MESSAGE: &str = "bt_header_axon_status_message";
    /// Axon processing time in seconds
    pub const BT_HEADER_AXON_PROCESS_TIME: &str = "bt_header_axon_process_time";

    // Synapse metadata headers
    /// Input object header (serialized synapse input)
    pub const BT_HEADER_INPUT_OBJ: &str = "bt_header_input_obj";
    /// Output object header (serialized synapse output)
    pub const BT_HEADER_OUTPUT_OBJ: &str = "bt_header_output_obj";
    /// Computed body hash for verification
    pub const COMPUTED_BODY_HASH: &str = "computed_body_hash";
    /// Synapse name/route
    pub const NAME: &str = "name";
    /// Request timeout in seconds
    pub const TIMEOUT: &str = "timeout";

    /// Get all dendrite header names as a slice
    pub fn dendrite_headers() -> &'static [&'static str] {
        &[
            BT_HEADER_DENDRITE_IP,
            BT_HEADER_DENDRITE_PORT,
            BT_HEADER_DENDRITE_VERSION,
            BT_HEADER_DENDRITE_NONCE,
            BT_HEADER_DENDRITE_UUID,
            BT_HEADER_DENDRITE_HOTKEY,
            BT_HEADER_DENDRITE_SIGNATURE,
        ]
    }

    /// Get all axon header names as a slice
    pub fn axon_headers() -> &'static [&'static str] {
        &[
            BT_HEADER_AXON_IP,
            BT_HEADER_AXON_PORT,
            BT_HEADER_AXON_VERSION,
            BT_HEADER_AXON_NONCE,
            BT_HEADER_AXON_UUID,
            BT_HEADER_AXON_HOTKEY,
            BT_HEADER_AXON_SIGNATURE,
            BT_HEADER_AXON_STATUS_CODE,
            BT_HEADER_AXON_STATUS_MESSAGE,
            BT_HEADER_AXON_PROCESS_TIME,
        ]
    }

    /// Get all metadata header names as a slice
    pub fn metadata_headers() -> &'static [&'static str] {
        &[
            BT_HEADER_INPUT_OBJ,
            BT_HEADER_OUTPUT_OBJ,
            COMPUTED_BODY_HASH,
            NAME,
            TIMEOUT,
        ]
    }
}

// =============================================================================
// SynapseType Trait
// =============================================================================

/// Trait for custom synapse types
///
/// Types implementing this trait can be used with the Dendrite client
/// for type-safe communication with Axon servers.
///
/// # Example
///
/// ```ignore
/// use bittensor_rs::types::synapse::{SynapseType, Synapse};
/// use serde::{Serialize, Deserialize};
/// use std::time::Duration;
///
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// struct MyCustomSynapse {
///     #[serde(flatten)]
///     pub base: Synapse,
///     pub query: String,
///     pub result: Option<String>,
/// }
///
/// impl SynapseType for MyCustomSynapse {
///     fn name() -> &'static str { "MyCustomSynapse" }
///     fn required_hash_fields() -> Vec<&'static str> { vec!["query"] }
/// }
/// ```
pub trait SynapseType: Serialize + DeserializeOwned + Send + Sync + 'static {
    /// Get the synapse name/route
    ///
    /// This should return a unique identifier for this synapse type,
    /// typically used as the HTTP endpoint path.
    fn name() -> &'static str;

    /// Get the timeout for this synapse
    ///
    /// Default is 12 seconds, matching the Python SDK default.
    fn timeout(&self) -> Duration {
        Duration::from_secs(12)
    }

    /// Get the fields that must be included in body hash computation
    ///
    /// Returns a list of field names that should be hashed for signature
    /// verification. The order matters for consistent hashing.
    fn required_hash_fields() -> Vec<&'static str> {
        vec![]
    }

    /// Deserialize from JSON bytes
    ///
    /// # Arguments
    ///
    /// * `data` - JSON-encoded bytes
    ///
    /// # Returns
    ///
    /// The deserialized synapse or an error
    fn from_json(data: &[u8]) -> Result<Self, BittensorError> {
        serde_json::from_slice(data).map_err(|e| {
            BittensorError::Serialization(SerializationError::with_type(
                format!("Failed to deserialize {}: {}", Self::name(), e),
                Self::name(),
            ))
        })
    }

    /// Serialize to JSON bytes
    ///
    /// # Returns
    ///
    /// JSON-encoded bytes or an error
    fn to_json(&self) -> Result<Vec<u8>, BittensorError> {
        serde_json::to_vec(self).map_err(|e| {
            BittensorError::Serialization(SerializationError::with_type(
                format!("Failed to serialize {}: {}", Self::name(), e),
                Self::name(),
            ))
        })
    }
}

// =============================================================================
// StreamingSynapse Trait
// =============================================================================

/// Trait for streaming synapse types that process data in chunks
///
/// This trait extends `SynapseType` to support streaming responses,
/// allowing incremental processing of large or continuous data streams.
///
/// # Example
///
/// ```ignore
/// use bittensor_rs::types::synapse::{SynapseType, StreamingSynapse, Synapse};
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// struct StreamingTextSynapse {
///     #[serde(flatten)]
///     pub base: Synapse,
///     pub prompt: String,
///     #[serde(skip)]
///     accumulated_response: String,
///     #[serde(skip)]
///     complete: bool,
/// }
///
/// impl SynapseType for StreamingTextSynapse {
///     fn name() -> &'static str { "StreamingTextSynapse" }
/// }
///
/// impl StreamingSynapse for StreamingTextSynapse {
///     type Chunk = String;
///     
///     fn process_chunk(&mut self, chunk: &[u8]) -> Option<Self::Chunk> {
///         String::from_utf8(chunk.to_vec()).ok()
///     }
///     
///     fn is_complete(&self) -> bool {
///         self.complete
///     }
///     
///     fn finalize(&mut self) -> Result<(), crate::errors::BittensorError> {
///         self.complete = true;
///         Ok(())
///     }
/// }
/// ```
pub trait StreamingSynapse: SynapseType {
    /// The type of each chunk produced by the stream
    type Chunk: Send;

    /// Process a chunk of data from the response stream
    ///
    /// # Arguments
    ///
    /// * `chunk` - Raw bytes from the response stream
    ///
    /// # Returns
    ///
    /// `Some(chunk)` if a complete chunk was parsed, `None` if more data is needed
    fn process_chunk(&mut self, chunk: &[u8]) -> Option<Self::Chunk>;

    /// Check if the stream is complete
    ///
    /// Returns `true` when no more chunks are expected
    fn is_complete(&self) -> bool;

    /// Finalize the stream
    ///
    /// Called when the stream ends (either normally or due to completion).
    /// Implementations should clean up any resources and mark the stream as complete.
    fn finalize(&mut self) -> Result<(), BittensorError>;
}

// =============================================================================
// TerminalInfo
// =============================================================================

/// Terminal information about a network endpoint
///
/// Contains metadata about either the Dendrite (client) or Axon (server)
/// side of a communication. Used for authentication and debugging.
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
    /// Create a new empty TerminalInfo
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the status code and message
    pub fn with_status(mut self, code: i32, message: &str) -> Self {
        self.status_code = Some(code);
        self.status_message = Some(message.to_string());
        self
    }

    /// Set the IP address
    pub fn with_ip(mut self, ip: impl Into<String>) -> Self {
        self.ip = Some(ip.into());
        self
    }

    /// Set the port number
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    /// Set the version
    pub fn with_version(mut self, version: u64) -> Self {
        self.version = Some(version);
        self
    }

    /// Set the nonce
    pub fn with_nonce(mut self, nonce: u64) -> Self {
        self.nonce = Some(nonce);
        self
    }

    /// Set the UUID
    pub fn with_uuid(mut self, uuid: impl Into<String>) -> Self {
        self.uuid = Some(uuid.into());
        self
    }

    /// Set the hotkey
    pub fn with_hotkey(mut self, hotkey: impl Into<String>) -> Self {
        self.hotkey = Some(hotkey.into());
        self
    }

    /// Set the signature
    pub fn with_signature(mut self, signature: impl Into<String>) -> Self {
        self.signature = Some(signature.into());
        self
    }

    /// Set the process time
    pub fn with_process_time(mut self, time: f64) -> Self {
        self.process_time = Some(time);
        self
    }
}

// =============================================================================
// Synapse
// =============================================================================

/// Base Synapse structure for network communication
///
/// This represents the core message format in Bittensor, containing
/// metadata for authentication, routing, and debugging, as well as
/// support for custom fields via the `extra` map.
///
/// # Example
///
/// ```
/// use bittensor_rs::types::synapse::Synapse;
///
/// let synapse = Synapse::new()
///     .with_name("MyQuery")
///     .with_timeout(30.0);
///
/// assert_eq!(synapse.name, Some("MyQuery".to_string()));
/// assert!(!synapse.is_success()); // No response yet
/// ```
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
    /// Computed body hash for signature verification
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
    /// Create a new Synapse with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the synapse name/route
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    /// Set the request timeout in seconds
    pub fn with_timeout(mut self, timeout: f64) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set the dendrite terminal info
    pub fn with_dendrite(mut self, dendrite: TerminalInfo) -> Self {
        self.dendrite = Some(dendrite);
        self
    }

    /// Set the axon terminal info
    pub fn with_axon(mut self, axon: TerminalInfo) -> Self {
        self.axon = Some(axon);
        self
    }

    /// Set the computed body hash
    pub fn with_body_hash(mut self, hash: impl Into<String>) -> Self {
        self.computed_body_hash = Some(hash.into());
        self
    }

    /// Check if request was successful (status code 200)
    pub fn is_success(&self) -> bool {
        if let Some(ref dendrite) = self.dendrite {
            if let Some(code) = dendrite.status_code {
                return code == 200;
            }
        }
        false
    }

    /// Check if request failed (status code != 200 or no status)
    pub fn is_failure(&self) -> bool {
        if let Some(ref dendrite) = self.dendrite {
            if let Some(code) = dendrite.status_code {
                return code != 200;
            }
        }
        true
    }

    /// Check if request timed out (status code 408)
    pub fn is_timeout(&self) -> bool {
        if let Some(ref dendrite) = self.dendrite {
            if let Some(code) = dendrite.status_code {
                return code == 408;
            }
        }
        false
    }

    /// Get total size of the synapse (body + header)
    pub fn get_total_size(&self) -> u64 {
        self.total_size.unwrap_or(0) + self.header_size.unwrap_or(0)
    }

    /// Set a custom field in the extra map
    pub fn set_field(&mut self, key: &str, value: serde_json::Value) {
        self.extra.insert(key.to_string(), value);
    }

    /// Get a custom field from the extra map
    pub fn get_field(&self, key: &str) -> Option<&serde_json::Value> {
        self.extra.get(key)
    }

    /// Compute the body hash for signature verification
    ///
    /// This method computes a SHA256 hash of specified fields for use in
    /// signature verification. The hash matches the Python SDK's body_hash.
    ///
    /// # Arguments
    ///
    /// * `fields` - Field names to include in the hash computation
    ///
    /// # Returns
    ///
    /// Hex-encoded SHA256 hash string
    pub fn compute_body_hash(&self, fields: &[&str]) -> String {
        let mut hasher = Sha256::new();

        // Sort fields for consistent ordering
        let mut sorted_fields: Vec<&str> = fields.to_vec();
        sorted_fields.sort();

        for field in sorted_fields {
            if let Some(value) = self.extra.get(field) {
                // Serialize the value to JSON for hashing
                if let Ok(json_bytes) = serde_json::to_vec(value) {
                    hasher.update(&json_bytes);
                }
            }
        }

        // Return hex-encoded hash
        hex::encode(hasher.finalize())
    }

    /// Compute the body hash using all extra fields
    ///
    /// Convenience method that hashes all fields in the extra map.
    pub fn compute_full_body_hash(&self) -> String {
        let fields: Vec<&str> = self.extra.keys().map(|s| s.as_str()).collect();
        self.compute_body_hash(&fields)
    }

    /// Verify that the stored body hash matches a computed hash
    ///
    /// # Arguments
    ///
    /// * `fields` - Field names to include in the hash computation
    ///
    /// # Returns
    ///
    /// `true` if the computed hash matches the stored `computed_body_hash`
    pub fn verify_body_hash(&self, fields: &[&str]) -> bool {
        if let Some(ref stored_hash) = self.computed_body_hash {
            let computed = self.compute_body_hash(fields);
            constant_time_compare(stored_hash.as_bytes(), computed.as_bytes())
        } else {
            // No stored hash means nothing to verify
            false
        }
    }

    /// Update the stored body hash with a freshly computed value
    ///
    /// # Arguments
    ///
    /// * `fields` - Field names to include in the hash computation
    pub fn update_body_hash(&mut self, fields: &[&str]) {
        self.computed_body_hash = Some(self.compute_body_hash(fields));
    }

    /// Get the timeout as a Duration
    pub fn timeout_duration(&self) -> Duration {
        Duration::from_secs_f64(self.timeout.unwrap_or(12.0))
    }
}

/// Constant-time string comparison to prevent timing attacks
fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}

impl SynapseType for Synapse {
    fn name() -> &'static str {
        "Synapse"
    }

    fn timeout(&self) -> Duration {
        self.timeout_duration()
    }
}

// =============================================================================
// Message
// =============================================================================

/// A message in a text prompt conversation
///
/// Represents a single message in a conversation, with a role (user, assistant, system)
/// and content. This matches the common chat API format used by many LLMs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Message {
    /// The role of the message author (e.g., "user", "assistant", "system")
    pub role: String,
    /// The content of the message
    pub content: String,
}

impl Message {
    /// Create a new message with the given role and content
    pub fn new(role: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            content: content.into(),
        }
    }

    /// Create a user message
    pub fn user(content: impl Into<String>) -> Self {
        Self::new("user", content)
    }

    /// Create an assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new("assistant", content)
    }

    /// Create a system message
    pub fn system(content: impl Into<String>) -> Self {
        Self::new("system", content)
    }
}

// =============================================================================
// TextPromptSynapse
// =============================================================================

/// Text prompt synapse for chat/completion requests
///
/// A pre-built synapse type for text-based LLM interactions, supporting
/// multi-turn conversations with role-based messages.
///
/// # Example
///
/// ```
/// use bittensor_rs::types::synapse::{TextPromptSynapse, Message};
///
/// let synapse = TextPromptSynapse::new(vec![
///     Message::system("You are a helpful assistant."),
///     Message::user("Hello, how are you?"),
/// ]);
///
/// assert_eq!(synapse.messages.len(), 2);
/// assert!(synapse.response.is_none());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextPromptSynapse {
    /// Base synapse fields
    #[serde(flatten)]
    pub base: Synapse,
    /// The conversation messages
    pub messages: Vec<Message>,
    /// The response from the axon (filled by server)
    pub response: Option<String>,
}

impl TextPromptSynapse {
    /// Create a new TextPromptSynapse with the given messages
    pub fn new(messages: Vec<Message>) -> Self {
        let mut base = Synapse::new();
        base.name = Some(Self::name().to_string());
        Self {
            base,
            messages,
            response: None,
        }
    }

    /// Create a new TextPromptSynapse with a single user message
    pub fn from_prompt(prompt: impl Into<String>) -> Self {
        Self::new(vec![Message::user(prompt)])
    }

    /// Create a new TextPromptSynapse with a system message and user message
    pub fn with_system_prompt(
        system_prompt: impl Into<String>,
        user_prompt: impl Into<String>,
    ) -> Self {
        Self::new(vec![
            Message::system(system_prompt),
            Message::user(user_prompt),
        ])
    }

    /// Add a message to the conversation
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    /// Add a user message to the conversation
    pub fn add_user_message(&mut self, content: impl Into<String>) {
        self.messages.push(Message::user(content));
    }

    /// Add an assistant message to the conversation
    pub fn add_assistant_message(&mut self, content: impl Into<String>) {
        self.messages.push(Message::assistant(content));
    }

    /// Set the response
    pub fn set_response(&mut self, response: impl Into<String>) {
        self.response = Some(response.into());
    }

    /// Get the response if available
    pub fn get_response(&self) -> Option<&str> {
        self.response.as_deref()
    }

    /// Set the timeout
    pub fn with_timeout(mut self, timeout: f64) -> Self {
        self.base.timeout = Some(timeout);
        self
    }

    /// Compute the body hash for this synapse
    pub fn compute_body_hash(&self) -> String {
        let mut hasher = Sha256::new();

        // Hash the messages array
        if let Ok(json_bytes) = serde_json::to_vec(&self.messages) {
            hasher.update(&json_bytes);
        }

        hex::encode(hasher.finalize())
    }

    /// Update the base synapse body hash
    pub fn update_body_hash(&mut self) {
        self.base.computed_body_hash = Some(self.compute_body_hash());
    }
}

impl SynapseType for TextPromptSynapse {
    fn name() -> &'static str {
        "TextPromptSynapse"
    }

    fn timeout(&self) -> Duration {
        self.base.timeout_duration()
    }

    fn required_hash_fields() -> Vec<&'static str> {
        vec!["messages"]
    }
}

// =============================================================================
// StreamingTextPromptSynapse
// =============================================================================

/// Streaming version of TextPromptSynapse for incremental response processing
///
/// This synapse accumulates text chunks as they arrive, allowing for
/// real-time display of generated text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingTextPromptSynapse {
    /// Base synapse fields
    #[serde(flatten)]
    pub base: Synapse,
    /// The conversation messages
    pub messages: Vec<Message>,
    /// Accumulated response text
    #[serde(default)]
    pub accumulated_response: String,
    /// Whether the stream is complete
    #[serde(skip, default)]
    complete: bool,
}

impl StreamingTextPromptSynapse {
    /// Create a new StreamingTextPromptSynapse with the given messages
    pub fn new(messages: Vec<Message>) -> Self {
        let mut base = Synapse::new();
        base.name = Some(Self::name().to_string());
        Self {
            base,
            messages,
            accumulated_response: String::new(),
            complete: false,
        }
    }

    /// Create from a single user prompt
    pub fn from_prompt(prompt: impl Into<String>) -> Self {
        Self::new(vec![Message::user(prompt)])
    }

    /// Get the current accumulated response
    pub fn response(&self) -> &str {
        &self.accumulated_response
    }

    /// Set the timeout
    pub fn with_timeout(mut self, timeout: f64) -> Self {
        self.base.timeout = Some(timeout);
        self
    }
}

impl SynapseType for StreamingTextPromptSynapse {
    fn name() -> &'static str {
        "StreamingTextPromptSynapse"
    }

    fn timeout(&self) -> Duration {
        self.base.timeout_duration()
    }

    fn required_hash_fields() -> Vec<&'static str> {
        vec!["messages"]
    }
}

impl StreamingSynapse for StreamingTextPromptSynapse {
    type Chunk = String;

    fn process_chunk(&mut self, chunk: &[u8]) -> Option<Self::Chunk> {
        // Try to parse the chunk as UTF-8 text
        match std::str::from_utf8(chunk) {
            Ok(text) => {
                if !text.is_empty() {
                    self.accumulated_response.push_str(text);
                    Some(text.to_string())
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    fn is_complete(&self) -> bool {
        self.complete
    }

    fn finalize(&mut self) -> Result<(), BittensorError> {
        self.complete = true;
        Ok(())
    }
}

// =============================================================================
// SynapseHeaders
// =============================================================================

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
        let mut headers: SynapseHeaders = SynapseHeaders {
            name: self.name.clone(),
            timeout: self.timeout.map(|t| t.to_string()),
            total_size: self.total_size.map(|s| s.to_string()),
            header_size: self.header_size.map(|s| s.to_string()),
            computed_body_hash: self.computed_body_hash.clone(),
            ..Default::default()
        };

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

    // =========================================================================
    // Synapse Tests
    // =========================================================================

    #[test]
    fn test_synapse_creation() {
        let synapse = Synapse::new().with_name("TestSynapse").with_timeout(30.0);

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

    #[test]
    fn test_synapse_builder_pattern() {
        let dendrite = TerminalInfo::new()
            .with_ip("192.168.1.1")
            .with_port(8080)
            .with_hotkey("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty");

        let synapse = Synapse::new()
            .with_name("TestSynapse")
            .with_timeout(30.0)
            .with_dendrite(dendrite);

        assert_eq!(synapse.name, Some("TestSynapse".to_string()));
        assert_eq!(synapse.timeout, Some(30.0));
        assert!(synapse.dendrite.is_some());
        let d = synapse.dendrite.as_ref().unwrap();
        assert_eq!(d.ip, Some("192.168.1.1".to_string()));
        assert_eq!(d.port, Some(8080));
    }

    #[test]
    fn test_timeout_duration() {
        let synapse = Synapse::new().with_timeout(30.5);
        let duration = synapse.timeout_duration();
        assert_eq!(duration, Duration::from_secs_f64(30.5));

        let default_synapse = Synapse::new();
        let default_duration = default_synapse.timeout_duration();
        assert_eq!(default_duration, Duration::from_secs(12));
    }

    // =========================================================================
    // Body Hash Tests
    // =========================================================================

    #[test]
    fn test_body_hash_computation() {
        let mut synapse = Synapse::new();
        synapse.set_field("input", serde_json::json!("hello"));
        synapse.set_field("value", serde_json::json!(42));

        let hash1 = synapse.compute_body_hash(&["input"]);
        let hash2 = synapse.compute_body_hash(&["input"]);

        // Same input should produce same hash
        assert_eq!(hash1, hash2);

        // Different fields should produce different hash
        let hash3 = synapse.compute_body_hash(&["value"]);
        assert_ne!(hash1, hash3);

        // Hash should be hex-encoded SHA256 (64 chars)
        assert_eq!(hash1.len(), 64);
        assert!(hash1.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_body_hash_field_ordering() {
        let mut synapse = Synapse::new();
        synapse.set_field("a", serde_json::json!(1));
        synapse.set_field("b", serde_json::json!(2));

        // Fields should be sorted, so order shouldn't matter
        let hash1 = synapse.compute_body_hash(&["a", "b"]);
        let hash2 = synapse.compute_body_hash(&["b", "a"]);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_body_hash_verification() {
        let mut synapse = Synapse::new();
        synapse.set_field("query", serde_json::json!("test query"));

        // Store the hash
        synapse.update_body_hash(&["query"]);

        // Verification should pass
        assert!(synapse.verify_body_hash(&["query"]));

        // Modify the field
        synapse.set_field("query", serde_json::json!("modified query"));

        // Verification should fail after modification
        assert!(!synapse.verify_body_hash(&["query"]));
    }

    #[test]
    fn test_body_hash_empty_fields() {
        let synapse = Synapse::new();

        // Empty fields should still produce a valid hash
        let hash = synapse.compute_body_hash(&[]);
        assert_eq!(hash.len(), 64);

        // Non-existent fields should be ignored
        let hash2 = synapse.compute_body_hash(&["nonexistent"]);
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_verify_body_hash_no_stored_hash() {
        let synapse = Synapse::new();

        // No stored hash should return false
        assert!(!synapse.verify_body_hash(&[]));
    }

    // =========================================================================
    // Message Tests
    // =========================================================================

    #[test]
    fn test_message_creation() {
        let msg = Message::new("user", "Hello, world!");
        assert_eq!(msg.role, "user");
        assert_eq!(msg.content, "Hello, world!");
    }

    #[test]
    fn test_message_convenience_constructors() {
        let user_msg = Message::user("User message");
        assert_eq!(user_msg.role, "user");

        let assistant_msg = Message::assistant("Assistant message");
        assert_eq!(assistant_msg.role, "assistant");

        let system_msg = Message::system("System message");
        assert_eq!(system_msg.role, "system");
    }

    #[test]
    fn test_message_serialization() {
        let msg = Message::user("Hello");
        let json = serde_json::to_string(&msg).expect("Failed to serialize");
        let deserialized: Message = serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(msg, deserialized);
    }

    // =========================================================================
    // TextPromptSynapse Tests
    // =========================================================================

    #[test]
    fn test_text_prompt_synapse_creation() {
        let synapse = TextPromptSynapse::new(vec![
            Message::system("You are helpful."),
            Message::user("Hello!"),
        ]);

        assert_eq!(synapse.messages.len(), 2);
        assert_eq!(synapse.messages[0].role, "system");
        assert_eq!(synapse.messages[1].role, "user");
        assert!(synapse.response.is_none());
        assert_eq!(synapse.base.name, Some("TextPromptSynapse".to_string()));
    }

    #[test]
    fn test_text_prompt_synapse_from_prompt() {
        let synapse = TextPromptSynapse::from_prompt("What is 2+2?");

        assert_eq!(synapse.messages.len(), 1);
        assert_eq!(synapse.messages[0].role, "user");
        assert_eq!(synapse.messages[0].content, "What is 2+2?");
    }

    #[test]
    fn test_text_prompt_synapse_with_system_prompt() {
        let synapse = TextPromptSynapse::with_system_prompt(
            "You are a math tutor.",
            "What is 2+2?",
        );

        assert_eq!(synapse.messages.len(), 2);
        assert_eq!(synapse.messages[0].role, "system");
        assert_eq!(synapse.messages[0].content, "You are a math tutor.");
        assert_eq!(synapse.messages[1].role, "user");
    }

    #[test]
    fn test_text_prompt_synapse_add_messages() {
        let mut synapse = TextPromptSynapse::new(vec![]);

        synapse.add_user_message("Hello");
        synapse.add_assistant_message("Hi there!");
        synapse.add_message(Message::new("custom", "Custom role"));

        assert_eq!(synapse.messages.len(), 3);
        assert_eq!(synapse.messages[0].role, "user");
        assert_eq!(synapse.messages[1].role, "assistant");
        assert_eq!(synapse.messages[2].role, "custom");
    }

    #[test]
    fn test_text_prompt_synapse_response() {
        let mut synapse = TextPromptSynapse::from_prompt("Hello");

        assert!(synapse.get_response().is_none());

        synapse.set_response("Hello to you too!");

        assert_eq!(synapse.get_response(), Some("Hello to you too!"));
    }

    #[test]
    fn test_text_prompt_synapse_body_hash() {
        let mut synapse = TextPromptSynapse::new(vec![
            Message::user("Hello"),
        ]);

        let hash = synapse.compute_body_hash();

        // Hash should be valid hex
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));

        // Same messages should produce same hash
        let synapse2 = TextPromptSynapse::new(vec![
            Message::user("Hello"),
        ]);
        assert_eq!(hash, synapse2.compute_body_hash());

        // Different messages should produce different hash
        let synapse3 = TextPromptSynapse::new(vec![
            Message::user("Goodbye"),
        ]);
        assert_ne!(hash, synapse3.compute_body_hash());

        // Update body hash should store it
        synapse.update_body_hash();
        assert!(synapse.base.computed_body_hash.is_some());
    }

    #[test]
    fn test_text_prompt_synapse_serialization() {
        let synapse = TextPromptSynapse::new(vec![
            Message::system("Be helpful"),
            Message::user("Hello"),
        ]).with_timeout(30.0);

        let json = serde_json::to_string(&synapse).expect("Failed to serialize");
        let deserialized: TextPromptSynapse =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(synapse.messages.len(), deserialized.messages.len());
        assert_eq!(synapse.base.timeout, deserialized.base.timeout);
    }

    #[test]
    fn test_text_prompt_synapse_type_trait() {
        assert_eq!(TextPromptSynapse::name(), "TextPromptSynapse");
        assert_eq!(
            TextPromptSynapse::required_hash_fields(),
            vec!["messages"]
        );

        let synapse = TextPromptSynapse::from_prompt("test").with_timeout(45.0);
        assert_eq!(synapse.timeout(), Duration::from_secs_f64(45.0));
    }

    // =========================================================================
    // StreamingTextPromptSynapse Tests
    // =========================================================================

    #[test]
    fn test_streaming_text_prompt_synapse_creation() {
        let synapse = StreamingTextPromptSynapse::new(vec![
            Message::user("Generate text"),
        ]);

        assert_eq!(synapse.messages.len(), 1);
        assert!(synapse.accumulated_response.is_empty());
        assert!(!synapse.is_complete());
    }

    #[test]
    fn test_streaming_text_prompt_synapse_process_chunk() {
        let mut synapse = StreamingTextPromptSynapse::from_prompt("test");

        // Process a chunk
        let chunk = synapse.process_chunk(b"Hello ");
        assert_eq!(chunk, Some("Hello ".to_string()));
        assert_eq!(synapse.response(), "Hello ");

        // Process another chunk
        let chunk = synapse.process_chunk(b"World!");
        assert_eq!(chunk, Some("World!".to_string()));
        assert_eq!(synapse.response(), "Hello World!");
    }

    #[test]
    fn test_streaming_text_prompt_synapse_empty_chunk() {
        let mut synapse = StreamingTextPromptSynapse::from_prompt("test");

        // Empty chunk should return None
        let chunk = synapse.process_chunk(b"");
        assert!(chunk.is_none());
    }

    #[test]
    fn test_streaming_text_prompt_synapse_invalid_utf8() {
        let mut synapse = StreamingTextPromptSynapse::from_prompt("test");

        // Invalid UTF-8 should return None
        let chunk = synapse.process_chunk(&[0xFF, 0xFE]);
        assert!(chunk.is_none());
    }

    #[test]
    fn test_streaming_text_prompt_synapse_finalize() {
        let mut synapse = StreamingTextPromptSynapse::from_prompt("test");

        assert!(!synapse.is_complete());

        synapse.finalize().expect("Finalize failed");

        assert!(synapse.is_complete());
    }

    #[test]
    fn test_streaming_text_prompt_synapse_type_trait() {
        assert_eq!(StreamingTextPromptSynapse::name(), "StreamingTextPromptSynapse");
        assert_eq!(
            StreamingTextPromptSynapse::required_hash_fields(),
            vec!["messages"]
        );
    }

    // =========================================================================
    // TerminalInfo Tests
    // =========================================================================

    #[test]
    fn test_terminal_info_builder() {
        let info = TerminalInfo::new()
            .with_ip("10.0.0.1")
            .with_port(9090)
            .with_version(123)
            .with_nonce(456)
            .with_uuid("test-uuid")
            .with_hotkey("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty")
            .with_signature("0x123abc")
            .with_process_time(0.5);

        assert_eq!(info.ip, Some("10.0.0.1".to_string()));
        assert_eq!(info.port, Some(9090));
        assert_eq!(info.version, Some(123));
        assert_eq!(info.nonce, Some(456));
        assert_eq!(info.uuid, Some("test-uuid".to_string()));
        assert_eq!(
            info.hotkey,
            Some("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty".to_string())
        );
        assert_eq!(info.signature, Some("0x123abc".to_string()));
        assert_eq!(info.process_time, Some(0.5));
    }

    // =========================================================================
    // Header Constants Tests
    // =========================================================================

    #[test]
    fn test_header_constants() {
        // Verify header names match expected format
        assert!(headers::BT_HEADER_DENDRITE_IP.starts_with("bt_header_"));
        assert!(headers::BT_HEADER_AXON_IP.starts_with("bt_header_"));

        // Verify helper functions return correct headers
        let dendrite_headers = headers::dendrite_headers();
        assert!(dendrite_headers.contains(&headers::BT_HEADER_DENDRITE_IP));
        assert!(dendrite_headers.contains(&headers::BT_HEADER_DENDRITE_SIGNATURE));

        let axon_headers = headers::axon_headers();
        assert!(axon_headers.contains(&headers::BT_HEADER_AXON_IP));
        assert!(axon_headers.contains(&headers::BT_HEADER_AXON_PROCESS_TIME));

        let metadata_headers = headers::metadata_headers();
        assert!(metadata_headers.contains(&headers::COMPUTED_BODY_HASH));
        assert!(metadata_headers.contains(&headers::NAME));
    }

    // =========================================================================
    // SynapseType Trait Tests
    // =========================================================================

    #[test]
    fn test_synapse_type_json_roundtrip() {
        let synapse = Synapse::new().with_name("Test").with_timeout(30.0);

        // Use SynapseType methods
        let json_bytes = synapse.to_json().expect("Serialization failed");
        let deserialized = Synapse::from_json(&json_bytes).expect("Deserialization failed");

        assert_eq!(synapse.name, deserialized.name);
        assert_eq!(synapse.timeout, deserialized.timeout);
    }

    #[test]
    fn test_synapse_type_from_json_error() {
        let invalid_json = b"not valid json {";
        let result = Synapse::from_json(invalid_json);
        assert!(result.is_err());
    }

    // =========================================================================
    // Constant Time Compare Tests
    // =========================================================================

    #[test]
    fn test_constant_time_compare() {
        assert!(constant_time_compare(b"hello", b"hello"));
        assert!(!constant_time_compare(b"hello", b"world"));
        assert!(!constant_time_compare(b"hello", b"hell"));
        assert!(!constant_time_compare(b"", b"a"));
        assert!(constant_time_compare(b"", b""));
    }
}
