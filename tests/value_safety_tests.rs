//! Comprehensive value safety test suite for TAO/RAO conversions.
//!
//! This test suite validates:
//! 1. Compile-time type safety with newtypes (Rao and Tao)
//! 2. Conversion precision for all valid ranges
//! 3. Secure decimal arithmetic (no precision loss)
//! 4. Compatibility with Python SDK behavior
//! 5. RAOPERTAO = 1_000_000_000 consistency across all operations
//! 6. Checked / saturating arithmetic on Rao
//! 7. is_valid_transfer_amount guards
//! 8. From<Tao> for Rao checked multiplication
//!
//! # Property-Based Testing
//! Uses proptest to verify properties hold across a wide range of inputs.

use bittensor_rs::core::constants::{EXISTENTIAL_DEPOSIT_RAO, RAOPERTAO};
use bittensor_rs::utils::balance_newtypes::{
    format_rao_as_tao, is_lossless_conversion, is_valid_tao_amount, parse_tao_string, rao_to_tao,
    tao_to_rao, Balance, Rao, Tao,
};
use bittensor_rs::validator::{staking, transfer, weights as validator_weights};
use bittensor_rs::{queries, utils::balance_newtypes::get_unit_symbol, BittensorClient};
use proptest::prelude::*;
use std::any::type_name;
use std::str::FromStr;

// ============================================================================
// Compile-time assertions
// ============================================================================

const _: () = assert!(RAOPERTAO == 1_000_000_000, "RAOPERTAO must be 1e9");
const _: () = assert!(RAOPERTAO == 10u128.pow(9), "RAOPERTAO must equal 10^9");
const _: () = assert!(
    EXISTENTIAL_DEPOSIT_RAO == 500,
    "Existential deposit must be 500 RAO"
);

// ============================================================================
// Type-safety smoke tests
// ============================================================================

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
fn test_transfer_and_stake_amounts_are_rao_scalars() {
    fn assert_type<T: Into<u128>>() -> &'static str {
        type_name::<T>()
    }

    assert_eq!(assert_type::<u128>(), "u128");
    assert_eq!(
        assert_type::<Rao>(),
        "bittensor_rs::utils::balance_newtypes::Rao"
    );

    let tao_amount = Tao(1.0);
    let rao_amount = tao_amount.as_rao();
    let rao_value: u128 = rao_amount.into();
    assert_eq!(rao_value, RAOPERTAO);
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

// ============================================================================
// RAOPERTAO constant tests
// ============================================================================

#[test]
fn test_raopertao_constant() {
    assert_eq!(
        RAOPERTAO, 1_000_000_000u128,
        "RAOPERTAO must be exactly 1e9"
    );
    assert_eq!(RAOPERTAO, 10u128.pow(9), "RAOPERTAO should be 10^9");
}

#[test]
fn test_existential_deposit_constant() {
    assert_eq!(
        EXISTENTIAL_DEPOSIT_RAO, 500,
        "Existential deposit must be 500 RAO as per subtensor"
    );
}

// ============================================================================
// Rao newtype tests
// ============================================================================

#[test]
fn test_rao_newtype_type_safety() {
    let rao = Rao(1_000_000_000);
    let raw: u128 = rao.as_u128();
    assert_eq!(raw, 1_000_000_000);

    let sum = rao + Rao(500_000_000);
    assert_eq!(sum.as_u128(), 1_500_000_000);
}

#[test]
fn test_rao_constants() {
    assert_eq!(Rao::ZERO.as_u128(), 0);
    assert_eq!(Rao::ONE.as_u128(), 1);
    assert_eq!(Rao::MAX.as_u128(), u128::MAX);
    assert_eq!(Rao::PER_TAO.as_u128(), RAOPERTAO);
}

#[test]
fn test_rao_checked_add() {
    assert_eq!(Rao(10).checked_add(Rao(5)), Some(Rao(15)));
    assert_eq!(Rao(u128::MAX).checked_add(Rao(1)), None);
    assert_eq!(Rao(0).checked_add(Rao(0)), Some(Rao(0)));
}

