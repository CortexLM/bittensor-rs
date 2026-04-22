# bittensor-axon

Axum-based neuron server with verification, blacklisting, priority routing, and synapse handler registration.

## Overview

The `bittensor-axon` crate provides the server half of the Bittensor synapse protocol. An axon is an HTTP server that receives signed requests from dendrites, verifies signatures and body hashes, filters blacklisted keys, assigns request priorities, and dispatches to handler closures registered per synapse type.

The crate has no optional features. Everything described here is always available.

### Crate

```toml
[dependencies]
bittensor-axon = "0.1"
```

### Prelude

```rust
use bittensor_axon::prelude::*;
```

The prelude re-exports:

| Item | Source |
|---|---|
| `Axon` | `axon` module |
| `AxonError` | `axon` module |
| `AxonConfig` | `config` module |
| `MiddlewareState` | `middleware` module |
| `RequestPriority` | `middleware` module |
| `blacklist_middleware` | `middleware` module |
| `body_hash_middleware` | `middleware` module |
| `priority_middleware` | `middleware` module |
| `verification_middleware` | `middleware` module |
| `SynapseRegistry` | `router` module |
| `register_synapse_route` | `router` module |

---

## Protocol Flow

When a dendrite sends a signed synapse request to an axon, the request passes through a layered middleware stack before reaching the handler:

```
Incoming HTTP POST
  |
  v
Extension layer  ──  MiddlewareState injected as request extension
  |
  v
VerificationMiddleware  ──  Checks bt_header_dendrite_signature and signing fields
  |
  v
BlacklistMiddleware  ──  Rejects requests from blacklisted hotkeys (403)
  |
  v
PriorityMiddleware  ──  Assigns numeric priority, injects x-request-priority header
  |
  v
BodyHashMiddleware  ──  Re-hashes body, compares to computed_body_hash (400 on mismatch)
  |
  v
Route handler  ──  Your closure for the matching synapse name
```

Middleware is applied in reverse order during layer construction (outermost runs first), matching the Python SDK's ordering: verification, blacklist, priority, body hash.

---

## Axon Server

```rust
pub struct Axon {
    // private fields
}

impl Axon {
    pub fn new(config: AxonConfig) -> Self;
    pub fn attach<H, T>(self, synapse_name: &str, handler: H) -> Self;
    pub async fn start(&mut self) -> Result<SocketAddr, AxonError>;
    pub fn stop(&self) -> Result<(), AxonError>;
    pub async fn forward(_request: axum::extract::Request) -> Response;
    pub fn middleware_state(&self) -> &MiddlewareState;
    pub fn config(&self) -> &AxonConfig;
    pub async fn blacklist(&self, hotkey: &str);
    pub async fn unblacklist(&self, hotkey: &str);
    pub async fn set_priority(&self, hotkey: &str, priority: u32);
}
```

### `Axon::new(config: AxonConfig) -> Self`

Creates a new axon from the given configuration. This call initializes the middleware chain and a 404 fallback for unregistered routes. The server does not bind to a port until you call `start`.

The `AxonConfig` parameter controls the bind address, port, external IP, and hotkey identity. See the `AxonConfig` section below.

```rust
let config = AxonConfig::default();
let axon = Axon::new(config);
```

### `fn attach<H, T>(self, synapse_name: &str, handler: H) -> Self`

Registers a POST handler at `/{synapse_name}`. Returns `self` for chaining. The handler must be an `axum::Handler` that can be converted to a tower service.

When a dendrite sends a request to `POST /TextPrompt`, the axon routes it to whatever handler was attached under that name. If no handler matches, the fallback returns 404.

```rust
let axon = Axon::new(config)
    .attach("TextPrompt", || async { "pong" })
    .attach("Embedding", my_embedding_handler);
```

### `async fn start(&mut self) -> Result<SocketAddr, AxonError>`

Binds a TCP listener on the configured address and spawns the server in a background tokio task. Returns the actual `SocketAddr`, which is useful when `port` is set to `0` (OS-assigned port).

The server runs with graceful shutdown support. Calling `stop` sends a signal through a broadcast channel, and the server task exits cleanly.

```rust
let mut axon = Axon::new(AxonConfig { port: 0, ..Default::default() });
let addr = axon.start().await?;
println!("Listening on {addr}");
```

