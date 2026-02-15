//! SCALE encoding/decoding utilities
//!
//! This module provides utilities for SCALE (Simple Concatenated Aggregate Little-Endian) encoding
//! and decoding, which is the serialization format used by Substrate-based blockchains.
//!
//! The SCALE encoding matches the expectations of the subtensor chain:
//! - Fixed-size integers are encoded as little-endian
//! - Compact integers use the Compact<T> encoding
//! - Vectors are prefixed with their length as a compact integer
//! - Options are encoded as 0 (None) or 1 + value (Some)

use anyhow::Result;
use parity_scale_codec::{Compact, Decode, Encode};

/// Encode a value to SCALE bytes
///
/// # Type Parameters
/// * `T` - The type to encode, must implement `Encode`
///
/// # Returns
/// The SCALE-encoded bytes as a Vec<u8>
pub fn encode_scale<T: Encode>(value: &T) -> Vec<u8> {
    value.encode()
}

/// Decode SCALE bytes to a value
///
/// # Type Parameters
/// * `T` - The type to decode, must implement `Decode`
///
/// # Arguments
/// * `bytes` - The SCALE-encoded bytes
///
/// # Returns
/// The decoded value or an error if decoding fails
pub fn decode_scale<T: Decode>(bytes: &[u8]) -> Result<T> {
    T::decode(&mut &bytes[..]).map_err(|e| anyhow::anyhow!("Failed to decode SCALE: {:?}", e))
}

/// Decode Option<T> from SCALE bytes
///
/// # Type Parameters
/// * `T` - The type to decode, must implement `Decode`
///
/// # Arguments
/// * `bytes` - The SCALE-encoded Option bytes
///
/// # Returns
/// `Ok(Some(T))` if the option is Some, `Ok(None)` if None, or an error
pub fn decode_scale_option<T: Decode>(bytes: &[u8]) -> Result<Option<T>> {
    if bytes.is_empty() {
        return Ok(None);
    }

    // Option encoding: 0 = None, 1 followed by value = Some
    match bytes[0] {
        0 => Ok(None), // None variant
        1 if bytes.len() > 1 => T::decode(&mut &bytes[1..])
            .map(Some)
            .map_err(|e| anyhow::anyhow!("Failed to decode Option: {:?}", e)),
        1 => Err(anyhow::anyhow!(
            "Invalid Option encoding: missing value after Some marker"
        )),
        _ => Err(anyhow::anyhow!(
            "Invalid Option encoding: unknown variant {}",
            bytes[0]
        )),
    }
}

/// Decode Vec<T> from SCALE bytes
///
/// # Type Parameters
/// * `T` - The element type to decode, must implement `Decode`
///
/// # Arguments
/// * `bytes` - The SCALE-encoded Vec bytes
///
/// # Returns
/// The decoded Vec or an error
pub fn decode_scale_vec<T: Decode>(bytes: &[u8]) -> Result<Vec<T>> {
    // Vec encoding: compact length + items
    let mut input = bytes;
    let len_compact = Compact::<u32>::decode(&mut input)
        .map_err(|e| anyhow::anyhow!("Failed to decode Vec length: {:?}", e))?;

    let len = len_compact.0 as usize;
    // Pre-allocate capacity for efficiency
    let mut items = Vec::with_capacity(len);
    for _ in 0..len {
        let item = T::decode(&mut input)
            .map_err(|e| anyhow::anyhow!("Failed to decode Vec item: {:?}", e))?;
        items.push(item);
    }

    // Check for trailing bytes (potential data integrity issue)
    if !input.is_empty() {
        return Err(anyhow::anyhow!(
            "Unexpected trailing bytes after decoding Vec: {} bytes remaining",
            input.len()
        ));
    }

    Ok(items)
}

/// Decode a tuple from SCALE bytes
///
/// Alias for `decode_scale<T>` specialized for tuples.
///
/// # Type Parameters
/// * `T` - The type to decode, must implement `Decode`
///
/// # Arguments
/// * `bytes` - The SCALE-encoded bytes
///
/// # Returns
/// The decoded tuple or an error
pub fn decode_scale_tuple<T: Decode>(bytes: &[u8]) -> Result<T> {
    decode_scale(bytes)
}

/// Encode a value to SCALE compact encoding
///
/// This is useful for encoding lengths and indices in the compact format
/// used by Substrate for variable-length integers.
///
/// # Type Parameters
/// * `T` - The type to encode, must implement `Encode`
pub fn encode_compact<T: Encode>(value: &T) -> Vec<u8> {
    value.encode()
}

/// Encode an unsigned integer as compact
///
/// # Arguments
/// * `value` - The integer to encode
///
/// # Returns
/// The compact-encoded bytes
pub fn encode_u128_compact(value: u128) -> Vec<u8> {
    Compact(value).encode()
}

#[cfg(test)]
mod tests {
    use super::*;
    use parity_scale_codec::Encode;

    #[test]
    fn test_encode_decode_scale() {
        let value: u64 = 42;
        let encoded = encode_scale(&value);
        let decoded: u64 = decode_scale(&encoded).unwrap();
        assert_eq!(value, decoded);
    }

    #[test]
    fn test_encode_decode_vec() {
        let value = vec![1u32, 2, 3];
        let encoded = encode_scale(&value);
        let decoded: Vec<u32> = decode_scale_vec(&encoded).unwrap();
        assert_eq!(value, decoded);
    }

    #[test]
    fn test_decode_scale_option_some() {
        let some_value: u32 = 42;
        let mut encoded = vec![1u8]; // Some marker
        encoded.extend_from_slice(&some_value.encode());

        let decoded: Option<u32> = decode_scale_option(&encoded).unwrap();
        assert_eq!(decoded, Some(42));
    }

    #[test]
    fn test_decode_scale_option_none() {
        let encoded = vec![0u8]; // None marker
        let decoded: Option<u32> = decode_scale_option(&encoded).unwrap();
        assert_eq!(decoded, None);
    }

    #[test]
    fn test_decode_scale_option_empty() {
        let encoded: Vec<u8> = vec![];
        let decoded: Option<u32> = decode_scale_option(&encoded).unwrap();
        assert_eq!(decoded, None);
    }

    #[test]
    fn test_decode_scale_option_invalid() {
        let encoded = vec![2u8]; // Invalid variant
        let result: Result<Option<u32>> = decode_scale_option(&encoded);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_scale_vec_trailing_bytes() {
        // Vec with 3 u32 values, but with extra trailing bytes
        let mut encoded = encode_scale(&vec![1u32, 2, 3]);
        encoded.push(0xFF); // Add trailing byte

        let result: Result<Vec<u32>> = decode_scale_vec(&encoded);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("trailing bytes"));
    }

    #[test]
    fn test_encode_compact_u128() {
        let value: u128 = 1_000_000_000;
        let encoded = encode_u128_compact(value);
        let decoded = Compact::<u128>::decode(&mut &encoded[..]).unwrap();
        assert_eq!(decoded.0, value);
    }

    #[test]
    fn test_tuple_encode_decode() {
        let value = (1u32, 2u64, 3u16);
        let encoded = encode_scale(&value);
        let decoded: (u32, u64, u16) = decode_scale_tuple(&encoded).unwrap();
        assert_eq!(value, decoded);
    }

    #[test]
    fn test_u128_roundtrip() {
        let value: u128 = u128::MAX;
        let encoded = encode_scale(&value);
        let decoded: u128 = decode_scale(&encoded).unwrap();
        assert_eq!(value, decoded);
    }
}
