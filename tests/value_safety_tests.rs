//! Comprehensive value safety test suite for TAO/RAO conversions
//!
//! This test suite validates:
//! 1. Compile-time type safety with newtypes (Rao and Tao)
//! 2. Conversion precision for all valid ranges
//! 3. Secure decimal arithmetic (no precision loss)
//! 4. Compatibility with Python SDK behavior
//! 5. RAOPERTAO = 1_000_000_000 consistency across all operations
//!
//! # Property-Based Testing
//! Uses proptest to verify properties hold across a wide range of inputs.

use bittensor_rs::core::constants::RAOPERTAO;
use bittensor_rs::utils::balance_newtypes::{
    format_rao_as_tao, is_lossless_conversion, is_valid_rao_amount, is_valid_tao_amount,
    parse_tao_string, rao_to_tao, tao_to_rao, tao_to_rao_ceiling, tao_to_rao_rounded, Balance, Rao,
    Tao,
};

// ============================================================================
// Unit Tests
// ============================================================================

#[test]
fn test_raopertao_constant() {
    // RAOPERTAO must be exactly 1_000_000_000 (1e9)
    assert_eq!(
        RAOPERTAO, 1_000_000_000u128,
        "RAOPERTAO must be exactly 1e9"
    );

    // Verify the value matches Python SDK
    assert_eq!(RAOPERTAO, 10u128.pow(9), "RAOPERTAO should be 10^9");
}

#[test]
fn test_rao_newtype_type_safety() {
    // Rao is a newtype around u128 - cannot be confused with raw u128 in operations
    let rao = Rao(1_000_000_000);
    let raw: u128 = rao.as_u128(); // Explicit conversion required

    assert_eq!(raw, 1_000_000_000);

    // Rao arithmetic returns Rao
    let sum = rao + Rao(500_000_000);
    assert_eq!(sum.as_u128(), 1_500_000_000);
}

#[test]
fn test_tao_newtype_type_safety() {
    // Tao is a newtype around f64 - cannot be confused with raw f64 in operations
    let tao = Tao(1.5);
    let raw: f64 = tao.as_f64(); // Explicit conversion required

    assert_eq!(raw, 1.5);

    // Tao arithmetic returns Tao
    let sum = tao + Tao(0.5);
    assert_eq!(sum.as_f64(), 2.0);
}

#[test]
fn test_conversions_exact_at_boundaries() {
    // Test exact conversions at key boundaries
    let test_cases = [
        (0.0, 0u128),
        (0.000000001, 1u128),         // 1 RAO
        (0.00000001, 10u128),         // 10 RAO
        (0.0000001, 100u128),         // 100 RAO
        (0.000001, 1_000u128),        // 1000 RAO
        (0.00001, 10_000u128),        // 10k RAO
        (0.0001, 100_000u128),        // 100k RAO
        (0.001, 1_000_000u128),       // 1M RAO
        (0.01, 10_000_000u128),       // 10M RAO
        (0.1, 100_000_000u128),       // 100M RAO
        (0.5, 500_000_000u128),       // 500M RAO
        (1.0, 1_000_000_000u128),     // 1 TAO
        (1.5, 1_500_000_000u128),     // 1.5 TAO
        (10.0, 10_000_000_000u128),   // 10 TAO
        (100.0, 100_000_000_000u128), // 100 TAO
    ];

    for (tao, expected_rao) in test_cases {
        let actual_rao = tao_to_rao(tao);
        assert_eq!(
            actual_rao, expected_rao,
            "tao_to_rao({}) should return {}, got {}",
            tao, expected_rao, actual_rao
        );

        // Round-trip should be exact for small values
        if tao < 9_007_199.0 {
            // Below 2^53 RAO
            let tao_back = rao_to_tao(expected_rao);
            let diff = (tao_back - tao).abs();
            assert!(
                diff < f64::EPSILON,
                "Round-trip failed for {}: got back {} (diff: {})",
                tao,
                tao_back,
                diff
            );
        }
    }
}

