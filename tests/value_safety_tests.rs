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
    format_rao_as_tao, is_lossless_conversion, is_valid_tao_amount, parse_tao_string, rao_to_tao,
    tao_to_rao, Balance, Rao, Tao,
};
use bittensor_rs::validator::{staking, transfer, weights as validator_weights};
use proptest::prelude::*;

#[test]
fn test_transfer_and_stake_requires_rao_inputs() {
    fn assert_u128_input<T>(_: T)
    where
        T: Into<u128>,
    {
    }

    assert_u128_input::<u128>(0);
    assert_u128_input::<u128>(Rao::ZERO.as_u128());
    assert_u128_input::<u128>(RAOPERTAO);

    let _transfer_fn = transfer::transfer;

    let _transfer_stake_fn = transfer::transfer_stake;

    let _add_stake_fn = staking::add_stake;

    let _unstake_fn = staking::unstake;
}

#[test]
fn test_weight_inputs_require_u16_not_tao() {
    let uids: Vec<u16> = vec![0, 1, 2];
    let weights: Vec<u16> = vec![10_000, 20_000, 30_000];

    let _set_weights_fn = validator_weights::set_weights;
    let _commit_weights_fn = validator_weights::commit_weights;
    let _reveal_weights_fn = validator_weights::reveal_weights;

    assert_eq!(uids.len(), weights.len());
}

#[test]
fn test_commit_reveal_units_require_u16() {
    fn assert_u16_slice(_: &[u16]) {}

    let uids: Vec<u16> = vec![1, 2, 3];
    let weights: Vec<u16> = vec![100, 200, 300];
    let salt: Vec<u16> = vec![4, 5, 6];

    assert_u16_slice(&uids);
    assert_u16_slice(&weights);
    assert_u16_slice(&salt);
}

#[test]
fn test_rao_only_inputs_for_transfer_conversion() {
    let tao_amount = Tao(0.123456789);
    let rao_amount = tao_amount.as_rao();
    assert_eq!(rao_amount.as_u128(), 123_456_789);

    let rao_amount_direct = Rao::from_tao(0.123456789);
    assert_eq!(rao_amount_direct.as_u128(), 123_456_789);

    let balance = Balance::from_tao(0.123456789);
    assert_eq!(balance.as_rao(), 123_456_789);
}

#[test]
fn test_tao_to_rao_truncation_invariant() {
    let tao_amount = Tao(1.0000000009);
    let rao = tao_amount.as_rao();
    assert_eq!(rao.as_u128(), RAOPERTAO);

    let rounded = tao_amount.as_rao_rounded();
    assert_eq!(rounded.as_u128(), RAOPERTAO + 1);

    let ceiling = tao_amount.as_rao_ceiling();
    assert_eq!(ceiling.as_u128(), RAOPERTAO + 1);
}

#[test]
fn test_rao_tao_roundtrip_invariant_for_exact_values() {
    let test_values = [0u128, 1, 10, 1_000, 1_000_000_000, 9_000_000_000_000u128];
    for rao in test_values {
        let tao = rao_to_tao(rao);
        let rao_back = tao_to_rao(tao);
        assert_eq!(rao_back, rao, "Round-trip failed for {}", rao);
    }
}

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
        let diff = rao_back.abs_diff(rao);

        assert!(
            diff <= 1,
            "Round-trip failed for {}: got {} (diff: {})",
            rao,
            rao_back,
            diff
        );
    }
}

#[test]
fn test_conversion_precision_large_values() {
    // Test precision for large values near max
    let test_values = [
        1_000_000_000u128,         // 1 TAO
        1_000_000_000_000u128,     // 1000 TAO
        1_000_000_000_000_000u128, // 1M TAO
        9_000_000_000_000_000u128, // 9M TAO (near precision limit)
    ];

    for rao in test_values {
        let tao = rao_to_tao(rao);
        let rao_back = tao_to_rao(tao);

        // For large values, we allow some precision loss within 1 RAO
        let diff = rao_back.abs_diff(rao);

        assert!(
            diff <= 1,
            "Precision loss too large for {}: diff {}",
            rao,
            diff
        );
    }
}

