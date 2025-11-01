use bittensor_rs::{
    delegate::DelegateInfoBase,
    metagraph::Metagraph,
    types::{AxonInfo, DelegateInfo, NeuronInfo, PrometheusInfo, SubnetInfo},
};
use sp_core::crypto::AccountId32;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;

#[test]
fn test_axon_info_creation() {
    let ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let axon = AxonInfo {
        version: 1,
        ip,
        port: 8080,
        ip_type: 4,
        protocol: 1,
        placeholder1: 0,
        placeholder2: 0,
    };

    assert_eq!(axon.version, 1);
    assert_eq!(axon.port, 8080);
    assert_eq!(axon.ip_type, 4); // IPv4
    assert!(axon.is_serving());
    assert_eq!(axon.to_endpoint(), "http://10.0.0.1:8080");
}

#[test]
fn test_prometheus_info_creation() {
    let prometheus = PrometheusInfo::from_chain_data(
        1000,                   // block
        1,                      // version
        "10.0.0.1".to_string(), // ip
        9090,                   // port
        4,                      // ip_type (IPv4)
    );

    assert_eq!(prometheus.version, 1);
    assert_eq!(prometheus.port, 9090);
    assert_eq!(prometheus.ip_type, 4);
}

#[test]
fn test_neuron_info_structure() {
    let mut stake_dict = HashMap::new();
    let coldkey =
        AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
    stake_dict.insert(coldkey, 1000000u128);

    let neuron = NeuronInfo {
        hotkey: AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap(),
        coldkey: AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap(),
        uid: 0,
        netuid: 1,
        active: true,
        axon_info: None,
        prometheus_info: None,
        stake: 1000000u128,
        stake_dict: stake_dict.clone(),
        total_stake: 1000000u128,
        root_stake: 0u128,
        rank: 100.0,
        emission: 1000.0,
        incentive: 50.0,
        consensus: 80.0,
        trust: 90.0,
        validator_trust: 95.0,
        dividends: 10.0,
        last_update: 1000,
        validator_permit: true,
        weights: vec![],
        bonds: vec![],
        pruning_score: 0,
        is_null: false,
        version: 100,
    };

    assert_eq!(neuron.total_stake, 1000000);
    assert_eq!(neuron.version, 100);
    assert!(!neuron.is_null);
    assert_eq!(neuron.stake_dict.len(), 1);
}

#[test]
fn test_metagraph_initialization() {
    let metagraph = Metagraph::new(5);
    assert_eq!(metagraph.netuid, 5);
    assert_eq!(metagraph.n, 0);
    assert!(metagraph.neurons.is_empty());
}

#[test]
fn test_subnet_info_creation() {
    let subnet = SubnetInfo {
        netuid: 1,
        neuron_count: 256,
        total_stake: 1000000,
        emission: 500000.0,
        name: Some("Test Subnet".to_string()),
        description: Some("A test subnet".to_string()),
    };

    assert_eq!(subnet.netuid, 1);
    assert_eq!(subnet.neuron_count, 256);
    assert_eq!(subnet.total_stake, 1000000);
    assert_eq!(subnet.emission, 500000.0);
    assert_eq!(subnet.name.as_ref().unwrap(), "Test Subnet");
}

#[test]
fn test_delegate_info_creation() {
    let hotkey = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
    let owner = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();

    let delegate = DelegateInfo::new(
        hotkey.clone(),
        owner.clone(),
        0.18, // 18% take
    );

    assert_eq!(delegate.base.take, 0.18);
    assert_eq!(delegate.base.hotkey_ss58, hotkey);
    assert_eq!(delegate.base.owner_ss58, owner);
    assert_eq!(delegate.nominators.len(), 0);
}

#[test]
fn test_account_id_from_string() {
    let account_str = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
    let account_id = AccountId32::from_str(account_str).unwrap();

    // Vérifier que l'AccountId a été créé correctement
    let bytes: &[u8] = account_id.as_ref();
    assert_eq!(bytes.len(), 32); // AccountId32 doit avoir 32 bytes
}

#[test]
fn test_hashmap_functionality() {
    let mut map = HashMap::new();
    let key1 = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
    let key2 = AccountId32::from_str("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty").unwrap();

    map.insert(key1.clone(), 100u128);
    map.insert(key2.clone(), 200u128);

    assert_eq!(map.get(&key1), Some(&100u128));
    assert_eq!(map.get(&key2), Some(&200u128));
    assert_eq!(map.len(), 2);
}

#[test]
fn test_metagraph_neuron_access() {
    let mut metagraph = Metagraph::new(1);

    let neuron = NeuronInfo {
        hotkey: AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap(),
        coldkey: AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap(),
        uid: 0,
        netuid: 1,
        active: true,
        axon_info: None,
        prometheus_info: None,
        stake: 1000u128,
        stake_dict: HashMap::new(),
        total_stake: 1000u128,
        root_stake: 0u128,
        rank: 100.0,
        emission: 1000.0,
        incentive: 50.0,
        consensus: 80.0,
        trust: 90.0,
        validator_trust: 95.0,
        dividends: 10.0,
        last_update: 1000,
        validator_permit: true,
        weights: vec![],
        bonds: vec![],
        pruning_score: 0,
        is_null: false,
        version: 100,
    };

    metagraph.neurons.insert(0, neuron);
    metagraph.n = 1;

    assert_eq!(metagraph.neurons.len(), 1);
    assert_eq!(metagraph.n, 1);
    assert_eq!(metagraph.neurons.get(&0).unwrap().total_stake, 1000);
}