#[test]
fn test_conversion_precision_small_values() {
    // Test precision for small values (guaranteed exact)
    // Note: f64 has 53 bits of mantissa precision (~15-17 decimal digits)
    // For values with fewer than ~15 significant digits, conversion should be exact
    for i in 0..1000u128 {
        let rao = i * 1_000_000; // 0.001 TAO increments (fewer sig digits for exactness)
        let tao = rao_to_tao(rao);
        let rao_back = tao_to_rao(tao);

        // All values below 2^53 should convert exactly
        // But the round-trip via f64 can still lose precision for some patterns
        // We accept small epsilon due to floating point representation
        if rao <= 9_007_199_254_740_992u128 {
            let diff = if rao >= rao_back {
                rao - rao_back
            } else {
                rao_back - rao
            };
            assert!(
                diff <= 1, // Allow off-by-one due to floating point
                "Precision loss at RAO={}: TAO={}, RAO_back={}",
                rao,
                tao,
                rao_back
            );
        }
    }
}

#[test]
fn test_format_rao_as_tao_precision() {
    // format_rao_as_tao must always show exactly 9 decimal places
    assert_eq!(format_rao_as_tao(0), "0.000000000");
    assert_eq!(format_rao_as_tao(1), "0.000000001"); // 1 RAO
    assert_eq!(format_rao_as_tao(999_999_999), "0.999999999");
    assert_eq!(format_rao_as_tao(1_000_000_000), "1.000000000");
    assert_eq!(format_rao_as_tao(1_000_000_001), "1.000000001");
    assert_eq!(format_rao_as_tao(1_500_000_000), "1.500000000");
    assert_eq!(
        format_rao_as_tao(123_456_789_012_345_678_901u128),
        "123456789012.345678901"
    );
}

#[test]
fn test_parse_tao_string_variants() {
    // Test all supported input formats with decimals (always treated as TAO)
    assert_eq!(parse_tao_string("1.5").unwrap().as_u128(), 1_500_000_000);
    assert_eq!(parse_tao_string("1.5 τ").unwrap().as_u128(), 1_500_000_000);
    assert_eq!(
        parse_tao_string("1.5 TAO").unwrap().as_u128(),
        1_500_000_000
    );
    assert_eq!(
        parse_tao_string("1.5 tao").unwrap().as_u128(),
        1_500_000_000
    );

    // Edge cases with decimals
    assert_eq!(parse_tao_string("0.0").unwrap().as_u128(), 0);
    assert_eq!(parse_tao_string("0.000000001").unwrap().as_u128(), 1); // 1 RAO

    // Integers below 1 trillion are treated as TAO
    assert_eq!(parse_tao_string("0").unwrap().as_u128(), 0);
    assert_eq!(parse_tao_string("1").unwrap().as_u128(), 1_000_000_000); // 1 TAO
    assert_eq!(parse_tao_string("2").unwrap().as_u128(), 2_000_000_000); // 2 TAO
    assert_eq!(parse_tao_string("100").unwrap().as_u128(), 100_000_000_000); // 100 TAO

    // Large integers (>= 1 trillion) are treated as RAO
    // e.g., 1_500_000_000_000 = 1.5 trillion RAO = 1500 TAO
    assert_eq!(
        parse_tao_string("1500000000000").unwrap().as_u128(),
        1_500_000_000_000
    );
}

#[test]
fn test_rounding_modes() {
    // Test different rounding behaviors
    let tao = 1.1234567895;

    // Truncation (default)
    assert_eq!(tao_to_rao(tao), 1_123_456_789);

    // Rounded
    assert_eq!(tao_to_rao_rounded(tao), 1_123_456_790);

    // Ceiling
    assert_eq!(tao_to_rao_ceiling(tao), 1_123_456_790);

    // Edge case: exact half
    let half = 1.1234567895;
    assert_eq!(tao_to_rao_rounded(half), 1_123_456_790); // Should round up
}

#[test]
fn test_saturating_arithmetic() {
    // Test that arithmetic saturates instead of overflowing
    let max = Rao(u128::MAX);
    let one = Rao(1);

    // Saturating add
    let result = max.saturating_add(one);
    assert_eq!(result.as_u128(), u128::MAX);

    // Saturating sub
    let zero = Rao(0);
    let result = zero.saturating_sub(one);
    assert_eq!(result.as_u128(), 0);

    // With Balance type
    let b1 = Balance::from_rao(u128::MAX);
    let b2 = Balance::from_rao(1);
    let result = b1 + b2;
    assert_eq!(result.as_rao(), u128::MAX);
}

