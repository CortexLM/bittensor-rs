# bittensor-synapse

Protocol types, header serialization, hashing, signing, and streaming for Bittensor synapse communication.

## Overview

The `bittensor-synapse` crate defines the wire protocol between neurons on the Bittensor network. Every request and response flowing from a dendrite (client) to an axon (server) is wrapped in a type that implements the `Synapse` trait. The trait governs how data is serialized into HTTP headers, how body hashes are computed, how requests are signed, and how streaming responses are consumed incrementally.

The crate has no optional features. Everything described here is always available.

### Crate

```toml
[dependencies]
bittensor-synapse = "0.1"
```

### Prelude

```rust
use bittensor_synapse::prelude::*;
```

The prelude re-exports every public item you need:

| Item | Source |
|---|---|
| `Synapse` | `synapse` module |
| `SynapseError` | `synapse` module |
| `StreamingSynapse` | `streaming` module |
| `TerminalInfo` | `terminal_info` module |
| `sha3_256_hex` | `hashing` module |
| `signing_message` | `signing` module |
| `keys` (header constants) | `header` module |
| `HashMap` | `std::collections` |

---

## Protocol Flow

Understanding the synapse protocol requires seeing the full request lifecycle:

1. A **dendrite** constructs a synapse value, calls `to_headers()` to produce a `HashMap<String, String>`, and computes `body_hash()` from the serialized body.
2. The dendrite signs the request using `signing_message()` to build the string `"{nonce}.{dendrite_hotkey}.{axon_hotkey}.{uuid}.{body_hash}"`, then signs that message with its Sr25519 keypair. The signature and metadata are attached as `bt-*` headers.
3. The **axon** receives the HTTP request. Its `VerificationMiddleware` checks that the signature header is present and that the signing fields are well-formed. `BlacklistMiddleware` rejects blacklisted hotkeys. `PriorityMiddleware` assigns a priority score. `BodyHashMiddleware` re-hashes the body and compares it to the `computed_body_hash` header.
4. The axon's handler processes the synapse, populates the `axon` field of `TerminalInfo` on the response, and sends it back.
5. The dendrite reads the response headers, extracts the axon's `TerminalInfo` via `TerminalInfo::from_headers_with_prefix`, and updates its synapse accordingly.

---

## Synapse Trait

```rust
pub trait Synapse: Send + Sync + Sized {
    type Output: DeserializeOwned;

    fn name(&self) -> &str;
    fn timeout(&self) -> f64;
    fn set_timeout(&mut self, timeout: f64);
    fn dendrite(&self) -> &TerminalInfo;
    fn set_dendrite(&mut self, info: TerminalInfo);
    fn axon(&self) -> &TerminalInfo;
    fn set_axon(&mut self, info: TerminalInfo);
    fn computed_body_hash(&self) -> &str;
    fn set_computed_body_hash(&mut self, hash: String);
    fn total_size(&self) -> u64;
    fn set_total_size(&mut self, size: u64);
    fn header_size(&self) -> u64;
    fn set_header_size(&mut self, size: u64);

    fn body_hash(&self) -> String
    where
        Self: serde::Serialize;

    fn to_headers(&self) -> HashMap<String, String>
    where
        Self: serde::Serialize;

    fn from_headers(headers: &HashMap<String, String>) -> Result<Self, SynapseError>;

    fn deserialize_body(body: &[u8]) -> Result<Self::Output, SynapseError>;
}
```

### Associated Types

#### `type Output: DeserializeOwned`

The deserialized type produced from the response body. When the axon returns a JSON body, `deserialize_body` parses it into this type. For a text generation synapse, `Output` might be a `struct TextOutput { completion: String }`.

### Required Methods

#### `fn name(&self) -> &str`

Returns the synapse route name. This string maps directly to the HTTP route on the axon: a synapse named `"TextPrompt"` is served at `POST /TextPrompt`. The name must be unique among all routes registered on a single axon.

```rust
fn name(&self) -> &str {
    "TextPrompt"
}
```

#### `fn timeout(&self) -> f64`

Returns the query timeout in seconds. The dendrite uses this to set its HTTP client timeout. The axon middleware also reads this header to enforce a processing deadline.

