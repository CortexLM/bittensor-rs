//! Cryptographic utilities for Bittensor operations
//!
//! This module provides cryptographic primitives for:
//! - Commit-reveal mechanism for weights
//! - Salt generation for cryptographic operations
//! - Blake2b hashing (used by Subtensor)
//!
//! # SR25519 Signature Compatibility
//!
//! The Bittensor network uses SR25519 (Schnorr signatures on Ristretto25519) for signing.
//! SR25519 signatures are exactly 64 bytes in the format defined by the schnorrkel crate:
//! - First 32 bytes: the Ristretto25519 point R (compressed)
//! - Second 32 bytes: the scalar s (response)
//!
//! This format is compatible with the Python Bittensor SDK's use of substrate-interface
//! and the sp_core::sr25519 implementation in Substrate.

use anyhow::Result;
use hex;
use parity_scale_codec::Encode;
use sp_core::blake2_256;

/// Generate a commit hash for weights using Blake2b (legacy format)
/// Uses u16 format for weights to match Subtensor's expected format
///
/// NOTE: This is a legacy function. For commit-reveal with subtensor,
/// use `generate_subtensor_commit_hash` instead.
pub fn commit_weights_hash(uids: &[u64], weights: &[u16], salt: &[u8]) -> Vec<u8> {
    // Serialize UIDs (as u64), weights (as u16), and salt
    let mut data = Vec::new();
    for uid in uids {
        data.extend_from_slice(&uid.to_le_bytes());
    }
    for weight in weights {
        data.extend_from_slice(&weight.to_le_bytes());
    }
    data.extend_from_slice(salt);

    // Hash with Blake2b
    blake2_256(&data).to_vec()
}

/// Generate a commit hash that matches subtensor's exact format.
///
/// Subtensor computes:
/// ```ignore
/// BlakeTwo256::hash_of(&(who, netuid_index, uids, values, salt, version_key))
/// ```
///
/// This is a SCALE-encoded tuple hashed with Blake2b-256.
///
/// # Arguments
/// * `account` - The hotkey's public key (32 bytes)
/// * `netuid` - The subnet ID (u16)
/// * `mechanism_id` - Optional mechanism ID for sub-subnet (default 0)
/// * `uids` - Vector of neuron UIDs (u16)
/// * `values` - Vector of weight values (u16)
/// * `salt` - Random salt (Vec<u16>)
/// * `version_key` - Network version key (u64)
///
/// # Returns
/// 32-byte Blake2b-256 hash matching subtensor's format
pub fn generate_subtensor_commit_hash(
    account: &[u8; 32],
    netuid: u16,
    mechanism_id: Option<u8>,
    uids: &[u16],
    values: &[u16],
    salt: &[u16],
    version_key: u64,
) -> [u8; 32] {
    // Calculate netuid_index (NetUidStorageIndex)
    // In subtensor: netuid_index = get_mechanism_storage_index(netuid, mecid)
    // GLOBAL_MAX_SUBNET_COUNT = 4096
    // Formula: mecid * 4096 + netuid
    // IMPORTANT: NetUidStorageIndex is u16 in subtensor, not u32!
    let mecid = mechanism_id.unwrap_or(0) as u16;
    let netuid_index: u16 = mecid.saturating_mul(4096).saturating_add(netuid);

    // Create SCALE-encodable tuple matching subtensor's format
    // The tuple is: (AccountId, NetUidStorageIndex, Vec<u16>, Vec<u16>, Vec<u16>, u64)
    // NetUidStorageIndex is a newtype over u16, so it SCALE encodes as u16
    let data = (
        account,         // [u8; 32] - AccountId
        netuid_index,    // u16 - NetUidStorageIndex (NOT u32!)
        uids.to_vec(),   // Vec<u16>
        values.to_vec(), // Vec<u16>
        salt.to_vec(),   // Vec<u16>
        version_key,     // u64
    );

    // SCALE encode and hash
    let encoded = data.encode();
    blake2_256(&encoded)
}

