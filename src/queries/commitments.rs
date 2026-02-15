use crate::chain::BittensorClient;
use crate::errors::{BittensorError, BittensorResult, ChainQueryError};
use crate::utils::decoders::vec::decode_vec;
use crate::utils::decoders::{decode_bytes, decode_u64};
use anyhow::{anyhow, Result};
use parity_scale_codec::Encode;
use sp_core::crypto::AccountId32;
use subxt::dynamic::Value;
use subxt::ext::scale_value::{Composite, ValueDef};

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
    let key = vec![Value::u128(netuid as u128)];
    if let Some(val) = client
        .storage_with_keys(SUBTENSOR_MODULE, "CRV3WeightCommitsV2", key)
        .await?
    {
        return Ok(decode_vec_of_bytes(&val));
    }
    Ok(Vec::new())
}

/// Get timelocked weight commits
pub async fn get_timelocked_weight_commits(
    client: &BittensorClient,
    netuid: u16,
) -> Result<Vec<Vec<u8>>> {
    let storage_index = crate::crv4::get_mechid_storage_index(netuid, 0);
    let key = vec![Value::u128(storage_index as u128)];
    if let Some(val) = client
        .storage_with_keys(SUBTENSOR_MODULE, "TimelockedWeightCommits", key)
        .await?
    {
        return Ok(decode_vec_of_bytes(&val));
    }
    Ok(Vec::new())
}

/// Get all commitments for a subnet from Commitments.CommitmentOf
pub async fn get_all_commitments(
    client: &BittensorClient,
    netuid: u16,
) -> Result<std::collections::HashMap<AccountId32, String>> {
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
        return Ok(decode_u64(&val).ok());
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
        return Ok(decode_commit_info_v2(&val));
    }
    Ok(Vec::new())
}

fn decode_metadata_like(value: &Value) -> String {
    decode_bytes_as_utf8(value)
}

fn decode_bytes_as_utf8(value: &Value) -> String {
    crate::utils::decoders::decode_string(value).unwrap_or_default()
}

fn decode_revealed_vec(value: &Value) -> Vec<(u64, String)> {
    decode_vec(value, |entry| {
        decode_revealed_entry(entry).ok_or_else(|| anyhow!("invalid"))
    })
    .unwrap_or_default()
}

fn decode_revealed_entry(value: &Value) -> Option<(u64, String)> {
    let fields = match &value.value {
        ValueDef::Composite(Composite::Named(fields)) => fields.iter().map(|(_, v)| v).collect(),
        ValueDef::Composite(Composite::Unnamed(values)) => values.iter().collect(),
        ValueDef::Variant(variant) => match &variant.values {
            Composite::Named(fields) => fields.iter().map(|(_, v)| v).collect(),
            Composite::Unnamed(values) => values.iter().collect(),
        },
        _ => Vec::new(),
    };
    if fields.len() < 2 {
        return None;
    }
    let block = decode_u64(fields[0]).ok()?;
    let message = decode_bytes_as_utf8(fields[1]);
    Some((block, message))
}

fn decode_commit_info_v2(value: &Value) -> Vec<(AccountId32, u64, String, u64)> {
    decode_vec(value, |entry| {
        decode_commit_info_v2_entry(entry).ok_or_else(|| anyhow!("invalid"))
    })
    .unwrap_or_default()
}

fn decode_commit_info_v2_entry(value: &Value) -> Option<(AccountId32, u64, String, u64)> {
    let fields = match &value.value {
        ValueDef::Composite(Composite::Named(fields)) => fields.iter().map(|(_, v)| v).collect(),
        ValueDef::Composite(Composite::Unnamed(values)) => values.iter().collect(),
        ValueDef::Variant(variant) => match &variant.values {
            Composite::Named(fields) => fields.iter().map(|(_, v)| v).collect(),
            Composite::Unnamed(values) => values.iter().collect(),
        },
        _ => Vec::new(),
    };
    if fields.len() < 4 {
        return None;
    }
    let hotkey = crate::utils::decoders::decode_account_id32(fields[0]).ok()?;
    let block = decode_u64(fields[1]).ok()?;
    let message = decode_bytes_as_utf8(fields[2]);
    let reveal_round = decode_u64(fields[3]).ok()?;
    Some((hotkey, block, message, reveal_round))
}