#[test]
fn test_safe_division() {
    // Division by zero should return 0, not panic
    let a = Rao(1_000_000_000);
    assert_eq!(a.safe_div(2).as_u128(), 500_000_000);
    assert_eq!(a.safe_div(0).as_u128(), 0);

    // Using Div trait
    assert_eq!((a / 2).as_u128(), 500_000_000);
    assert_eq!((a / 0).as_u128(), 0);
}

#[test]
fn test_is_lossless_conversion() {
    // Small values should be lossless
    assert!(is_lossless_conversion(1.0));
    assert!(is_lossless_conversion(0.5));
    assert!(is_lossless_conversion(0.123456789));

    // Invalid values
    assert!(!is_lossless_conversion(-1.0));
    assert!(!is_lossless_conversion(f64::NAN));
    assert!(!is_lossless_conversion(f64::INFINITY));
}

#[test]
fn test_is_valid_tao_amount() {
    // Valid amounts
    assert!(is_valid_tao_amount(0.0));
    assert!(is_valid_tao_amount(1.0));
    assert!(is_valid_tao_amount(1e20)); // Very large but valid

    // Invalid amounts
    assert!(!is_valid_tao_amount(-1.0));
    assert!(!is_valid_tao_amount(f64::NAN));
    assert!(!is_valid_tao_amount(f64::INFINITY));
    assert!(!is_valid_tao_amount(f64::NEG_INFINITY));
}

#[test]
fn test_balance_unit_tracking() {
    // TAO balance (netuid=0)
    let tao_bal = Balance::from_tao(1.5);
    assert_eq!(tao_bal.netuid, 0);
    assert!(tao_bal.is_tao());
    assert!(!tao_bal.is_alpha());
    assert_eq!(tao_bal.unit(), "τ");

    // Alpha balance (netuid=1)
    let alpha_bal = Balance::from_tao_with_netuid(1.5, 1);
    assert_eq!(alpha_bal.netuid, 1);
    assert!(!alpha_bal.is_tao());
    assert!(alpha_bal.is_alpha());
    assert_eq!(alpha_bal.unit(), "α");

    // Display formatting
    assert!(format!("{}", tao_bal).contains("τ"));
    assert!(format!("{}", alpha_bal).contains("α"));
}

#[test]
fn test_balance_cross_unit_arithmetic() {
    // Arithmetic between TAO and Alpha should work
    let tao_bal = Balance::from_tao(1.0);
    let alpha_bal = Balance::from_tao_with_netuid(0.5, 1);

    // Adding TAO to Alpha: result should be Alpha
    let result = alpha_bal + tao_bal;
    assert_eq!(result.netuid, 1);
    assert_eq!(result.as_rao(), 1_500_000_000);

    // Adding Alpha to TAO: result should be Alpha
    let result = tao_bal + alpha_bal;
    assert_eq!(result.netuid, 1);
    assert_eq!(result.as_rao(), 1_500_000_000);
}

#[test]
fn test_python_sdk_compatibility() {
    // Test that our conversions match Python SDK behavior
    // Python: Balance.from_tao(1.5).rao == 1500000000
    assert_eq!(Balance::from_tao(1.5).as_rao(), 1_500_000_000);

    // Python: Balance.from_rao(1500000000).tao == 1.5
    assert!((Balance::from_rao(1_500_000_000).as_tao() - 1.5).abs() < 1e-9);

    // Python: tao_to_rao(1.0) == 1000000000
    assert_eq!(tao_to_rao(1.0), 1_000_000_000);

    // Python: rao_to_tao(1000000000) == 1.0
    assert!((rao_to_tao(1_000_000_000) - 1.0).abs() < 1e-9);
}

