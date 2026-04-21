# bittensor-synapse

Protocol types for Bittensor synapse communication: headers, hashing, signing, and streaming.

## Quick Start

```rust
use bittensor_synapse::{Synapse, TerminalInfo};

// Implement Synapse for your own request/response type
struct MySynapse {
    prompt: String,
    completion: Option<String>,
}

impl Synapse for MySynapse {
    fn name() -> &'static str { "MySynapse" }
    fn to_headers(&self) -> Vec<(String, String)> { /* ... */ }
    // ... other trait methods
}
```

## Feature Flags

No optional features — all functionality is always enabled.

## API Overview

| Module | Purpose |
|---|---|
| `synapse` | `Synapse` trait — define request/response protocol types |
| `header` | Header name constants used by the Bittensor protocol |
| `hashing` | SHA-3 hashing for body verification |
| `signing` | Message signing for request authentication |
| `streaming` | `StreamingSynapse` trait for chunked responses |
| `terminal_info` | `TerminalInfo` — axon response metadata |
