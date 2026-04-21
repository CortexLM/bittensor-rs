//! Encrypted extrinsic submission via MEV Shield.
//!
//! Fetches the on-chain NextKey (ML-KEM-768 public key), encrypts the
//! extrinsic payload, and submits it via `submit_encrypted_extrinsic`.

use crate::mev_shield::encrypt::{
    EncryptedPayload, ML_KEM_768_EK_SIZE, MevShieldEncrypt, MevShieldEncryptError,
};

/// Errors from MEV Shield submission.
#[derive(Debug, thiserror::Error)]
pub enum MevShieldSubmitError {
    #[error("Encryption error: {0}")]
    Encryption(#[from] MevShieldEncryptError),

    #[error("Chain error: {0}")]
    Chain(String),

    #[error("NextKey not available on chain")]
    NextKeyUnavailable,

    #[error("NextKey has invalid length: {0}")]
    NextKeyInvalidLength(usize),
}

/// Result type for MEV Shield operations.
type Result<T> = std::result::Result<T, MevShieldSubmitError>;

/// MEV Shield submitter — handles encrypted extrinsic submission.
pub struct MevShieldSubmit;

impl MevShieldSubmit {
    /// Encrypt an extrinsic payload using an ML-KEM-768 public key
    /// fetched from chain, and format it for `submit_encrypted_extrinsic`.
    ///
    /// This function handles the encryption step. The actual RPC call
    /// to submit the encrypted extrinsic is done by the caller, since
    /// it depends on the subxt client which is already set up elsewhere.
    pub fn encrypt_extrinsic(
        next_key_bytes: &[u8],
        extrinsic_data: &[u8],
    ) -> Result<EncryptedPayload> {
        if next_key_bytes.len() != ML_KEM_768_EK_SIZE {
            return Err(MevShieldSubmitError::NextKeyInvalidLength(next_key_bytes.len()));
        }
        MevShieldEncrypt::encrypt(next_key_bytes, extrinsic_data).map_err(Into::into)
    }

    /// Decode an on-chain response after encrypted extrinsic processing.
    ///
    /// The response from the chain after `submit_encrypted_extrinsic`
    /// contains the decrypted result. This function verifies it
    /// matches the expected format.
    pub fn decode_response(response: &[u8]) -> Result<Vec<u8>> {
        // The on-chain response format: length prefix (4 bytes LE) + data
        if response.len() < 4 {
            return Err(MevShieldSubmitError::Chain("response too short".to_string()));
        }
        let len = u32::from_le_bytes(
            response[..4]
                .try_into()
                .map_err(|_| MevShieldSubmitError::Chain("invalid length prefix".to_string()))?,
        ) as usize;
        if response.len() < 4 + len {
            return Err(MevShieldSubmitError::Chain(format!(
                "response truncated: expected {len} bytes, got {}",
                response.len() - 4
            )));
        }
        Ok(response[4..4 + len].to_vec())
    }

    /// Format the encrypted payload as SCALE-encoded bytes for
    /// submission to the `submit_encrypted_extrinsic` call.
    ///
    /// SCALE encoding: kem_ciphertext_len (u32 LE) + kem_ciphertext +
    ///                 encrypted_extrinsic_len (u32 LE) + encrypted_extrinsic
    pub fn scale_encode_payload(payload: &EncryptedPayload) -> Vec<u8> {
        let mut out = Vec::new();
        let ct_len = payload.kem_ciphertext.len() as u32;
        out.extend_from_slice(&ct_len.to_le_bytes());
        out.extend_from_slice(&payload.kem_ciphertext);
        let ext_len = payload.encrypted_extrinsic.len() as u32;
        out.extend_from_slice(&ext_len.to_le_bytes());
        out.extend_from_slice(&payload.encrypted_extrinsic);
        out
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Test 1: Encrypt extrinsic with valid key ----

    #[test]
    fn encrypt_extrinsic_with_valid_key() {
        let (_, ek) = MevShieldEncrypt::generate_keypair();
        let extrinsic = b"0x1234abcd";
        let result = MevShieldSubmit::encrypt_extrinsic(&ek, extrinsic);
        assert!(result.is_ok());
        let payload = result.unwrap();
        assert!(!payload.kem_ciphertext.is_empty());
        assert!(!payload.encrypted_extrinsic.is_empty());
    }

    // ---- Test 2: Reject invalid NextKey length ----

    #[test]
    fn reject_invalid_next_key_length() {
        let bad_key = vec![0u8; 100];
        let result = MevShieldSubmit::encrypt_extrinsic(&bad_key, b"test");
        assert!(matches!(result, Err(MevShieldSubmitError::NextKeyInvalidLength(100))));
    }

    // ---- Test 3: SCALE encode payload round-trip ----

    #[test]
    fn scale_encode_decode_roundtrip() {
        let payload = EncryptedPayload {
            kem_ciphertext: vec![0xAA; 1088],
            encrypted_extrinsic: vec![0xBB; 256],
        };
        let encoded = MevShieldSubmit::scale_encode_payload(&payload);

        // Verify structure
        let ct_len = u32::from_le_bytes(encoded[..4].try_into().unwrap()) as usize;
        assert_eq!(ct_len, 1088);
        assert_eq!(&encoded[4..4 + 1088], &payload.kem_ciphertext[..]);
        let ext_len = u32::from_le_bytes(encoded[4 + 1088..8 + 1088].try_into().unwrap()) as usize;
        assert_eq!(ext_len, 256);
        assert_eq!(&encoded[8 + 1088..], &payload.encrypted_extrinsic[..]);
    }

    // ---- Test 4: Decode valid response ----

    #[test]
    fn decode_valid_response() {
        let data = b"hello chain";
        let mut response = vec![];
        response.extend_from_slice(&(data.len() as u32).to_le_bytes());
        response.extend_from_slice(data);
        let decoded = MevShieldSubmit::decode_response(&response).unwrap();
        assert_eq!(decoded, data);
    }

    // ---- Test 5: Reject too-short response ----

    #[test]
    fn reject_short_response() {
        let result = MevShieldSubmit::decode_response(&[0u8; 2]);
        assert!(result.is_err());
    }

    // ---- Test 6: Reject truncated response ----

    #[test]
    fn reject_truncated_response() {
        let mut response = vec![];
        response.extend_from_slice(&1000u32.to_le_bytes()); // claims 1000 bytes
        response.extend_from_slice(b"short");
        let result = MevShieldSubmit::decode_response(&response);
        assert!(result.is_err());
    }

    // ---- Test 7: Decode empty data response ----

    #[test]
    fn decode_empty_data_response() {
        let mut response = vec![];
        response.extend_from_slice(&0u32.to_le_bytes());
        let decoded = MevShieldSubmit::decode_response(&response).unwrap();
        assert!(decoded.is_empty());
    }
}