fn decode_timelocked_weight_commit_info(value: &Value) -> Vec<(AccountId32, WeightCommitInfo)> {
    decode_vec(value, |entry| {
        decode_timelocked_commit_entry(entry).ok_or_else(|| anyhow!("invalid"))
    })
    .unwrap_or_default()
}

fn decode_timelocked_commit_entry(value: &Value) -> Option<(AccountId32, WeightCommitInfo)> {
    let fields = match &value.value {
        ValueDef::Composite(Composite::Named(fields)) => fields.iter().map(|(_, v)| v).collect(),
        ValueDef::Composite(Composite::Unnamed(values)) => values.iter().collect(),
        ValueDef::Variant(variant) => match &variant.values {
            Composite::Named(fields) => fields.iter().map(|(_, v)| v).collect(),
            Composite::Unnamed(values) => values.iter().collect(),
        },
        _ => Vec::new(),
    };

    if fields.len() < 4 {
        return None;
    }

    let hotkey = crate::utils::decoders::decode_account_id32(fields[0]).ok()?;
    let block = decode_u64(fields[1]).ok()?;
    let commit = decode_bytes(fields[2]).ok()?;
    let reveal_round = decode_u64(fields[3]).ok()?;
    Some((hotkey, WeightCommitInfo::new(block, commit, reveal_round)))
}

fn decode_vec_of_bytes(value: &Value) -> Vec<Vec<u8>> {
    decode_vec(value, |entry| {
        decode_bytes(entry).map_err(|e| anyhow!("{e}"))
    })
    .unwrap_or_default()
}

/// Get weight commitment for a hotkey on a subnet
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
    let fields = match &value.value {
        ValueDef::Composite(Composite::Named(fields)) => fields.iter().map(|(_, v)| v).collect(),
        ValueDef::Composite(Composite::Unnamed(values)) => values.iter().collect(),
        ValueDef::Variant(variant) => match &variant.values {
            Composite::Named(fields) => fields.iter().map(|(_, v)| v).collect(),
            Composite::Unnamed(values) => values.iter().collect(),
        },
        _ => Vec::new(),
    };
    if fields.len() < 3 {
        return None;
    }
    let block = decode_u64(fields[0]).ok()?;
    let commit_hash = decode_bytes(fields[1]).ok()?;
    let reveal_round = decode_u64(fields[2]).ok()?;
    Some(WeightCommitInfo::new(block, commit_hash, reveal_round))
}

/// Get all weight commitments for a subnet
pub async fn get_all_weight_commitments(
    client: &BittensorClient,
    netuid: u16,
) -> BittensorResult<Vec<(AccountId32, WeightCommitInfo)>> {
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
pub async fn get_pending_weight_commits(
    client: &BittensorClient,
    netuid: u16,
) -> BittensorResult<Vec<(AccountId32, WeightCommitInfo)>> {
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

/// Get CRv4 timelocked commits for a subnet mechanism.
pub async fn get_timelocked_weight_commits_v4(
    client: &BittensorClient,
    netuid: u16,
    mechanism_id: u8,
) -> Result<Vec<(AccountId32, WeightCommitInfo)>> {
    let netuid_index = crate::crv4::get_mechid_storage_index(netuid, mechanism_id);
    let key = vec![Value::u128(netuid_index as u128)];
    if let Some(val) = client
        .storage_with_keys(SUBTENSOR_MODULE, "TimelockedWeightCommits", key)
        .await?
    {
        let commits = decode_timelocked_weight_commit_info(&val);
        return Ok(commits);
    }
    Ok(Vec::new())
}

/// Check if a hotkey has a pending weight commitment on a subnet
pub async fn has_pending_commitment(
    client: &BittensorClient,
    netuid: u16,
    hotkey: &AccountId32,
) -> BittensorResult<bool> {
    let commitment = get_weight_commitment(client, netuid, hotkey).await?;
    Ok(commitment.is_some())
}

/// Get the last commit block for a hotkey on a subnet
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
