//! Weight comparison tests - verify Rust implementation matches Python behavior
//! Run python_comparison.py first to generate expected values

use bittensor_rs::utils::balance::{rao_to_tao, tao_to_rao, Balance};
use bittensor_rs::utils::weights::{
    normalize_max_weight, normalize_weights, u16_normalized_float, U16_MAX,
};

#[test]
fn test_u16_normalized_float_matches_python() {
    // Python: u16_normalized_float(0) = 0.0
    assert_eq!(u16_normalized_float(0), 0.0);

    // Python: u16_normalized_float(65535) ≈ 1.0
    assert!((u16_normalized_float(U16_MAX) - 1.0).abs() < 0.0001);

    // Python: u16_normalized_float(32767) ≈ 0.5
    assert!((u16_normalized_float(32767) - 0.5).abs() < 0.01);
}

#[test]
fn test_balance_conversion_matches_python() {
    // Python: Balance.from_rao(1_000_000_000).tao == 1.0
    let bal = Balance::from_rao(1_000_000_000);
    assert!((bal.as_tao() - 1.0).abs() < 0.0001);

    // Python: Balance.from_tao(1.5).rao == 1_500_000_000
    let bal2 = Balance::from_tao(1.5);
    assert_eq!(bal2.as_rao(), 1_500_000_000);

    // Test arithmetic: 1 TAO + 1.5 TAO = 2.5 TAO
    let bal3 = bal + bal2;
    assert!((bal3.as_tao() - 2.5).abs() < 0.0001);
}

#[test]
fn test_rao_tao_conversion() {
    // 1 TAO = 1e9 RAO
    assert_eq!(tao_to_rao(1.0), 1_000_000_000);
    assert!((rao_to_tao(1_000_000_000) - 1.0).abs() < 0.0001);

    // 0.5 TAO = 5e8 RAO
    assert_eq!(tao_to_rao(0.5), 500_000_000);
    assert!((rao_to_tao(500_000_000) - 0.5).abs() < 0.0001);
}

#[test]
fn test_normalize_max_weight_even_weights() {
    // Test case from Python: even weights with limit 0.3
    let weights = vec![0.25f32, 0.25, 0.25, 0.25];
    let limit = 0.3f32;

    let result = normalize_max_weight(&weights, limit);

    // All weights should be equal (0.25 each)
    for w in &result {
        assert!((*w - 0.25).abs() < 0.01, "Weight {} should be ~0.25", w);
    }

    // Sum should be 1.0
    let sum: f32 = result.iter().sum();
    assert!((sum - 1.0).abs() < 0.01, "Sum {} should be 1.0", sum);
}

#[test]
fn test_normalize_max_weight_uneven_weights() {
    // Test case from Python: uneven weights with limit 0.35
    let weights = vec![0.1f32, 0.2, 0.3, 0.4];
    let limit = 0.35f32;

    let result = normalize_max_weight(&weights, limit);

    // Max weight should not exceed limit
    let max_w = result.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    assert!(
        max_w <= limit + 0.01,
        "Max weight {} exceeds limit {}",
        max_w,
        limit
    );

    // Sum should be 1.0
    let sum: f32 = result.iter().sum();
    assert!((sum - 1.0).abs() < 0.01, "Sum {} should be 1.0", sum);
}

#[test]
fn test_normalize_max_weight_dominant_weight() {
    // Test case from Python: one dominant weight with limit 0.4
    let weights = vec![0.05f32, 0.05, 0.1, 0.8];
    let limit = 0.4f32;

    let result = normalize_max_weight(&weights, limit);

    // Max weight should be capped at limit
    let max_w = result.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    assert!(
        max_w <= limit + 0.01,
        "Max weight {} exceeds limit {}",
        max_w,
        limit
    );

    // Sum should be 1.0
    let sum: f32 = result.iter().sum();
    assert!((sum - 1.0).abs() < 0.01, "Sum {} should be 1.0", sum);

    // The original dominant weight (0.8) should be reduced
    // and other weights should be relatively increased
}

#[test]
fn test_normalize_weights_basic() {
    // Test basic weight normalization for emit
    let uids = vec![0u64, 1, 2];
    let weights = vec![0.25f32, 0.5, 0.25];

    let (result_uids, result_weights) = normalize_weights(&uids, &weights).unwrap();

    // Should have 3 results
    assert_eq!(result_uids.len(), 3);
    assert_eq!(result_weights.len(), 3);

    // UIDs should be preserved (converted to u16)
    assert_eq!(result_uids, vec![0u16, 1, 2]);

    // Middle weight should be approximately double the others
    // In u16 representation, 0.5 ≈ 32767, 0.25 ≈ 16383
    let mid = result_weights[1] as f32;
    let side = result_weights[0] as f32;
    assert!(
        (mid / side - 2.0).abs() < 0.1,
        "Middle weight should be ~2x side weights"
    );
}

#[test]
fn test_normalize_weights_filters_zeros() {
    // Test that zero weights are filtered out
    let uids = vec![0u64, 1, 2, 3];
    let weights = vec![0.5f32, 0.0, 0.5, 0.0];

    let (result_uids, result_weights) = normalize_weights(&uids, &weights).unwrap();

    // Should only have 2 results (non-zero weights)
    assert_eq!(result_uids.len(), 2);
    assert_eq!(result_weights.len(), 2);

    // Should be UIDs 0 and 2
    assert_eq!(result_uids[0], 0);
    assert_eq!(result_uids[1], 2);
}

#[test]
fn test_normalize_weights_empty() {
    let uids: Vec<u64> = vec![];
    let weights: Vec<f32> = vec![];

    let (result_uids, result_weights) = normalize_weights(&uids, &weights).unwrap();

    assert!(result_uids.is_empty());
    assert!(result_weights.is_empty());
}

#[test]
fn test_normalize_weights_all_zeros() {
    // When all weights are zero, should distribute evenly
    let uids = vec![0u64, 1, 2];
    let weights = vec![0.0f32, 0.0, 0.0];

    let (result_uids, result_weights) = normalize_weights(&uids, &weights).unwrap();

    // With even distribution, all weights should be non-zero
    assert_eq!(result_uids.len(), 3);
    assert_eq!(result_weights.len(), 3);

    // All weights should be approximately equal
    let first = result_weights[0];
    for w in &result_weights {
        assert!(
            (*w as i32 - first as i32).abs() < 100,
            "Weights should be approximately equal"
        );
    }
}
