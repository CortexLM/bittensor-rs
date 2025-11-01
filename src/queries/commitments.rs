use crate::chain::BittensorClient;
use anyhow::Result;
use subxt::dynamic::Value;
use parity_scale_codec::Encode;
use sp_core::crypto::AccountId32;

const SUBTENSOR_MODULE: &str = "SubtensorModule";
const COMMITMENTS_PALLET: &str = "Commitments";

/// Get commitment: SubtensorModule.Commits[(netuid, block, uid)] -> bytes
pub async fn get_commitment(client: &BittensorClient, netuid: u16, block: u64, uid: u64) -> Result<Option<String>> {
    let keys = vec![Value::u128(netuid as u128), Value::u128(block as u128), Value::u128(uid as u128)];
    if let Some(val) = client
        .storage_with_keys(SUBTENSOR_MODULE, "Commits", keys)
        .await?
    {
        return Ok(Some(decode_bytes_as_utf8(&val)));
    }
    Ok(None)
}

/// Get revealed commitments for a hotkey on a specific netuid
pub async fn get_revealed_commitment_by_hotkey(
    client: &BittensorClient,
    netuid: u16,
    hotkey: &AccountId32,
) -> Result<Vec<(u64, String)>> {
    let keys = vec![Value::u128(netuid as u128), Value::from_bytes(&hotkey.encode())];
    if let Some(val) = client
        .storage_with_keys(COMMITMENTS_PALLET, "RevealedCommitments", keys)
        .await?
    {
        return Ok(decode_revealed_vec(&val));
    }
    Ok(Vec::new())
}

/// Get revealed commitment (latest) for (netuid, hotkey)
pub async fn get_revealed_commitment(
    client: &BittensorClient,
    netuid: u16,
    hotkey: &AccountId32,
) -> Result<Option<(u64, String)>> {
    let keys = vec![Value::u128(netuid as u128), Value::from_bytes(&hotkey.encode())];
    if let Some(val) = client
        .storage_with_keys(COMMITMENTS_PALLET, "RevealedCommitments", keys)
        .await?
    {
        let items = decode_revealed_vec(&val);
        if let Some(last) = items.last() { return Ok(Some(last.clone())); }
    }
    Ok(None)
}

/// Get current weight commit info: SubtensorModule.CRV3WeightCommitsV2[(netuid)] -> Vec<Vec<u8>>
pub async fn get_current_weight_commit_info(client: &BittensorClient, netuid: u16) -> Result<Vec<Vec<u8>>> {
    if let Some(val) = client
        .storage_with_keys(SUBTENSOR_MODULE, "CRV3WeightCommitsV2", vec![Value::u128(netuid as u128)])
        .await?
    {
        return Ok(extract_vec_of_bytes(&val));
    }
    Ok(Vec::new())
}

/// Get timelocked weight commits
pub async fn get_timelocked_weight_commits(client: &BittensorClient, netuid: u16) -> Result<Vec<Vec<u8>>> {
    if let Some(val) = client
        .storage_with_keys(SUBTENSOR_MODULE, "TimelockedWeightCommits", vec![Value::u128(netuid as u128)])
        .await?
    {
        return Ok(extract_vec_of_bytes(&val));
    }
    Ok(Vec::new())
}

/// Get all commitments for a subnet from Commitments.CommitmentOf
pub async fn get_all_commitments(
    client: &BittensorClient,
    netuid: u16,
) -> Result<std::collections::HashMap<AccountId32, String>> {
    // Gather all hotkeys from subnet, then read each commitment entry
    let n_val = client
        .storage_with_keys("SubtensorModule", "SubnetworkN", vec![Value::u128(netuid as u128)])
        .await?;
    let n = n_val.and_then(|v| crate::utils::value_decode::decode_u64(&v).ok()).unwrap_or(0);
    let mut map = std::collections::HashMap::new();
    for uid in 0..n {
        if let Some(hk_val) = client
            .storage_with_keys("SubtensorModule", "Keys", vec![Value::u128(netuid as u128), Value::u128(uid as u128)])
            .await?
        {
            if let Ok(hk) = crate::utils::value_decode::decode_account_id32(&hk_val) {
                if let Some(val) = client
                    .storage_with_keys(COMMITMENTS_PALLET, "CommitmentOf", vec![
                        Value::u128(netuid as u128),
                        Value::from_bytes(&hk.encode()),
                    ])
                    .await?
                {
                    let msg = decode_metadata_like(&val);
                    if !msg.is_empty() { map.insert(hk, msg); }
                }
            }
        }
    }
    Ok(map)
}

