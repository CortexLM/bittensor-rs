//! Weight normalization and conversion utilities for bittensor.
//!
//! These functions mirror the Python bittensor weight normalization logic,
//! converting float weights into u16/u64 chain representations.

use crate::error::BittensorError;

const U16_MAX: f32 = u16::MAX as f32;
const U64_MAX_F: f64 = u64::MAX as f64;

/// Normalize float weights so that the max weight equals `u16::MAX` (65535).
///
/// Rules (matching Python exactly):
/// - If all weights are zero, set all to `u16::MAX` (uniform distribution).
/// - If max weight is zero, set all to `u16::MAX`.
/// - Clamp negative weights to 0.
/// - Normalize: each weight = (weight / max_weight) * u16::MAX.
/// - Round and clamp to `[0, u16::MAX]`.
pub fn normalize_weights_max_u16(weights: &[f32]) -> Vec<u16> {
    let clamped: Vec<f32> = weights.iter().map(|w| w.max(0.0)).collect();

    let max_weight = clamped.iter().copied().fold(0.0f32, f32::max);

    if max_weight == 0.0 {
        return vec![u16::MAX; weights.len()];
    }

    clamped
        .iter()
        .map(|&w| {
            let normalized = (w as f64 / max_weight as f64) * U16_MAX as f64;
            normalized.round().clamp(0.0, U16_MAX as f64) as u16
        })
        .collect()
}

/// Normalize float weights so that the max weight equals `u64::MAX`.
///
/// Same rules as [`normalize_weights_max_u16`] but targeting u64.
pub fn normalize_weights_max_u64(weights: &[f32]) -> Vec<u64> {
    let clamped: Vec<f32> = weights.iter().map(|w| w.max(0.0)).collect();

    let max_weight = clamped.iter().copied().fold(0.0f32, f32::max);

    if max_weight == 0.0 {
        return vec![u64::MAX; weights.len()];
    }

    clamped
        .iter()
        .map(|&w| {
            let normalized = (w as f64 / max_weight as f64) * U64_MAX_F;
            normalized.round().clamp(0.0, U64_MAX_F) as u64
        })
        .collect()
}

/// Validate and convert weight destinations and values to chain format.
///
/// Validates that:
/// - `dests` and `weights` have the same length
/// - No duplicate destinations
/// - All destinations are within u16 range
/// - No negative weights (already u16, so always non-negative)
///
/// Returns `(dests, weights, netuid, version_key)` on success.
pub fn convert_weights_to_chain(
    dests: Vec<u16>,
    weights: Vec<u16>,
    netuid: u16,
    version_key: u64,
) -> Result<(Vec<u16>, Vec<u16>, u16, u64), BittensorError> {
    if dests.len() != weights.len() {
        return Err(BittensorError::Validation(format!(
            "dests and weights must have the same length, got {} and {}",
            dests.len(),
            weights.len()
        )));
    }

    let mut seen = std::collections::HashSet::new();
    for &dest in &dests {
        if !seen.insert(dest) {
            return Err(BittensorError::Validation(format!("duplicate destination uid: {dest}")));
        }
    }

    let mut filtered_dests = Vec::with_capacity(dests.len());
    let mut filtered_weights = Vec::with_capacity(weights.len());
    for (dest, &weight) in dests.iter().zip(weights.iter()) {
        if weight != 0 {
            filtered_dests.push(*dest);
            filtered_weights.push(weight);
        }
    }

    Ok((filtered_dests, filtered_weights, netuid, version_key))
}

