//! Weight utilities for Bittensor
//! Provides weight normalization, processing, and conversion functions
//! matching Python's bittensor.utils.weight_utils

use anyhow::Result;

/// Maximum value for u16 weights
pub const U16_MAX: u16 = 65535;
/// Maximum value for u32
pub const U32_MAX: u32 = 4294967295;

/// Normalize weights to ensure they sum to 1.0 and convert to fixed point representation
/// Returns (uids as Vec<u16>, weights as Vec<u16>) matching Subtensor's expected format
/// Weights use u16::MAX (65535) as the scale factor to represent 1.0
pub fn normalize_weights(uids: &[u64], weights: &[f32]) -> Result<(Vec<u16>, Vec<u16>)> {
    if uids.len() != weights.len() {
        return Err(anyhow::anyhow!(
            "UIDS and weights must have the same length"
        ));
    }

    if weights.is_empty() {
        return Ok((vec![], vec![]));
    }

    // Calculate sum
    let sum: f32 = weights.iter().sum();

    // Normalize weights
    let normalized: Vec<f32> = if sum.abs() > f32::EPSILON {
        weights.iter().map(|w| w / sum).collect()
    } else {
        // If sum is zero, distribute evenly
        let count = weights.len() as f32;
        weights.iter().map(|_| 1.0 / count).collect()
    };

    // Convert to fixed point (u16) - using u16::MAX (65535) as the scale factor
    // This matches Subtensor's expected format where u16::MAX represents 1.0
    let scale = U16_MAX as f32;
    let weight_vals: Vec<u16> = normalized
        .iter()
        .map(|w| {
            let val = (w * scale) as u16;
            val
        })
        .collect();

    // Filter out zero weights and convert uids to u16
    let mut filtered_uids = Vec::new();
    let mut filtered_vals = Vec::new();

    for (uid, val) in uids.iter().zip(weight_vals.iter()) {
        if *val > 0 {
            // Convert uid from u64 to u16 (Subtensor expects Vec<u16>)
            filtered_uids.push(*uid as u16);
            filtered_vals.push(*val);
        }
    }

    Ok((filtered_uids, filtered_vals))
}

/// Convert weights back from fixed point (u16) to float
pub fn denormalize_weights(weight_vals: &[u16]) -> Vec<f32> {
    let scale = U16_MAX as f64;
    weight_vals
        .iter()
        .map(|val| (*val as f64 / scale) as f32)
        .collect()
}

/// Normalize a u16 value to float [0, 1]
pub fn u16_normalized_float(value: u16) -> f64 {
    value as f64 / U16_MAX as f64
}

/// Normalize a u64 value to float [0, 1] using U64_MAX
pub fn u64_normalized_float(value: u64) -> f64 {
    value as f64 / u64::MAX as f64
}

/// Convert float [0, 1] to u16
pub fn float_to_u16(value: f64) -> u16 {
    ((value.clamp(0.0, 1.0)) * U16_MAX as f64) as u16
}

/// Normalize max weight to ensure no single weight exceeds limit
/// Similar to Python's normalize_max_weight function
pub fn normalize_max_weight(weights: &[f32], limit: f32) -> Vec<f32> {
    if weights.is_empty() {
        return vec![];
    }

    let epsilon = 1e-7f32;
    let n = weights.len() as f32;

    // Check if we need normalization at all
    let sum: f32 = weights.iter().sum();
    if sum == 0.0 || n * limit <= 1.0 {
        return vec![1.0 / n; weights.len()];
    }

    // Sort and get estimation
    let mut sorted: Vec<f32> = weights.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let estimation: Vec<f32> = sorted.iter().map(|v| v / sum).collect();

    if estimation.iter().cloned().fold(f32::NEG_INFINITY, f32::max) <= limit {
        // No need to clip
        return weights.iter().map(|w| w / sum).collect();
    }

    // Find cumulative sum
    let mut cumsum = Vec::with_capacity(estimation.len());
    let mut acc = 0.0f32;
    for e in &estimation {
        acc += e;
        cumsum.push(acc);
    }

    // Determine cutoff index
    let mut n_values = 0;
    for i in 0..estimation.len() {
        let estimation_sum: f32 = (0..estimation.len())
            .map(|j| if j > i { estimation[j] } else { 0.0 })
            .sum();
        if estimation[i] / (estimation_sum + cumsum[i] + epsilon) < limit {
            n_values += 1;
        }
    }
    n_values = n_values.max(1);

    // Calculate cutoff
    let cutoff_scale = (limit * cumsum[n_values - 1] - epsilon)
        / (1.0 - (limit * (estimation.len() - n_values) as f32));
    let cutoff = cutoff_scale * sum;

    // Apply cutoff
    let clipped: Vec<f32> = weights
        .iter()
        .map(|w| if *w > cutoff { cutoff } else { *w })
        .collect();

    let clipped_sum: f32 = clipped.iter().sum();
    if clipped_sum > 0.0 {
        clipped.iter().map(|w| w / clipped_sum).collect()
    } else {
        vec![1.0 / n; weights.len()]
    }
}

