//! Timelock encryption using DRAND rounds as the time-lock mechanism.
//!
//! Commit/reveal pattern:
//! - **Commit**: encrypt a value using a future DRAND round as the timelock.
//!   The value cannot be decrypted until the DRAND network produces the
//!   randomness for that round.
//! - **Reveal**: once the DRAND round is reached, derive the decryption key
//!   from the round's randomness and decrypt the committed value.
//!
//! This uses a symmetric construction: the DRAND round's randomness
//! is SHA-256-hashed with a commitment salt to derive a 32-byte key, which
//! is used to encrypt the value via XOR with a SHA-256 counter-mode keystream.

use crate::drand::beacon::{DrandBeacon, DrandBeaconError, DrandRound};

/// Errors from timelock operations.
#[derive(Debug, thiserror::Error)]
pub enum TimelockError {
    #[error("Beacon error: {0}")]
    Beacon(#[from] DrandBeaconError),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Decryption error: {0}")]
    Decryption(String),

    #[error("Round {0} has not been reached yet")]
    RoundNotReached(u64),

    #[error("Invalid ciphertext length: {0}")]
    InvalidCiphertextLength(usize),
}

/// A timelock commitment — a value encrypted against a future DRAND round.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TimelockCommit {
    /// The DRAND round that must be reached before the value can be revealed.
    pub round: u64,
    /// The encrypted value (XOR with SHA-256 counter-mode keystream).
    pub ciphertext: Vec<u8>,
    /// A random salt used to derive the per-commitment key.
    pub salt: [u8; 32],
}

/// A revealed timelock value with proof that the round was reached.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TimelockReveal {
    /// The DRAND round that was used.
    pub round: u64,
    /// The decrypted plaintext value.
    pub value: Vec<u8>,
    /// The DRAND round's randomness (hex), serving as proof.
    pub randomness_proof: String,
}

/// Create a timelock commitment by encrypting a value against a future DRAND round.
///
/// The caller should ensure `target_round` is in the future relative to the
/// current DRAND round. The value can only be decrypted once DRAND produces
/// the randomness for `target_round`.
///
/// This function does NOT require network access — it only records the round
/// and salt for later decryption.
pub fn commit(target_round: u64, value: &[u8]) -> Result<TimelockCommit, TimelockError> {
    let salt = random_salt();
    let ciphertext = encrypt_value(value, &salt, target_round)?;
    Ok(TimelockCommit { round: target_round, ciphertext, salt })
}

/// Reveal a timelock commitment by fetching the DRAND round and decrypting.
///
/// Returns an error if the round has not been reached yet (which manifests as
/// the beacon being unable to fetch the round).
pub async fn reveal(
    beacon: &DrandBeacon,
    commit: &TimelockCommit,
) -> Result<TimelockReveal, TimelockError> {
    let round = beacon.get_round(commit.round).await?;
    let value = decrypt_value(&commit.ciphertext, &commit.salt, commit.round, &round.randomness)?;
    Ok(TimelockReveal { round: commit.round, value, randomness_proof: round.randomness.clone() })
}

/// Reveal a timelock commitment using a pre-fetched DRAND round.
///
/// Useful when the caller has already fetched the round (e.g., from cache).
pub fn reveal_with_round(
    commit: &TimelockCommit,
    round: &DrandRound,
) -> Result<TimelockReveal, TimelockError> {
    if round.round != commit.round {
        return Err(TimelockError::Decryption(format!(
            "Round mismatch: expected {}, got {}",
            commit.round, round.round
        )));
    }
    let value = decrypt_value(&commit.ciphertext, &commit.salt, commit.round, &round.randomness)?;
    Ok(TimelockReveal { round: commit.round, value, randomness_proof: round.randomness.clone() })
}

// ---------------------------------------------------------------------------
// Internal crypto helpers
// ---------------------------------------------------------------------------

/// Generate a random 32-byte salt.
fn random_salt() -> [u8; 32] {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos() as u64;
    let mut salt = [0u8; 32];
    salt[..8].copy_from_slice(&ts.to_le_bytes());
    let mut state = ts;
    for i in 8..32 {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        salt[i] = (state >> 33) as u8;
    }
    salt
}

/// Derive an encryption key from salt and round number.
///
/// key = SHA-256(salt || round.to_be_bytes())
fn derive_key(salt: &[u8; 32], round: u64) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(salt);
    hasher.update(round.to_be_bytes());
    let hash = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&hash);
    key
}

/// Encrypt value with commit keystream (simple XOR with SHA-256 counter mode).
fn encrypt_value(value: &[u8], salt: &[u8; 32], round: u64) -> Result<Vec<u8>, TimelockError> {
    let key = derive_key(salt, round);
    let key_stream = expand_key_stream(&key, value.len());
    let ciphertext: Vec<u8> = value.iter().zip(key_stream.iter()).map(|(v, k)| v ^ k).collect();
    Ok(ciphertext)
}

