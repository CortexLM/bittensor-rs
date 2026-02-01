use crate::chain::BittensorClient;
use crate::errors::{BittensorError, BittensorResult, ChainQueryError};
use anyhow::Result;
use parity_scale_codec::Encode;
use sp_core::crypto::AccountId32;
use subxt::dynamic::Value;

const SUBTENSOR_MODULE: &str = "SubtensorModule";
const COMMITMENTS_PALLET: &str = "Commitments";

/// Weight commitment information stored on chain
#[derive(Debug, Clone)]
pub struct WeightCommitInfo {
    /// The block number when the commitment was made
    pub block: u64,
    /// The committed data (typically hash of weights)
    pub commit_hash: Vec<u8>,
    /// The reveal round number
    pub reveal_round: u64,
}

impl WeightCommitInfo {
    /// Create a new WeightCommitInfo
    pub fn new(block: u64, commit_hash: Vec<u8>, reveal_round: u64) -> Self {
        Self {
            block,
            commit_hash,
            reveal_round,
        }
    }

    /// Get the commit hash as a hex string
    pub fn commit_hash_hex(&self) -> String {
        hex::encode(&self.commit_hash)
    }
}

/// Get commitment: SubtensorModule.Commits[(netuid, block, uid)] -> bytes
pub async fn get_commitment(
    client: &BittensorClient,
    netuid: u16,
    block: u64,
    uid: u64,
) -> Result<Option<String>> {
    let keys = vec![
        Value::u128(netuid as u128),
        Value::u128(block as u128),
        Value::u128(uid as u128),
    ];
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
    let keys = vec![
        Value::u128(netuid as u128),
        Value::from_bytes(hotkey.encode()),
    ];
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
    let keys = vec![
        Value::u128(netuid as u128),
        Value::from_bytes(hotkey.encode()),
    ];
    if let Some(val) = client
        .storage_with_keys(COMMITMENTS_PALLET, "RevealedCommitments", keys)
        .await?
    {
        let items = decode_revealed_vec(&val);
        if let Some(last) = items.last() {
            return Ok(Some(last.clone()));
        }
    }
    Ok(None)
}

/// Get current weight commit info: SubtensorModule.CRV3WeightCommitsV2[(netuid)] -> Vec<Vec<u8>>
pub async fn get_current_weight_commit_info(
    client: &BittensorClient,
    netuid: u16,
) -> Result<Vec<Vec<u8>>> {
    if let Some(val) = client
        .storage_with_keys(
            SUBTENSOR_MODULE,
            "CRV3WeightCommitsV2",
            vec![Value::u128(netuid as u128)],
        )
        .await?
    {
        return Ok(extract_vec_of_bytes(&val));
    }
    Ok(Vec::new())
}