### `fn stop(&self) -> Result<(), AxonError>`

Sends a shutdown signal to the background server task. Returns `AxonError::Shutdown` if the broadcast channel is already closed (e.g. the server has already stopped).

```rust
axon.stop()?;
```

### `async fn forward(_request: axum::extract::Request) -> Response`

A default handler for unregistered routes. Returns HTTP 404 with the body `"no handler registered"`. You can use this as a fallback if you construct your own router.

### `fn middleware_state(&self) -> &MiddlewareState`

Returns a reference to the shared middleware state. Use this to inspect the current blacklist or priority map.

### `fn config(&self) -> &AxonConfig`

Returns a reference to the original `AxonConfig` used to build this axon.

### `async fn blacklist(&self, hotkey: &str)`

Adds a hotkey to the blacklist. Requests from this hotkey will receive HTTP 403 from `blacklist_middleware`. The blacklist is stored in an `Arc<RwLock<HashSet<String>>>` and can be modified at runtime.

```rust
axon.blacklist("5BadActorHotkey").await;
```

### `async fn unblacklist(&self, hotkey: &str)`

Removes a hotkey from the blacklist. Subsequent requests from this key will be accepted.

```rust
axon.unblacklist("5BadActorHotkey").await;
```

### `async fn set_priority(&self, hotkey: &str, priority: u32)`

Sets the numeric priority for a given hotkey. Higher values mean the request is served first. Unrecognized hotkeys default to priority 0.

The priority is attached as the `x-request-priority` response header and also injected into request extensions as a `RequestPriority` value, so handler code can read it.

```rust
axon.set_priority("5HighStakeKey", 10).await;
axon.set_priority("5LowStakeKey", 1).await;
```

---

## AxonConfig

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxonConfig {
    pub ip: String,
    pub port: u16,
    pub max_connections: usize,
    pub external_ip: Option<String>,
    pub hotkey: Option<String>,
}
```

Controls how the axon binds and identifies itself on the network.

### Fields

| Field | Type | Default | Description |
|---|---|---|---|
| `ip` | `String` | `"0.0.0.0"` | Bind address for the TCP listener |
| `port` | `u16` | `8090` | Listen port. Use `0` for OS-assigned |
| `max_connections` | `usize` | `0` | Maximum concurrent connections. `0` means unlimited |
| `external_ip` | `Option<String>` | `None` | IP advertised to the network. Falls back to `ip` |
| `hotkey` | `Option<String>` | `None` | Hotkey identity for verification. If `None`, verification is skipped |

### Constructors

#### `AxonConfig::new() -> Self`

Creates a config with default values.

#### `fn bind_addr(&self) -> String`

Returns the `ip:port` string for TCP binding.

```rust
let cfg = AxonConfig { ip: "127.0.0.1".to_string(), port: 3000, ..Default::default() };
assert_eq!(cfg.bind_addr(), "127.0.0.1:3000");
```

#### `fn external_ip_or_ip(&self) -> &str`

Returns the external IP if set, otherwise the bind IP. Use this when advertising the axon endpoint to the chain or to dendrites.

```rust
let cfg = AxonConfig {
    ip: "10.0.0.1".to_string(),
    external_ip: Some("1.2.3.4".to_string()),
    ..Default::default()
};
assert_eq!(cfg.external_ip_or_ip(), "1.2.3.4");
```

---

## Middleware

All middleware functions share the same signature:

```rust
pub async fn some_middleware(request: Request, next: Next) -> Response
```

They are axum middleware functions added via `middleware::from_fn`. They run in the order they are layered, with the outermost running first.

### Shared State: MiddlewareState

```rust
#[derive(Debug, Clone)]
pub struct MiddlewareState {
    pub axon_hotkey: Option<String>,
    pub blacklist: Arc<RwLock<HashSet<String>>>,
    pub priority_map: Arc<RwLock<HashMap<String, u32>>>,
}
```

Injected as an axum `Extension` during `Axon::new`. Every middleware reads from this shared state.

| Field | Type | Description |
|---|---|---|
| `axon_hotkey` | `Option<String>` | The axon's own hotkey. If `None`, verification is bypassed |
| `blacklist` | `Arc<RwLock<HashSet<String>>>` | Set of hotkeys to reject |
| `priority_map` | `Arc<RwLock<HashMap<String, u32>>>` | Priority values per hotkey |

### RequestPriority Extension

```rust
#[derive(Debug, Clone, Copy)]
pub struct RequestPriority(pub u32);
```

Injected into request extensions by `priority_middleware`. Handler code can extract it:

```rust
use axum::Extension;

