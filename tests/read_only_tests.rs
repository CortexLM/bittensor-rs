//! Comprehensive read-only operation tests matching Python SDK behavior
//! These tests verify that the Rust SDK produces identical results to the Python SDK

use bittensor_rs::utils::balance::{rao_to_tao, tao_to_rao, Balance};
use bittensor_rs::utils::weights::{
    convert_weight_uids_and_vals_to_tensor, normalize_max_weight, normalize_weights,
    u16_normalized_float,
};
use sp_core::crypto::AccountId32;
use std::str::FromStr;

// =============================================================================
// Balance Tests (matching Python bittensor.utils.balance)
// =============================================================================

#[test]
fn test_balance_rao_to_tao_conversion() {
    assert_eq!(rao_to_tao(1_000_000_000), 1.0);
    assert_eq!(rao_to_tao(500_000_000), 0.5);
    assert_eq!(rao_to_tao(0), 0.0);
    assert_eq!(rao_to_tao(1), 1e-9);
}

#[test]
fn test_balance_tao_to_rao_conversion() {
    assert_eq!(tao_to_rao(1.0), 1_000_000_000);
    assert_eq!(tao_to_rao(0.5), 500_000_000);
    assert_eq!(tao_to_rao(0.0), 0);
    assert_eq!(tao_to_rao(0.000000001), 1);
}

#[test]
fn test_balance_roundtrip() {
    let original_rao: u128 = 123_456_789_012;
    let tao = rao_to_tao(original_rao);
    let back_to_rao = tao_to_rao(tao);
    assert_eq!(original_rao, back_to_rao);
}

#[test]
fn test_balance_struct_operations() {
    let balance = Balance::from_rao(1_000_000_000);
    assert_eq!(balance.rao, 1_000_000_000);
    assert_eq!(balance.as_tao(), 1.0);

    let balance2 = Balance::from_tao(2.5);
    assert_eq!(balance2.as_tao(), 2.5);
    assert_eq!(balance2.rao, 2_500_000_000);
}

#[test]
fn test_balance_arithmetic() {
    let b1 = Balance::from_tao(1.0);
    let b2 = Balance::from_tao(2.0);

    let sum = b1 + b2;
    assert_eq!(sum.as_tao(), 3.0);

    let diff = b2 - b1;
    assert_eq!(diff.as_tao(), 1.0);
}

#[test]
fn test_balance_display() {
    let balance = Balance::from_tao(1.5);
    let display = format!("{}", balance);
    assert!(display.contains("TAO"));
}

// =============================================================================
// Weight Normalization Tests (matching Python bittensor.utils.weight_utils)
// =============================================================================

#[test]
fn test_normalize_weights_basic() {
    let uids: Vec<u64> = vec![0, 1, 2];
    let weights: Vec<f32> = vec![1.0, 2.0, 3.0];

    let (result_uids, result_weights) = normalize_weights(&uids, &weights).unwrap();

    assert_eq!(result_uids.len(), 3);
    assert_eq!(result_weights.len(), 3);

    let total: u32 = result_weights.iter().map(|&w| w as u32).sum();
    assert!(total <= u16::MAX as u32 + 3);
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
    let uids: Vec<u64> = vec![0, 1, 2, 3];
    let weights: Vec<f32> = vec![0.0, 0.0, 0.0, 0.0];

    let (result_uids, result_weights) = normalize_weights(&uids, &weights).unwrap();

    // When all weights are zero, Rust SDK returns all UIDs with equal distribution
    assert_eq!(result_uids.len(), 4);
    assert_eq!(result_weights.len(), 4);
    // All weights should be equal
    assert!(result_weights.iter().all(|&w| w == result_weights[0]));
}

#[test]
fn test_normalize_weights_filters_zeros() {
    let uids: Vec<u64> = vec![0, 1, 2, 3, 4];
    let weights: Vec<f32> = vec![1.0, 0.0, 2.0, 0.0, 3.0];

    let (result_uids, result_weights) = normalize_weights(&uids, &weights).unwrap();

    assert_eq!(result_uids.len(), 3);
    assert_eq!(result_weights.len(), 3);
    assert!(result_uids.contains(&0));
    assert!(result_uids.contains(&2));
    assert!(result_uids.contains(&4));
}

#[test]
fn test_normalize_weights_single_element() {
    let uids: Vec<u64> = vec![5];
    let weights: Vec<f32> = vec![100.0];

    let (result_uids, result_weights) = normalize_weights(&uids, &weights).unwrap();

    assert_eq!(result_uids.len(), 1);
    assert_eq!(result_uids[0], 5);
    assert_eq!(result_weights[0], u16::MAX);
}