/// Full pipeline: validate, normalize, and convert float weights to chain format.
///
/// 1. Validate input (non-empty, no duplicates after zero filtering).
/// 2. Normalize weights to u16 via [`normalize_weights_max_u16`].
/// 3. Convert to chain format via [`convert_weights_to_chain`].
///
/// Returns `(dests, weights, netuid, version_key)`.
pub fn process_weights_for_settings(
    netuid: u16,
    weights: Vec<f32>,
    version_key: u64,
) -> Result<(Vec<u16>, Vec<u16>, u16, u64), BittensorError> {
    if weights.is_empty() {
        return Err(BittensorError::Validation("weights must not be empty".into()));
    }

    let dests: Vec<u16> = (0..weights.len()).map(|i| i as u16).collect();

    let normalized = normalize_weights_max_u16(&weights);

    convert_weights_to_chain(dests, normalized, netuid, version_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── normalize_weights_max_u16 ──

    #[test]
    fn normalize_u16_all_zeros() {
        let weights = vec![0.0, 0.0, 0.0];
        let result = normalize_weights_max_u16(&weights);
        assert_eq!(result, vec![u16::MAX, u16::MAX, u16::MAX]);
    }

    #[test]
    fn normalize_u16_negative_weights() {
        let weights = vec![-1.0, -5.0, 0.0];
        let result = normalize_weights_max_u16(&weights);
        // All clamped to 0 → uniform
        assert_eq!(result, vec![u16::MAX, u16::MAX, u16::MAX]);
    }

    #[test]
    fn normalize_u16_single_nonzero() {
        let weights = vec![0.0, 5.0, 0.0];
        let result = normalize_weights_max_u16(&weights);
        assert_eq!(result[0], 0);
        assert_eq!(result[1], u16::MAX);
        assert_eq!(result[2], 0);
    }

    #[test]
    fn normalize_u16_multiple_values() {
        let weights = vec![1.0, 2.0, 3.0];
        let result = normalize_weights_max_u16(&weights);
        // Max is 3.0; normalized: (1/3)*65535, (2/3)*65535, 65535
        assert_eq!(result[2], u16::MAX);
        assert!(result[1] > result[0]);
        // Verify proportionality: result[1] ≈ 2*result[0]
        let ratio = result[1] as f64 / result[0] as f64;
        assert!((ratio - 2.0).abs() < 0.05, "expected ratio ~2.0, got {ratio}");
    }

    #[test]
    fn normalize_u16_all_equal() {
        let weights = vec![5.0, 5.0, 5.0];
        let result = normalize_weights_max_u16(&weights);
        // All equal → all map to u16::MAX
        assert_eq!(result, vec![u16::MAX, u16::MAX, u16::MAX]);
    }

    #[test]
    fn normalize_u16_mixed_negative_and_positive() {
        let weights = vec![-10.0, 5.0, -3.0, 10.0];
        let result = normalize_weights_max_u16(&weights);
        assert_eq!(result[0], 0);
        assert_eq!(result[3], u16::MAX);
        // result[1] should be ~5/10 * 65535 = 32767.5
        assert!((result[1] as i32 - 32768).abs() <= 1, "expected ~32768, got {}", result[1]);
        assert_eq!(result[2], 0);
    }

    #[test]
    fn normalize_u16_empty() {
        let weights: Vec<f32> = vec![];
        let result = normalize_weights_max_u16(&weights);
        assert!(result.is_empty());
    }

    #[test]
    fn normalize_u16_large_values() {
        let weights = vec![1e10, 2e10];
        let result = normalize_weights_max_u16(&weights);
        assert_eq!(result[1], u16::MAX);
        // ratio should be ~0.5
        let ratio = result[0] as f64 / result[1] as f64;
        assert!((ratio - 0.5).abs() < 0.01, "expected ratio ~0.5, got {ratio}");
    }

    // ── normalize_weights_max_u64 ──

    #[test]
    fn normalize_u64_all_zeros() {
        let weights = vec![0.0, 0.0];
        let result = normalize_weights_max_u64(&weights);
        assert_eq!(result, vec![u64::MAX, u64::MAX]);
    }

    #[test]
    fn normalize_u64_single_nonzero() {
        let weights = vec![0.0, 3.0, 0.0];
        let result = normalize_weights_max_u64(&weights);
        assert_eq!(result[0], 0);
        assert_eq!(result[1], u64::MAX);
        assert_eq!(result[2], 0);
    }

    #[test]
    fn normalize_u64_proportional() {
        let weights = vec![1.0, 3.0];
        let result = normalize_weights_max_u64(&weights);
        assert_eq!(result[1], u64::MAX);
        let ratio = result[0] as f64 / result[1] as f64;
        assert!((ratio - (1.0 / 3.0)).abs() < 1e-15, "expected ratio ~0.333, got {ratio}");
    }

    #[test]
    fn normalize_u64_negative_clamped() {
        let weights = vec![-5.0, 2.0];
        let result = normalize_weights_max_u64(&weights);
        assert_eq!(result[0], 0);
        assert_eq!(result[1], u64::MAX);
    }

    // ── convert_weights_to_chain ──

    #[test]
    fn convert_chain_mismatched_lengths() {
        let result = convert_weights_to_chain(vec![1, 2], vec![100], 0, 0);
        assert!(result.is_err());
        match result {
            Err(BittensorError::Validation(msg)) => {
                assert!(msg.contains("same length"));
            }
            _ => panic!("expected Validation error"),
        }
    }

    #[test]
    fn convert_chain_duplicate_dests() {
        let result = convert_weights_to_chain(vec![1, 1], vec![100, 200], 0, 0);
        assert!(result.is_err());
        match result {
            Err(BittensorError::Validation(msg)) => {
                assert!(msg.contains("duplicate"));
            }
            _ => panic!("expected Validation error"),
        }
    }

    #[test]
    fn convert_chain_filters_zeros() {
        let (dests, weights, netuid, vk) =
            convert_weights_to_chain(vec![0, 1, 2], vec![100, 0, 300], 5, 42).expect("ok");
        assert_eq!(dests, vec![0, 2]);
        assert_eq!(weights, vec![100, 300]);
        assert_eq!(netuid, 5);
        assert_eq!(vk, 42);
    }

    #[test]
    fn convert_chain_all_zeros_filtered() {
        let (dests, weights, _netuid, _vk) =
            convert_weights_to_chain(vec![0, 1], vec![0, 0], 1, 1).expect("ok");
        assert!(dests.is_empty());
        assert!(weights.is_empty());
    }

    #[test]
    fn convert_chain_valid() {
        let (dests, weights, netuid, vk) =
            convert_weights_to_chain(vec![0, 1, 2], vec![100, 200, 300], 3, 99).expect("ok");
        assert_eq!(dests, vec![0, 1, 2]);
        assert_eq!(weights, vec![100, 200, 300]);
        assert_eq!(netuid, 3);
        assert_eq!(vk, 99);
    }

    // ── process_weights_for_settings ──

    #[test]
    fn process_empty_weights() {
        let result = process_weights_for_settings(1, vec![], 0);
        assert!(result.is_err());
    }

    #[test]
    fn process_all_zero_weights() {
        // All zeros → normalized to all u16::MAX → but then filtered (no zeros)
        let (dests, weights, netuid, vk) =
            process_weights_for_settings(1, vec![0.0, 0.0, 0.0], 5).expect("ok");
        assert_eq!(dests, vec![0, 1, 2]);
        assert_eq!(weights, vec![u16::MAX, u16::MAX, u16::MAX]);
        assert_eq!(netuid, 1);
        assert_eq!(vk, 5);
    }

    #[test]
    fn process_mixed_weights() {
        let (dests, weights, netuid, vk) =
            process_weights_for_settings(2, vec![0.0, 5.0, 0.0, 10.0], 7).expect("ok");
        // UIDs 0,2 get normalized to 0, filtered out
        assert_eq!(dests, vec![1, 3]);
        assert_eq!(weights[1], u16::MAX); // max value
        assert_eq!(netuid, 2);
        assert_eq!(vk, 7);
    }

    #[test]
    fn process_negative_weights_uniform() {
        // All negative → all clamped to 0 → normalized to u16::MAX (uniform)
        let (dests, weights, netuid, vk) =
            process_weights_for_settings(0, vec![-1.0, -2.0, -3.0], 0).expect("ok");
        assert_eq!(dests, vec![0, 1, 2]);
        assert!(weights.iter().all(|&w| w == u16::MAX));
        assert_eq!(netuid, 0);
        assert_eq!(vk, 0);
    }
}