/// Convert weight UIDs and values to dense tensor representation
pub fn convert_weight_uids_and_vals_to_tensor(n: usize, uids: &[u16], weights: &[u16]) -> Vec<f32> {
    let mut row_weights = vec![0.0f32; n];

    for (uid, weight) in uids.iter().zip(weights.iter()) {
        if (*uid as usize) < n {
            row_weights[*uid as usize] = *weight as f32;
        }
    }

    // Normalize
    let sum: f32 = row_weights.iter().sum();
    if sum > 0.0 {
        for w in &mut row_weights {
            *w /= sum;
        }
    }

    row_weights
}

/// Convert bond UIDs and values to dense tensor representation
pub fn convert_bond_uids_and_vals_to_tensor(n: usize, uids: &[u16], bonds: &[u64]) -> Vec<u64> {
    let mut row_bonds = vec![0u64; n];

    for (uid, bond) in uids.iter().zip(bonds.iter()) {
        if (*uid as usize) < n {
            row_bonds[*uid as usize] = *bond;
        }
    }

    row_bonds
}

/// Process weights for a subnet with constraints
pub fn process_weights(
    uids: &[u64],
    weights: &[f32],
    num_neurons: usize,
    min_allowed_weights: Option<u64>,
    max_weight_limit: Option<f64>,
    exclude_quantile: u64,
) -> Result<(Vec<u16>, Vec<f32>)> {
    let min_allowed = min_allowed_weights.unwrap_or(0) as usize;
    let max_limit = max_weight_limit.unwrap_or(1.0) as f32;
    let quantile = exclude_quantile as f64 / U16_MAX as f64;

    // Find non-zero weights
    let non_zero: Vec<(u64, f32)> = uids
        .iter()
        .zip(weights.iter())
        .filter(|(_, w)| **w > 0.0)
        .map(|(u, w)| (*u, *w))
        .collect();

    if non_zero.is_empty() || num_neurons < min_allowed {
        // Return uniform weights
        let uniform = vec![1.0 / num_neurons as f32; num_neurons];
        let all_uids: Vec<u16> = (0..num_neurons as u16).collect();
        return Ok((all_uids, uniform));
    }

    if non_zero.len() < min_allowed {
        // Create minimum weights
        let mut result_weights = vec![1e-5f32; num_neurons];
        for (uid, weight) in &non_zero {
            if (*uid as usize) < num_neurons {
                result_weights[*uid as usize] += weight;
            }
        }
        let normalized = normalize_max_weight(&result_weights, max_limit);
        let all_uids: Vec<u16> = (0..num_neurons as u16).collect();
        return Ok((all_uids, normalized));
    }

    // Apply quantile exclusion
    let max_exclude = (non_zero.len() - min_allowed).max(0) as f64 / non_zero.len() as f64;
    let effective_quantile = quantile.min(max_exclude);

    let mut sorted_weights: Vec<f32> = non_zero.iter().map(|(_, w)| *w).collect();
    sorted_weights.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let quantile_idx = ((effective_quantile * sorted_weights.len() as f64) as usize)
        .min(sorted_weights.len().saturating_sub(1));
    let lowest_quantile = sorted_weights.get(quantile_idx).copied().unwrap_or(0.0);

    // Filter by quantile
    let filtered: Vec<(u64, f32)> = non_zero
        .into_iter()
        .filter(|(_, w)| *w >= lowest_quantile)
        .collect();

    let filtered_uids: Vec<u16> = filtered.iter().map(|(u, _)| *u as u16).collect();
    let filtered_weights: Vec<f32> = filtered.iter().map(|(_, w)| *w).collect();

    // Normalize with max weight limit
    let normalized = normalize_max_weight(&filtered_weights, max_limit);

    Ok((filtered_uids, normalized))
}