/// Generate commit hash for mechanism weights (sub-subnet)
///
/// This is a convenience wrapper around `generate_subtensor_commit_hash` for mechanism weights.
pub fn generate_mechanism_commit_hash(
    account: &[u8; 32],
    netuid: u16,
    mechanism_id: u8,
    uids: &[u16],
    values: &[u16],
    salt: &[u16],
    version_key: u64,
) -> [u8; 32] {
    generate_subtensor_commit_hash(
        account,
        netuid,
        Some(mechanism_id),
        uids,
        values,
        salt,
        version_key,
    )
}

/// Verify a commit hash matches the provided data.
///
/// Used to validate reveals before submission.
#[allow(clippy::too_many_arguments)]
pub fn verify_commit_hash(
    commit_hash: &[u8; 32],
    account: &[u8; 32],
    netuid: u16,
    mechanism_id: Option<u8>,
    uids: &[u16],
    values: &[u16],
    salt: &[u16],
    version_key: u64,
) -> bool {
    let computed = generate_subtensor_commit_hash(
        account,
        netuid,
        mechanism_id,
        uids,
        values,
        salt,
        version_key,
    );
    computed == *commit_hash
}

/// Convert commit hash to hex string
pub fn commit_hash_to_hex(hash: &[u8]) -> String {
    hex::encode(hash)
}

/// Parse hex string to commit hash
pub fn hex_to_commit_hash(hex_str: &str) -> Result<Vec<u8>> {
    let decoded = hex::decode(hex_str).map_err(|e| anyhow::anyhow!("Invalid hex string: {}", e))?;
    Ok(decoded)
}

