//! SCALE encoding/decoding utilities
use anyhow::Result;
use parity_scale_codec::{Compact, Decode, Encode};

/// Encode a value to SCALE bytes
pub fn encode_scale<T: Encode>(value: &T) -> Vec<u8> {
    value.encode()
}

/// Decode SCALE bytes to a value
pub fn decode_scale<T: Decode>(bytes: &[u8]) -> Result<T> {
    T::decode(&mut &bytes[..]).map_err(|e| anyhow::anyhow!("Failed to decode SCALE: {:?}", e))
}

/// Decode Option<T> from SCALE bytes
pub fn decode_scale_option<T: Decode>(bytes: &[u8]) -> Result<Option<T>> {
    if bytes.is_empty() {
        return Ok(None);
    }

    // Option encoding: 0 = None, 1 followed by value = Some
    if bytes[0] == 0 {
        Ok(None)
    } else if bytes.len() > 1 {
        T::decode(&mut &bytes[1..])
            .map(Some)
            .map_err(|e| anyhow::anyhow!("Failed to decode Option: {:?}", e))
    } else {
        Err(anyhow::anyhow!("Invalid Option encoding"))
    }
}

/// Decode Vec<T> from SCALE bytes
pub fn decode_scale_vec<T: Decode>(bytes: &[u8]) -> Result<Vec<T>> {
    // Vec encoding: compact length + items
    let mut input = bytes;
    let len_compact = Compact::<u32>::decode(&mut input)
        .map_err(|e| anyhow::anyhow!("Failed to decode Vec length: {:?}", e))?;

    let len = len_compact.0 as usize;
    let mut items = Vec::new();
    for _ in 0..len {
        let item = T::decode(&mut input)
            .map_err(|e| anyhow::anyhow!("Failed to decode Vec item: {:?}", e))?;
        items.push(item);
    }

    Ok(items)
}

/// Decode a tuple from SCALE bytes
pub fn decode_scale_tuple<T: Decode>(bytes: &[u8]) -> Result<T> {
    decode_scale(bytes)
}