async fn my_handler(Extension(pri): Extension<RequestPriority>) -> &'static str {
    println!("Priority: {}", pri.0);
    "ok"
}
```

### Middleware Header Names

The middleware reads and writes specific header keys:

| Constant | Value | Set By | Read By |
|---|---|---|---|
| `headers::NONCE` | `bt_header_dendrite_nonce` | Dendrite | Verification, Priority |
| `headers::DENDRITE_HOTKEY` | `bt_header_dendrite_hotkey` | Dendrite | Blacklist, Priority, Verification |
| `headers::AXON_HOTKEY` | `bt_header_axon_hotkey` | Dendrite | Verification |
| `headers::UUID` | `bt_header_dendrite_uuid` | Dendrite | Verification |
| `headers::COMPUTED_BODY_HASH` | `computed_body_hash` | Dendrite | Verification, BodyHash |
| `headers::SIGNATURE` | `bt_header_dendrite_signature` | Dendrite | Verification |
| `headers::REQUEST_PRIORITY` | `x-request-priority` | Priority (response) | Downstream handlers |

### 1. verification_middleware

Checks that the dendrite signature is present and that the signing fields can be extracted. If `MiddlewareState::axon_hotkey` is `None`, verification is skipped entirely (useful for local testing).

Rejection: HTTP 401 if the `bt_header_dendrite_signature` header is missing or signing fields are malformed.

In a full implementation, this middleware would also verify the Sr25519 signature against the dendrite hotkey's public key. The current version validates that the header is present and that all five signing fields parse correctly.

### 2. blacklist_middleware

Looks up the `bt_header_dendrite_hotkey` header in the shared blacklist set. If the hotkey is found, the request is rejected.

Rejection: HTTP 403 with body `"hotkey is blacklisted"`.

If the hotkey header is missing or empty, the request passes through (no key to filter on).

### 3. priority_middleware

Looks up the `bt_header_dendrite_hotkey` in the priority map. If the hotkey is not found, priority defaults to `0`.

Side effects:
- Injects `RequestPriority(priority)` into request extensions
- Adds `x-request-priority: {priority}` to the response headers

This middleware never rejects a request.

### 4. body_hash_middleware

Reads the `computed_body_hash` header. If present, it consumes the request body, re-hashes it with SHA3-256, and compares against the header value using constant-time comparison to prevent timing attacks.

Rejection: HTTP 400 with body `"body hash mismatch"` if the hashes differ, or `"failed to read body"` if the body cannot be read.

If the `computed_body_hash` header is absent, the middleware passes the request through without checking.

---

## SynapseRegistry

```rust
#[derive(Debug, Clone)]
pub struct SynapseRegistry {
    handlers: Arc<RwLock<HashMap<String, String>>>,
}

impl SynapseRegistry {
    pub fn new() -> Self;
    pub async fn register(&self, synapse_name: &str, route_path: &str);
    pub async fn get_route(&self, synapse_name: &str) -> Option<String>;
    pub async fn len(&self) -> usize;
    pub async fn is_empty(&self) -> bool;
}
```

A thread-safe registry that maps synapse names to route paths. The `Axon::attach` method uses `register_synapse_route` internally, but you can use the `SynapseRegistry` directly if you need to inspect registered routes at runtime.

### `fn new() -> Self`

Creates an empty registry.

### `async fn register(&self, synapse_name: &str, route_path: &str)`

Adds or overwrites a mapping from synapse name to route path.

### `async fn get_route(&self, synapse_name: &str) -> Option<String>`

Looks up the route path for a synapse name. Returns `None` if not registered.

### `async fn len(&self) -> usize`

Returns the number of registered handlers.

### `async fn is_empty(&self) -> bool`

Returns `true` if no handlers are registered.

---

## register_synapse_route

```rust
pub fn register_synapse_route<H, T>(router: Router, synapse_name: &str, handler: H) -> Router
where
    H: Handler<T, ()>,
    T: 'static,