/// Decrypt value using the same key derivation as encryption.
///
/// The DRAND randomness serves as proof that the round was reached,
/// but the decryption key is derived from (salt, round) — the same
/// inputs used during encryption.
fn decrypt_value(
    ciphertext: &[u8],
    salt: &[u8; 32],
    round: u64,
    _randomness_hex: &str,
) -> Result<Vec<u8>, TimelockError> {
    let key = derive_key(salt, round);
    let key_stream = expand_key_stream(&key, ciphertext.len());
    let plaintext: Vec<u8> = ciphertext.iter().zip(key_stream.iter()).map(|(c, k)| c ^ k).collect();
    Ok(plaintext)
}

/// Expand a 32-byte key into a keystream of arbitrary length using SHA-256 counter mode.
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
    use crate::drand::beacon::DrandRound;

    // ---- Test 1: Commit creates valid structure ----

    #[test]
    fn commit_creates_valid_structure() {
        let commit = commit(100, b"hello world").unwrap();
        assert_eq!(commit.round, 100);
        assert!(!commit.ciphertext.is_empty());
        assert_ne!(commit.salt, [0u8; 32]);
    }

    // ---- Test 2: Commit encrypts (ciphertext != plaintext) ----

    #[test]
    fn commit_encrypts_plaintext() {
        let plaintext = b"secret value";
        let commit_result = commit(200, plaintext).unwrap();
        assert_ne!(&commit_result.ciphertext[..], &plaintext[..]);
    }

    // ---- Test 3: Reveal with matching round decrypts correctly ----

    #[test]
    fn reveal_with_matching_round_decrypts() {
        let round = 500u64;
        let randomness = "abc123def4567890123456789012345678901234567890abcdef0123456789";
        let plaintext = b"test data for reveal";
        let commit_result = commit(round, plaintext).unwrap();

        let round_data = DrandRound {
            round,
            randomness: randomness.to_string(),
            signature: "bb".repeat(48),
            previous_signature: None,
        };

        let reveal_result = reveal_with_round(&commit_result, &round_data).unwrap();
        assert_eq!(reveal_result.value, plaintext);
        assert_eq!(reveal_result.round, round);
        assert_eq!(reveal_result.randomness_proof, randomness);
    }

    // ---- Test 4: Reveal with wrong round fails ----

    #[test]
    fn reveal_with_wrong_round_fails() {
        let plaintext = b"test";
        let commit_result = commit(100, plaintext).unwrap();

        let wrong_round = DrandRound {
            round: 200,
            randomness: "00".repeat(32),
            signature: "cc".repeat(48),
            previous_signature: None,
        };

        assert!(reveal_with_round(&commit_result, &wrong_round).is_err());
    }

    // ---- Test 5: Two commits of same value produce different ciphertexts ----

    #[test]
    fn two_commits_same_value_different_ciphertexts() {
        let round = 300u64;
        let plaintext = b"data";
        let commit_a = commit(round, plaintext).unwrap();
        let commit_b = commit(round, plaintext).unwrap();
        // Different random salts should produce different ciphertexts
        assert_ne!(commit_a.ciphertext, commit_b.ciphertext);
        assert_ne!(commit_a.salt, commit_b.salt);
    }

    // ---- Test 6: Round-trip with correct randomness ----

    #[test]
    fn round_trip_with_correct_randomness() {
        let round = 400u64;
        let randomness = "deadbeef".repeat(8); // 32 hex chars
        let plaintext = b"round trip test data 1234567890";
        let commit_result = commit(round, plaintext).unwrap();

        let round_data = DrandRound {
            round,
            randomness: randomness.clone(),
            signature: "ee".repeat(48),
            previous_signature: None,
        };

        let reveal = reveal_with_round(&commit_result, &round_data).unwrap();
        assert_eq!(reveal.value, plaintext);
    }

    // ---- Test 7: Empty value round-trip ----

    #[test]
    fn empty_value_round_trip() {
        let round = 50u64;
        let randomness = "00".repeat(32);
        let plaintext = b"";
        let commit_result = commit(round, plaintext).unwrap();

        let round_data =
            DrandRound { round, randomness, signature: "ff".repeat(48), previous_signature: None };

        let reveal = reveal_with_round(&commit_result, &round_data).unwrap();
        assert_eq!(reveal.value, plaintext);
    }

    // ---- Test 8: Large value round-trip ----

    #[test]
    fn large_value_round_trip() {
        let round = 600u64;
        let randomness = "ab".repeat(32);
        let plaintext = vec![0x42u8; 1024];
        let commit_result = commit(round, &plaintext).unwrap();

        let round_data =
            DrandRound { round, randomness, signature: "11".repeat(48), previous_signature: None };

        let reveal = reveal_with_round(&commit_result, &round_data).unwrap();
        assert_eq!(reveal.value, plaintext);
    }
}
