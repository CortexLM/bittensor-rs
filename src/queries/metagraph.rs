//! Metagraph synchronization queries
//!
//! Functions for fetching metagraph data from the chain.

use crate::error::{Error, Result};
use crate::metagraph::Metagraph;
use crate::queries::chain_info::{
    decode_bool, decode_u16, decode_u64, extract_account_bytes, get_block_number,
};
use crate::queries::subnets::{get_subnetwork_n, get_tempo, subnet_exists};
use crate::types::AxonInfo;
use crate::utils::ss58::account_to_ss58;
use crate::utils::u16_normalized_float;
use scale_value::{Composite, Primitive, ValueDef};
use sp_core::crypto::AccountId32;
use subxt::dynamic::{storage, Value};
use subxt::OnlineClient;
use subxt::PolkadotConfig;

const SUBTENSOR_MODULE: &str = "SubtensorModule";

/// Sync metagraph from chain
///
/// This function fetches all neuron data for a subnet and populates the metagraph.
pub async fn sync_metagraph(
    client: &OnlineClient<PolkadotConfig>,
    netuid: u16,
    network: &str,
    lite: bool,
) -> Result<Metagraph> {
    // Check if subnet exists
    if !subnet_exists(client, netuid).await? {
        return Err(Error::SubnetNotFound(netuid));
    }

    let mut metagraph = Metagraph::new(netuid, network);
    
    // Get block and basic info
    metagraph.block = get_block_number(client).await?;
    metagraph.n = get_subnetwork_n(client, netuid).await?;
    metagraph.tempo = get_tempo(client, netuid).await?;

    if metagraph.n == 0 {
        return Ok(metagraph);
    }

    // Initialize vectors
    let n = metagraph.n as usize;
    metagraph.uids = (0..metagraph.n).collect();
    metagraph.stake = vec![0; n];
    metagraph.tao_stake = vec![0; n];
    metagraph.alpha_stake = vec![0; n];
    metagraph.ranks = vec![0.0; n];
    metagraph.trust = vec![0.0; n];
    metagraph.consensus = vec![0.0; n];
    metagraph.validator_trust = vec![0.0; n];
    metagraph.incentive = vec![0.0; n];
    metagraph.emission = vec![0; n];
    metagraph.dividends = vec![0.0; n];
    metagraph.active = vec![false; n];
    metagraph.last_update = vec![0; n];
    metagraph.validator_permit = vec![false; n];
    metagraph.pruning_score = vec![0; n];
    metagraph.hotkeys = vec![String::new(); n];
    metagraph.coldkeys = vec![String::new(); n];
    metagraph.axons = vec![AxonInfo::default(); n];
    metagraph.block_at_registration = vec![0; n];

    // Fetch hotkeys (Keys storage)
    let keys_query = storage(SUBTENSOR_MODULE, "Keys", vec![Value::u128(netuid as u128)]);
    let mut keys_iter = client
        .storage()
        .at_latest()
        .await?
        .iter(keys_query)
        .await?;

    while let Some(Ok(kv)) = keys_iter.next().await {
        let key = &kv.key_bytes;
        if let Some(uid) = extract_uid_from_key(key) {
            if uid < n as u16 {
                if let Some(account) = decode_account(&kv.value) {
                    metagraph.hotkeys[uid as usize] = account_to_ss58(&account);
                }
            }
        }
    }

    // Fetch rank
    fetch_u16_array(client, netuid, "Rank", &mut metagraph.ranks).await?;
    
    // Fetch trust
    fetch_u16_array(client, netuid, "Trust", &mut metagraph.trust).await?;
    
    // Fetch consensus
    fetch_u16_array(client, netuid, "Consensus", &mut metagraph.consensus).await?;
    
    // Fetch validator_trust
    fetch_u16_array(client, netuid, "ValidatorTrust", &mut metagraph.validator_trust).await?;
    
    // Fetch incentive
    fetch_u16_array(client, netuid, "Incentive", &mut metagraph.incentive).await?;
    
    // Fetch dividends
    fetch_u16_array(client, netuid, "Dividends", &mut metagraph.dividends).await?;
    
    // Fetch emission
    fetch_u64_array(client, netuid, "Emission", &mut metagraph.emission).await?;
    
    // Fetch last_update
    fetch_u64_array(client, netuid, "LastUpdate", &mut metagraph.last_update).await?;
    
    // Fetch active
    fetch_bool_array(client, netuid, "Active", &mut metagraph.active).await?;
    
    // Fetch validator_permit
    fetch_bool_array(client, netuid, "ValidatorPermit", &mut metagraph.validator_permit).await?;
    
    // Fetch pruning_score
    fetch_u16_raw_array(client, netuid, "PruningScores", &mut metagraph.pruning_score).await?;

    // Fetch axon info if not lite
    if !lite {
        fetch_axon_info(client, netuid, &mut metagraph.axons, &metagraph.hotkeys).await?;
    }

    // Fetch stake info
    fetch_stake_info(client, netuid, &metagraph.hotkeys, &mut metagraph.stake).await?;

    Ok(metagraph)
}