#[test]
fn test_normalize_weights_preserves_relative_order() {
    let uids: Vec<u64> = vec![0, 1, 2];
    let weights: Vec<f32> = vec![1.0, 2.0, 3.0];

    let (_, result_weights) = normalize_weights(&uids, &weights).unwrap();

    assert!(result_weights[0] < result_weights[1]);
    assert!(result_weights[1] < result_weights[2]);
}

// =============================================================================
// Normalize Max Weight Tests (matching Python normalize_max_weight)
// =============================================================================

#[test]
fn test_normalize_max_weight_limit_001() {
    let weights: Vec<f32> = (0..1000).map(|i| (i as f32) / 1000.0).collect();
    let limit = 0.01;

    let result = normalize_max_weight(&weights, limit);

    assert!(result.iter().all(|&w| w <= limit + 0.0001));
}

#[test]
fn test_normalize_max_weight_limit_002() {
    let weights: Vec<f32> = (0..1000).map(|i| (i as f32) / 1000.0).collect();
    let limit = 0.02;

    let result = normalize_max_weight(&weights, limit);

    assert!(result.iter().all(|&w| w <= limit + 0.0001));
}

#[test]
fn test_normalize_max_weight_limit_003() {
    let weights: Vec<f32> = (0..1000).map(|i| (i as f32) / 1000.0).collect();
    let limit = 0.03;

    let result = normalize_max_weight(&weights, limit);

    assert!(result.iter().all(|&w| w <= limit + 0.0001));
}

#[test]
fn test_normalize_max_weight_all_zeros() {
    let weights: Vec<f32> = vec![0.0; 2000];
    let limit = 0.01;

    let result = normalize_max_weight(&weights, limit);

    let expected_max = 1.0 / 2000.0;
    let max_val = result.iter().cloned().fold(0.0f32, f32::max);
    assert!((max_val - expected_max).abs() < 0.0001);
}

#[test]
fn test_normalize_max_weight_even_distribution() {
    let weights: Vec<f32> = vec![1.0; 10];
    let limit = 0.1;

    let result = normalize_max_weight(&weights, limit);

    let expected = 0.1;
    for w in &result {
        assert!((w - expected).abs() < 0.0001);
    }
}

#[test]
fn test_normalize_max_weight_dominant_weight() {
    let mut weights: Vec<f32> = vec![0.01; 99];
    weights.push(100.0);
    let limit = 0.5;

    let result = normalize_max_weight(&weights, limit);

    assert!(result.iter().cloned().fold(0.0f32, f32::max) <= limit + 0.0001);
}

#[test]
fn test_normalize_max_weight_sums_to_one() {
    let weights: Vec<f32> = (0..100).map(|i| (i as f32 + 1.0) / 100.0).collect();
    let limit = 0.1;

    let result = normalize_max_weight(&weights, limit);
    let sum: f32 = result.iter().sum();

    assert!((sum - 1.0).abs() < 0.01);
}

// =============================================================================
// U16 Normalized Float Tests (matching Python u16_normalized_float)
// =============================================================================

#[test]
fn test_u16_normalized_float_max() {
    assert!((u16_normalized_float(u16::MAX) - 1.0).abs() < 0.0001);
}

#[test]
fn test_u16_normalized_float_zero() {
    assert_eq!(u16_normalized_float(0), 0.0);
}

#[test]
fn test_u16_normalized_float_half() {
    let half = u16::MAX / 2;
    let result = u16_normalized_float(half);
    assert!((result - 0.5).abs() < 0.01);
}

#[test]
fn test_u16_normalized_float_range() {
    for i in (0..=u16::MAX).step_by(1000) {
        let result = u16_normalized_float(i);
        assert!(result >= 0.0 && result <= 1.0);
    }
}

#[test]
fn test_u16_normalized_float_monotonic() {
    let mut prev = 0.0;
    for i in (0..=u16::MAX).step_by(100) {
        let result = u16_normalized_float(i);
        assert!(result >= prev);
        prev = result;
    }
}

// =============================================================================
// Convert Weight UIDs and Vals to Tensor Tests
// =============================================================================

#[test]
fn test_convert_weight_uids_and_vals_to_tensor_basic() {
    let n = 3;
    let uids: Vec<u16> = vec![0, 1, 2];
    let weights: Vec<u16> = vec![15, 5, 80];

    let result = convert_weight_uids_and_vals_to_tensor(n, &uids, &weights);

    assert_eq!(result.len(), 3);
    let sum: f32 = result.iter().sum();
    assert!((sum - 1.0).abs() < 0.01);
}

