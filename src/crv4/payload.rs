//! CRv4 Payload structures
//!
//! Defines the payload format that matches subtensor's expected structure.

use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

/// Weights payload for CRv4 timelock encryption
///
/// This structure MUST match subtensor's `WeightsTlockPayload`:
/// ```ignore
/// pub struct WeightsTlockPayload {
///     pub hotkey: Vec<u8>,
///     pub uids: Vec<u16>,
///     pub values: Vec<u16>,
///     pub version_key: u64,
/// }
/// ```
///
/// The payload is SCALE-encoded before encryption.
#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct WeightsTlockPayload {
    /// Hotkey public key bytes (32 bytes for sr25519)
    /// IMPORTANT: Must match the hotkey signing the transaction!
    pub hotkey: Vec<u8>,
    /// Neuron UIDs to set weights for
    pub uids: Vec<u16>,
    /// Weight values (0-65535 scale)
    pub values: Vec<u16>,
    /// Network version key
    pub version_key: u64,
}

impl WeightsTlockPayload {
    /// Create a new weights payload
    pub fn new(hotkey: Vec<u8>, uids: Vec<u16>, values: Vec<u16>, version_key: u64) -> Self {
        Self {
            hotkey,
            uids,
            values,
            version_key,
        }
    }

    /// Encode payload to bytes using SCALE codec
    pub fn encode_payload(&self) -> Vec<u8> {
        self.encode()
    }

    /// Decode payload from bytes
    pub fn decode_payload(data: &[u8]) -> Result<Self, parity_scale_codec::Error> {
        Self::decode(&mut &data[..])
    }
}

/// Legacy payload format (without hotkey verification)
/// Used for backwards compatibility with older chain versions
#[derive(Clone, Debug, Encode, Decode)]
pub struct LegacyWeightsTlockPayload {
    pub uids: Vec<u16>,
    pub values: Vec<u16>,
    pub version_key: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payload_encode_decode() {
        let payload = WeightsTlockPayload {
            hotkey: vec![1u8; 32],
            uids: vec![0, 1, 2],
            values: vec![10000, 20000, 35535],
            version_key: 1,
        };

        let encoded = payload.encode_payload();
        let decoded = WeightsTlockPayload::decode_payload(&encoded).unwrap();

        assert_eq!(payload.hotkey, decoded.hotkey);
        assert_eq!(payload.uids, decoded.uids);
        assert_eq!(payload.values, decoded.values);
        assert_eq!(payload.version_key, decoded.version_key);
    }
}
