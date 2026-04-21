# bittensor-core

Shared types, errors, balance arithmetic, weight utilities, and proof-of-work for the bittensor-rs SDK.

## Quick Start

```rust
use bittensor_core::prelude::*;

// Balance arithmetic (1 TAO = 10^9 rao)
let balance = Balance::from_tao(1.5);
let fee = Balance::from_rao(500_000);
let net = balance - fee;
println!("Net: {net}"); // "1.499999500 TAO"

// Error classification
let err = BittensorError::timeout("connection timed out");
assert_eq!(err.category(), ErrorCategory::Network);
```

## Feature Flags

No optional features — all functionality is always enabled.

## API Overview

| Module | Purpose |
|---|---|
| `balance` | `Balance` type with checked/saturating arithmetic and TAO↔rao conversion |
| `config` | `NetworkConfig` for chain endpoints (finney, test, local) |
| `error` | `BittensorError`, `ErrorCategory`, `RetryConfig` |
| `pow` | `PowSolution` for registration difficulty checks |
| `types` | `AxonInfo`, `NeuronInfo`, `SubnetHyperparameters`, etc. |
| `weight_utils` | Weight normalization and validation |