#### `fn set_timeout(&mut self, timeout: f64)`

Updates the timeout value. Called by the dendrite before sending if the caller overrides the default.

#### `fn dendrite(&self) -> &TerminalInfo`

Returns a reference to the dendrite's `TerminalInfo`, carrying the requesting neuron's identity and connection metadata.

#### `fn set_dendrite(&mut self, info: TerminalInfo)`

Replaces the dendrite `TerminalInfo`. The dendrite calls this after signing to inject its hotkey, nonce, uuid, and other fields.

#### `fn axon(&self) -> &TerminalInfo`

Returns a reference to the axon's `TerminalInfo`. After a response arrives, the dendrite populates this with the server's metadata.

#### `fn set_axon(&mut self, info: TerminalInfo)`

Replaces the axon `TerminalInfo`. Called by the dendrite after receiving a response.

#### `fn computed_body_hash(&self) -> &str`

Returns the stored SHA3-256 hex digest of the request body. This hash was computed at send time and is verified by the axon's `BodyHashMiddleware`.

#### `fn set_computed_body_hash(&mut self, hash: String)`

Stores the computed body hash. Called internally by `to_headers` and by the dendrite before sending.

#### `fn total_size(&self) -> u64`

Returns the total size of the request body in bytes.

#### `fn set_total_size(&mut self, size: u64)`

Sets the total body size.

#### `fn header_size(&self) -> u64`

Returns the size of all headers combined (key lengths plus value lengths) in bytes.

#### `fn set_header_size(&mut self, size: u64)`

Sets the header size value.

### Provided Methods

#### `fn body_hash(&self) -> String`

Computes the SHA3-256 hash of the serialized synapse body. The default implementation serializes `self` to a JSON byte vector via `serde_json::to_vec` and passes it to `sha3_256_hex`. Implementations can override this if they need to hash only a subset of fields.

Requires `Self: serde::Serialize`.

```rust
let hash = my_synapse.body_hash();
// "a7ffc6f8bf1ed766..."
```

#### `fn to_headers(&self) -> HashMap<String, String>`

Serializes the synapse into a flat header map for HTTP transmission. The output matches the Python SDK's `to_headers()` method exactly:

- Top-level fields: `name`, `timeout`, `header_size`, `total_size`, `computed_body_hash`
- Axon metadata: `bt_header_axon_{field}` for each non-None field in `self.axon()`
- Dendrite metadata: `bt_header_dendrite_{field}` for each non-None field in `self.dendrite()`

Fields that are `None` in `TerminalInfo` are omitted entirely, not set to empty strings.

Requires `Self: serde::Serialize`.

```rust
let headers = my_synapse.to_headers();
assert!(headers.contains_key("name"));
assert!(headers.contains_key("timeout"));
assert!(headers.contains_key("computed_body_hash"));
assert!(headers.contains_key("bt_header_axon_hotkey"));
assert!(headers.contains_key("bt_header_dendrite_hotkey"));
```

#### `fn from_headers(headers: &HashMap<String, String>) -> Result<Self, SynapseError>`

Deserializes a synapse from a header map. The default implementation returns `SynapseError::DeserializationFailed` because the reconstruction logic depends on the concrete type. Every synapse type must provide its own implementation that reads the top-level keys and prefixed `TerminalInfo` fields.

```rust
impl Synapse for MySynapse {
    // ...
    fn from_headers(headers: &HashMap<String, String>) -> Result<Self, SynapseError> {
        let name = parse_header(headers, "name")?;
        let timeout = parse_header_f64(headers, "timeout")?;
        let dendrite = TerminalInfo::from_headers_with_prefix(headers, keys::DENDRITE_PREFIX);
        let axon = TerminalInfo::from_headers_with_prefix(headers, keys::AXON_PREFIX);
        Ok(MySynapse {
            name_val: name.to_string(),
            timeout_val: timeout,
            dendrite_info: dendrite,
            axon_info: axon,
            // ... remaining fields
        })
    }
}
```

#### `fn deserialize_body(body: &[u8]) -> Result<Self::Output, SynapseError>`

