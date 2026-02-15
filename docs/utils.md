# Utilities

This document describes utility functions for encoding, decoding, address conversion, and weight normalization.

## Weight Operations

### normalize_weights

Normalize weights to ensure they sum to 1.0 and convert to fixed-point representation for Subtensor.

```rust
pub fn normalize_weights(
    uids: &[u64],
    weights: &[f32]
) -> Result<(Vec<u16>, Vec<u16>)>
```

**Parameters:**
- `uids`: Vector of neuron UIDs
- `weights`: Vector of weight values (f32)

**Returns:**
- `(Vec<u16>, Vec<u16>)`: Tuple of (UIDs, weights) in u16 format
- Weights are scaled by `u16::MAX` (65535) where 65535 represents 1.0

**Example:**
```rust
use bittensor_rs::utils::normalize_weights;

let uids = vec![1, 2, 3];
let weights = vec![0.5, 0.3, 0.2];
let (uid_vals, weight_vals) = normalize_weights(&uids, &weights)?;
```

### denormalize_weights

Convert weights from fixed-point (u16) back to float values.

```rust
pub fn denormalize_weights(weight_vals: &[u16]) -> Vec<f32>
```

**Parameters:**
- `weight_vals`: Vector of u16 weight values

**Returns:**
- `Vec<f32>`: Normalized float weights

**Example:**
```rust
use bittensor_rs::utils::denormalize_weights;

let weight_vals = vec![32767, 19660, 13107]; // ~0.5, 0.3, 0.2
let weights = denormalize_weights(&weight_vals);
```

## Address Conversion

### account_to_ss58

Convert an AccountId32 to SS58 string representation.

```rust
pub fn account_to_ss58(account: &AccountId32) -> String
```

**Parameters:**
- `account`: AccountId32 to convert

**Returns:**
- `String`: SS58-encoded address

### ss58_to_account

Convert an SS58 string to AccountId32.

```rust
pub fn ss58_to_account(ss58: &str) -> Result<AccountId32>
```

**Parameters:**
- `ss58`: SS58-encoded address string

**Returns:**
- `Result<AccountId32>`: Decoded account ID

**Example:**
```rust
use bittensor_rs::utils::{ss58_to_account, account_to_ss58};
use sp_core::crypto::AccountId32;

let ss58 = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
let account = ss58_to_account(ss58)?;
let back_to_ss58 = account_to_ss58(&account);
```

## Balance Utilities

### rao_to_tao

Convert RAO (smallest on-chain unit) to TAO for display.

```rust
pub fn rao_to_tao(rao: u128) -> f64
```

**Parameters:**
- `rao`: Amount in RAO

**Returns:**
- `f64`: Amount in TAO (1 TAO = 10^9 RAO). Values are display-only and may lose precision above 2^53 RAO.

### tao_to_rao

Convert TAO to RAO using truncation toward zero.

```rust
pub fn tao_to_rao(tao: f64) -> u128
```

**Parameters:**
- `tao`: Amount in TAO

**Returns:**
- `u128`: Amount in RAO (on-chain unit)

**Example:**
```rust
use bittensor_rs::utils::{rao_to_tao, tao_to_rao};

let rao = 1_000_000_000u128;
let tao = rao_to_tao(rao); // 1.0
let back_to_rao = tao_to_rao(tao); // 1_000_000_000
```

## SCALE Encoding/Decoding

### encode_scale

Encode a value using SCALE encoding.

```rust
pub fn encode_scale<T: Encode>(value: &T) -> Vec<u8>
```

**Parameters:**
- `value`: Value implementing `Encode` trait

**Returns:**
- `Vec<u8>`: SCALE-encoded bytes

### decode_scale

Decode a SCALE-encoded value.

```rust
pub fn decode_scale<T: Decode>(bytes: &[u8]) -> Result<T>
```

**Parameters:**
- `bytes`: SCALE-encoded bytes

**Returns:**
- `Result<T>`: Decoded value

## Value Decoding

Utilities for decoding `Value` types from subxt storage results.

### decode_vec_u16

Decode a vector of u16 values from a `Value`.

```rust
pub fn decode_vec_u16(value: &Value) -> Result<Vec<u16>>
```

### decode_vec_u64

Decode a vector of u64 values from a `Value`.

```rust
pub fn decode_vec_u64(value: &Value) -> Result<Vec<u64>>
```

### decode_vec_bool

Decode a vector of boolean values from a `Value`.

```rust
pub fn decode_vec_bool(value: &Value) -> Result<Vec<bool>>
```

### decode_option

Decode an `Option<T>` from a `Value`.

```rust
pub fn decode_option<T>(value: &Value) -> Result<Option<T>>
```

## Cryptographic Utilities

### commit_weights_hash

Generate a commitment hash for weight values in commit-reveal schemes.

```rust
pub fn commit_weights_hash(
    uids: &[u16],
    weights: &[u16],
    salt: &[u16]
) -> Result<[u8; 32]>
```

**Parameters:**
- `uids`: Vector of u16 UIDs
- `weights`: Vector of u16 weight values
- `salt`: Salt for commitment

**Returns:**
- `Result<[u8; 32]>`: 32-byte commitment hash

**Example:**
```rust
use bittensor_rs::utils::commit_weights_hash;

let uids = vec![1u16, 2u16, 3u16];
let weights = vec![32767u16, 19660u16, 13107u16];
let salt = vec![42u16, 43u16, 44u16];
let commitment = commit_weights_hash(&uids, &weights, &salt)?;
```

## Error Handling

All utility functions return `Result<T>` types. Handle errors appropriately:

```rust
use bittensor_rs::utils::*;

match normalize_weights(&uids, &weights) {
    Ok((uid_vals, weight_vals)) => {
        // Use normalized values
    }
    Err(e) => {
        eprintln!("Error normalizing weights: {}", e);
    }
}
```

## Usage

Import utilities from the main crate:

```rust
use bittensor_rs::utils::{
    normalize_weights, denormalize_weights,
    ss58_to_account, account_to_ss58,
    rao_to_tao, tao_to_rao,
    commit_weights_hash
};
```

## Performance Considerations

- Weight normalization automatically filters out zero weights to reduce payload size
- Address conversions are lightweight operations
- SCALE encoding/decoding is optimized for Substrate compatibility
- Value decoding handles various format representations (U128, U64, U8) for robustness