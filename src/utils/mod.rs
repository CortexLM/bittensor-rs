//! Utility functions for the Bittensor SDK

pub mod balance;
pub mod networking;
pub mod ss58;

pub use balance::Balance;
pub use ss58::{is_valid_ss58_address, ss58_decode, ss58_encode};

/// Normalize u16 value to f64 (0.0 - 1.0)
pub fn u16_normalized_float(value: u16) -> f64 {
    value as f64 / u16::MAX as f64
}

/// Normalize u64 value to f64 (0.0 - 1.0)
pub fn u64_normalized_float(value: u64) -> f64 {
    value as f64 / u64::MAX as f64
}

/// Convert f64 (0.0 - 1.0) to u16
pub fn float_to_u16(value: f64) -> u16 {
    (value.clamp(0.0, 1.0) * u16::MAX as f64) as u16
}

/// Convert f64 (0.0 - 1.0) to u64
pub fn float_to_u64(value: f64) -> u64 {
    (value.clamp(0.0, 1.0) * u64::MAX as f64) as u64
}

/// Normalize weights to sum to 1.0
pub fn normalize_weights(weights: &[f64]) -> Vec<f64> {
    let sum: f64 = weights.iter().sum();
    if sum == 0.0 {
        return weights.to_vec();
    }
    weights.iter().map(|w| w / sum).collect()
}

/// Convert normalized f64 weights to u16 weights
pub fn weights_to_u16(weights: &[f64]) -> Vec<u16> {
    let normalized = normalize_weights(weights);
    normalized.iter().map(|w| float_to_u16(*w)).collect()
}

/// Convert u16 weights to normalized f64 weights
pub fn weights_from_u16(weights: &[u16]) -> Vec<f64> {
    weights.iter().map(|w| u16_normalized_float(*w)).collect()
}
