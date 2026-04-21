//! CPU-based Proof-of-Work solver for bittensor registration.
//!
//! Implements the POW algorithm: increment nonce from 0,
//! compute `seal = blake2b(nonce_seed || nonce_le_bytes || block_hash || block_number_le_bytes)`,
//! and check if `seal * difficulty < 2^256 - 1` (matching Python bittensor).

use blake2::Blake2b512;
use blake2::digest::Digest;

use crate::error::BittensorError;

/// Result of a successful POW solve.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PowSolution {
    /// The nonce that satisfies the difficulty condition.
    pub nonce: u64,
    /// The 32-byte blake2b seal for the winning nonce.
    pub seal: [u8; 32],
}

/// Solve the POW challenge by finding a nonce where
/// `seal_number * difficulty < 2^256 - 1` (the Python bittensor rule).
///
/// `seal_number` is the 32-byte blake2b digest interpreted as a big-endian u256.
/// Higher difficulty = harder to solve (fewer valid seals).
pub fn solve_pow(
    nonce_seed: &[u8],
    difficulty: u64,
    block_hash: [u8; 32],
    block_number: u64,
) -> Result<PowSolution, BittensorError> {
    if difficulty == 0 {
        let seal = compute_seal(nonce_seed, 0, &block_hash, block_number);
        let mut seal_bytes = [0u8; 32];
        seal_bytes.copy_from_slice(&seal[..32]);
        return Ok(PowSolution { nonce: 0, seal: seal_bytes });
    }

    let threshold = compute_pow_threshold(difficulty);
    let mut nonce: u64 = 0;

    loop {
        let seal = compute_seal(nonce_seed, nonce, &block_hash, block_number);

        if seal_below_threshold(&seal, &threshold) {
            let mut seal_bytes = [0u8; 32];
            seal_bytes.copy_from_slice(&seal[..32]);
            return Ok(PowSolution { nonce, seal: seal_bytes });
        }

        nonce = nonce
            .checked_add(1)
            .ok_or_else(|| BittensorError::Validation("POW nonce overflow".into()))?;
    }
}

/// Compute the blake2b seal hash for a given nonce.
pub fn compute_seal(
    nonce_seed: &[u8],
    nonce: u64,
    block_hash: &[u8; 32],
    block_number: u64,
) -> Vec<u8> {
    let mut hasher = Blake2b512::new();
    Digest::update(&mut hasher, nonce_seed);
    Digest::update(&mut hasher, nonce.to_le_bytes());
    Digest::update(&mut hasher, block_hash);
    Digest::update(&mut hasher, block_number.to_le_bytes());
    let result = hasher.finalize();
    result[..32].to_vec()
}

/// Check if a seal meets the difficulty requirement.
///
/// Python rule: `seal_number * difficulty < 2^256 - 1`
/// Equivalent to: `seal_number < floor((2^256 - 1) / difficulty)`
/// We precompute the threshold and compare seal bytes against it.
pub fn seal_meets_difficulty(seal: &[u8], difficulty: u64) -> bool {
    if difficulty == 0 {
        return true;
    }
    let threshold = compute_pow_threshold(difficulty);
    seal_below_threshold(seal, &threshold)
}

/// Compute `floor((2^256 - 1) / difficulty)` as 32 big-endian bytes.
fn compute_pow_threshold(difficulty: u64) -> [u8; 32] {
    let max_u256: [u8; 32] = [0xFF; 32];
    u256_div_u64(max_u256, difficulty)
}

/// Divide a 256-bit big-endian number by a u64, returning the quotient as 32 big-endian bytes.
fn u256_div_u64(dividend: [u8; 32], divisor: u64) -> [u8; 32] {
    let mut remainder: u128 = 0;
    let mut quotient = [0u8; 32];

    for i in 0..32 {
        remainder = (remainder << 8) | dividend[i] as u128;
        let q = (remainder / divisor as u128) as u8;
        remainder %= divisor as u128;
        quotient[i] = q;
    }

    quotient
}