Parses a JSON byte slice into the `Output` type. Wraps `serde_json::from_slice`. Returns `SynapseError::DeserializationFailed` if the JSON is malformed or does not match the output schema.

```rust
let body = br#"{"completion":"hello world"}"#;
let output = MySynapse::deserialize_body(body)?;
```

---

## Header Constants

The `header::keys` module defines constant strings for every header used in the synapse protocol.

```rust
use bittensor_synapse::header::keys;
```

### Top-level Keys

| Constant | Value | Description |
|---|---|---|
| `keys::NAME` | `"name"` | Synapse route name |
| `keys::TIMEOUT` | `"timeout"` | Query timeout in seconds |
| `keys::HEADER_SIZE` | `"header_size"` | Total size of all header keys plus values |
| `keys::TOTAL_SIZE` | `"total_size"` | Size of the request body in bytes |
| `keys::COMPUTED_BODY_HASH` | `"computed_body_hash"` | SHA3-256 hex digest of the body |

### Prefixes

| Constant | Value | Description |
|---|---|---|
| `keys::AXON_PREFIX` | `"bt_header_axon_"` | Prefix for all axon metadata headers |
| `keys::DENDRITE_PREFIX` | `"bt_header_dendrite_"` | Prefix for all dendrite metadata headers |
| `keys::INPUT_OBJ_PREFIX` | `"bt_header_input_obj_"` | Prefix for input object serialization headers |

When serialized, a `TerminalInfo` field like `hotkey` under the axon prefix becomes the header `bt_header_axon_hotkey`.

---

## Hashing Module

```rust
use bittensor_synapse::hashing::sha3_256_hex;
```

### `fn sha3_256_hex(data: &[u8]) -> String`

Computes the FIPS 202 SHA3-256 digest of the input bytes and returns the lowercase hexadecimal encoding. This is the same algorithm as Python's `hashlib.sha3_256(data).hexdigest()`.

**Not Keccak-256.** The FIPS 202 standard and Keccak produce different digests for the same input. Ethereum uses Keccak-256, but Bittensor uses FIPS 202 SHA3-256. Using the wrong variant will produce hash mismatches that the axon's `BodyHashMiddleware` rejects with HTTP 400.

#### Parameters

| Name | Type | Description |
|---|---|---|
| `data` | `&[u8]` | Raw bytes to hash |

#### Returns

A 64-character lowercase hex string.

#### Example

```rust
use bittensor_synapse::hashing::sha3_256_hex;

let hash = sha3_256_hex(b"hello");
assert_eq!(hash.len(), 64);
assert_eq!(hash, "3338be694f50c5d33fb5496c37c2f9be3f0a3f5c6a21d6bb7b5a870a2eb6ec75");
```

#### Known Test Vectors

| Input | Expected Digest |
|---|---|
| `""` (empty) | `a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a` |
| `"abc"` | `3a985da74fe225b2045c172d6bd390bd855f086e3e9d525b46bfe24511431532` |

---

## Signing Module

```rust
use bittensor_synapse::signing::signing_message;
```

### `fn signing_message(nonce: u64, dendrite_hotkey: &str, axon_hotkey: &str, uuid: &str, body_hash: &str) -> String`

Constructs the signing message from the five verification fields, using the same dot-separated format as the Python SDK. The resulting string is what gets signed by the dendrite's Sr25519 keypair and verified by the axon's `VerificationMiddleware`.

#### Format

```
{nonce}.{dendrite_hotkey}.{axon_hotkey}.{uuid}.{body_hash}
```

No spaces between fields. Exactly four dots separating five components.

#### Parameters

| Name | Type | Description |
|---|---|---|
| `nonce` | `u64` | Monotonically increasing counter, typically the current Unix timestamp in milliseconds |
| `dendrite_hotkey` | `&str` | SS58-encoded public key of the requesting neuron |
| `axon_hotkey` | `&str` | SS58-encoded public key of the target axon |
| `uuid` | `&str` | Unique request identifier (v4 UUID) |
| `body_hash` | `&str` | SHA3-256 hex digest of the request body |

#### Returns

A single `String` with all five fields joined by dots.

