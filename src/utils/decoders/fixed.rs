use crate::utils::primitive;
use anyhow::{anyhow, Result};
use subxt::dynamic::Value;

/// Decode a fixed-point number (U64F64) from a Value
pub fn decode_fixed_u64f64(value: &Value) -> Result<f64> {
    let s = format!("{:?}", value);

    // Look for pattern Composite { bits: U128(n) }
    // Try to extract bits field
    if let Some(pos) = s.find("bits") {
        let after = &s[pos..];
        if let Some(u128_pos) = after.find("U128(") {
            let start = u128_pos + 5;
            if let Some(end) = after[start..].find(')') {
                let num_str = &after[start..start + end];
                if let Ok(bits) = num_str.trim().parse::<u128>() {
                    return Ok(fixed_u128_to_f64(bits, 64));
                }
            }
        }
    }

    // Fallback: try direct u128
    if let Ok(bits) = primitive::decode_u128(value) {
        return Ok(fixed_u128_to_f64(bits, 64));
    }

    Err(anyhow!("Cannot decode fixed-point number from value"))
}

/// Convert a fixed-point u128 to f64
pub fn fixed_u128_to_f64(bits: u128, frac_bits: u32) -> f64 {
    let fractional_mask: u128 = (1u128 << frac_bits) - 1u128;
    let fractional_part: u128 = bits & fractional_mask;
    let integer_part: u128 = bits >> frac_bits;
    let frac_float = (fractional_part as f64) / ((1u128 << frac_bits) as f64);
    (integer_part as f64) + frac_float
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_point_conversion() {
        // Test some known values
        let one = 1u128 << 64; // 1.0 in U64F64
        assert_eq!(fixed_u128_to_f64(one, 64), 1.0);

        let half = 1u128 << 63; // 0.5 in U64F64
        assert_eq!(fixed_u128_to_f64(half, 64), 0.5);
    }
}
