# bittensor-dendrite

HTTP client for querying axons with request signing, timeout handling, and SSE streaming support.

## Overview

The `bittensor-dendrite` crate provides the client half of the Bittensor synapse protocol. A dendrite constructs a signed HTTP POST request from a synapse value, sends it to a remote axon endpoint, and populates the response metadata on the way back. It supports both single-shot queries and Server-Sent Events (SSE) streaming.

When a hotkey is configured, every outbound request carries cryptographic proof of origin via `bt-*` headers. The axon's middleware verifies the signature before dispatching the request to the handler. If no hotkey is set, requests are sent unsigned, which is useful for local testing.

The crate has no optional features. Everything described here is always available.

### Crate

```toml
[dependencies]
bittensor-dendrite = "0.1"
```

### Prelude

```rust
use bittensor_dendrite::prelude::*;
```

The prelude re-exports:

| Item | Source |
|---|---|
| `Dendrite` | `dendrite` module |
| `DendriteConfig` | `config` module |
| `SignedRequest` | `signing` module |

---

## Quick Start

```rust,no_run
use bittensor_dendrite::prelude::*;
use bittensor_core::types::AxonInfo;
use subxt_signer::sr25519::dev::alice;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = DendriteConfig::new()
        .with_timeout_secs(30)
        .with_hotkey(alice());
    let dendrite = Dendrite::new(config)?;

    let axon = AxonInfo {
        ip: 2130706433,  // 127.0.0.1
        port: 8091,
        ip_type: 4,
        protocol: 0,
        version: 1,
        hotkey: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".into(),
        coldkey: "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty".into(),
    };

    // Send a signed synapse request to the axon
    let response = dendrite.query(my_synapse, &axon).await?;
    println!("Axon status: {:?}", response.axon().status_code);
    Ok(())
}
```

---

## Protocol Flow

When a dendrite queries an axon, the following steps occur:

1. The dendrite serializes the synapse to JSON, producing the request body.
2. It computes the SHA3-256 hash of the body bytes.
3. It generates a monotonic nonce (derived from the current Unix timestamp in milliseconds, incrementing for concurrent requests).
4. It calls `sign_request` to build the signing message, signs it with the dendrite's Sr25519 keypair, and attaches `bt-*` headers to the outgoing request.
5. It sets the synapse's `dendrite` field to a `TerminalInfo` carrying the dendrite hotkey, nonce, and uuid.
6. It merges the synapse's `to_headers()` output into the request headers.
7. The axon receives the request, runs its middleware chain, and invokes the handler.
8. The axon sends back a response with `bt_header_axon_*` headers carrying the axon's `TerminalInfo`.
9. The dendrite reads the response headers, reconstructs the axon's `TerminalInfo`, sets it on the synapse, and returns the updated synapse to the caller.

If the axon returns HTTP 401, the dendrite produces `BittensorError::Signing`. Any other non-2xx status becomes `BittensorError::Rpc`. Timeouts become `BittensorError::Timeout`. Connection failures become `BittensorError::Network`.

---

## Dendrite Client

```rust
pub struct Dendrite {
    client: reqwest::Client,
    hotkey: Option<Keypair>,
    nonce: AtomicU64,
}

impl Dendrite {
    pub fn new(config: DendriteConfig) -> Result<Self, BittensorError>;
    pub async fn query<S: Synapse + Serialize>(
        &self,
        synapse: S,
        axon_info: &AxonInfo,
    ) -> Result<S, BittensorError>;
    pub async fn forward<S: Synapse + Serialize>(
        &self,
        synapse: S,
        axon_info: &AxonInfo,
    ) -> Result<S, BittensorError>;
    pub async fn call<S: Synapse + Serialize>(
        &self,
        synapse: S,
        axon_info: &AxonInfo,
    ) -> Result<S, BittensorError>;
    pub async fn call_stream<S>(
        &self,
        synapse: S,
        axon_info: &AxonInfo,
    ) -> Result<S::StreamItem, BittensorError>
    where
        S: StreamingSynapse + Serialize;
}
```

### `Dendrite::new(config: DendriteConfig) -> Result<Self, BittensorError>`

Creates a new dendrite from the given configuration. Builds a `reqwest::Client` with the configured timeout and connection pool settings. Initializes the nonce counter to the current Unix timestamp in milliseconds.