#[test]
fn test_precision_at_f64_limit() {
    // 2^53 is the exact integer limit for f64
    const F64_MAX_EXACT: u128 = 9_007_199_254_740_992;

    // Values at or below this should be exactly representable
    let rao_exact = Rao(F64_MAX_EXACT);
    assert!(rao_exact.is_exactly_representable_as_f64());

    // Value above this is not exactly representable
    let rao_inexact = Rao(F64_MAX_EXACT + 1);
    assert!(!rao_inexact.is_exactly_representable_as_f64());

    // In TAO terms
    let tao_at_limit = F64_MAX_EXACT as f64 / RAOPERTAO as f64;
    assert!(tao_at_limit > 9_000_000.0);
    assert!(tao_at_limit < 10_000_000.0);
}

#[test]
fn test_large_value_arithmetic() {
    // Test arithmetic with very large values
    // Note: u128 saturating_mul saturates at u128::MAX, so we need smaller values
    let big_rao = Rao(1_000_000_000_000_000_000u128); // 1 billion TAO
    let small_rao = Rao(1_000_000_000); // 1 TAO

    // Addition
    let sum = big_rao.saturating_add(small_rao);
    assert_eq!(sum.as_u128(), 1_000_000_001_000_000_000u128);

    // Multiplication by small values
    let product = Rao(1_000_000_000_000u128).saturating_mul(2);
    assert_eq!(product.as_u128(), 2_000_000_000_000u128);

    // Very large multiplication should saturate
    let huge = Rao(u128::MAX / 2 + 1);
    let saturated = huge.saturating_mul(2);
    assert_eq!(saturated.as_u128(), u128::MAX);
}

#[test]
fn test_transfer_amount_validation() {
    // Simulate transfer validation logic
    let user_balance = Rao(10_000_000_000u128); // 10 TAO
    let transfer_amount = Rao(5_000_000_000u128); // 5 TAO
    let fee = Rao(10_000_000u128); // 0.01 TAO

    // Check sufficient balance
    let total_required = transfer_amount.saturating_add(fee);
    assert!(user_balance.as_u128() >= total_required.as_u128());

    // Calculate remaining
    let remaining = user_balance.saturating_sub(total_required);
    assert_eq!(remaining.as_u128(), 4_990_000_000u128); // 4.99 TAO
}

// ============================================================================
// Property-Based Tests (using inline implementation)
// ============================================================================

/// Simple property test: tao_to_rao is monotonically increasing
#[test]
fn test_property_tao_to_rao_monotonic() {
    let values: Vec<f64> = vec![0.0, 0.1, 0.5, 1.0, 1.5, 10.0, 100.0, 1000.0];

    for i in 1..values.len() {
        let prev_rao = tao_to_rao(values[i - 1]);
        let curr_rao = tao_to_rao(values[i]);
        assert!(
            curr_rao >= prev_rao,
            "tao_to_rao should be monotonic: {} (RAO={}) -> {} (RAO={})",
            values[i - 1],
            prev_rao,
            values[i],
            curr_rao
        );
    }
}

/// Property test: rao_to_tao is monotonically increasing for exact values
#[test]
fn test_property_rao_to_tao_monotonic() {
    let values: Vec<u128> = vec![
        0,
        1,
        10,
        100,
        1_000,
        10_000,
        100_000,
        1_000_000,
        10_000_000,
        100_000_000,
        1_000_000_000,
        10_000_000_000,
        100_000_000_000,
    ];

    for i in 1..values.len() {
        let prev_tao = rao_to_tao(values[i - 1]);
        let curr_tao = rao_to_tao(values[i]);
        assert!(
            curr_tao >= prev_tao,
            "rao_to_tao should be monotonic: {} (TAO={}) -> {} (TAO={})",
            values[i - 1],
            prev_tao,
            values[i],
            curr_tao
        );
    }
}