/// Get all revealed commitments for a subnet: Commitments.RevealedCommitments[(netuid, hotkey)]
pub async fn get_all_revealed_commitments(
    client: &BittensorClient,
    netuid: u16,
) -> Result<std::collections::HashMap<AccountId32, Vec<(u64, String)>>> {
    let n_val = client
        .storage_with_keys("SubtensorModule", "SubnetworkN", vec![Value::u128(netuid as u128)])
        .await?;
    let n = n_val.and_then(|v| crate::utils::value_decode::decode_u64(&v).ok()).unwrap_or(0);
    let mut map = std::collections::HashMap::new();
    for uid in 0..n {
        if let Some(hk_val) = client
            .storage_with_keys("SubtensorModule", "Keys", vec![Value::u128(netuid as u128), Value::u128(uid as u128)])
            .await?
        {
            if let Ok(hk) = crate::utils::value_decode::decode_account_id32(&hk_val) {
                if let Some(val) = client
                    .storage_with_keys(COMMITMENTS_PALLET, "RevealedCommitments", vec![
                        Value::u128(netuid as u128),
                        Value::from_bytes(&hk.encode()),
                    ])
                    .await?
                {
                    let entries = decode_revealed_vec(&val);
                    if !entries.is_empty() { map.insert(hk, entries); }
                }
            }
        }
    }
    Ok(map)
}

/// Get last commitment bonds reset block: Commitments.LastBondsReset[(netuid, hotkey)] -> bytes containing block
pub async fn get_last_commitment_bonds_reset_block(
    client: &BittensorClient,
    netuid: u16,
    hotkey: &AccountId32,
) -> Result<Option<u64>> {
    if let Some(val) = client
        .storage_with_keys(COMMITMENTS_PALLET, "LastBondsReset", vec![
            Value::u128(netuid as u128),
            Value::from_bytes(&hotkey.encode()),
        ])
        .await?
    {
        let s = format!("{:?}", val);
        return Ok(extract_last_u64_from_str(&s));
    }
    Ok(None)
}

/// Get CRV3 weight commit info v2, decoded to (hotkey, block, message, reveal_round)
pub async fn get_current_weight_commit_info_v2(
    client: &BittensorClient,
    netuid: u16,
) -> Result<Vec<(AccountId32, u64, String, u64)>> {
    if let Some(val) = client
        .storage_with_keys(SUBTENSOR_MODULE, "CRV3WeightCommitsV2", vec![Value::u128(netuid as u128)])
        .await?
    {
        let s = format!("{:?}", val);
        // Split into commit-like chunks; heuristic using ")})," boundary
        let mut out = Vec::new();
        for part in s.split(")}),") {
            if let Some(hk) = extract_first_account_from_str(part) {
                let block_first = extract_first_u64_from_str(part);
                let reveal_last = extract_last_u64_from_str(part);
                let msg = decode_bytes_as_utf8_from_str(part);
                if block_first.is_some() || !msg.is_empty() || reveal_last.is_some() {
                    out.push((hk, block_first.unwrap_or(0), msg, reveal_last.unwrap_or(0)));
                }
            }
        }
        return Ok(out);
    }
    Ok(Vec::new())
}

// helper decoders unchanged below
fn decode_metadata_like(value: &Value) -> String { decode_bytes_as_utf8(value) }