/// Parse hex string to 32-byte commit hash
pub fn hex_to_commit_hash_32(hex_str: &str) -> Result<[u8; 32]> {
    let bytes = hex::decode(hex_str).map_err(|e| anyhow::anyhow!("Invalid hex string: {}", e))?;
    if bytes.len() != 32 {
        return Err(anyhow::anyhow!(
            "Hash must be 32 bytes, got {}",
            bytes.len()
        ));
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(arr)
}

/// Generate random salt as Vec<u16> for commit-reveal
///
/// Subtensor expects salt as Vec<u16>, not Vec<u8>
pub fn generate_salt(len: usize) -> Vec<u16> {
    use rand::Rng;
    let mut rng = rand::rng();
    (0..len).map(|_| rng.random::<u16>()).collect()
}

/// Convert u8 salt to u16 salt (for backwards compatibility)
pub fn salt_u8_to_u16(salt: &[u8]) -> Vec<u16> {
    salt.iter().map(|b| *b as u16).collect()
}

/// Verify that a signature is valid (64 bytes for SR25519)
///
/// This function performs basic format validation on SR25519 signatures.
/// For full verification, use `Keypair::verify` or `sr25519::Pair::verify`.
///
/// # Arguments
/// * `signature` - The signature bytes
///
/// # Returns
/// `true` if the signature format is valid (64 bytes)
pub fn is_valid_sr25519_signature(signature: &[u8]) -> bool {
    signature.len() == 64
}

/// Get SR25519 signature format description for debugging
///
/// # Arguments
/// * `signature` - The signature bytes
///
/// # Returns
/// A description of the signature format
pub fn signature_format_info(signature: &[u8]) -> String {
    if signature.len() != 64 {
        return format!(
            "Invalid SR25519 signature: expected 64 bytes, got {}",
            signature.len()
        );
    }
    format!(
        "Valid SR25519 signature: 64 bytes (R point: 32 bytes, scalar s: 32 bytes)\n\
         R (compressed): 0x{}...\n\
         s (response):   0x{}",
        hex::encode(&signature[0..4]),
        hex::encode(&signature[32..36])
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_subtensor_commit_hash() {
        let account = [1u8; 32];
        let netuid = 1u16;
        let uids = vec![0u16, 1, 2];
        let values = vec![1000u16, 2000, 3000];
        let salt = vec![123u16, 456];
        let version_key = 1u64;

        let hash = generate_subtensor_commit_hash(
            &account,
            netuid,
            None,
            &uids,
            &values,
            &salt,
            version_key,
        );

        // Hash should be deterministic
        let hash2 = generate_subtensor_commit_hash(
            &account,
            netuid,
            None,
            &uids,
            &values,
            &salt,
            version_key,
        );
        assert_eq!(hash, hash2);

        // Different salt should produce different hash
        let hash3 = generate_subtensor_commit_hash(
            &account,
            netuid,
            None,
            &uids,
            &values,
            &[789u16],
            version_key,
        );
        assert_ne!(hash, hash3);
    }

    #[test]
    fn test_verify_commit_hash() {
        let account = [2u8; 32];
        let netuid = 5u16;
        let uids = vec![1u16, 2];
        let values = vec![30000u16, 35535];
        let salt = vec![1u16, 2, 3, 4];
        let version_key = 100u64;

        let hash = generate_subtensor_commit_hash(
            &account,
            netuid,
            None,
            &uids,
            &values,
            &salt,
            version_key,
        );

        assert!(verify_commit_hash(
            &hash,
            &account,
            netuid,
            None,
            &uids,
            &values,
            &salt,
            version_key,
        ));

        // Wrong version key should fail
        assert!(!verify_commit_hash(
            &hash,
            &account,
            netuid,
            None,
            &uids,
            &values,
            &salt,
            version_key + 1,
        ));
    }

    #[test]
    fn test_mechanism_commit_hash() {
        let account = [3u8; 32];
        let netuid = 1u16;
        let mechanism_id = 2u8;
        let uids = vec![0u16];
        let values = vec![65535u16];
        let salt = vec![100u16];
        let version_key = 1u64;

        // Main mechanism (id=0) should differ from mechanism 2
        let hash_main = generate_subtensor_commit_hash(
            &account,
            netuid,
            Some(0),
            &uids,
            &values,
            &salt,
            version_key,
        );
        let hash_mec2 = generate_mechanism_commit_hash(
            &account,
            netuid,
            mechanism_id,
            &uids,
            &values,
            &salt,
            version_key,
        );
        assert_ne!(hash_main, hash_mec2);
    }

    #[test]
    fn test_generate_salt() {
        let salt = generate_salt(8);
        assert_eq!(salt.len(), 8);

        // Salt should be random (very unlikely to be all zeros)
        let all_zero = salt.iter().all(|&s| s == 0);
        assert!(!all_zero);
    }

    #[test]
    fn test_is_valid_sr25519_signature() {
        // Valid signature is exactly 64 bytes
        let valid_sig = [0u8; 64];
        assert!(is_valid_sr25519_signature(&valid_sig));

        // Invalid signatures
        assert!(!is_valid_sr25519_signature(&[0u8; 63])); // Too short
        assert!(!is_valid_sr25519_signature(&[0u8; 65])); // Too long
        assert!(!is_valid_sr25519_signature(&[0u8; 0])); // Empty
    }

    #[test]
    fn test_signature_format_info() {
        let valid_sig = [0xABu8; 64];
        let info = signature_format_info(&valid_sig);
        assert!(info.contains("Valid SR25519"));

        let invalid_sig = [0u8; 32];
        let info = signature_format_info(&invalid_sig);
        assert!(info.contains("Invalid SR25519"));
    }

    #[test]
    fn test_hex_encoding_roundtrip() {
        let hash = [0xDEu8, 0xAD, 0xBE, 0xEF];
        let hex_str = commit_hash_to_hex(&hash);
        let decoded = hex_to_commit_hash(&hex_str).unwrap();
        assert_eq!(hash.to_vec(), decoded);

        // Test 32-byte hash
        let hash32 = [0xABu8; 32];
        let hex_str32 = commit_hash_to_hex(&hash32);
        let decoded32 = hex_to_commit_hash_32(&hex_str32).unwrap();
        assert_eq!(hash32, decoded32);
    }
}