Returns `BittensorError::Network` if the `reqwest::Client` fails to build (rare - typically a TLS backend issue).

```rust
let config = DendriteConfig::default();
let dendrite = Dendrite::new(config)?;
```

### `async fn query<S: Synapse + Serialize>(&self, synapse: S, axon_info: &AxonInfo) -> Result<S, BittensorError>`

Sends a signed synapse request and returns the synapse with response metadata populated. This is the primary query method.

#### Parameters

| Name | Type | Description |
|---|---|---|
| `synapse` | `S` | The synapse value to send. Must implement `Synapse` and `Serialize`. |
| `axon_info` | `&AxonInfo` | Target axon metadata, including its IP, port, protocol, and hotkey. |

#### Returns

`Ok(S)` with the synapse's `axon` field populated from the response headers, or an error:

| Error | Condition |
|---|---|
| `BittensorError::Timeout` | The request exceeded the configured timeout |
| `BittensorError::Network` | Connection refused, DNS failure, or other transport error |
| `BittensorError::Signing` | The axon returned HTTP 401 |
| `BittensorError::Rpc` | The axon returned any other non-2xx status code |
| `BittensorError::Codec` | The synapse body failed to serialize to JSON |

```rust,no_run
let axon = AxonInfo {
    ip: 2130706433,  // 127.0.0.1
    port: 8091,
    ip_type: 4,
    protocol: 0,
    version: 1,
    hotkey: "5AxonHotkey".into(),
    coldkey: "5AxonColdkey".into(),
};
let synapse = TextPromptSynapse::new("What is Bittensor?", 128);
let response = dendrite.query(synapse, &axon).await?;
println!("Axon status: {:?}", response.axon().status_code);
```

### `async fn forward<S: Synapse + Serialize>(&self, synapse: S, axon_info: &AxonInfo) -> Result<S, BittensorError>`

An alias for `query`. Provided for API parity with the Python SDK, where `forward` and `query` have identical behavior but differ in name for readability in validator code. Calls `self.query(synapse, axon_info).await` directly.

### `async fn call<S: Synapse + Serialize>(&self, synapse: S, axon_info: &AxonInfo) -> Result<S, BittensorError>`

Another alias for `query`. Use `call` when the intent is "invoke a remote procedure and get the full result," and `query` when the intent is "ask a question." They are functionally identical.

### `async fn call_stream<S>(&self, synapse: S, axon_info: &AxonInfo) -> Result<S::StreamItem, BittensorError>`

Sends a signed synapse request and processes the SSE response stream. For `StreamingSynapse` types, each chunk is parsed into the stream item type. The method returns the first successfully parsed item.

#### SSE Protocol

The dendrite reads the response body as a byte stream and buffers incoming chunks. When a double-newline (`\n\n`) delimiter is found, the buffered data is treated as one SSE event. Within each event, lines starting with `data: ` are extracted. If the data payload is the literal `[DONE]`, that event is skipped. Otherwise, the data bytes are passed to `S::process_chunk`, and on success the resulting `StreamItem` is returned immediately.

If the stream ends without producing any items, returns `BittensorError::Network("stream ended without producing items")`.

#### Parameters

| Name | Type | Description |
|---|---|---|
| `synapse` | `S` | A `StreamingSynapse + Serialize` value to send. |
| `axon_info` | `&AxonInfo` | Target axon metadata. |

#### Returns

`Ok(S::StreamItem)` for the first parsed SSE event, or an error:

| Error | Condition |
|---|---|
| `BittensorError::Timeout` | The request exceeded the configured timeout |
| `BittensorError::Network` | Connection failed, or the stream ended without items |
| `BittensorError::Signing` | The axon returned HTTP 401 |
| `BittensorError::Rpc` | The axon returned a non-2xx, non-401 status |
| `BittensorError::Codec` | JSON serialization or `process_chunk` parsing failed |

```rust,no_run
let item = dendrite.call_stream(streaming_synapse, &axon).await?;
println!("Stream chunk: {item}");
```

---

## DendriteConfig

```rust
#[derive(Debug, Clone)]
pub struct DendriteConfig {
    pub timeout_secs: u64,
    pub max_connections: usize,
    pub hotkey: Option<Keypair>,
}
```

### Fields