fn decode_bytes_as_utf8(value: &Value) -> String {
    let s = format!("{:?}", value);
    let mut bytes = Vec::new();
    let mut rem = s.as_str();
    while let Some(pos) = rem.find("U128(") {
        let after = &rem[pos + 5..];
        if let Some(end) = after.find(')') {
            if let Ok(n) = after[..end].trim().parse::<u128>() { if n <= 255 { bytes.push(n as u8); } }
            rem = &after[end+1..];
        } else { break; }
    }
    String::from_utf8_lossy(&bytes).to_string()
}

#[allow(dead_code)]
fn decode_revealed_tuple(value: &Value) -> Result<(u64, String)> {
    let s = format!("{:?}", value);
    let mut block: u64 = 0;
    if let Some(pos) = s.rfind("U64(") { let after = &s[pos+4..]; if let Some(end) = after.find(')') { block = after[..end].trim().parse::<u64>().unwrap_or(0); } }
    Ok((block, decode_bytes_as_utf8(value)))
}

fn decode_revealed_vec(value: &Value) -> Vec<(u64, String)> {
    let s = format!("{:?}", value);
    let mut out = Vec::new();
    for part in s.split(")),") {
        if part.contains("U128(") && part.contains("U64(") {
            let msg = decode_bytes_as_utf8_from_str(part);
            let block = extract_last_u64_from_str(part);
            if block.is_some() || !msg.is_empty() { out.push((block.unwrap_or(0), msg)); }
        }
    }
    out
}

fn decode_bytes_as_utf8_from_str(s: &str) -> String {
    let mut bytes = Vec::new();
    let mut rem = s;
    while let Some(pos) = rem.find("U128(") {
        let after = &rem[pos + 5..];
        if let Some(end) = after.find(')') {
            if let Ok(n) = after[..end].trim().parse::<u128>() { if n <= 255 { bytes.push(n as u8); } }
            rem = &after[end+1..];
        } else { break; }
    }
    String::from_utf8_lossy(&bytes).to_string()
}

fn extract_last_u64_from_str(s: &str) -> Option<u64> {
    if let Some(pos) = s.rfind("U64(") { let after = &s[pos+4..]; if let Some(end) = after.find(')') { return after[..end].trim().parse::<u64>().ok(); } }
    None
}

fn extract_first_u64_from_str(s: &str) -> Option<u64> {
    if let Some(pos) = s.find("U64(") { let after = &s[pos+4..]; if let Some(end) = after.find(')') { return after[..end].trim().parse::<u64>().ok(); } }
    None
}

fn extract_vec_of_bytes(value: &Value) -> Vec<Vec<u8>> {
    let s = format!("{:?}", value);
    let mut groups: Vec<Vec<u8>> = Vec::new();
    let mut current: Vec<u8> = Vec::new();
    let mut rem = s.as_str();
    while let Some(pos) = rem.find("U128(") {
        let after = &rem[pos + 5..];
        if let Some(end) = after.find(')') {
            if let Ok(n) = after[..end].trim().parse::<u128>() { if n <= 255 { current.push(n as u8); } }
            if let Some(close) = after[end..].find(")") { if close > 0 && !current.is_empty() { groups.push(std::mem::take(&mut current)); } }
            rem = &after[end+1..];
        } else { break; }
    }
    if !current.is_empty() { groups.push(current); }
    groups
}

fn extract_first_account_from_str(s: &str) -> Option<AccountId32> {
    if let Some(pos) = s.find("0x") {
        let hexstr: String = s[pos+2..].chars().take_while(|c| c.is_ascii_hexdigit()).collect();
        if hexstr.len() >= 64 {
            if let Ok(bytes) = hex::decode(&hexstr[0..64]) {
                if bytes.len()==32 {
                    if let Ok(arr) = <[u8;32]>::try_from(bytes.as_slice()) {
                        return Some(AccountId32::from(arr));
                    }
                }
            }
        }
    }
    None
}
