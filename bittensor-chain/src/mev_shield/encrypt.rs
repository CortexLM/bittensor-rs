//! ML-KEM-768 (Kyber) encryption for MEV Shield.
//!
//! Provides post-quantum key encapsulation using the `ml-kem` crate.
//! The encryptor takes a chain-provided ML-KEM-768 public key, encapsulates
//! a shared secret, then uses the shared secret as a symmetric key for
//! XOR-based encryption of the extrinsic payload.

use ml_kem::kem::{Decapsulate, DecapsulationKey, Encapsulate, EncapsulationKey};
use ml_kem::{Encoded, EncodedSizeUser, KemCore, MlKem768, MlKem768Params};

/// Errors from MEV Shield encryption.
#[derive(Debug, thiserror::Error)]
pub enum MevShieldEncryptError {
    #[error("Invalid public key length: expected {expected}, got {got}")]
    InvalidPublicKeyLength { expected: usize, got: usize },

    #[error("Invalid ciphertext length: expected {expected}, got {got}")]
    InvalidCiphertextLength { expected: usize, got: usize },

    #[error("Key encapsulation failed: {0}")]
    EncapsulationFailed(String),

    #[error("Key decapsulation failed: {0}")]
    DecapsulationFailed(String),

    #[error("Hex decode error: {0}")]
    HexDecode(String),
}

/// ML-KEM-768 encapsulation key size in bytes (1184).
pub const ML_KEM_768_EK_SIZE: usize = 1184;

/// ML-KEM-768 decapsulation key size in bytes (2400).
pub const ML_KEM_768_DK_SIZE: usize = 2400;

/// ML-KEM-768 ciphertext size in bytes (1088).
pub const ML_KEM_768_CT_SIZE: usize = 1088;

/// ML-KEM-768 shared secret size in bytes (32).
pub const ML_KEM_768_SS_SIZE: usize = 32;

/// An encrypted payload ready for chain submission.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EncryptedPayload {
    /// ML-KEM-768 ciphertext (encapsulation of the shared secret).
    pub kem_ciphertext: Vec<u8>,
    /// The extrinsic payload encrypted with the shared secret.
    pub encrypted_extrinsic: Vec<u8>,
}

/// MEV Shield encryptor using ML-KEM-768.
pub struct MevShieldEncrypt;

impl MevShieldEncrypt {
    /// Encrypt an extrinsic payload using an ML-KEM-768 public key.
    ///
    /// 1. Deserialize the public key bytes into an `EncapsulationKey`.
    /// 2. Encapsulate to produce (ciphertext, shared_secret).
    /// 3. XOR-encrypt the payload with the shared secret (via SHA-256 counter mode).
    /// 4. Return the KEM ciphertext + encrypted payload.
    pub fn encrypt(
        public_key_bytes: &[u8],
        plaintext: &[u8],
    ) -> Result<EncryptedPayload, MevShieldEncryptError> {
        if public_key_bytes.len() != ML_KEM_768_EK_SIZE {
            return Err(MevShieldEncryptError::InvalidPublicKeyLength {
                expected: ML_KEM_768_EK_SIZE,
                got: public_key_bytes.len(),
            });
        }

        // Build encapsulation key from raw bytes
        let ek_encoded = Encoded::<EncapsulationKey<MlKem768Params>>::try_from(public_key_bytes)
            .map_err(|_| MevShieldEncryptError::InvalidPublicKeyLength {
                expected: ML_KEM_768_EK_SIZE,
                got: public_key_bytes.len(),
            })?;
        let ek = EncapsulationKey::<MlKem768Params>::from_bytes(&ek_encoded);

        // Encapsulate: produces (ciphertext, shared_secret)
        let (ct, ss) = ek
            .encapsulate(&mut rand::thread_rng())
            .map_err(|e| MevShieldEncryptError::EncapsulationFailed(format!("{e:?}")))?;

        // XOR-encrypt payload with the shared secret (expanded via SHA-256 counter mode)
        let key = bytes32_from_slice(ss.as_slice())?;
        let encrypted = xor_encrypt(&key, plaintext);

        Ok(EncryptedPayload { kem_ciphertext: ct.to_vec(), encrypted_extrinsic: encrypted })
    }

    /// Decrypt an encrypted payload using an ML-KEM-768 decapsulation key.
    ///
    /// 1. Decapsulate the KEM ciphertext to recover the shared secret.
    /// 2. XOR-decrypt the encrypted extrinsic.
    pub fn decrypt(
        decapsulation_key_bytes: &[u8],
        payload: &EncryptedPayload,
    ) -> Result<Vec<u8>, MevShieldEncryptError> {
        if payload.kem_ciphertext.len() != ML_KEM_768_CT_SIZE {
            return Err(MevShieldEncryptError::InvalidCiphertextLength {
                expected: ML_KEM_768_CT_SIZE,
                got: payload.kem_ciphertext.len(),
            });
        }

        let dk_encoded =
            Encoded::<DecapsulationKey<MlKem768Params>>::try_from(decapsulation_key_bytes)
                .map_err(|_| {
                    MevShieldEncryptError::DecapsulationFailed(format!(
                        "invalid decapsulation key length: {}",
                        decapsulation_key_bytes.len()
                    ))
                })?;
        let dk = DecapsulationKey::<MlKem768Params>::from_bytes(&dk_encoded);

        let ct: &ml_kem::Ciphertext<MlKem768> =
            std::convert::TryFrom::try_from(&payload.kem_ciphertext[..]).map_err(|_| {
                MevShieldEncryptError::InvalidCiphertextLength {
                    expected: ML_KEM_768_CT_SIZE,
                    got: payload.kem_ciphertext.len(),
                }
            })?;

        // Decapsulate to recover shared secret
        let ss = dk
            .decapsulate(ct)
            .map_err(|e| MevShieldEncryptError::DecapsulationFailed(format!("{e:?}")))?;

        let key = bytes32_from_slice(ss.as_slice())?;
        let decrypted = xor_encrypt(&key, &payload.encrypted_extrinsic);
        Ok(decrypted)
    }

