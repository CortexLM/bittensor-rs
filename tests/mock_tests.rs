use bittensor_rs::{
    delegate::DelegateInfoBase,
    queries::SubnetHyperparameters,
    types::{DelegateInfo, DelegatedInfo, SubnetInfo},
};
use sp_core::crypto::AccountId32;
use std::collections::HashMap;
use std::str::FromStr;

#[test]
fn test_subnet_info_basic() {
    let subnet = SubnetInfo {
        netuid: 1,
        neuron_count: 256,
        total_stake: 1000000,
        emission: 100000,
        name: Some("Test Subnet".to_string()),
        description: Some("Test description".to_string()),
    };

    assert_eq!(subnet.netuid, 1);
    assert_eq!(subnet.neuron_count, 256);
    assert_eq!(subnet.name.as_ref().unwrap(), "Test Subnet");
}

#[test]
fn test_subnet_hyperparameters() {
    let params = SubnetHyperparameters {
        rho: 0,
        kappa: 0,
        weights_version: 1,
        weights_rate_limit: 0,
        min_stake: 100000,
        min_burn: 1000,
        max_burn: 1000000,
        bonds_moving_avg: 0,
        max_regs_per_block: 0,
        adjustment_alpha: 100,
        target_regs_per_interval: 10,
        adjustment_interval: 0,
        immunity_period: 5000,
        min_allowed_weights: 1,
        max_weights_limit: 0,
        max_weight_limit: 65535,
        tempo: 99,
        min_difficulty: 0,
        max_difficulty: 0,
        difficulty: 0,
        activity_cutoff: 0,
        registration_allowed: false,
        max_validators: 0,
        max_allowed_uids: 4096,
        serving_rate_limit: 0,
        commit_reveal_weights_interval: 0,
        commit_reveal_weights_enabled: false,
        alpha_high: 0,
        alpha_low: 0,
        liquid_alpha_enabled: false,
    };

    assert_eq!(params.tempo, 99);
    assert_eq!(params.max_allowed_uids, 4096);
    assert_eq!(params.weights_version, 1);
}

#[test]
fn test_delegate_info_structure() {
    let mut nominators = HashMap::new();
    let mut subnet_stakes = HashMap::new();
    subnet_stakes.insert(1u16, 1000000u128);

    let nominator =
        AccountId32::from_str("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty").unwrap();
    nominators.insert(nominator, subnet_stakes);

    let mut total_stake = HashMap::new();
    total_stake.insert(1u16, 1000000u128);

    let delegate = DelegateInfo {
        base: DelegateInfoBase {
            hotkey_ss58: AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY")
                .unwrap(),
            owner_ss58: AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY")
                .unwrap(),
            take: 0.18,
            validator_permits: vec![1, 3],
            registrations: vec![1, 3, 5],
            return_per_1000: 150,
            total_daily_return: 10000,
        },
        total_stake: total_stake.clone(),
        nominators: nominators.clone(),
    };

    assert_eq!(delegate.base.take, 0.18);
    assert_eq!(delegate.base.registrations.len(), 3);
    assert_eq!(delegate.nominators.len(), 1);
    assert_eq!(delegate.base.return_per_1000, 150);
}

#[test]
fn test_delegated_info_structure() {
    let delegated = DelegatedInfo {
        base: DelegateInfoBase {
            hotkey_ss58: AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY")
                .unwrap(),
            owner_ss58: AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY")
                .unwrap(),
            take: 0.18,
            validator_permits: vec![1],
            registrations: vec![1],
            return_per_1000: 150,
            total_daily_return: 10000,
        },
        netuid: 1,
        stake: 500000,
    };

    assert_eq!(delegated.netuid, 1);
    assert_eq!(delegated.stake, 500000);
    assert_eq!(delegated.base.take, 0.18);
}

#[test]
fn test_hashmap_operations() {
    let mut stake_map: HashMap<u16, u128> = HashMap::new();
    stake_map.insert(1, 100000);
    stake_map.insert(2, 200000);
    stake_map.insert(3, 300000);

    assert_eq!(stake_map.get(&1), Some(&100000));
    assert_eq!(stake_map.get(&2), Some(&200000));
    assert_eq!(stake_map.len(), 3);

    let total: u128 = stake_map.values().sum();
    assert_eq!(total, 600000);
}

#[test]
fn test_vector_operations() {
    let mut registrations = vec![1u16, 3, 5, 7];

    registrations.push(9);
    assert_eq!(registrations.len(), 5);
    assert_eq!(registrations[4], 9);

    assert!(registrations.contains(&3));
    assert!(!registrations.contains(&4));

    let sum: u16 = registrations.iter().sum();
    assert_eq!(sum, 25);
}

#[test]
fn test_account_id_operations() {
    let account1 =
        AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
    let account2 =
        AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
    let account3 =
        AccountId32::from_str("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty").unwrap();

    assert_eq!(account1, account2);
    assert_ne!(account1, account3);

    let mut map = HashMap::new();
    map.insert(account1.clone(), 100);
    map.insert(account3.clone(), 200);

    assert_eq!(map.get(&account1), Some(&100));
    assert_eq!(map.get(&account3), Some(&200));
}