```

Adds a POST route at `/{synapse_name}` to the given axum `Router`. This is what `Axon::attach` calls internally. You can also use it directly if you are constructing a custom router.

```rust
use bittensor_axon::router::register_synapse_route;
use axum::Router;

let router = Router::new();
let router = register_synapse_route(router, "TextPrompt", || async { "response" });
```

---

## AxonError

```rust
#[derive(Debug, thiserror::Error)]
pub enum AxonError {
    #[error("bind error: {0}")]
    Bind(String),
    #[error("shutdown error: {0}")]
    Shutdown(String),
}
```

### Variants

| Variant | When | Description |
|---|---|---|
| `Bind(msg)` | TCP listener fails to bind | Port already in use, invalid address, etc. |
| `Shutdown(msg)` | Failed to send shutdown signal | The broadcast channel was already closed |

---

## Complete Example: Neuron Server with Middleware

This example shows a complete axon server that registers two synapse handlers, configures the middleware stack, manages a blacklist, and shuts down gracefully on CTRL+C.

```rust
use bittensor_axon::prelude::*;
use bittensor_core::config::NetworkConfig;
use bittensor_chain::prelude::SubtensorClient;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), AxonError> {
    // Build the axon with default config (0.0.0.0:8090)
    // In production, load hotkey from wallet and set it here
    let config = AxonConfig {
        port: 8091,
        hotkey: Some("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".to_string()),
        ..Default::default()
    };

    let mut axon = Axon::new(config)
        .attach("TextPrompt", text_prompt_handler)
        .attach("Embedding", embedding_handler);

    // Start listening
    let addr = axon.start().await?;
    println!("Axon serving at {addr}");

    // Configure runtime security policies
    axon.blacklist("5KnownBadActor").await;
    axon.set_priority("5HighStakeValidator", 10).await;
    axon.set_priority("5MediumStakeValidator", 5).await;

    // Wait for CTRL+C, then shut down
    signal::ctrl_c()
        .await
        .expect("failed to listen for ctrl+c");

    println!("Shutting down...");
    axon.stop()
}

// Handler: receives a POST to /TextPrompt
async fn text_prompt_handler() -> &'static str {
    // In a real server, you would deserialize the synapse body,
    // run inference, and return the result.
    "placeholder completion"
}

// Handler: receives a POST to /Embedding
async fn embedding_handler() -> &'static str {
    // Return embedding vectors here
    "[0.1, 0.2, 0.3]"
}
```

### OS-Assigned Port for Testing

When writing tests, use port `0` to let the OS pick an available port:

```rust
#[tokio::test]
async fn test_axon_bind() {
    let config = AxonConfig { port: 0, ..Default::default() };
    let mut axon = Axon::new(config);
    let addr = axon.start().await.unwrap();
    assert!(addr.port() > 0);
    axon.stop().unwrap();
}
```

### Custom Router with Middleware

If you need finer control, you can build the router and middleware stack manually:

```rust
use axum::Router;
use axum::routing::post;
use axum::middleware;
use bittensor_axon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

async fn ok_handler() -> &'static str {
    "ok"
}

#[tokio::main]
async fn main() {
    let state = MiddlewareState {
        axon_hotkey: Some("5MyHotkey".to_string()),
        blacklist: Arc::new(RwLock::new(HashSet::new())),
        priority_map: Arc::new(RwLock::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/TextPrompt", post(ok_handler))
        .fallback(|| async { axum::http::StatusCode::NOT_FOUND.into_response() })
        .layer(middleware::from_fn(body_hash_middleware))
        .layer(middleware::from_fn(priority_middleware))
        .layer(middleware::from_fn(blacklist_middleware))
        .layer(middleware::from_fn(verification_middleware))
        .layer(axum::Extension(state));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8091").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

### Runtime Blacklist Management

You can add or remove hotkeys from the blacklist while the server is running. The changes take effect immediately for the next request:

```rust
// Block a key that is sending bad requests
axon.blacklist("5SpamKey").await;

// Later, unblock it if the issue is resolved
axon.unblacklist("5SpamKey").await;

// Check current blacklist contents
let blacklist = axon.middleware_state().blacklist.read().await;
for key in blacklist.iter() {
    println!("Blocked: {key}");
}
```