| Field | Type | Default | Description |
|---|---|---|---|
| `timeout_secs` | `u64` | `12` | HTTP request timeout in seconds |
| `max_connections` | `usize` | `100` | Maximum idle connections per host in the connection pool |
| `hotkey` | `Option<Keypair>` | `None` | Sr25519 keypair for signing. If `None`, requests are sent unsigned |

### Constructors

#### `DendriteConfig::new() -> Self`

Creates a config with default values. Equivalent to `DendriteConfig::default()`.

```rust
let config = DendriteConfig::new();
assert_eq!(config.timeout_secs, 12);
assert_eq!(config.max_connections, 100);
assert!(config.hotkey.is_none());
```

#### `fn with_timeout_secs(self, secs: u64) -> Self`

Sets the request timeout. Consumes and returns `self` for chaining.

```rust
let config = DendriteConfig::new().with_timeout_secs(30);
assert_eq!(config.timeout_secs, 30);
```

#### `fn with_max_connections(self, max: usize) -> Self`

Sets the maximum number of idle connections per host. Consumes and returns `self`.

```rust
let config = DendriteConfig::new().with_max_connections(50);
assert_eq!(config.max_connections, 50);
```

#### `fn with_hotkey(self, keypair: Keypair) -> Self`

Sets the signing keypair. When set, every outbound request carries `bt-signature`, `bt-dendrite-hotkey`, `bt-nonce`, `bt-uuid`, and `bt-body-hash` headers. Consumes and returns `self`.

```rust
use subxt_signer::sr25519::dev::alice;

let config = DendriteConfig::new().with_hotkey(alice());
let dendrite = Dendrite::new(config)?;
```

### Builder Pattern Example

```rust
use subxt_signer::sr25519::dev::alice;

let keypair = alice();
let config = DendriteConfig::new()
    .with_timeout_secs(30)
    .with_max_connections(200)
    .with_hotkey(keypair);
let dendrite = Dendrite::new(config)?;
```

---

## Request Signing

### `sign_request`

```rust
pub fn sign_request(
    keypair: &Keypair,
    axon_hotkey: &str,
    body: &[u8],
    nonce: u64,
) -> Result<SignedRequest, BittensorError>
```

Signs an outgoing synapse request and returns the complete `SignedRequest` with all headers populated. This function is called internally by `Dendrite::query`, but you can also call it directly when constructing custom HTTP clients.

#### Parameters

| Name | Type | Description |
|---|---|---|
| `keypair` | `&Keypair` | The dendrite's Sr25519 signing keypair |
| `axon_hotkey` | `&str` | SS58-encoded hotkey of the target axon |
| `body` | `&[u8]` | Serialized request body (may be empty) |
| `nonce` | `u64` | Monotonically increasing counter (caller supplies) |

#### Returns

`Ok(SignedRequest)` on success, or `BittensorError::Signing` if a header name or value cannot be encoded (extremely unlikely with well-formed SS58 addresses).

### SignedRequest

```rust
pub struct SignedRequest {
    pub nonce: u64,
    pub uuid: String,
    pub body_hash: String,
    pub dendrite_hotkey: String,
    pub headers: HeaderMap,
}
```

| Field | Type | Description |
|---|---|---|
| `nonce` | `u64` | The monotonic counter used in the signing message |
| `uuid` | `String` | A v4 UUID identifying this request |
| `body_hash` | `String` | SHA3-256 hex digest of the request body |
| `dendrite_hotkey` | `String` | SS58-encoded public key of the signer |
| `headers` | `HeaderMap` | Complete set of `bt-*` headers to attach to the request |

### Header Output

The returned `HeaderMap` contains:

| Header | Value |
|---|---|
| `bt-nonce` | The supplied nonce as a decimal string |
| `bt-dendrite-hotkey` | SS58 address of the signer |
| `bt-axon-hotkey` | The `axon_hotkey` parameter |
| `bt-uuid` | A new v4 UUID string |
| `bt-body-hash` | SHA3-256 hex digest of `body` |
| `bt-signature` | `0x`-prefixed hex-encoded 64-byte Sr25519 signature |
| `accept` | `application/json` (for `query`/`forward`/`call`) or `text/event-stream` (for `call_stream`) |

### Signing Process