/// Compare two 32-byte slices as big-endian integers: returns true if a < b.
fn seal_below_threshold(seal: &[u8], threshold: &[u8; 32]) -> bool {
    for i in 0..32 {
        if seal[i] < threshold[i] {
            return true;
        }
        if seal[i] > threshold[i] {
            return false;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solve_pow_easy_difficulty() {
        let nonce_seed = b"test-seed";
        let block_hash = [0xAB; 32];
        let difficulty = 1;

        let solution = solve_pow(nonce_seed, difficulty, block_hash, 100)
            .expect("should solve with difficulty=1");

        assert!(seal_meets_difficulty(&solution.seal, difficulty));
    }

    #[test]
    fn solve_pow_returns_correct_seal() {
        let nonce_seed = b"seed";
        let block_hash = [0x00; 32];
        let difficulty = 1;

        let solution = solve_pow(nonce_seed, difficulty, block_hash, 0).expect("solve");

        let expected_seal = compute_seal(nonce_seed, solution.nonce, &block_hash, 0);
        assert_eq!(solution.seal.to_vec(), expected_seal[..32]);
    }

    #[test]
    fn compute_seal_deterministic() {
        let seal1 = compute_seal(b"seed", 42, &[1u8; 32], 100);
        let seal2 = compute_seal(b"seed", 42, &[1u8; 32], 100);
        assert_eq!(seal1, seal2);
    }

    #[test]
    fn compute_seal_different_nonce() {
        let seal1 = compute_seal(b"seed", 0, &[0u8; 32], 0);
        let seal2 = compute_seal(b"seed", 1, &[0u8; 32], 0);
        assert_ne!(seal1, seal2);
    }

    #[test]
    fn seal_meets_difficulty_zero_difficulty() {
        let seal = [0u8; 32];
        assert!(seal_meets_difficulty(&seal, 0));
    }

    #[test]
    fn seal_meets_difficulty_high_seal_fails() {
        let seal = [0xFFu8; 32];
        let difficulty = 1_000_000u64;
        assert!(!seal_meets_difficulty(&seal, difficulty));
    }

    #[test]
    fn seal_meets_difficulty_low_seal_passes() {
        let seal = [0u8; 32];
        let difficulty = 1_000_000u64;
        assert!(seal_meets_difficulty(&seal, difficulty));
    }

    #[test]
    fn solve_pow_medium_difficulty() {
        let nonce_seed = b"medium-test";
        let block_hash = [0x55; 32];
        let difficulty = 1_000_000u64;

        let solution = solve_pow(nonce_seed, difficulty, block_hash, 500).expect("should solve");

        assert!(seal_meets_difficulty(&solution.seal, difficulty));
    }

    #[test]
    fn pow_solution_nonce_starts_at_zero() {
        let nonce_seed = b"zero-nonce";
        let block_hash = [0x11; 32];

        let solution = solve_pow(nonce_seed, 1, block_hash, 0).expect("solve");
        let recomputed = compute_seal(nonce_seed, solution.nonce, &block_hash, 0);
        assert_eq!(solution.seal.to_vec(), recomputed[..32]);
    }

    #[test]
    fn u256_div_u64_correctness() {
        let max_u256: [u8; 32] = [0xFF; 32];
        let result = u256_div_u64(max_u256, 256);
        assert_eq!(result[0], 0);
        assert_eq!(result[1], 0xFF);
        assert_eq!(result[31], 0xFF);
    }

    #[test]
    fn threshold_decreases_with_difficulty() {
        let t1 = compute_pow_threshold(1);
        let t2 = compute_pow_threshold(2);
        let t1000 = compute_pow_threshold(1000);
        assert!(seal_below_threshold(&t2, &t1));
        assert!(seal_below_threshold(&t1000, &t2));
    }

    #[test]
    fn seal_below_threshold_equal_returns_false() {
        let a: [u8; 32] = [0x42; 32];
        assert!(!seal_below_threshold(&a, &a));
    }

    #[test]
    fn seal_below_threshold_less() {
        let mut a = [0u8; 32];
        let mut b = [0u8; 32];
        a[31] = 5;
        b[31] = 10;
        assert!(seal_below_threshold(&a, &b));
    }
}