/// Get timelocked weight commits
pub async fn get_timelocked_weight_commits(
    client: &BittensorClient,
    netuid: u16,
) -> Result<Vec<Vec<u8>>> {
    if let Some(val) = client
        .storage_with_keys(
            SUBTENSOR_MODULE,
            "TimelockedWeightCommits",
            vec![Value::u128(netuid as u128)],
        )
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
        .storage_with_keys(
            "SubtensorModule",
            "SubnetworkN",
            vec![Value::u128(netuid as u128)],
        )
        .await?;
    let n = n_val
        .and_then(|v| crate::utils::decoders::decode_u64(&v).ok())
        .unwrap_or(0);
    let mut map = std::collections::HashMap::new();
    for uid in 0..n {
        if let Some(hk_val) = client
            .storage_with_keys(
                "SubtensorModule",
                "Keys",
                vec![Value::u128(netuid as u128), Value::u128(uid as u128)],
            )
            .await?
        {
            if let Ok(hk) = crate::utils::decoders::decode_account_id32(&hk_val) {
                if let Some(val) = client
                    .storage_with_keys(
                        COMMITMENTS_PALLET,
                        "CommitmentOf",
                        vec![Value::u128(netuid as u128), Value::from_bytes(hk.encode())],
                    )
                    .await?
                {
                    let msg = decode_metadata_like(&val);
                    if !msg.is_empty() {
                        map.insert(hk, msg);
                    }
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
        .storage_with_keys(
            "SubtensorModule",
            "SubnetworkN",
            vec![Value::u128(netuid as u128)],
        )
        .await?;
    let n = n_val
        .and_then(|v| crate::utils::decoders::decode_u64(&v).ok())
        .unwrap_or(0);
    let mut map = std::collections::HashMap::new();
    for uid in 0..n {
        if let Some(hk_val) = client
            .storage_with_keys(
                "SubtensorModule",
                "Keys",
                vec![Value::u128(netuid as u128), Value::u128(uid as u128)],
            )
            .await?
        {
            if let Ok(hk) = crate::utils::decoders::decode_account_id32(&hk_val) {
                if let Some(val) = client
                    .storage_with_keys(
                        COMMITMENTS_PALLET,
                        "RevealedCommitments",
                        vec![Value::u128(netuid as u128), Value::from_bytes(hk.encode())],
                    )
                    .await?
                {
                    let entries = decode_revealed_vec(&val);
                    if !entries.is_empty() {
                        map.insert(hk, entries);
                    }
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
        .storage_with_keys(
            COMMITMENTS_PALLET,
            "LastBondsReset",
            vec![
                Value::u128(netuid as u128),
                Value::from_bytes(hotkey.encode()),
            ],
        )
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
        .storage_with_keys(
            SUBTENSOR_MODULE,
            "CRV3WeightCommitsV2",
            vec![Value::u128(netuid as u128)],
        )
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
fn decode_metadata_like(value: &Value) -> String {
    decode_bytes_as_utf8(value)
}

fn decode_bytes_as_utf8(value: &Value) -> String {
    let s = format!("{:?}", value);
    let mut bytes = Vec::new();
    let mut rem = s.as_str();
    while let Some(pos) = rem.find("U128(") {
        let after = &rem[pos + 5..];
        if let Some(end) = after.find(')') {
            if let Ok(n) = after[..end].trim().parse::<u128>() {
                if n <= 255 {
                    bytes.push(n as u8);
                }
            }
            rem = &after[end + 1..];
        } else {
            break;
        }
    }
    String::from_utf8_lossy(&bytes).to_string()
}

#[allow(dead_code)]
fn decode_revealed_tuple(value: &Value) -> Result<(u64, String)> {
    let s = format!("{:?}", value);
    let mut block: u64 = 0;
    if let Some(pos) = s.rfind("U64(") {
        let after = &s[pos + 4..];
        if let Some(end) = after.find(')') {
            block = after[..end].trim().parse::<u64>().unwrap_or(0);
        }
    }
    Ok((block, decode_bytes_as_utf8(value)))
}

fn decode_revealed_vec(value: &Value) -> Vec<(u64, String)> {
    let s = format!("{:?}", value);
    let mut out = Vec::new();
    for part in s.split(")),") {
        if part.contains("U128(") && part.contains("U64(") {
            let msg = decode_bytes_as_utf8_from_str(part);
            let block = extract_last_u64_from_str(part);
            if block.is_some() || !msg.is_empty() {
                out.push((block.unwrap_or(0), msg));
            }
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
            if let Ok(n) = after[..end].trim().parse::<u128>() {
                if n <= 255 {
                    bytes.push(n as u8);
                }
            }
            rem = &after[end + 1..];
        } else {
            break;
        }
    }
    String::from_utf8_lossy(&bytes).to_string()
}

fn extract_last_u64_from_str(s: &str) -> Option<u64> {
    if let Some(pos) = s.rfind("U64(") {
        let after = &s[pos + 4..];
        if let Some(end) = after.find(')') {
            return after[..end].trim().parse::<u64>().ok();
        }
    }
    None
}

fn extract_first_u64_from_str(s: &str) -> Option<u64> {
    if let Some(pos) = s.find("U64(") {
        let after = &s[pos + 4..];
        if let Some(end) = after.find(')') {
            return after[..end].trim().parse::<u64>().ok();
        }
    }
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
            if let Ok(n) = after[..end].trim().parse::<u128>() {
                if n <= 255 {
                    current.push(n as u8);
                }
            }
            if let Some(close) = after[end..].find(")") {
                if close > 0 && !current.is_empty() {
                    groups.push(std::mem::take(&mut current));
                }
            }
            rem = &after[end + 1..];
        } else {
            break;
        }
    }
    if !current.is_empty() {
        groups.push(current);
    }
    groups
}

fn extract_first_account_from_str(s: &str) -> Option<AccountId32> {
    if let Some(pos) = s.find("0x") {
        let hexstr: String = s[pos + 2..]
            .chars()
            .take_while(|c| c.is_ascii_hexdigit())
            .collect();
        if hexstr.len() >= 64 {
            if let Ok(bytes) = hex::decode(&hexstr[0..64]) {
                if bytes.len() == 32 {
                    if let Ok(arr) = <[u8; 32]>::try_from(bytes.as_slice()) {
                        return Some(AccountId32::from(arr));
                    }
                }
            }
        }
    }
    None
}

/// Get weight commitment for a hotkey on a subnet
///
/// Queries the CRV3WeightCommits storage to get the weight commitment
/// information for a specific hotkey on a subnet.
///
/// # Arguments
/// * `client` - The Bittensor client
/// * `netuid` - The subnet ID
/// * `hotkey` - The hotkey account to query
///
/// # Returns
/// The WeightCommitInfo if found, None otherwise
pub async fn get_weight_commitment(
    client: &BittensorClient,
    netuid: u16,
    hotkey: &AccountId32,
) -> BittensorResult<Option<WeightCommitInfo>> {
    let keys = vec![
        Value::u128(netuid as u128),
        Value::from_bytes(hotkey.encode()),
    ];

    match client
        .storage_with_keys(SUBTENSOR_MODULE, "CRV3WeightCommits", keys)
        .await
    {
        Ok(Some(val)) => {
            let info = decode_weight_commit_info(&val);
            Ok(info)
        }
        Ok(None) => Ok(None),
        Err(e) => Err(BittensorError::ChainQuery(ChainQueryError::with_storage(
            format!("Failed to query CRV3WeightCommits: {}", e),
            SUBTENSOR_MODULE,
            "CRV3WeightCommits",
        ))),
    }
}

/// Decode WeightCommitInfo from a Value
fn decode_weight_commit_info(value: &Value) -> Option<WeightCommitInfo> {
    let s = format!("{:?}", value);

    // Extract block (u64), commit_hash (bytes), reveal_round (u64)
    let block = extract_first_u64_from_str(&s).unwrap_or(0);

    // Extract bytes for commit_hash
    let mut commit_hash = Vec::new();
    let mut rem = s.as_str();
    while let Some(pos) = rem.find("U128(") {
        let after = &rem[pos + 5..];
        if let Some(end) = after.find(')') {
            if let Ok(n) = after[..end].trim().parse::<u128>() {
                if n <= 255 {
                    commit_hash.push(n as u8);
                }
            }
            rem = &after[end + 1..];
        } else {
            break;
        }
    }

    let reveal_round = extract_last_u64_from_str(&s).unwrap_or(0);

    if block > 0 || !commit_hash.is_empty() || reveal_round > 0 {
        Some(WeightCommitInfo::new(block, commit_hash, reveal_round))
    } else {
        None
    }
}

/// Get all weight commitments for a subnet
///
/// Queries all weight commitments from all neurons registered on the subnet.
///
/// # Arguments
/// * `client` - The Bittensor client
/// * `netuid` - The subnet ID
///
/// # Returns
/// A vector of (AccountId32, WeightCommitInfo) tuples for all commitments
pub async fn get_all_weight_commitments(
    client: &BittensorClient,
    netuid: u16,
) -> BittensorResult<Vec<(AccountId32, WeightCommitInfo)>> {
    // Get the number of neurons in the subnet
    let n_val = client
        .storage_with_keys(
            SUBTENSOR_MODULE,
            "SubnetworkN",
            vec![Value::u128(netuid as u128)],
        )
        .await
        .map_err(|e| {
            BittensorError::ChainQuery(ChainQueryError::with_storage(
                format!("Failed to query SubnetworkN: {}", e),
                SUBTENSOR_MODULE,
                "SubnetworkN",
            ))
        })?;

    let n = n_val
        .and_then(|v| crate::utils::decoders::decode_u64(&v).ok())
        .unwrap_or(0);

    let mut commitments = Vec::new();

    for uid in 0..n {
        // Get the hotkey for this UID
        let hk_val = client
            .storage_with_keys(
                SUBTENSOR_MODULE,
                "Keys",
                vec![Value::u128(netuid as u128), Value::u128(uid as u128)],
            )
            .await
            .map_err(|e| {
                BittensorError::ChainQuery(ChainQueryError::with_storage(
                    format!("Failed to query Keys: {}", e),
                    SUBTENSOR_MODULE,
                    "Keys",
                ))
            })?;

        if let Some(hk_val) = hk_val {
            if let Ok(hotkey) = crate::utils::decoders::decode_account_id32(&hk_val) {
                // Get the commitment for this hotkey
                if let Ok(Some(commit_info)) = get_weight_commitment(client, netuid, &hotkey).await
                {
                    commitments.push((hotkey, commit_info));
                }
            }
        }
    }

    Ok(commitments)
}

/// Get pending weight commits for a subnet
///
/// Returns weight commits that have been submitted but not yet revealed.
///
/// # Arguments
/// * `client` - The Bittensor client
/// * `netuid` - The subnet ID
///
/// # Returns
/// A vector of (AccountId32, WeightCommitInfo) for pending commits
pub async fn get_pending_weight_commits(
    client: &BittensorClient,
    netuid: u16,
) -> BittensorResult<Vec<(AccountId32, WeightCommitInfo)>> {
    // Query the V2 commits storage which contains pending commits
    let commits_v2 = get_current_weight_commit_info_v2(client, netuid)
        .await
        .map_err(|e| {
            BittensorError::ChainQuery(ChainQueryError::with_storage(
                format!("Failed to query CRV3WeightCommitsV2: {}", e),
                SUBTENSOR_MODULE,
                "CRV3WeightCommitsV2",
            ))
        })?;

    let mut result = Vec::new();
    for (hotkey, block, msg, reveal_round) in commits_v2 {
        let commit_hash = msg.into_bytes();
        result.push((
            hotkey,
            WeightCommitInfo::new(block, commit_hash, reveal_round),
        ));
    }

    Ok(result)
}

/// Check if a hotkey has a pending weight commitment on a subnet
///
/// # Arguments
/// * `client` - The Bittensor client
/// * `netuid` - The subnet ID
/// * `hotkey` - The hotkey to check
///
/// # Returns
/// true if the hotkey has a pending commitment
pub async fn has_pending_commitment(
    client: &BittensorClient,
    netuid: u16,
    hotkey: &AccountId32,
) -> BittensorResult<bool> {
    let commitment = get_weight_commitment(client, netuid, hotkey).await?;
    Ok(commitment.is_some())
}

/// Get the last commit block for a hotkey on a subnet
///
/// # Arguments
/// * `client` - The Bittensor client
/// * `netuid` - The subnet ID
/// * `hotkey` - The hotkey to query
///
/// # Returns
/// The block number of the last commitment, or None if no commitment exists
pub async fn get_last_commit_block(
    client: &BittensorClient,
    netuid: u16,
    hotkey: &AccountId32,
) -> BittensorResult<Option<u64>> {
    match get_weight_commitment(client, netuid, hotkey).await? {
        Some(info) => Ok(Some(info.block)),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weight_commit_info_new() {
        let info = WeightCommitInfo::new(100, vec![1, 2, 3, 4], 5);
        assert_eq!(info.block, 100);
        assert_eq!(info.commit_hash, vec![1, 2, 3, 4]);
        assert_eq!(info.reveal_round, 5);
    }

    #[test]
    fn test_weight_commit_info_commit_hash_hex() {
        let info = WeightCommitInfo::new(100, vec![0xde, 0xad, 0xbe, 0xef], 5);
        assert_eq!(info.commit_hash_hex(), "deadbeef");
    }

    #[test]
    fn test_weight_commit_info_clone() {
        let info = WeightCommitInfo::new(100, vec![1, 2, 3], 5);
        let cloned = info.clone();
        assert_eq!(cloned.block, info.block);
        assert_eq!(cloned.commit_hash, info.commit_hash);
        assert_eq!(cloned.reveal_round, info.reveal_round);
    }

    #[test]
    fn test_weight_commit_info_debug() {
        let info = WeightCommitInfo::new(100, vec![1, 2, 3], 5);
        let debug_str = format!("{:?}", info);
        assert!(debug_str.contains("WeightCommitInfo"));
        assert!(debug_str.contains("100"));
        assert!(debug_str.contains("5"));
    }
}