1. Compute `body_hash = sha3_256_hex(body)`.
2. Derive `dendrite_hotkey` from the keypair's public key via SS58 encoding.
3. Generate a new v4 UUID.
4. Construct the signing message: `"{nonce}.{dendrite_hotkey}.{axon_hotkey}.{uuid}.{body_hash}"`.
5. Sign the message bytes with the Sr25519 keypair.
6. Hex-encode the 64-byte signature with a `0x` prefix.
7. Build and populate the `HeaderMap`.

```rust
use bittensor_dendrite::signing::sign_request;
use subxt_signer::sr25519::dev::alice;

let keypair = alice();
let signed = sign_request(&keypair, "5AxonHotkey123", br#"{"prompt":"hello"}"#, 42)?;

assert!(signed.headers.contains_key("bt-signature"));
assert!(signed.headers.contains_key("bt-nonce"));
assert_eq!(signed.nonce, 42);
```

---

## Axon URL Construction

The dendrite constructs the target URL from `AxonInfo` fields:

```rust
fn axon_url(axon_info: &AxonInfo) -> String {
    let protocol = if axon_info.protocol == 0 { "http" } else { "https" };
    let ip = if axon_info.ip == 0 { "127.0.0.1".to_string() } else { ip_from_u64(axon_info.ip) };
    format!("{protocol}://{ip}:{}", axon_info.port)
}
```

The `protocol` field maps as follows:

| `protocol` value | URL scheme |
|---|---|
| `0` | `http` |
| `1` | `https` |

The `ip` field is a packed `u64` in network byte order. The conversion extracts four octets:

| Bit range | Octet |
|---|---|
| bits 24-31 | first octet (most significant) |
| bits 16-23 | second octet |
| bits 8-15 | third octet |
| bits 0-7 | fourth octet (least significant) |

For example, `2130706433` (hex `0x7F000001`) converts to `127.0.0.1`. If `ip` is `0`, the dendrite substitutes `127.0.0.1`.

Example results:

| `AxonInfo` | URL |
|---|---|
| `ip=2130706433, port=8090, protocol=0` | `http://127.0.0.1:8090` |
| `ip=2130706433, port=443, protocol=1` | `https://127.0.0.1:443` |
| `ip=16843009, port=80, protocol=0` | `http://1.1.1.1:80` |
| `ip=0, port=8080, protocol=0` | `http://127.0.0.1:8080` |

---

## Error Handling

The dendrite maps every failure mode into `bittensor_core::error::BittensorError` variants. Callers can check `error.is_retryable()` to decide whether to retry.

| Error Variant | Condition | Retryable |
|---|---|---|
| `BittensorError::Timeout(msg)` | HTTP request exceeded the configured timeout | Yes |
| `BittensorError::Network(msg)` | Connection refused, DNS failure, or other transport error | Yes |
| `BittensorError::Signing(msg)` | The axon returned 401, or a header could not be encoded | No |
| `BittensorError::Rpc(msg)` | The axon returned a non-2xx, non-401 status code | No (by default) |
| `BittensorError::Codec(msg)` | JSON serialization of the synapse body failed | No |

The `BittensorError::category()` method classifies errors into `ErrorCategory` values (`Transient`, `RateLimit`, `Auth`, `Config`, `Network`, `Permanent`), each of which carries a `RetryConfig` with tuned `max_retries`, `base_delay_ms`, `max_delay_ms`, and `backoff_factor` values.

---

## Code Examples

### Basic Query

```rust,no_run
use bittensor_dendrite::prelude::*;
use bittensor_core::types::AxonInfo;
use subxt_signer::sr25519::dev::alice;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = DendriteConfig::new().with_timeout_secs(15).with_hotkey(alice());
    let dendrite = Dendrite::new(config)?;

    let axon = AxonInfo {
        ip: 2130706433,
        port: 8091,
        ip_type: 4,
        protocol: 0,
        version: 1,
        hotkey: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".into(),
        coldkey: "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty".into(),
    };

    let synapse = MySynapse::new("hello");
    let response = dendrite.query(synapse, &axon).await?;
    println!("Response name: {}", response.name());
    Ok(())
}
```

### Streaming Query