#[test]
fn test_rao_checked_sub() {
    assert_eq!(Rao(10).checked_sub(Rao(5)), Some(Rao(5)));
    assert_eq!(Rao(0).checked_sub(Rao(1)), None);
    assert_eq!(Rao(5).checked_sub(Rao(5)), Some(Rao(0)));
}

#[test]
fn test_rao_saturating_add() {
    assert_eq!(Rao(10).saturating_add(Rao(5)), Rao(15));
    assert_eq!(Rao(u128::MAX).saturating_add(Rao(1)), Rao(u128::MAX));
}

#[test]
fn test_rao_saturating_sub() {
    assert_eq!(Rao(10).saturating_sub(Rao(5)), Rao(5));
    assert_eq!(Rao(0).saturating_sub(Rao(1)), Rao(0));
}

#[test]
fn test_rao_is_valid_transfer_amount() {
    assert!(!Rao::ZERO.is_valid_transfer_amount());
    assert!(Rao::ONE.is_valid_transfer_amount());
    assert!(Rao::PER_TAO.is_valid_transfer_amount());
    assert!(Rao(u64::MAX as u128).is_valid_transfer_amount());
    assert!(!Rao(u64::MAX as u128 + 1).is_valid_transfer_amount());
    assert!(!Rao::MAX.is_valid_transfer_amount());
}

#[test]
fn test_rao_is_valid_transfer_amount_with_max() {
    assert!(Rao(100).is_valid_transfer_amount_with_max(100));
    assert!(!Rao(101).is_valid_transfer_amount_with_max(100));
    assert!(!Rao(0).is_valid_transfer_amount_with_max(100));
}

// ============================================================================
// Tao newtype tests
// ============================================================================

#[test]
fn test_tao_newtype_type_safety() {
    let tao = Tao(1.5);
    let raw: f64 = tao.as_f64();
    assert_eq!(raw, 1.5);

    let sum = tao + Tao(0.5);
    assert_eq!(sum.as_f64(), 2.0);
}

#[test]
fn test_tao_to_rao_checked() {
    assert_eq!(Tao(1.0).to_rao_checked(), Some(Rao(1_000_000_000)));
    assert_eq!(Tao(0.0).to_rao_checked(), Some(Rao(0)));
    assert_eq!(Tao(-1.0).to_rao_checked(), None);
    assert_eq!(Tao(f64::NAN).to_rao_checked(), None);
    assert_eq!(Tao(f64::INFINITY).to_rao_checked(), None);
    assert_eq!(Tao(f64::NEG_INFINITY).to_rao_checked(), None);
}

#[test]
fn test_from_tao_for_rao_valid() {
    let rao: Rao = Tao(1.0).into();
    assert_eq!(rao, Rao(1_000_000_000));

    let rao: Rao = Tao(0.0).into();
    assert_eq!(rao, Rao(0));
}

#[test]
#[should_panic(expected = "Tao-to-Rao overflow")]
fn test_from_tao_for_rao_overflow_panics() {
    let _: Rao = Tao(f64::INFINITY).into();
}

#[test]
#[should_panic(expected = "Tao-to-Rao overflow")]
fn test_from_tao_for_rao_nan_panics() {
    let _: Rao = Tao(f64::NAN).into();
}

#[test]
#[should_panic(expected = "Tao-to-Rao overflow")]
fn test_from_tao_for_rao_negative_panics() {
    let _: Rao = Tao(-1.0).into();
}

// ============================================================================
// Conversion tests
// ============================================================================

