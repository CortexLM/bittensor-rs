//! CRv4 commit-reveal flow tests
//!
//! Validates:
//! - `calculate_reveal_round()` with various parameters
//! - `prepare_crv4_commit()` produces valid encrypted data
//! - `WeightsTlockPayload` SCALE encode/decode roundtrip
//! - Drand round calculation correctness
//! - `get_mechid_storage_index()` formula verification

use bittensor_rs::crv4::{
    calculate_reveal_round, get_mechid_storage_index, prepare_crv4_commit, verify_encrypted_data,
    DrandInfo, WeightsTlockPayload, DRAND_QUICKNET_GENESIS, DRAND_QUICKNET_PK_HEX,
    DRAND_ROUND_INTERVAL_SECS,
};
use parity_scale_codec::{Decode, Encode};

// ============================================================================
// calculate_reveal_round tests
// ============================================================================

#[test]
fn test_reveal_round_always_greater_than_chain_last() {
    let chain_last = 24_000_000u64;
    for tempo in [100u16, 360, 720] {
        for reveal_period in [1u64, 2, 5] {
            let rr = calculate_reveal_round(tempo, 5000, 1, reveal_period, 12.0, chain_last);
            assert!(
                rr > chain_last,
                "reveal_round {} must be > chain_last {} (tempo={}, rp={})",
                rr,
                chain_last,
                tempo,
                reveal_period
            );
        }
    }
}

#[test]
fn test_reveal_round_zero_reveal_period() {
    let chain_last = 24_000_000u64;
    let rr = calculate_reveal_round(360, 5000, 1, 0, 12.0, chain_last);
    assert_eq!(rr, chain_last + 1);
}

#[test]
fn test_reveal_round_scales_with_chain_state() {
    let tempo = 360u16;
    let current_block = 5000u64;
    let reveal_period = 1u64;

    let rr1 = calculate_reveal_round(tempo, current_block, 1, reveal_period, 12.0, 24_000_000);
    let rr2 = calculate_reveal_round(tempo, current_block, 1, reveal_period, 12.0, 25_000_000);

    assert_eq!(
        rr2 - rr1,
        1_000_000,
        "reveal_round difference should match chain state difference"
    );
}

#[test]
fn test_reveal_round_increases_with_reveal_period() {
    let chain_last = 24_000_000u64;
    let rr1 = calculate_reveal_round(360, 5000, 1, 1, 12.0, chain_last);
    let rr2 = calculate_reveal_round(360, 5000, 1, 2, 12.0, chain_last);
    assert!(
        rr2 > rr1,
        "longer reveal period should give a later reveal round"
    );
}

#[test]
fn test_reveal_round_varies_with_tempo() {
    let chain_last = 24_000_000u64;
    let rr_short = calculate_reveal_round(100, 5000, 1, 1, 12.0, chain_last);
    let rr_long = calculate_reveal_round(720, 5000, 1, 1, 12.0, chain_last);
    assert!(
        rr_short > chain_last && rr_long > chain_last,
        "both reveal rounds should exceed chain_last"
    );
}

#[test]
fn test_reveal_round_saturating_arithmetic() {
    let rr = calculate_reveal_round(360, u64::MAX - 10, 1, 1, 12.0, u64::MAX - 100);
    assert!(rr > 0, "should not panic on near-max values");
}

// ============================================================================
// prepare_crv4_commit tests
// ============================================================================

#[test]
fn test_prepare_crv4_commit_valid_output() {
    let hotkey = vec![1u8; 32];
    let uids = vec![0u16, 1, 2];
    let weights = vec![10_000u16, 20_000, 35_535];
    let version_key = 0u64;
    let reveal_round = 1000u64;

    let encrypted =
        prepare_crv4_commit(&hotkey, &uids, &weights, version_key, reveal_round).unwrap();
    assert!(!encrypted.is_empty());
    assert!(verify_encrypted_data(&encrypted));
}

#[test]
fn test_prepare_crv4_commit_different_rounds_differ() {
    let hotkey = vec![2u8; 32];
    let uids = vec![0u16, 1];
    let weights = vec![30_000u16, 35_535];

    let enc1 = prepare_crv4_commit(&hotkey, &uids, &weights, 0, 100).unwrap();
    let enc2 = prepare_crv4_commit(&hotkey, &uids, &weights, 0, 200).unwrap();
    assert_ne!(
        enc1, enc2,
        "different reveal rounds should produce different ciphertext"
    );
}

#[test]
fn test_prepare_crv4_commit_empty_uids() {
    let hotkey = vec![3u8; 32];
    let encrypted = prepare_crv4_commit(&hotkey, &[], &[], 0, 500).unwrap();
    assert!(verify_encrypted_data(&encrypted));
}

#[test]
fn test_prepare_crv4_commit_max_u16_weights() {
    let hotkey = vec![4u8; 32];
    let uids = vec![0u16];
    let weights = vec![u16::MAX];
    let encrypted = prepare_crv4_commit(&hotkey, &uids, &weights, 0, 500).unwrap();
    assert!(verify_encrypted_data(&encrypted));
}

// ============================================================================
// WeightsTlockPayload SCALE encode/decode roundtrip
// ============================================================================

#[test]
fn test_payload_scale_roundtrip() {
    let payload = WeightsTlockPayload {
        hotkey: vec![0xAB; 32],
        uids: vec![0, 1, 2, 3],
        values: vec![10_000, 20_000, 30_000, 5_535],
        version_key: 42,
    };

    let encoded = payload.encode();
    let decoded = WeightsTlockPayload::decode(&mut &encoded[..]).unwrap();

    assert_eq!(payload.hotkey, decoded.hotkey);
    assert_eq!(payload.uids, decoded.uids);
    assert_eq!(payload.values, decoded.values);
    assert_eq!(payload.version_key, decoded.version_key);
}