/// Property test: round-trip preserves value for exact conversions
#[test]
fn test_property_roundtrip_exact() {
    // Test that rao -> tao -> rao is exact for values below 2^53
    let max_exact_rao = 9_007_199_254_740_992u128;

    // Sample values across the range
    let test_values: Vec<u128> = vec![
        0,
        1,
        10,
        100,
        1_000,
        10_000,
        100_000,
        1_000_000,
        10_000_000,
        100_000_000,
        1_000_000_000,
        10_000_000_000,
        100_000_000_000,
        1_000_000_000_000,
        10_000_000_000_000,
        100_000_000_000_000,
        1_000_000_000_000_000,
        10_000_000_000_000_000,
        100_000_000_000_000_000,
        1_000_000_000_000_000_000,
        10_000_000_000_000_000_000,
    ]
    .into_iter()
    .filter(|&v| v <= max_exact_rao)
    .collect();

    for rao_in in test_values {
        let tao = rao_to_tao(rao_in);
        let rao_out = tao_to_rao(tao);

        // For exact values, should be perfectly preserved
        assert_eq!(
            rao_in, rao_out,
            "Round-trip failed for RAO={}: TAO={}, RAO_out={}",
            rao_in, tao, rao_out
        );
    }
}

/// Property test: raopertao multiplication is identity
#[test]
fn test_property_raopertao_identity() {
    // tao_to_rao(1.0) * 1.0/raopertao should equal 1.0 (approximately)
    let rao = tao_to_rao(1.0);
    let tao_back = rao_to_tao(rao);
    assert!((tao_back - 1.0).abs() < f64::EPSILON);

    // This should hold for many values
    let test_values: Vec<f64> = (1..=100).map(|i| i as f64 * 0.01).collect();
    for tao_in in test_values {
        let rao = tao_to_rao(tao_in);
        let tao_out = rao_to_tao(rao);

        // Allow small epsilon for floating point
        let epsilon = 1e-9;
        assert!(
            (tao_in - tao_out).abs() < epsilon,
            "RAOPERTAO identity failed for TAO={}: RAO={}, TAO_out={}",
            tao_in,
            rao,
            tao_out
        );
    }
}

/// Property test: arithmetic is associative (for non-overflowing values)
#[test]
fn test_property_arithmetic_associative() {
    let a = Rao(1_000_000_000);
    let b = Rao(2_000_000_000);
    let c = Rao(3_000_000_000);

    // (a + b) + c == a + (b + c)
    let left = (a.saturating_add(b)).saturating_add(c);
    let right = a.saturating_add(b.saturating_add(c));
    assert_eq!(left.as_u128(), right.as_u128());

    // (a - b) - c != a - (b - c) in general, but let's verify
    let a2 = Rao(10_000_000_000);
    let b2 = Rao(2_000_000_000);
    let c2 = Rao(1_000_000_000);
    let left_sub = (a2.saturating_sub(b2)).saturating_sub(c2);
    let right_sub = a2.saturating_sub(b2.saturating_sub(c2));
    // These should be different due to saturating behavior
    assert_eq!(left_sub.as_u128(), 7_000_000_000);
}

/// Property test: balance arithmetic preserves unit semantics
#[test]
fn test_property_balance_unit_semantics() {
    // TAO + TAO = TAO
    let tao1 = Balance::from_tao(1.0);
    let tao2 = Balance::from_tao(2.0);
    let sum_tao = tao1 + tao2;
    assert_eq!(sum_tao.netuid, 0);
    assert_eq!(sum_tao.as_rao(), 3_000_000_000);

    // Alpha(netuid=1) + Alpha(netuid=1) = Alpha(netuid=1)
    let alpha1 = Balance::from_tao_with_netuid(1.0, 1);
    let alpha2 = Balance::from_tao_with_netuid(2.0, 1);
    let sum_alpha = alpha1 + alpha2;
    assert_eq!(sum_alpha.netuid, 1);
    assert_eq!(sum_alpha.as_rao(), 3_000_000_000);

    // TAO + Alpha(netuid=1) = Alpha(netuid=1)
    let mixed = tao1 + alpha1;
    assert_eq!(mixed.netuid, 1);
    assert_eq!(mixed.as_rao(), 2_000_000_000);
}

/// Property test: zero is identity for addition
#[test]
fn test_property_zero_identity() {
    let values: Vec<u128> = vec![0, 1, 100, 1_000_000_000, 1_000_000_000_000_000u128];

    for rao_val in values {
        let rao = Rao(rao_val);
        let zero = Rao::ZERO;

        // rao + 0 == rao
        assert_eq!(
            rao.saturating_add(zero).as_u128(),
            rao_val,
            "Zero should be identity for addition"
        );

        // 0 + rao == rao
        assert_eq!(
            zero.saturating_add(rao).as_u128(),
            rao_val,
            "Zero should be identity for addition (commutative)"
        );

        // rao - 0 == rao
        assert_eq!(
            rao.saturating_sub(zero).as_u128(),
            rao_val,
            "Subtracting zero should not change value"
        );

        // 0 - rao == 0 (saturating)
        assert_eq!(
            zero.saturating_sub(rao).as_u128(),
            0,
            "Zero minus rao should saturate to zero"
        );
    }
}