#### Example

```rust
use bittensor_synapse::signing::signing_message;

let msg = signing_message(
    1234567890,
    "5DendriteHotkey123",
    "5AxonHotkey456",
    "550e8400-e29b-41d4-a716-446655440000",
    "a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a",
);
assert_eq!(
    msg,
    "1234567890.5DendriteHotkey123.5AxonHotkey456.550e8400-e29b-41d4-a716-446655440000.a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a"
);
```

---

## StreamingSynapse Trait

```rust
pub trait StreamingSynapse: Synapse {
    type StreamItem;

    fn process_chunk(chunk: &[u8]) -> Result<Self::StreamItem, SynapseError>;
}
```

Extension of the `Synapse` trait for Server-Sent Events (SSE) responses. Instead of waiting for the full body, a streaming synapse processes incremental chunks from the server as they arrive.

### Associated Types

#### `type StreamItem`

The type produced from each SSE data chunk. For a text generation synapse, this might be a `String` token fragment.

### Required Methods

#### `fn process_chunk(chunk: &[u8]) -> Result<Self::StreamItem, SynapseError>`

Parse a single `data: ` payload from an SSE event. The `chunk` parameter contains the raw bytes after the `data: ` prefix and newline have been stripped. If the chunk is `[DONE]`, the dendrite stops the stream.

#### Parameters

| Name | Type | Description |
|---|---|---|
| `chunk` | `&[u8]` | Raw bytes of a single SSE data payload |

#### Returns

`Ok(StreamItem)` on successful parsing, or `Err(SynapseError)` if the chunk is malformed.

#### Example

```rust
use bittensor_synapse::prelude::*;

struct StreamingText {
    // Synapse fields...
}

impl StreamingSynapse for StreamingText {
    type StreamItem = String;

    fn process_chunk(chunk: &[u8]) -> Result<String, SynapseError> {
        String::from_utf8(chunk.to_vec())
            .map_err(|e| SynapseError::DeserializationFailed(e.to_string()))
    }
}
```

---

## TerminalInfo Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TerminalInfo {
    pub status_code: Option<u16>,
    pub status_message: Option<String>,
    pub process_time: Option<f64>,
    pub ip: Option<String>,
    pub port: Option<u16>,
    pub version: Option<u32>,
    pub nonce: Option<u64>,
    pub uuid: Option<String>,
    pub hotkey: Option<String>,
    pub signature: Option<String>,
}
```

Carries metadata about one endpoint of a synapse communication. A synapse has two `TerminalInfo` fields: `dendrite` (the requester) and `axon` (the responder). Every field is optional because not all metadata is available at every stage of the lifecycle.

### Fields

| Field | Type | Description |
|---|---|---|
| `status_code` | `Option<u16>` | HTTP status code (200 for success, 408 for timeout, etc.) |
| `status_message` | `Option<String>` | Human-readable status text (e.g. "OK", "Timeout") |
| `process_time` | `Option<f64>` | Seconds the axon spent processing the request |
| `ip` | `Option<String>` | IP address of the terminal |
| `port` | `Option<u16>` | TCP port of the terminal |
| `version` | `Option<u32>` | Bittensor protocol version running on the terminal |
| `nonce` | `Option<u64>` | Monotonically increasing counter to prevent replay attacks |
| `uuid` | `Option<String>` | Unique request identifier |
| `hotkey` | `Option<String>` | SS58-encoded hotkey of the terminal's wallet |
| `signature` | `Option<String>` | Sr25519 signature over the signing message |

### Constructors

#### `TerminalInfo::new() -> Self`

Creates a `TerminalInfo` with all fields set to `None`.

```rust
let info = TerminalInfo::new();
assert!(info.hotkey.is_none());
```

`TerminalInfo` also implements `Default`, which calls `new()`.

### Serialization Methods

#### `fn to_headers_with_prefix(&self, prefix: &str) -> HashMap<String, String>`

Serializes non-None fields into a header map. Each field name is prefixed with the given string. For example, passing `"bt_header_axon_"` with `hotkey = Some("5Grw...")` produces the entry `"bt_header_axon_hotkey" => "5Grw..."`. Fields that are `None` are omitted entirely.

```rust
let info = TerminalInfo {
    hotkey: Some("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".to_string()),
    nonce: Some(12345),
    ..Default::default()
};
let headers = info.to_headers_with_prefix("bt_header_axon_");
assert_eq!(headers.len(), 2);
assert_eq!(headers.get("bt_header_axon_hotkey").unwrap(), "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY");
```

#### `fn from_headers_with_prefix(headers: &HashMap<String, String>, prefix: &str) -> Self`

Reconstructs a `TerminalInfo` from a header map. Looks up each field name with the given prefix. Fields that are missing or unparseable (e.g. a non-numeric string in the `nonce` field) are set to `None`.

```rust
let mut headers = HashMap::new();
headers.insert("bt_header_dendrite_hotkey".to_string(), "5DendKey".to_string());
headers.insert("bt_header_dendrite_nonce".to_string(), "99999".to_string());