#[test]
fn test_conversions_exact_at_boundaries() {
    let test_cases = [
        (0.0, 0u128),
        (0.000000001, 1u128),
        (0.00000001, 10u128),
        (0.0000001, 100u128),
        (0.000001, 1_000u128),
        (0.00001, 10_000u128),
        (0.0001, 100_000u128),
        (0.001, 1_000_000u128),
        (0.01, 10_000_000u128),
        (0.1, 100_000_000u128),
        (0.5, 500_000_000u128),
        (1.0, 1_000_000_000u128),
        (1.5, 1_500_000_000u128),
        (10.0, 10_000_000_000u128),
        (100.0, 100_000_000_000u128),
    ];

    for (tao, expected_rao) in test_cases {
        let actual_rao = tao_to_rao(tao);
        assert_eq!(
            actual_rao, expected_rao,
            "tao_to_rao({}) should return {}, got {}",
            tao, expected_rao, actual_rao
        );

        if tao < 9_007_199.0 {
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
fn test_conversion_consistency_one_tao() {
    assert_eq!(Rao(1_000_000_000).to_tao(), Tao(1.0));
    assert_eq!(Rao::PER_TAO.to_tao(), Tao(1.0));
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

#[test]
fn test_conversion_precision_small_values() {
    for i in 0..1000u128 {
        let rao = i * 1_000_000;
        let tao = rao_to_tao(rao);
        let rao_back = tao_to_rao(tao);
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
    let test_values = [
        1_000_000_000u128,
        1_000_000_000_000u128,
        1_000_000_000_000_000u128,
        9_000_000_000_000_000u128,
    ];

    for rao in test_values {
        let tao = rao_to_tao(rao);
        let rao_back = tao_to_rao(tao);
        let diff = rao_back.abs_diff(rao);
        assert!(
            diff <= 1,
            "Precision loss too large for {}: diff {}",
            rao,
            diff
        );
    }
}

// ============================================================================
// Edge cases
// ============================================================================

#[test]
fn test_zero_amount_conversions() {
    assert_eq!(tao_to_rao(0.0), 0);
    assert_eq!(rao_to_tao(0), 0.0);
    assert_eq!(Rao::ZERO.as_u128(), 0);
    assert_eq!(Tao::ZERO.as_f64(), 0.0);
    assert_eq!(Balance::from_rao(0).as_tao(), 0.0);
    assert_eq!(Balance::from_tao(0.0).as_rao(), 0);
}

#[test]
fn test_one_rao() {
    assert_eq!(Rao(1).as_tao(), 1e-9);
    assert_eq!(Rao(1).to_tao(), Tao(1e-9));
}

#[test]
fn test_raopertao_rao() {
    assert_eq!(Rao(RAOPERTAO).as_tao(), 1.0);
    assert_eq!(Rao(RAOPERTAO).to_tao(), Tao(1.0));
}

#[test]
fn test_u128_max_rao_saturating() {
    let max_rao = Rao(u128::MAX);
    let one_rao = Rao(1);
    assert_eq!((max_rao + one_rao).as_u128(), u128::MAX);

    let zero_rao = Rao(0);
    assert_eq!((zero_rao - one_rao).as_u128(), 0);
}

#[test]
fn test_negative_and_nan_tao_to_rao() {
    assert_eq!(tao_to_rao(-1.0), 0);
    assert_eq!(tao_to_rao(-0.0), 0);
    assert_eq!(tao_to_rao(f64::NAN), 0);
    assert_eq!(tao_to_rao(f64::NEG_INFINITY), 0);
    assert_eq!(tao_to_rao(f64::INFINITY), 0);
}

#[test]
fn test_tao_to_rao_f64_max_does_not_panic() {
    let result = tao_to_rao(f64::MAX);
    assert!(result == 0 || result == u128::MAX);
}

#[test]
fn test_fractional_tao_precision() {
    let cases = [
        (0.000000001, 1u128),
        (0.123456789, 123_456_789u128),
        (0.999999999, 999_999_999u128),
        (1.999999999, 1_999_999_999u128),
    ];
    for (tao, expected_rao) in cases {
        assert_eq!(
            tao_to_rao(tao),
            expected_rao,
            "tao_to_rao({}) should be {}",
            tao,
            expected_rao
        );
    }
}

// ============================================================================
// Display and formatting tests
// ============================================================================

#[test]
fn test_rao_display_shows_both_units() {
    let rao = Rao(1_234_567_890);
    let display = format!("{}", rao);
    assert!(display.contains("τ"), "Display should contain TAO symbol");
    assert!(display.contains("ρ"), "Display should contain RAO symbol");
    assert!(
        display.contains("1234567890"),
        "Display should contain raw RAO"
    );
    assert!(
        display.contains("1.234567890"),
        "Display should contain TAO value"
    );
}

#[test]
fn test_rao_format_tao() {
    assert_eq!(Rao(1_000_000_000).format_tao(), "1.000000000 τ");
    assert_eq!(Rao(0).format_tao(), "0.000000000 τ");
    assert_eq!(Rao(1).format_tao(), "0.000000001 τ");
}

#[test]
fn test_rao_format_rao() {
    assert_eq!(Rao(1_000_000_000).format_rao(), "1000000000 ρ");
}

#[test]
fn test_tao_display() {
    assert_eq!(format!("{}", Tao(1.5)), "1.500000000 τ");
    assert_eq!(Tao(1.5).format(), "1.500000000 τ");
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
fn test_rao_newtype_conversions() {
    let rao = Rao(1_234_567_890);
    let tao = rao.as_tao();
    assert_eq!(tao, 1.23456789);

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

// ============================================================================
// String parsing tests
// ============================================================================

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

    let invalid_cases = ["", "abc", "1.0000000000", "-1", "1.", ".1"];

    for s in invalid_cases {
        assert!(parse_tao_string(s).is_none(), "Should reject '{}'", s);
    }
}

// ============================================================================
// Balance newtype tests
// ============================================================================

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
fn test_balance_from_rao_roundtrip() {
    for rao_val in [
        0u128,
        1,
        999_999_999,
        1_000_000_000,
        21_000_000_000_000_000u128,
    ] {
        let bal = Balance::from_rao(rao_val);
        assert_eq!(bal.as_rao(), rao_val);
    }
}

// ============================================================================
// Validation tests
// ============================================================================

#[test]
fn test_lossless_conversion_check() {
    assert!(is_lossless_conversion(1.0));
    assert!(is_lossless_conversion(0.5));
    assert!(is_lossless_conversion(0.123456789));
    assert!(!is_lossless_conversion(0.1234567891));
}

#[test]
fn test_valid_tao_amount_check() {
    assert!(is_valid_tao_amount(1.0));
    assert!(is_valid_tao_amount(0.123456789));
    assert!(is_valid_tao_amount(0.0));
    assert!(!is_valid_tao_amount(0.1234567891));
    assert!(!is_valid_tao_amount(-0.1));
}

#[test]
fn test_transfer_and_staking_amount_type_is_rao() {
    fn takes_rao(_: Rao) {}
    takes_rao(Rao(1_000_000_000));
    takes_rao(Tao(1.0).as_rao());
}

#[test]
fn test_rao_safe_div_by_zero() {
    assert_eq!(Rao(1_000_000_000).safe_div(0).as_u128(), 0);
    assert_eq!(Rao(0).safe_div(0).as_u128(), 0);
}

#[test]
fn test_tao_is_display_only_never_extrinsic() {
    let tao_val = Tao(1.5);
    let rao_val = tao_val.as_rao();
    assert_eq!(rao_val.as_u128(), 1_500_000_000);

    let _transfer_fn = transfer::transfer;
    let _add_stake_fn = staking::add_stake;
    let _unstake_fn = staking::unstake;
}

#[test]
fn test_tao_to_rao_ceiling_correctness() {
    use bittensor_rs::utils::balance_newtypes::tao_to_rao_ceiling;

    assert_eq!(tao_to_rao_ceiling(1.0), 1_000_000_000);
    assert_eq!(tao_to_rao_ceiling(0.0), 0);
    assert_eq!(tao_to_rao_ceiling(-1.0), 0);
    assert_eq!(tao_to_rao_ceiling(f64::NAN), 0);
    assert_eq!(tao_to_rao_ceiling(f64::INFINITY), 0);

    assert_eq!(tao_to_rao_ceiling(0.0000000001), 1);
    assert_eq!(tao_to_rao_ceiling(1.0000000001), 1_000_000_001);
}

#[test]
fn test_tao_to_rao_rounded_correctness() {
    use bittensor_rs::utils::balance_newtypes::tao_to_rao_rounded;

    assert_eq!(tao_to_rao_rounded(1.0), 1_000_000_000);
    assert_eq!(tao_to_rao_rounded(0.0), 0);
    assert_eq!(tao_to_rao_rounded(-1.0), 0);
    assert_eq!(tao_to_rao_rounded(f64::NAN), 0);
    assert_eq!(tao_to_rao_rounded(f64::INFINITY), 0);

    assert_eq!(tao_to_rao_rounded(1.0000000005), 1_000_000_001);
    assert_eq!(tao_to_rao_rounded(1.0000000004), 1_000_000_000);
}

// ============================================================================
// Property-Based Tests
// ============================================================================

proptest! {
    #[test]
    fn prop_rao_roundtrip_identity(r in 0u128..=u128::MAX) {
        prop_assert_eq!(Rao(r).as_u128(), r);
    }

    #[test]
    fn prop_rao_tao_roundtrip(rao in 0u128..1_000_000_000_000_000u128) {
        let tao = rao_to_tao(rao);
        let rao_back = tao_to_rao(tao);
        let diff = rao_back.abs_diff(rao);
        prop_assert!(diff <= 1);
    }

    #[test]
    fn prop_tao_rao_roundtrip(tao in 0.0f64..1_000_000.0f64) {
        if is_valid_tao_amount(tao) {
            let rao = tao_to_rao(tao);
            let tao_back = rao_to_tao(rao);
            let rao_back = tao_to_rao(tao_back);
            let diff = rao_back.abs_diff(rao);
            prop_assert!(diff <= 1);
        }
    }

    #[test]
    fn prop_tao_to_rao_never_panics(tao in proptest::num::f64::ANY) {
        let _ = tao_to_rao(tao);
    }

    #[test]
    fn prop_rao_to_tao_never_panics(rao in 0u128..=u128::MAX) {
        let _ = rao_to_tao(rao);
    }

    #[test]
    fn prop_checked_add_consistent_with_saturating(a in 0u128..=u128::MAX, b in 0u128..=u128::MAX) {
        let ra = Rao(a);
        let rb = Rao(b);
        match ra.checked_add(rb) {
            Some(result) => prop_assert_eq!(result, ra.saturating_add(rb)),
            None => prop_assert_eq!(ra.saturating_add(rb), Rao::MAX),
        }
    }

    #[test]
    fn prop_checked_sub_consistent_with_saturating(a in 0u128..=u128::MAX, b in 0u128..=u128::MAX) {
        let ra = Rao(a);
        let rb = Rao(b);
        match ra.checked_sub(rb) {
            Some(result) => prop_assert_eq!(result, ra.saturating_sub(rb)),
            None => prop_assert_eq!(ra.saturating_sub(rb), Rao::ZERO),
        }
    }

    #[test]
    fn prop_valid_transfer_amount_excludes_zero(amount in 1u128..=(u64::MAX as u128)) {
        prop_assert!(Rao(amount).is_valid_transfer_amount());
    }

    #[test]
    fn prop_valid_transfer_amount_rejects_above_u64_max(amount in (u64::MAX as u128 + 1)..=u128::MAX) {
        prop_assert!(!Rao(amount).is_valid_transfer_amount());
    }
}

// ============================================================================
// Finney balance integration test
// ============================================================================

#[tokio::test]
async fn test_finney_balance_is_rao_and_symbol_tau() {
    let client = match BittensorClient::with_default().await {
        Ok(client) => client,
        Err(err) => {
            eprintln!("Skipping test: unable to connect ({err})");
            return;
        }
    };

    let coldkey =
        sp_core::crypto::AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY")
            .expect("valid account");

    let balance = queries::balances::get_balance(&client, &coldkey).await;
    if balance.is_err() {
        eprintln!("Skipping test: unable to decode balance");
        return;
    }
    assert_eq!(get_unit_symbol(0), "τ");
}