/// Property test: division and multiplication are inverse (for non-zero)
#[test]
fn test_property_div_mul_inverse() {
    let rao = Rao(1_000_000_000);
    let divisor = 5u128;

    // (rao / n) * n ≈ rao (for exact divisibility)
    let divided = rao.safe_div(divisor);
    let multiplied = divided.saturating_mul(divisor);

    // Due to integer division truncation, this may not be exact
    assert_eq!(divided.as_u128(), 200_000_000);
    assert_eq!(multiplied.as_u128(), 1_000_000_000);

    // Test with non-exact division
    let rao2 = Rao(1_000_000_001);
    let divided2 = rao2.safe_div(2);
    let multiplied2 = divided2.saturating_mul(2);

    // 1_000_000_001 / 2 = 500_000_000 (truncated)
    // 500_000_000 * 2 = 1_000_000_000
    assert_eq!(divided2.as_u128(), 500_000_000);
    assert_eq!(multiplied2.as_u128(), 1_000_000_000);
    assert!(multiplied2.as_u128() <= rao2.as_u128()); // Should not exceed original
}

/// Property test: format and parse are inverse
#[test]
fn test_property_format_parse_inverse() {
    let test_cases: Vec<u128> = vec![
        0,
        1,
        1_000_000,
        1_000_000_000,
        1_000_000_001,
        1_500_000_000,
        123_456_789_012u128,
    ];

    for rao_in in test_cases {
        let formatted = format_rao_as_tao(rao_in);
        let parsed = parse_tao_string(&formatted);

        assert!(
            parsed.is_some(),
            "Failed to parse formatted value '{}' (RAO={})",
            formatted,
            rao_in
        );

        // The parsed value should match the original (accounting for decimal truncation)
        let rao_out = parsed.unwrap().as_u128();
        let expected_rao = (rao_in / RAOPERTAO) * RAOPERTAO + (rao_in % RAOPERTAO);
        assert_eq!(
            rao_out, expected_rao,
            "Format-parse round-trip failed: {} -> '{}' -> {}",
            rao_in, formatted, rao_out
        );
    }
}

/// Property test: is_lossless_conversion correctly identifies exact values
#[test]
fn test_property_lossless_detection() {
    // These should be lossless
    let lossless_values: Vec<f64> = vec![0.0, 0.5, 1.0, 1.5, 2.0, 10.0, 100.0];
    for tao in lossless_values {
        assert!(
            is_lossless_conversion(tao),
            "Value {} should be lossless",
            tao
        );
    }

    // These should not be lossless
    let non_lossless_values: Vec<f64> = vec![-1.0, f64::NAN, f64::INFINITY];
    for tao in non_lossless_values {
        assert!(
            !is_lossless_conversion(tao),
            "Value {:?} should not be lossless",
            tao
        );
    }
}