    /// Generate a new ML-KEM-768 keypair for testing.
    /// Returns (decapsulation_key_bytes, encapsulation_key_bytes).
    pub fn generate_keypair() -> (Vec<u8>, Vec<u8>) {
        let (dk, ek) = MlKem768::generate(&mut rand::thread_rng());
        (dk.as_bytes().to_vec(), ek.as_bytes().to_vec())
    }
}

/// Convert a slice to a [u8; 32], returning an error if the length is wrong.
fn bytes32_from_slice(slice: &[u8]) -> Result<[u8; 32], MevShieldEncryptError> {
    let mut arr = [0u8; 32];
    if slice.len() != 32 {
        return Err(MevShieldEncryptError::EncapsulationFailed(format!(
            "shared secret has wrong length: {}",
            slice.len()
        )));
    }
    arr.copy_from_slice(slice);
    Ok(arr)
}

/// XOR encryption with SHA-256 counter-mode key expansion.
fn xor_encrypt(key: &[u8; 32], data: &[u8]) -> Vec<u8> {
    let key_stream = expand_key_stream(key, data.len());
    data.iter().zip(key_stream.iter()).map(|(d, k)| d ^ k).collect()
}

/// Expand a 32-byte key into an arbitrary-length keystream via SHA-256 counter mode.
fn expand_key_stream(key: &[u8; 32], len: usize) -> Vec<u8> {
    use sha2::{Digest, Sha256};
    let mut stream = Vec::with_capacity(len);
    let mut counter = 0u64;
    while stream.len() < len {
        let mut hasher = Sha256::new();
        hasher.update(key);
        hasher.update(counter.to_be_bytes());
        let hash = hasher.finalize();
        let remaining = len - stream.len();
        let take = remaining.min(32);
        stream.extend_from_slice(&hash[..take]);
        counter += 1;
    }
    stream
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Test 1: Keypair generation produces correct sizes ----

    #[test]
    fn keypair_sizes() {
        let (dk, ek) = MevShieldEncrypt::generate_keypair();
        assert_eq!(ek.len(), ML_KEM_768_EK_SIZE);
        assert_eq!(dk.len(), ML_KEM_768_DK_SIZE);
    }

    // ---- Test 2: Encrypt/decrypt round-trip ----

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let (dk, ek) = MevShieldEncrypt::generate_keypair();
        let plaintext = b"hello mev shield world";
        let encrypted = MevShieldEncrypt::encrypt(&ek, plaintext).unwrap();
        let decrypted = MevShieldEncrypt::decrypt(&dk, &encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    // ---- Test 3: KEM ciphertext is correct size ----

    #[test]
    fn kem_ciphertext_size() {
        let (_, ek) = MevShieldEncrypt::generate_keypair();
        let encrypted = MevShieldEncrypt::encrypt(&ek, b"test").unwrap();
        assert_eq!(encrypted.kem_ciphertext.len(), ML_KEM_768_CT_SIZE);
    }

    // ---- Test 4: Wrong decapsulation key fails ----

    #[test]
    fn wrong_decapsulation_key_fails() {
        let (_, ek) = MevShieldEncrypt::generate_keypair();
        let (dk2, _) = MevShieldEncrypt::generate_keypair();
        let encrypted = MevShieldEncrypt::encrypt(&ek, b"secret data").unwrap();
        let result = MevShieldEncrypt::decrypt(&dk2, &encrypted);
        // Decapsulation is infallible in ML-KEM (returns Kbar on failure),
        // so the decrypted output won't match the plaintext
        match result {
            Ok(decrypted) => assert_ne!(decrypted, b"secret data"),
            Err(_) => {} // also acceptable
        }
    }

    // ---- Test 5: Invalid public key length is rejected ----

    #[test]
    fn invalid_public_key_length_rejected() {
        let short_key = vec![0u8; 100];
        let result = MevShieldEncrypt::encrypt(&short_key, b"test");
        assert!(matches!(result, Err(MevShieldEncryptError::InvalidPublicKeyLength { .. })));
    }

    // ---- Test 6: Invalid ciphertext length is rejected ----

    #[test]
    fn invalid_ciphertext_length_rejected() {
        let (dk, _) = MevShieldEncrypt::generate_keypair();
        let payload = EncryptedPayload {
            kem_ciphertext: vec![0u8; 100], // wrong size
            encrypted_extrinsic: vec![1, 2, 3],
        };
        let result = MevShieldEncrypt::decrypt(&dk, &payload);
        assert!(matches!(result, Err(MevShieldEncryptError::InvalidCiphertextLength { .. })));
    }

    // ---- Test 7: Empty plaintext round-trip ----

    #[test]
    fn empty_plaintext_roundtrip() {
        let (dk, ek) = MevShieldEncrypt::generate_keypair();
        let plaintext = b"";
        let encrypted = MevShieldEncrypt::encrypt(&ek, plaintext).unwrap();
        let decrypted = MevShieldEncrypt::decrypt(&dk, &encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }
}
