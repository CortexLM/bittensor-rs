//! CRv4 Timelock Encryption
//!
//! Implements the TLE (Timelock Encryption) using DRAND Quicknet beacon.
//! The encryption is based on BLS12-381 curve (TinyBLS381).

use crate::crv4::{DrandInfo, WeightsTlockPayload, DRAND_QUICKNET_PK_HEX};
use anyhow::Result;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use parity_scale_codec::Encode;
use rand_chacha::{rand_core::SeedableRng, ChaCha20Rng};
use sha2::{Digest, Sha256};
use tle::{
    curves::drand::TinyBLS381, ibe::fullident::Identity,
    stream_ciphers::AESGCMStreamCipherProvider, tlock::tle,
};
use w3f_bls::EngineBLS;

/// Encrypt weights payload for CRv4 commit
///
/// # Arguments
/// * `hotkey` - Hotkey public key bytes (32 bytes)
/// * `uids` - Neuron UIDs
/// * `weights` - Weight values (u16, 0-65535)
/// * `version_key` - Network version key
/// * `reveal_round` - DRAND round number for decryption
///
/// # Returns
/// Encrypted and compressed ciphertext bytes
pub fn prepare_crv4_commit(
    hotkey: &[u8],
    uids: &[u16],
    weights: &[u16],
    version_key: u64,
    reveal_round: u64,
) -> Result<Vec<u8>> {
    // Create payload
    let payload = WeightsTlockPayload {
        hotkey: hotkey.to_vec(),
        uids: uids.to_vec(),
        values: weights.to_vec(),
        version_key,
    };

    // SCALE encode the payload
    let serialized_payload = payload.encode();

    // Encrypt with TLE
    encrypt_for_round(&serialized_payload, reveal_round)
}

/// Encrypt arbitrary data for a specific DRAND round
///
/// Uses the TLE (Timelock Encryption) scheme with:
/// - BLS12-381 curve (TinyBLS381)
/// - AES-GCM stream cipher
/// - DRAND Quicknet public key
pub fn encrypt_for_round(data: &[u8], reveal_round: u64) -> Result<Vec<u8>> {
    // Get DRAND public key
    let pk_bytes = hex::decode(DRAND_QUICKNET_PK_HEX)
        .map_err(|e| anyhow::anyhow!("Failed to decode DRAND public key: {}", e))?;

    let pub_key = <TinyBLS381 as EngineBLS>::PublicKeyGroup::deserialize_compressed(&*pk_bytes)
        .map_err(|e| anyhow::anyhow!("Failed to deserialize DRAND public key: {:?}", e))?;

    // Create identity from round number
    // Identity = SHA256(round.to_be_bytes())
    let message = {
        let mut hasher = Sha256::new();
        hasher.update(reveal_round.to_be_bytes());
        hasher.finalize().to_vec()
    };
    let identity = Identity::new(b"", vec![message]);

    // Generate ephemeral secret key (random 32 bytes)
    let rng = ChaCha20Rng::from_entropy();
    let esk: [u8; 32] = rand::random();

    // Encrypt using TLE
    let ciphertext = tle::<TinyBLS381, AESGCMStreamCipherProvider, ChaCha20Rng>(
        pub_key, esk, data, identity, rng,
    )
    .map_err(|e| anyhow::anyhow!("TLE encryption failed: {:?}", e))?;

    // Serialize compressed
    let mut commit_bytes = Vec::new();
    ciphertext
        .serialize_compressed(&mut commit_bytes)
        .map_err(|e| anyhow::anyhow!("Failed to serialize ciphertext: {:?}", e))?;

    Ok(commit_bytes)
}

/// Encrypt data for a future time (in blocks)
///
/// # Arguments
/// * `data` - Data to encrypt
/// * `blocks_until_reveal` - Number of blocks until data should be revealed
/// * `block_time` - Block time in seconds (default 12.0)
///
/// # Returns
/// (encrypted_data, reveal_round)
pub fn encrypt_for_blocks(
    data: &[u8],
    blocks_until_reveal: u64,
    block_time: f64,
) -> Result<(Vec<u8>, u64)> {
    let drand_info = DrandInfo::quicknet();

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let secs_until_reveal = (blocks_until_reveal as f64 * block_time) as u64;
    let reveal_time = now + secs_until_reveal;

    // Add buffer for safety
    let reveal_round = drand_info.round_at_time(reveal_time + drand_info.period);

    let encrypted = encrypt_for_round(data, reveal_round)?;

    Ok((encrypted, reveal_round))
}

/// Verify that encrypted data is valid
///
/// This doesn't decrypt, just checks the structure is valid.
pub fn verify_encrypted_data(encrypted: &[u8]) -> bool {
    use tle::tlock::TLECiphertext;

    let reader = &mut &encrypted[..];
    TLECiphertext::<TinyBLS381>::deserialize_compressed(reader).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_for_round() {
        let data = b"test payload data";
        let reveal_round = 1000u64;

        let encrypted = encrypt_for_round(data, reveal_round);
        assert!(encrypted.is_ok());

        let encrypted = encrypted.unwrap();
        assert!(!encrypted.is_empty());

        // Verify it's valid ciphertext
        assert!(verify_encrypted_data(&encrypted));
    }

    #[test]
    fn test_prepare_crv4_commit() {
        let hotkey = vec![1u8; 32];
        let uids = vec![0u16, 1, 2];
        let weights = vec![10000u16, 20000, 35535];
        let version_key = 1u64;
        let reveal_round = 1000u64;

        let encrypted = prepare_crv4_commit(&hotkey, &uids, &weights, version_key, reveal_round);
        assert!(encrypted.is_ok());

        let encrypted = encrypted.unwrap();
        assert!(!encrypted.is_empty());
        assert!(verify_encrypted_data(&encrypted));
    }

    #[test]
    fn test_encrypt_for_blocks() {
        let data = b"some data to encrypt";
        let blocks = 100u64;
        let block_time = 12.0;

        let result = encrypt_for_blocks(data, blocks, block_time);
        assert!(result.is_ok());

        let (encrypted, reveal_round) = result.unwrap();
        assert!(!encrypted.is_empty());
        assert!(reveal_round > 0);
        assert!(verify_encrypted_data(&encrypted));
    }
}