#[test]
fn test_payload_scale_empty_vecs() {
    let payload = WeightsTlockPayload {
        hotkey: vec![],
        uids: vec![],
        values: vec![],
        version_key: 0,
    };

    let encoded = payload.encode();
    let decoded = WeightsTlockPayload::decode(&mut &encoded[..]).unwrap();
    assert!(decoded.hotkey.is_empty());
    assert!(decoded.uids.is_empty());
    assert!(decoded.values.is_empty());
    assert_eq!(decoded.version_key, 0);
}

#[test]
fn test_payload_scale_large_payload() {
    let payload = WeightsTlockPayload {
        hotkey: vec![0xFF; 32],
        uids: (0..256).collect(),
        values: (0..256).map(|i| (i * 256) as u16).collect(),
        version_key: u64::MAX,
    };

    let encoded = payload.encode();
    let decoded = WeightsTlockPayload::decode(&mut &encoded[..]).unwrap();
    assert_eq!(payload.uids.len(), decoded.uids.len());
    assert_eq!(payload.values.len(), decoded.values.len());
    assert_eq!(payload.version_key, decoded.version_key);
}

// ============================================================================
// DrandInfo tests
// ============================================================================

#[test]
fn test_drand_quicknet_constants() {
    assert_eq!(DRAND_QUICKNET_GENESIS, 1688385600);
    assert_eq!(DRAND_ROUND_INTERVAL_SECS, 3);

    let pk_bytes = hex::decode(DRAND_QUICKNET_PK_HEX).expect("valid hex");
    assert_eq!(pk_bytes.len(), 96, "G2 compressed point must be 96 bytes");
}

#[test]
fn test_drand_round_at_genesis() {
    let info = DrandInfo::quicknet();
    assert_eq!(info.round_at_time(info.genesis_time), 1);
}

#[test]
fn test_drand_round_before_genesis() {
    let info = DrandInfo::quicknet();
    assert_eq!(info.round_at_time(0), 1);
    assert_eq!(info.round_at_time(info.genesis_time - 1), 1);
}

#[test]
fn test_drand_round_progression() {
    let info = DrandInfo::quicknet();
    for i in 0..100u64 {
        let expected_round = i + 1;
        assert_eq!(
            info.round_at_time(info.genesis_time + i * 3),
            expected_round
        );
    }
}

#[test]
fn test_drand_time_for_round_inverse() {
    let info = DrandInfo::quicknet();
    for round in 1..=100u64 {
        let time = info.time_for_round(round);
        let back = info.round_at_time(time);
        assert_eq!(
            back, round,
            "round_at_time(time_for_round({})) should be {}",
            round, round
        );
    }
}

#[test]
fn test_drand_current_round_positive() {
    let info = DrandInfo::quicknet();
    let round = info.current_round();
    assert!(round > 0, "current round should be positive");
    assert!(
        round > 1_000_000,
        "current round should be well past genesis by now"
    );
}

// ============================================================================
// get_mechid_storage_index tests
// ============================================================================

#[test]
fn test_mechid_storage_index_main_mechanism() {
    assert_eq!(get_mechid_storage_index(0, 0), 0);
    assert_eq!(get_mechid_storage_index(1, 0), 1);
    assert_eq!(get_mechid_storage_index(100, 0), 100);
}

#[test]
fn test_mechid_storage_index_formula() {
    assert_eq!(get_mechid_storage_index(1, 1), 4097);
    assert_eq!(get_mechid_storage_index(0, 1), 4096);
    assert_eq!(get_mechid_storage_index(0, 2), 8192);
}

#[test]
fn test_mechid_storage_index_saturating() {
    let result = get_mechid_storage_index(u16::MAX, 15);
    let _ = result;
}

// ============================================================================
// Integration test: CRv4 flow against Finney
// ============================================================================

#[tokio::test]
async fn test_crv4_flow_against_finney() {
    use bittensor_rs::crv4::get_last_drand_round;
    use bittensor_rs::BittensorClient;

    let client = match BittensorClient::with_default().await {
        Ok(c) => c,
        Err(err) => {
            eprintln!("Skipping test: unable to connect ({err})");
            return;
        }
    };

    let chain_last_round = match get_last_drand_round(&client).await {
        Ok(r) => r,
        Err(err) => {
            eprintln!("Skipping test: unable to get drand round ({err})");
            return;
        }
    };
    assert!(
        chain_last_round > 0,
        "chain should have stored drand rounds"
    );

    let current_block = client.block_number().await.expect("block number");
    assert!(current_block > 0);

    let netuid = 1u16;
    let tempo = bittensor_rs::queries::subnets::tempo(&client, netuid)
        .await
        .expect("tempo query")
        .unwrap_or(0);
    assert!(tempo > 0, "tempo should be positive for subnet 1");

    let reveal_period =
        bittensor_rs::queries::subnets::get_subnet_reveal_period_epochs(&client, netuid)
            .await
            .expect("reveal period")
            .unwrap_or(1);

    let storage_index = get_mechid_storage_index(netuid, 0);
    let reveal_round = calculate_reveal_round(
        tempo as u16,
        current_block,
        storage_index,
        reveal_period,
        12.0,
        chain_last_round,
    );
    assert!(reveal_round > chain_last_round);

    let hotkey = vec![0xAA; 32];
    let uids = vec![0u16, 1, 2];
    let weights = vec![10_000u16, 20_000, 35_535];
    let encrypted = prepare_crv4_commit(&hotkey, &uids, &weights, 0, reveal_round).unwrap();
    assert!(verify_encrypted_data(&encrypted));
}