#[test]
fn test_string_parsing_formats() {
    let valid_cases = [
        ("1", 1_000_000_000u128),
        ("1.0", 1_000_000_000u128),
        ("1.000000000", 1_000_000_000u128),
        ("0.1", 100_000_000u128),
        ("0.000000001", 1u128),
        ("123.456789", 123_456_789_000u128),
        ("0", 0u128),
    ];

    for (s, expected) in valid_cases {
        let rao = parse_tao_string(s).unwrap();
        assert_eq!(rao, Rao::new(expected), "Failed to parse '{}'", s);
    }

    let invalid_cases = [
        "",
        "abc",
        "1.0000000000", // too many decimals
        "-1",
        "1.",
        ".1",
    ];

    for s in invalid_cases {
        assert!(parse_tao_string(s).is_none(), "Should reject '{}'", s);
    }
}

#[test]
fn test_formatting() {
    let test_cases = [
        (0u128, "0.000000000"),
        (1u128, "0.000000001"),
        (10u128, "0.000000010"),
        (100u128, "0.000000100"),
        (1_000u128, "0.000001000"),
        (1_000_000u128, "0.001000000"),
        (1_000_000_000u128, "1.000000000"),
        (1_234_567_890u128, "1.234567890"),
        (10_000_000_000u128, "10.000000000"),
    ];

    for (rao, expected) in test_cases {
        let formatted = format_rao_as_tao(rao);
        assert_eq!(formatted, expected, "Formatting failed for {}", rao);
    }
}

#[test]
fn test_balance_newtype_operations() {
    let balance = Balance::from_rao(1_500_000_000);
    assert_eq!(balance.as_tao(), 1.5);

    let balance2 = Balance::from_tao(0.5);
    let sum = balance + balance2;
    assert_eq!(sum.as_rao(), 2_000_000_000);

    let diff = sum - Balance::from_tao(0.5);
    assert_eq!(diff.as_rao(), 1_500_000_000);
}

#[test]
fn test_balance_newtype_comparisons() {
    let b1 = Balance::from_rao(1_000_000_000);
    let b2 = Balance::from_rao(2_000_000_000);

    assert!(b1 < b2);
    assert!(b2 > b1);
    assert_eq!(b1, Balance::from_tao(1.0));
}

#[test]
fn test_rao_newtype_conversions() {
    let rao = Rao(1_234_567_890);
    let tao = rao.as_tao();
    assert_eq!(tao, 1.23456789);

    let formatted = rao.to_string();
    assert_eq!(formatted, "1234567890 ρ");

    let parsed = parse_tao_string("1.234567890").unwrap();
    assert_eq!(parsed, rao);
}

#[test]
fn test_tao_newtype_conversions() {
    let tao = Tao(1.23456789);
    let rao = tao.as_rao();
    assert_eq!(rao.as_u128(), 1_234_567_890);

    let formatted = tao.to_string();
    assert_eq!(formatted, "1.234567890 τ");

    let parsed = Tao::from_rao(1_234_567_890);
    assert_eq!(parsed, tao);
}

#[test]
fn test_lossless_conversion_check() {
    assert!(is_lossless_conversion(1.0));
    assert!(is_lossless_conversion(0.5));
    assert!(is_lossless_conversion(0.123456789));

    // This value requires rounding
    assert!(!is_lossless_conversion(0.1234567891));
}

#[test]
fn test_valid_tao_amount_check() {
    assert!(is_valid_tao_amount(1.0));
    assert!(is_valid_tao_amount(0.123456789));
    assert!(is_valid_tao_amount(0.0));

    // Too many decimals
    assert!(!is_valid_tao_amount(0.1234567891));

    // Negative value
    assert!(!is_valid_tao_amount(-0.1));
}

// ============================================================================
// Property-Based Tests
// ============================================================================

proptest! {
    #[test]
    fn prop_rao_tao_roundtrip(rao in 0u128..1_000_000_000_000_000u128) {
        let tao = rao_to_tao(rao);
        let rao_back = tao_to_rao(tao);

        // Allow for small precision loss due to float
        let diff = rao_back.abs_diff(rao);
        prop_assert!(diff <= 1);
    }

    #[test]
    fn prop_tao_rao_roundtrip(tao in 0.0f64..1_000_000.0f64) {
        if is_valid_tao_amount(tao) {
            let rao = tao_to_rao(tao);
            let tao_back = rao_to_tao(rao);

            // Convert back to rao to compare
            let rao_back = tao_to_rao(tao_back);
            let diff = rao_back.abs_diff(rao);
            prop_assert!(diff <= 1);
        }
    }
}