#[test]
fn test_convert_weight_uids_and_vals_to_tensor_sparse() {
    let n = 4;
    let uids: Vec<u16> = vec![1, 3];
    let weights: Vec<u16> = vec![50, 50];

    let result = convert_weight_uids_and_vals_to_tensor(n, &uids, &weights);

    assert_eq!(result.len(), 4);
    assert_eq!(result[0], 0.0);
    assert!((result[1] - 0.5).abs() < 0.01);
    assert_eq!(result[2], 0.0);
    assert!((result[3] - 0.5).abs() < 0.01);
}

#[test]
fn test_convert_weight_uids_and_vals_to_tensor_empty() {
    let n = 5;
    let uids: Vec<u16> = vec![];
    let weights: Vec<u16> = vec![];

    let result = convert_weight_uids_and_vals_to_tensor(n, &uids, &weights);

    assert_eq!(result.len(), 5);
    assert!(result.iter().all(|&w| w == 0.0));
}

#[test]
fn test_convert_weight_uids_and_vals_to_tensor_single() {
    let n = 1;
    let uids: Vec<u16> = vec![0];
    let weights: Vec<u16> = vec![100];

    let result = convert_weight_uids_and_vals_to_tensor(n, &uids, &weights);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0], 1.0);
}

#[test]
fn test_convert_weight_uids_and_vals_to_tensor_all_zeros() {
    let n = 4;
    let uids: Vec<u16> = vec![0, 1, 2, 3];
    let weights: Vec<u16> = vec![0, 0, 0, 0];

    let result = convert_weight_uids_and_vals_to_tensor(n, &uids, &weights);

    assert_eq!(result.len(), 4);
    assert!(result.iter().all(|&w| w == 0.0));
}

// =============================================================================
// AccountId Tests
// =============================================================================

#[test]
fn test_account_id_from_valid_ss58() {
    let valid_address = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
    let account = AccountId32::from_str(valid_address);
    assert!(account.is_ok());
}

#[test]
fn test_account_id_from_invalid_ss58() {
    let invalid_address = "invalid_address";
    let account = AccountId32::from_str(invalid_address);
    assert!(account.is_err());
}

#[test]
fn test_account_id_byte_length() {
    let address = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
    let account = AccountId32::from_str(address).unwrap();
    let bytes: &[u8] = account.as_ref();
    assert_eq!(bytes.len(), 32);
}

#[test]
fn test_account_id_equality() {
    let address = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
    let account1 = AccountId32::from_str(address).unwrap();
    let account2 = AccountId32::from_str(address).unwrap();
    assert_eq!(account1, account2);
}

#[test]
fn test_account_id_from_bytes() {
    let bytes = [1u8; 32];
    let account = AccountId32::from(bytes);
    let back: &[u8] = account.as_ref();
    assert_eq!(back, bytes);
}

// =============================================================================
// Type Structure Tests
// =============================================================================

#[test]
fn test_neuron_info_structure() {
    use bittensor_rs::types::NeuronInfo;
    use std::collections::HashMap;

    let hotkey = AccountId32::from([1u8; 32]);
    let coldkey = AccountId32::from([2u8; 32]);

    let neuron = NeuronInfo {
        netuid: 1,
        uid: 0,
        hotkey: hotkey.clone(),
        coldkey: coldkey.clone(),
        active: true,
        stake: 1000000,
        stake_dict: HashMap::new(),
        total_stake: 1000000,
        root_stake: 0,
        rank: 0.5,
        emission: 100.0,
        incentive: 0.5,
        consensus: 0.8,
        trust: 0.9,
        validator_trust: 0.95,
        dividends: 0.1,
        last_update: 1000,
        validator_permit: true,
        weights: vec![],
        bonds: vec![],
        pruning_score: 0,
        is_null: false,
        version: 100,
        axon_info: None,
        prometheus_info: None,
    };

    assert_eq!(neuron.uid, 0);
    assert_eq!(neuron.netuid, 1);
    assert!(neuron.validator_permit);
    assert!(!neuron.is_null);
    assert!(neuron.active);
}

#[test]
fn test_subnet_info_structure() {
    use bittensor_rs::types::SubnetInfo;

    let subnet = SubnetInfo {
        netuid: 1,
        neuron_count: 256,
        total_stake: 1000000000,
        emission: 0.5,
        name: Some("test".to_string()),
        description: None,
    };

    assert_eq!(subnet.netuid, 1);
    assert_eq!(subnet.neuron_count, 256);
    assert_eq!(subnet.name, Some("test".to_string()));
}