/// Extract UID from storage key
fn extract_uid_from_key(key: &[u8]) -> Option<u16> {
    if key.len() >= 2 {
        let uid_bytes = &key[key.len() - 2..];
        Some(u16::from_le_bytes([uid_bytes[0], uid_bytes[1]]))
    } else {
        None
    }
}

/// Decode AccountId32 from storage value
fn decode_account(value: &subxt::dynamic::DecodedValueThunk) -> Option<AccountId32> {
    if let Ok(val) = value.to_value() {
        if let Some(bytes) = extract_account_bytes(&val) {
            return Some(AccountId32::from(bytes));
        }
        // Try direct primitive
        if let ValueDef::Primitive(Primitive::U256(bytes)) = &val.value {
            return Some(AccountId32::from(*bytes));
        }
    }
    None
}

/// Fetch u16 values and convert to f64 (normalized)
async fn fetch_u16_array(
    client: &OnlineClient<PolkadotConfig>,
    netuid: u16,
    storage_name: &str,
    output: &mut [f64],
) -> Result<()> {
    let query = storage(
        SUBTENSOR_MODULE,
        storage_name,
        vec![Value::u128(netuid as u128)],
    );
    let mut iter = client.storage().at_latest().await?.iter(query).await?;

    while let Some(Ok(kv)) = iter.next().await {
        let key = &kv.key_bytes;
        if let Some(uid) = extract_uid_from_key(key) {
            if (uid as usize) < output.len() {
                if let Some(val) = decode_u16(&kv.value) {
                    output[uid as usize] = u16_normalized_float(val);
                }
            }
        }
    }
    Ok(())
}

/// Fetch u16 values (raw, not normalized)
async fn fetch_u16_raw_array(
    client: &OnlineClient<PolkadotConfig>,
    netuid: u16,
    storage_name: &str,
    output: &mut [u16],
) -> Result<()> {
    let query = storage(
        SUBTENSOR_MODULE,
        storage_name,
        vec![Value::u128(netuid as u128)],
    );
    let mut iter = client.storage().at_latest().await?.iter(query).await?;

    while let Some(Ok(kv)) = iter.next().await {
        let key = &kv.key_bytes;
        if let Some(uid) = extract_uid_from_key(key) {
            if (uid as usize) < output.len() {
                if let Some(val) = decode_u16(&kv.value) {
                    output[uid as usize] = val;
                }
            }
        }
    }
    Ok(())
}

/// Fetch u64 values
async fn fetch_u64_array(
    client: &OnlineClient<PolkadotConfig>,
    netuid: u16,
    storage_name: &str,
    output: &mut [u64],
) -> Result<()> {
    let query = storage(
        SUBTENSOR_MODULE,
        storage_name,
        vec![Value::u128(netuid as u128)],
    );
    let mut iter = client.storage().at_latest().await?.iter(query).await?;

    while let Some(Ok(kv)) = iter.next().await {
        let key = &kv.key_bytes;
        if let Some(uid) = extract_uid_from_key(key) {
            if (uid as usize) < output.len() {
                if let Some(val) = decode_u64(&kv.value) {
                    output[uid as usize] = val;
                }
            }
        }
    }
    Ok(())
}