let info = TerminalInfo::from_headers_with_prefix(&headers, "bt_header_dendrite_");
assert_eq!(info.hotkey, Some("5DendKey".to_string()));
assert_eq!(info.nonce, Some(99999));
assert!(info.ip.is_none()); // missing fields become None
```

### TerminalInfo Header Key Constants

The `terminal_info::header_keys` module provides constant strings for each field name used in serialization.

```rust
use bittensor_synapse::terminal_info::header_keys;

assert_eq!(header_keys::STATUS_CODE, "status_code");
assert_eq!(header_keys::STATUS_MESSAGE, "status_message");
assert_eq!(header_keys::PROCESS_TIME, "process_time");
assert_eq!(header_keys::IP, "ip");
assert_eq!(header_keys::PORT, "port");
assert_eq!(header_keys::VERSION, "version");
assert_eq!(header_keys::NONCE, "nonce");
assert_eq!(header_keys::UUID, "uuid");
assert_eq!(header_keys::HOTKEY, "hotkey");
assert_eq!(header_keys::SIGNATURE, "signature");
```

---

## SynapseError

```rust
#[derive(Debug, thiserror::Error)]
pub enum SynapseError {
    #[error("missing required header: {0}")]
    MissingHeader(String),
    #[error("invalid header value for {key}: {source}")]
    InvalidHeaderValue {
        key: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    #[error("deserialization failed: {0}")]
    DeserializationFailed(String),
}
```

All errors produced by the synapse crate.

### Variants

| Variant | When | Description |
|---|---|---|
| `MissingHeader(key)` | A required header is absent from the map | `from_headers` could not find a mandatory key |
| `InvalidHeaderValue { key, source }` | A header exists but cannot be parsed | The string in the header is not a valid number, etc. |
| `DeserializationFailed(msg)` | JSON body parsing fails | `deserialize_body` or `process_chunk` encountered invalid data |

---

## Helper Functions

### `fn parse_header<'a>(headers: &'a HashMap<String, String>, key: &str) -> Result<&'a str, SynapseError>`

Extracts a required header value as a string slice. Returns `SynapseError::MissingHeader` if the key is absent.

### `fn parse_header_f64(headers: &HashMap<String, String>, key: &str) -> Result<f64, SynapseError>`

Extracts a required header value and parses it as `f64`. Returns `SynapseError::MissingHeader` if absent, or `SynapseError::InvalidHeaderValue` if the string is not a valid float.

### `fn parse_header_u64(headers: &HashMap<String, String>, key: &str) -> Result<u64, SynapseError>`

Extracts a required header value and parses it as `u64`. Returns `SynapseError::MissingHeader` if absent, or `SynapseError::InvalidHeaderValue` if the string is not a valid integer.

---

## Full Example: Custom Synapse Type

This example shows implementing a complete custom synapse for text prompt inference. It covers every required trait method, header serialization, and deserialization.