#[test]
fn test_axon_info_structure() {
    use bittensor_rs::types::AxonInfo;
    use std::net::{IpAddr, Ipv4Addr};

    let axon = AxonInfo {
        version: 1,
        ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        port: 8091,
        ip_type: 4,
        protocol: 4,
        placeholder1: 0,
        placeholder2: 0,
    };

    assert_eq!(axon.port, 8091);
    assert!(axon.is_serving());
}

#[test]
fn test_prometheus_info_structure() {
    use bittensor_rs::types::PrometheusInfo;

    let prometheus = PrometheusInfo {
        block: 1000,
        version: 1,
        ip: "127.0.0.1".to_string(),
        port: 9090,
        ip_type: 4,
    };

    assert_eq!(prometheus.port, 9090);
    assert_eq!(prometheus.block, 1000);
}

// =============================================================================
// Synapse Tests
// =============================================================================

#[test]
fn test_synapse_creation() {
    use bittensor_rs::types::Synapse;

    let synapse = Synapse::new();
    // Synapse has a default timeout of 12.0
    assert_eq!(synapse.timeout, Some(12.0));
}

#[test]
fn test_synapse_with_timeout() {
    use bittensor_rs::types::Synapse;

    let synapse = Synapse::new().with_timeout(30.0);
    assert_eq!(synapse.timeout, Some(30.0));
}

#[test]
fn test_synapse_with_name() {
    use bittensor_rs::types::Synapse;

    let mut synapse = Synapse::new();
    synapse.name = Some("test_synapse".to_string());
    assert_eq!(synapse.name, Some("test_synapse".to_string()));
}

// =============================================================================
// DynamicInfo Tests
// =============================================================================

#[test]
fn test_dynamic_info_creation() {
    use bittensor_rs::types::DynamicInfo;

    let info = DynamicInfo::default();
    assert_eq!(info.netuid, 0);
    assert_eq!(info.tempo, 0);
}

#[test]
fn test_dynamic_info_fields() {
    use bittensor_rs::types::DynamicInfo;

    let mut info = DynamicInfo::default();
    info.netuid = 1;
    info.tempo = 360;
    info.emission_value = 1_000_000_000;

    assert_eq!(info.netuid, 1);
    assert_eq!(info.tempo, 360);
}

// =============================================================================
// MetagraphInfo Tests
// =============================================================================

#[test]
fn test_metagraph_info_creation() {
    use bittensor_rs::types::MetagraphInfo;

    let info = MetagraphInfo::new(1);
    assert_eq!(info.netuid, 1);
}

#[test]
fn test_metagraph_info_n_method() {
    use bittensor_rs::types::MetagraphInfo;

    let mut info = MetagraphInfo::new(1);
    info.hotkeys = (0..256).map(|i| format!("hotkey_{}", i)).collect();
    assert_eq!(info.n(), 256);
}

// =============================================================================
// Config Tests
// =============================================================================

#[test]
fn test_config_default() {
    use bittensor_rs::Config;

    let config = Config::default();
    assert!(config.subtensor.network.len() > 0);
}

#[test]
fn test_axon_config_default() {
    use bittensor_rs::AxonConfig;

    let config = AxonConfig::default();
    assert_eq!(config.port, 8091);
    assert_eq!(config.ip, "0.0.0.0");
}

#[test]
fn test_subtensor_config_default() {
    use bittensor_rs::SubtensorConfig;

    let config = SubtensorConfig::default();
    assert_eq!(config.network, "finney");
}

// =============================================================================
// Edge Case Tests
// =============================================================================

#[test]
fn test_large_weight_array() {
    let weights: Vec<f32> = (0..10000).map(|i| i as f32).collect();
    let result = normalize_max_weight(&weights, 0.001);

    assert_eq!(result.len(), 10000);
    assert!(result.iter().all(|&w| w <= 0.001 + 0.0001));
}

#[test]
fn test_very_small_weights() {
    let weights: Vec<f32> = vec![1e-10, 2e-10, 3e-10];
    let result = normalize_max_weight(&weights, 0.5);

    let sum: f32 = result.iter().sum();
    assert!((sum - 1.0).abs() < 0.01);
}

#[test]
fn test_mixed_weight_magnitudes() {
    let weights: Vec<f32> = vec![0.001, 1.0, 1000.0];
    let result = normalize_max_weight(&weights, 0.9);

    assert!(result.iter().all(|&w| w <= 0.9 + 0.0001));
    let sum: f32 = result.iter().sum();
    assert!((sum - 1.0).abs() < 0.01);
}