/// Fetch bool values
async fn fetch_bool_array(
    client: &OnlineClient<PolkadotConfig>,
    netuid: u16,
    storage_name: &str,
    output: &mut [bool],
) -> Result<()> {
    let query = storage(
        SUBTENSOR_MODULE,
        storage_name,
        vec![Value::u128(netuid as u128)],
    );
    let mut iter = client.storage().at_latest().await?.iter(query).await?;

    while let Some(Ok(kv)) = iter.next().await {
        let key = &kv.key_bytes;
        if let Some(uid) = extract_uid_from_key(key) {
            if (uid as usize) < output.len() {
                if let Some(val) = decode_bool(&kv.value) {
                    output[uid as usize] = val;
                }
            }
        }
    }
    Ok(())
}

/// Fetch axon info for all neurons
async fn fetch_axon_info(
    client: &OnlineClient<PolkadotConfig>,
    netuid: u16,
    axons: &mut [AxonInfo],
    hotkeys: &[String],
) -> Result<()> {
    let query = storage(
        SUBTENSOR_MODULE,
        "Axons",
        vec![Value::u128(netuid as u128)],
    );
    let mut iter = client.storage().at_latest().await?.iter(query).await?;

    while let Some(Ok(kv)) = iter.next().await {
        let key = &kv.key_bytes;
        if let Some(uid) = extract_uid_from_key(key) {
            if (uid as usize) < axons.len() {
                if let Ok(val) = kv.value.to_value() {
                    if let Some(axon) = parse_axon_info(&val, hotkeys.get(uid as usize)) {
                        axons[uid as usize] = axon;
                    }
                }
            }
        }
    }
    Ok(())
}

/// Parse AxonInfo from decoded value
fn parse_axon_info<T>(
    val: &scale_value::Value<T>,
    hotkey: Option<&String>,
) -> Option<AxonInfo> {
    if let ValueDef::Composite(Composite::Named(fields)) = &val.value {
        let mut axon = AxonInfo::default();
        
        for (name, value) in fields {
            match name.as_str() {
                "version" => {
                    if let ValueDef::Primitive(Primitive::U128(n)) = &value.value {
                        axon.version = *n as u32;
                    }
                }
                "ip" => {
                    if let ValueDef::Primitive(Primitive::U128(n)) = &value.value {
                        // Will be converted later based on ip_type
                        let ip_int = *n;
                        axon.ip = format!("{}", ip_int); // temporary
                    }
                }
                "port" => {
                    if let ValueDef::Primitive(Primitive::U128(n)) = &value.value {
                        axon.port = *n as u16;
                    }
                }
                "ip_type" => {
                    if let ValueDef::Primitive(Primitive::U128(n)) = &value.value {
                        axon.ip_type = *n as u8;
                    }
                }
                "protocol" => {
                    if let ValueDef::Primitive(Primitive::U128(n)) = &value.value {
                        axon.protocol = *n as u8;
                    }
                }
                _ => {}
            }
        }
        
        // Convert IP integer to string
        if let Ok(ip_int) = axon.ip.parse::<u128>() {
            axon.ip = AxonInfo::ip_from_int(ip_int, axon.ip_type);
        }
        
        if let Some(hk) = hotkey {
            axon.hotkey = hk.clone();
        }
        
        return Some(axon);
    }
    None
}

/// Fetch stake info for neurons
async fn fetch_stake_info(
    client: &OnlineClient<PolkadotConfig>,
    _netuid: u16,
    hotkeys: &[String],
    stakes: &mut [u64],
) -> Result<()> {
    // For simplicity, query TotalHotkeyStake for each hotkey
    for (idx, hotkey) in hotkeys.iter().enumerate() {
        if hotkey.is_empty() {
            continue;
        }
        
        // Decode hotkey to bytes
        if let Ok(account_bytes) = crate::utils::ss58::ss58_decode(hotkey) {
            let val = crate::queries::chain_info::query_storage_value(
                client,
                SUBTENSOR_MODULE,
                "TotalHotkeyStake",
                vec![Value::from_bytes(account_bytes)],
            )
            .await?;
            
            if let Some(v) = val.and_then(|v| decode_u64(&v)) {
                stakes[idx] = v;
            }
        }
    }
    Ok(())
}