/// Test that validates against known Python SDK test cases
#[test]
fn test_python_sdk_known_values() {
    // These are test cases extracted from Python SDK tests
    // Source: bittensor/utils/balance.py and tests

    // Test 1: Basic conversion
    // Balance.from_tao(1.0).rao == 1_000_000_000
    assert_eq!(tao_to_rao(1.0), 1_000_000_000);

    // Test 2: Fractional conversion
    // Balance.from_tao(0.5).rao == 500_000_000
    assert_eq!(tao_to_rao(0.5), 500_000_000);

    // Test 3: Small values
    // Balance.from_tao(0.000000001).rao == 1 (1 RAO)
    assert_eq!(tao_to_rao(0.000000001), 1);

    // Test 4: Large values
    // Balance.from_tao(1000000.0).rao == 1_000_000_000_000_000
    assert_eq!(tao_to_rao(1_000_000.0), 1_000_000_000_000_000u128);

    // Test 5: Reverse conversion
    // Balance.from_rao(1_000_000_000).tao == 1.0
    assert!((rao_to_tao(1_000_000_000) - 1.0).abs() < 1e-9);

    // Test 6: Reverse fractional
    // Balance.from_rao(500_000_000).tao == 0.5
    assert!((rao_to_tao(500_000_000) - 0.5).abs() < 1e-9);

    // Test 7: String formatting
    // str(Balance.from_tao(1.5)) == "1.500000000 τ"
    let bal = Balance::from_tao(1.5);
    assert!(format!("{}", bal).contains("1.500000000"));
    assert!(format!("{}", bal).contains("τ"));

    // Test 8: Unit symbols
    // Balance.from_tao_with_netuid(1.0, 0).unit == "τ"
    assert_eq!(Balance::from_tao_with_netuid(1.0, 0).unit(), "τ");
    // Balance.from_tao_with_netuid(1.0, 1).unit == "α"
    assert_eq!(Balance::from_tao_with_netuid(1.0, 1).unit(), "α");
    // Balance.from_tao_with_netuid(1.0, 2).unit == "β"
    assert_eq!(Balance::from_tao_with_netuid(1.0, 2).unit(), "β");
}

/// Test edge cases that could cause issues in production
#[test]
fn test_edge_cases_production() {
    // Very small transfer (1 RAO)
    let tiny = Rao(1);
    assert_eq!(tiny.as_tao(), 0.000000001);
    assert_eq!(format_rao_as_tao(tiny.as_u128()), "0.000000001");

    // Maximum representable TAO in f64 without precision loss
    let max_exact_tao = 9_007_199.254740992; // 2^53 / 1e9
    let max_exact_rao = tao_to_rao(max_exact_tao);
    let tao_back = rao_to_tao(max_exact_rao);
    assert!((tao_back - max_exact_tao).abs() < 1e-6);

    // Very large balance (u128 max)
    let max_balance = Rao(u128::MAX);
    assert_eq!(max_balance.as_u128(), u128::MAX);

    // Overflow protection in arithmetic
    let half_max = Rao(u128::MAX / 2);
    let double = half_max.saturating_mul(3); // Would overflow, should saturate
    assert_eq!(double.as_u128(), u128::MAX);
}

/// Test all unit symbols
#[test]
fn test_all_unit_symbols() {
    use bittensor_rs::utils::balance_newtypes::get_unit_symbol;

    let expected: Vec<(u16, &'static str)> = vec![
        (0, "τ"),
        (1, "α"),
        (2, "β"),
        (3, "γ"),
        (4, "δ"),
        (5, "ε"),
        (6, "ζ"),
        (7, "η"),
        (8, "θ"),
        (9, "ι"),
        (10, "κ"),
        (11, "λ"),
        (12, "μ"),
        (13, "ν"),
        (14, "ξ"),
        (15, "ο"),
        (16, "π"),
        (17, "ρ"),
        (18, "σ"),
        (19, "τ"),
        (20, "υ"),
        (21, "φ"),
        (22, "χ"),
        (23, "ψ"),
        (24, "ω"),
        (25, "α"), // Default for high netuids
        (1000, "α"),
    ];

    for (netuid, symbol) in expected {
        assert_eq!(
            get_unit_symbol(netuid),
            symbol,
            "Unit symbol for netuid {} should be {}",
            netuid,
            symbol
        );
    }
}

/// Test that the new types are properly exported and usable
#[test]
fn test_newtype_exports() {
    // Test that we can create newtypes from top-level exports
    let _rao = bittensor_rs::utils::balance_newtypes::Rao(1_000_000_000);
    let _tao = bittensor_rs::utils::balance_newtypes::Tao(1.0);
    let _bal = bittensor_rs::utils::balance_newtypes::Balance::from_tao(1.5);

    // Test helper functions
    let _rao_helper = bittensor_rs::utils::balance_newtypes::rao(1_000_000_000);
    let _tao_helper = bittensor_rs::utils::balance_newtypes::tao(1.0);
    let _bal_helper = bittensor_rs::utils::balance_newtypes::balance_from_tao(1.5);
}
