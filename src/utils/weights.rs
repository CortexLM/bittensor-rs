use anyhow::Result;

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
    let scale = u16::MAX as f32;
    let weight_vals: Vec<u16> = normalized
        .iter()
        .map(|w| {
            let val = (w * scale) as u16;
            val.min(u16::MAX)
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
    let scale = u16::MAX as f64;
    weight_vals
        .iter()
        .map(|val| (*val as f64 / scale) as f32)
        .collect()
}
