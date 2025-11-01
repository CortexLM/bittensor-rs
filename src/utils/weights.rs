use anyhow::Result;

/// Normalize weights to ensure they sum to 1.0 and convert to fixed point representation
pub fn normalize_weights(uids: &[u64], weights: &[f32]) -> Result<(Vec<u64>, Vec<u64>)> {
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

    // Convert to fixed point (u64) - using u32::MAX as the scale factor
    let scale = u32::MAX as u64;
    let weight_vals: Vec<u64> = normalized
        .iter()
        .map(|w| {
            let val = (w * scale as f32) as u64;
            val.min(scale)
        })
        .collect();

    // Filter out zero weights
    let mut filtered_uids = Vec::new();
    let mut filtered_vals = Vec::new();

    for (uid, val) in uids.iter().zip(weight_vals.iter()) {
        if *val > 0 {
            filtered_uids.push(*uid);
            filtered_vals.push(*val);
        }
    }

    Ok((filtered_uids, filtered_vals))
}

/// Convert weights back from fixed point to float
pub fn denormalize_weights(weight_vals: &[u64]) -> Vec<f32> {
    let scale = u32::MAX as f64;
    weight_vals
        .iter()
        .map(|val| (*val as f64 / scale) as f32)
        .collect()
}