```rust
use bittensor_synapse::prelude::*;
use bittensor_synapse::{Synapse, SynapseError, StreamingSynapse, TerminalInfo, sha3_256_hex, signing_message, header::keys};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------- Request / Response types ----------

#[derive(Debug, Serialize, Deserialize)]
pub struct PromptResponse {
    pub completion: String,
}

// ---------- Synapse type ----------

#[derive(Debug, Serialize, Deserialize)]
pub struct TextPromptSynapse {
    // Protocol fields
    name_val: String,
    timeout_val: f64,
    dendrite_info: TerminalInfo,
    axon_info: TerminalInfo,
    computed_hash: String,
    total: u64,
    header: u64,

    // Application fields (serialized into body)
    pub prompt: String,
    pub max_tokens: u32,
    pub temperature: f64,
}

impl TextPromptSynapse {
    pub fn new(prompt: impl Into<String>, max_tokens: u32) -> Self {
        Self {
            name_val: "TextPrompt".to_string(),
            timeout_val: 12.0,
            dendrite_info: TerminalInfo::default(),
            axon_info: TerminalInfo::default(),
            computed_hash: String::new(),
            total: 0,
            header: 0,
            prompt: prompt.into(),
            max_tokens,
            temperature: 0.7,
        }
    }
}

impl Synapse for TextPromptSynapse {
    type Output = PromptResponse;

    fn name(&self) -> &str { &self.name_val }

    fn timeout(&self) -> f64 { self.timeout_val }

    fn set_timeout(&mut self, t: f64) { self.timeout_val = t; }

    fn dendrite(&self) -> &TerminalInfo { &self.dendrite_info }

    fn set_dendrite(&mut self, info: TerminalInfo) { self.dendrite_info = info; }

    fn axon(&self) -> &TerminalInfo { &self.axon_info }

    fn set_axon(&mut self, info: TerminalInfo) { self.axon_info = info; }

    fn computed_body_hash(&self) -> &str { &self.computed_hash }

    fn set_computed_body_hash(&mut self, h: String) { self.computed_hash = h; }

    fn total_size(&self) -> u64 { self.total }

    fn set_total_size(&mut self, s: u64) { self.total = s; }

    fn header_size(&self) -> u64 { self.header }

    fn set_header_size(&mut self, s: u64) { self.header = s; }

    fn from_headers(headers: &HashMap<String, String>) -> Result<Self, SynapseError> {
        let name = parse_header(headers, keys::NAME)?;
        let timeout = parse_header_f64(headers, keys::TIMEOUT)?;
        let dendrite = TerminalInfo::from_headers_with_prefix(headers, keys::DENDRITE_PREFIX);
        let axon = TerminalInfo::from_headers_with_prefix(headers, keys::AXON_PREFIX);

        Ok(Self {
            name_val: name.to_string(),
            timeout_val: timeout,
            dendrite_info: dendrite,
            axon_info: axon,
            computed_hash: String::new(),
            total: 0,
            header: 0,
            prompt: String::new(),   // body fields are not in headers
            max_tokens: 0,
            temperature: 0.0,
        })
    }
}

// ---------- Streaming extension ----------

impl StreamingSynapse for TextPromptSynapse {
    type StreamItem = String;

    fn process_chunk(chunk: &[u8]) -> Result<String, SynapseError> {
        String::from_utf8(chunk.to_vec())
            .map_err(|e| SynapseError::DeserializationFailed(e.to_string()))
    }
}

// ---------- Usage ----------

fn main() -> Result<(), SynapseError> {
    let synapse = TextPromptSynapse::new("What is the capital of France?", 128);

    // Serialize to headers
    let headers = synapse.to_headers();
    println!("Route name: {}", headers.get("name").unwrap_or(&"".to_string()));
    println!("Header count: {}", headers.len());

    // Compute body hash
    let body = serde_json::to_vec(&synapse).unwrap();
    let hash = sha3_256_hex(&body);
    println!("Body hash: {hash}");

    // Build signing message
    let msg = signing_message(
        1700000000000,
        "5DendriteKey",
        "5AxonKey",
        "550e8400-e29b-41d4-a716-446655440000",
        &hash,
    );
    println!("Signing message: {msg}");

    // Deserialize response body
    let response_body = br#"{"completion":"Paris"}"#;
    let output = TextPromptSynapse::deserialize_body(response_body)?;
    println!("Completion: {}", output.completion);

    Ok(())
}
```