```rust,no_run
use bittensor_dendrite::prelude::*;
use bittensor_core::types::AxonInfo;
use subxt_signer::sr25519::dev::alice;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = DendriteConfig::new().with_timeout_secs(60).with_hotkey(alice());
    let dendrite = Dendrite::new(config)?;

    let axon = AxonInfo {
        ip: 16843009,  // 1.1.1.1
        port: 443,
        ip_type: 4,
        protocol: 1,   // https
        version: 1,
        hotkey: "5TargetKey".into(),
        coldkey: "5TargetColdkey".into(),
    };

    let synapse = StreamingTextSynapse::new("Continue this story:", 512);

    // call_stream returns the first parsed SSE item
    let first_chunk = dendrite.call_stream(synapse, &axon).await?;
    println!("First chunk: {first_chunk}");

    Ok(())
}
```

### Custom Configuration

```rust
use bittensor_dendrite::prelude::*;
use subxt_signer::sr25519::dev::alice;

let config = DendriteConfig::new()
    .with_timeout_secs(30)
    .with_max_connections(200)
    .with_hotkey(alice());
let dendrite = Dendrite::new(config)?;
```

### Error Handling with Retry

```rust,no_run
use bittensor_dendrite::prelude::*;
use bittensor_core::error::BittensorError;
use bittensor_core::types::AxonInfo;
use std::time::Duration;

async fn query_with_retry(
    dendrite: &Dendrite,
    synapse: MySynapse,
    axon: &AxonInfo,
    max_attempts: u32,
) -> Result<MySynapse, BittensorError> {
    let mut attempt = 0;
    loop {
        match dendrite.query(synapse.clone(), axon).await {
            Ok(resp) => return Ok(resp),
            Err(e) if !e.is_retryable() => return Err(e),
            Err(e) if attempt >= max_attempts => return Err(e),
            Err(e) => {
                attempt += 1;
                let delay = Duration::from_millis(500 * 2u64.pow(attempt - 1));
                eprintln!("Attempt {attempt} failed ({e}), retrying in {:?}", delay);
                tokio::time::sleep(delay).await;
            }
        }
    }
}
```

### Unsigned Requests (Testing)

When no hotkey is set, the dendrite sends plain HTTP requests without `bt-*` signing headers. This is useful for local testing or querying non-Bittensor HTTP endpoints:

```rust,no_run
let config = DendriteConfig::new().with_timeout_secs(5);
let dendrite = Dendrite::new(config)?;

// No bt-signature header will be sent
let result = dendrite.query(my_synapse, &axon).await?;
```

The axon's `VerificationMiddleware` skips signature validation when `axon_hotkey` is `None`, so unsigned requests pass through on a test axon.

---

## Comparison with Python SDK

| Feature | Python `bittensor.dendrite` | Rust `bittensor-dendrite` |
|---|---|---|
| HTTP client | `aiohttp.ClientSession` | `reqwest::Client` |
| Signing | NaCl/Sr25519 via `nacl.signing` | Sr25519 via `subxt_signer` |
| Request format | POST with JSON body | POST with JSON body (identical) |
| Header format | `bt-*` headers (identical keys) | `bt-*` headers (identical keys) |
| Signing message | `"{nonce}.{dendrite}.{axon}.{uuid}.{hash}"` | `"{nonce}.{dendrite}.{axon}.{uuid}.{hash}"` (identical) |
| Streaming | SSE via `aiohttp` | SSE via `reqwest::bytes_stream` |
| SSE sentinel | `[DONE]` | `[DONE]` (identical) |
| Nonce source | `time.time()` float | `SystemTime` millis (monotonic AtomicU64) |
| Connection pool | `aiohttp.TCPConnector` limit | `reqwest` pool_max_idle_per_host |
| Timeout | Per-synapse `timeout` field | Config-level `timeout_secs` (12s default) |
| Error on 401 | Raises `StatusCodeError` | `BittensorError::Signing` |
| Error on non-2xx | Raises `StatusCodeError` | `BittensorError::Rpc` |
| Retry logic | None (caller responsibility) | None (caller responsibility) |
| `forward` alias | Present | Present (calls `query`) |
| `call` alias | Present | Present (calls `query`) |
| `call_stream` | Present | Present (returns first `StreamItem`) |
| `query` return type | Returns synapse object | Returns synapse with `axon` field populated |
| Thread safety | N/A (async, single-threaded) | `Send + Sync` (safe across tokio tasks) |

The Rust crate produces wire-compatible requests and can query any Python SDK axon, and vice versa. The signing message format is identical, so cross-SDK signature verification works without modification.