/// Generate commit hash for weights (for commit-reveal pattern)
pub fn generate_weight_hash(
    address: &[u8; 32],
    netuid: u16,
    uids: &[u16],
    values: &[u16],
    version_key: u64,
    salt: &[u16],
) -> [u8; 32] {
    let mut data = Vec::new();

    // Write account address
    data.extend_from_slice(address);

    // Write netuid (little endian)
    data.extend_from_slice(&netuid.to_le_bytes());

    // Write uids as SCALE-encoded Vec<u16>
    // Compact length prefix
    let uids_len = uids.len();
    if uids_len < 64 {
        data.push((uids_len as u8) << 2);
    } else {
        // Simplified: handle larger lengths
        data.extend_from_slice(&(uids_len as u32).to_le_bytes());
    }
    for uid in uids {
        data.extend_from_slice(&uid.to_le_bytes());
    }

    // Write values as SCALE-encoded Vec<u16>
    let values_len = values.len();
    if values_len < 64 {
        data.push((values_len as u8) << 2);
    } else {
        data.extend_from_slice(&(values_len as u32).to_le_bytes());
    }
    for val in values {
        data.extend_from_slice(&val.to_le_bytes());
    }

    // Write salt as SCALE-encoded Vec<u16>
    let salt_len = salt.len();
    if salt_len < 64 {
        data.push((salt_len as u8) << 2);
    } else {
        data.extend_from_slice(&(salt_len as u32).to_le_bytes());
    }
    for s in salt {
        data.extend_from_slice(&s.to_le_bytes());
    }

    // Write version key (little endian)
    data.extend_from_slice(&version_key.to_le_bytes());

    // Blake2b-256 hash
    use sp_core::blake2_256;
    blake2_256(&data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_weights() {
        let uids = vec![0, 1, 2];
        let weights = vec![1.0, 2.0, 1.0];

        let (result_uids, result_weights) = normalize_weights(&uids, &weights).unwrap();

        assert_eq!(result_uids.len(), 3);
        assert_eq!(result_weights.len(), 3);

        // Sum should be approximately U16_MAX
        let sum: u32 = result_weights.iter().map(|w| *w as u32).sum();
        assert!(sum > 60000); // Should be close to 65535
    }

    #[test]
    fn test_denormalize_weights() {
        let weights = vec![U16_MAX / 2, U16_MAX / 2];
        let denorm = denormalize_weights(&weights);

        assert!((denorm[0] - 0.5).abs() < 0.01);
        assert!((denorm[1] - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_u16_normalized_float() {
        assert_eq!(u16_normalized_float(0), 0.0);
        assert!((u16_normalized_float(U16_MAX) - 1.0).abs() < 0.0001);
        assert!((u16_normalized_float(U16_MAX / 2) - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_normalize_max_weight() {
        let weights = vec![0.1, 0.2, 0.7];
        let normalized = normalize_max_weight(&weights, 0.5);

        // All weights should be <= 0.5
        for w in &normalized {
            assert!(*w <= 0.5 + 0.01);
        }

        // Should sum to 1.0
        let sum: f32 = normalized.iter().sum();
        assert!((sum - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_convert_weight_uids_and_vals_to_tensor() {
        let uids = vec![0, 2];
        let weights = vec![32767, 32768];

        let tensor = convert_weight_uids_and_vals_to_tensor(4, &uids, &weights);

        assert_eq!(tensor.len(), 4);
        assert!(tensor[0] > 0.0);
        assert_eq!(tensor[1], 0.0);
        assert!(tensor[2] > 0.0);
        assert_eq!(tensor[3], 0.0);
    }
}
