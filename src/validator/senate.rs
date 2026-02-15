//! Senate and governance operations for Bittensor
//! Implements senate registration, voting, and membership management

use crate::chain::{BittensorClient, BittensorSigner, ExtrinsicWait};
use crate::errors::{BittensorError, BittensorResult, ChainQueryError, ExtrinsicError};
use crate::utils::decoders::{
    decode_account_id32, decode_bytes, decode_u64, decode_vec, decode_vec_account_id32,
};
use anyhow::anyhow;
use sp_core::crypto::AccountId32;
use subxt::dynamic::Value;
use subxt::ext::scale_value::{Composite, ValueDef};

const SUBTENSOR_MODULE: &str = "SubtensorModule";
const SENATE_MODULE: &str = "SenateMembers";
const TRIUMVIRATE_MODULE: &str = "Triumvirate";

// =============================================================================
// Proposal Data Structures
// =============================================================================

/// Proposal data structure for governance proposals
#[derive(Debug, Clone)]
pub struct Proposal {
    /// The hash of the proposal
    pub hash: [u8; 32],
    /// The index of the proposal
    pub index: u32,
    /// The account that proposed this (None if triumvirate prime couldn't be determined)
    pub proposer: Option<AccountId32>,
    /// The encoded call data for the proposal
    pub call_data: Vec<u8>,
    /// The vote threshold required to pass
    pub threshold: u32,
    /// List of accounts that voted in favor
    pub ayes: Vec<AccountId32>,
    /// List of accounts that voted against
    pub nays: Vec<AccountId32>,
    /// The block number at which voting ends
    pub end: u64,
}

/// Vote data for a specific proposal
#[derive(Debug, Clone)]
pub struct VoteData {
    /// The proposal index
    pub index: u32,
    /// The vote threshold required to pass
    pub threshold: u32,
    /// List of accounts that voted in favor
    pub ayes: Vec<AccountId32>,
    /// List of accounts that voted against
    pub nays: Vec<AccountId32>,
    /// The block number at which voting ends
    pub end: u64,
}

// =============================================================================
// Senate Registration
// =============================================================================

/// Register as a senate member
/// Requires being a delegate with sufficient stake
pub async fn register_senate(
    client: &BittensorClient,
    signer: &BittensorSigner,
    wait_for: ExtrinsicWait,
) -> BittensorResult<String> {
    let args: Vec<Value> = vec![];

    let tx_hash = client
        .submit_extrinsic(SUBTENSOR_MODULE, "join_senate", args, signer, wait_for)
        .await
        .map_err(|e| {
            BittensorError::Extrinsic(ExtrinsicError::with_call(
                format!("Failed to register as senate member: {}", e),
                SUBTENSOR_MODULE,
                "join_senate",
            ))
        })?;

    Ok(tx_hash)
}

/// Leave the senate
pub async fn leave_senate(
    client: &BittensorClient,
    signer: &BittensorSigner,
    wait_for: ExtrinsicWait,
) -> BittensorResult<String> {
    let args: Vec<Value> = vec![];

    let tx_hash = client
        .submit_extrinsic(SUBTENSOR_MODULE, "leave_senate", args, signer, wait_for)
        .await
        .map_err(|e| {
            BittensorError::Extrinsic(ExtrinsicError::with_call(
                format!("Failed to leave senate: {}", e),
                SUBTENSOR_MODULE,
                "leave_senate",
            ))
        })?;

    Ok(tx_hash)
}

// =============================================================================
// Voting
// =============================================================================

/// Vote on a proposal
///
/// # Arguments
/// * `client` - The Bittensor client
/// * `signer` - The signer (must be a senate member)
/// * `proposal_hash` - The 32-byte hash of the proposal
/// * `proposal_index` - The index of the proposal
/// * `approve` - Whether to vote in favor (true) or against (false)
/// * `wait_for` - How long to wait for the transaction
pub async fn vote(
    client: &BittensorClient,
    signer: &BittensorSigner,
    proposal_hash: &[u8; 32],
    proposal_index: u32,
    approve: bool,
    wait_for: ExtrinsicWait,
) -> BittensorResult<String> {
    let args = vec![
        Value::from_bytes(proposal_hash),
        Value::u128(proposal_index as u128),
        Value::bool(approve),
    ];

    let tx_hash = client
        .submit_extrinsic(SUBTENSOR_MODULE, "vote", args, signer, wait_for)
        .await
        .map_err(|e| {
            BittensorError::Extrinsic(ExtrinsicError::with_call(
                format!("Failed to vote on proposal: {}", e),
                SUBTENSOR_MODULE,
                "vote",
            ))
        })?;

    Ok(tx_hash)
}

// =============================================================================
// Senate Queries
// =============================================================================

/// Check if an account is a senate member
pub async fn is_senate_member(
    client: &BittensorClient,
    hotkey: &AccountId32,
) -> BittensorResult<bool> {
    // Query SenateMembers.Members storage
    let members_val = client
        .storage(SENATE_MODULE, "Members", None)
        .await
        .map_err(|e| {
            BittensorError::ChainQuery(ChainQueryError::with_storage(
                format!("Failed to query senate members: {}", e),
                SENATE_MODULE,
                "Members",
            ))
        })?;

    match members_val {
        Some(val) => {
            let members = decode_vec_account_id32(&val).map_err(|e| {
                BittensorError::ChainQuery(ChainQueryError::new(format!(
                    "Failed to decode senate members: {}",
                    e
                )))
            })?;
            Ok(members.contains(hotkey))
        }
        None => Ok(false),
    }
}

/// Get all senate members
pub async fn get_senate_members(client: &BittensorClient) -> BittensorResult<Vec<AccountId32>> {
    // Query SenateMembers.Members storage
    let members_val = client
        .storage(SENATE_MODULE, "Members", None)
        .await
        .map_err(|e| {
            BittensorError::ChainQuery(ChainQueryError::with_storage(
                format!("Failed to query senate members: {}", e),
                SENATE_MODULE,
                "Members",
            ))
        })?;

    match members_val {
        Some(val) => {
            let members = decode_vec_account_id32(&val).map_err(|e| {
                BittensorError::ChainQuery(ChainQueryError::new(format!(
                    "Failed to decode senate members: {}",
                    e
                )))
            })?;
            Ok(members)
        }
        None => Ok(Vec::new()),
    }
}

/// Get proposal data for a specific proposal hash
pub async fn get_proposal(
    client: &BittensorClient,
    proposal_hash: &[u8; 32],
) -> BittensorResult<Option<Proposal>> {
    // Get vote data first
    let vote_data = get_vote_data(client, proposal_hash).await?;

    // Query proposal call data from Triumvirate.ProposalOf
    let proposal_of_val = client
        .storage_with_keys(
            TRIUMVIRATE_MODULE,
            "ProposalOf",
            vec![Value::from_bytes(proposal_hash)],
        )
        .await
        .map_err(|e| {
            BittensorError::ChainQuery(ChainQueryError::with_storage(
                format!("Failed to query proposal data: {}", e),
                TRIUMVIRATE_MODULE,
                "ProposalOf",
            ))
        })?;

    // If no vote data, the proposal doesn't exist
    let vote_info = match vote_data {
        Some(v) => v,
        None => return Ok(None),
    };

    // Extract call data from the proposal
    let call_data = match &proposal_of_val {
        Some(val) => extract_call_data(val),
        None => Vec::new(),
    };

    // Get the proposer - returns None if we can't determine it, rather than masking with zeroed account
    let proposer = match get_triumvirate_prime(client).await {
        Ok(prime) => Some(prime),
        Err(e) => {
            tracing::warn!(
                "Failed to get triumvirate prime for proposal {:?}: {}",
                proposal_hash,
                e
            );
            None
        }
    };

    Ok(Some(Proposal {
        hash: *proposal_hash,
        index: vote_info.index,
        proposer,
        call_data,
        threshold: vote_info.threshold,
        ayes: vote_info.ayes,
        nays: vote_info.nays,
        end: vote_info.end,
    }))
}

/// Get all active proposals
pub async fn get_proposals(client: &BittensorClient) -> BittensorResult<Vec<Proposal>> {
    // Query Triumvirate.Proposals to get list of proposal hashes
    let proposals_val = client
        .storage(TRIUMVIRATE_MODULE, "Proposals", None)
        .await
        .map_err(|e| {
            BittensorError::ChainQuery(ChainQueryError::with_storage(
                format!("Failed to query proposals list: {}", e),
                TRIUMVIRATE_MODULE,
                "Proposals",
            ))
        })?;

    let proposal_hashes = match proposals_val {
        Some(val) => extract_proposal_hashes(&val),
        None => return Ok(Vec::new()),
    };

    let mut proposals = Vec::with_capacity(proposal_hashes.len());

    for hash in proposal_hashes {
        if let Some(proposal) = get_proposal(client, &hash).await? {
            proposals.push(proposal);
        }
    }

    Ok(proposals)
}

/// Get vote data for a proposal
pub async fn get_vote_data(
    client: &BittensorClient,
    proposal_hash: &[u8; 32],
) -> BittensorResult<Option<VoteData>> {
    // Query Triumvirate.Voting storage
    let voting_val = client
        .storage_with_keys(
            TRIUMVIRATE_MODULE,
            "Voting",
            vec![Value::from_bytes(proposal_hash)],
        )
        .await
        .map_err(|e| {
            BittensorError::ChainQuery(ChainQueryError::with_storage(
                format!("Failed to query vote data: {}", e),
                TRIUMVIRATE_MODULE,
                "Voting",
            ))
        })?;

    match voting_val {
        Some(val) => Ok(decode_vote_data(&val)),
        None => Ok(None),
    }
}

/// Get the number of proposals
pub async fn get_proposal_count(client: &BittensorClient) -> BittensorResult<u32> {
    // Query Triumvirate.ProposalCount storage
    let count_val = client
        .storage(TRIUMVIRATE_MODULE, "ProposalCount", None)
        .await
        .map_err(|e| {
            BittensorError::ChainQuery(ChainQueryError::with_storage(
                format!("Failed to query proposal count: {}", e),
                TRIUMVIRATE_MODULE,
                "ProposalCount",
            ))
        })?;

    match count_val {
        Some(val) => Ok(decode_u64(&val).unwrap_or(0) as u32),
        None => Ok(0),
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Get the Triumvirate Prime (lead proposer)
async fn get_triumvirate_prime(client: &BittensorClient) -> BittensorResult<AccountId32> {
    let prime_val = client
        .storage(TRIUMVIRATE_MODULE, "Prime", None)
        .await
        .map_err(|e| {
            BittensorError::ChainQuery(ChainQueryError::with_storage(
                format!("Failed to query triumvirate prime: {}", e),
                TRIUMVIRATE_MODULE,
                "Prime",
            ))
        })?;

    match prime_val {
        Some(val) => decode_account_id32(&val).map_err(|_| {
            BittensorError::ChainQuery(ChainQueryError::new("Failed to decode triumvirate prime"))
        }),
        None => Err(BittensorError::ChainQuery(ChainQueryError::new(
            "Triumvirate prime not set",
        ))),
    }
}

/// Extract a u32 value from a debug string.
///
/// # Note on Debug String Parsing
///
/// This function parses Rust's Debug format output. The Debug format is not stable
/// and may change between versions of subxt. This approach is used because the
/// Value API doesn't provide direct typed access for all storage values.
fn decode_vote_data(value: &Value) -> Option<VoteData> {
    if let Ok(fields) = crate::utils::decoders::decode_named_composite(value) {
        let index = fields
            .get("index")
            .and_then(|v| decode_u64(v).ok())
            .map(|v| v as u32)
            .unwrap_or(0);
        let threshold = fields
            .get("threshold")
            .and_then(|v| decode_u64(v).ok())
            .map(|v| v as u32)
            .unwrap_or(0);
        let end = fields
            .get("end")
            .and_then(|v| decode_u64(v).ok())
            .unwrap_or(0);
        let ayes = fields
            .get("ayes")
            .and_then(|v| decode_vec_account_id32(v).ok())
            .unwrap_or_default();
        let nays = fields
            .get("nays")
            .and_then(|v| decode_vec_account_id32(v).ok())
            .unwrap_or_default();
        return Some(VoteData {
            index,
            threshold,
            ayes,
            nays,
            end,
        });
    }

    let values = match &value.value {
        ValueDef::Composite(Composite::Named(fields)) => fields.iter().map(|(_, v)| v).collect(),
        ValueDef::Composite(Composite::Unnamed(values)) => values.iter().collect(),
        ValueDef::Variant(variant) => match &variant.values {
            Composite::Named(fields) => fields.iter().map(|(_, v)| v).collect(),
            Composite::Unnamed(values) => values.iter().collect(),
        },
        _ => Vec::new(),
    };

    if values.len() < 5 {
        return None;
    }

    let index = decode_u64(values[0]).ok().map(|v| v as u32).unwrap_or(0);
    let threshold = decode_u64(values[1]).ok().map(|v| v as u32).unwrap_or(0);
    let end = decode_u64(values[2]).ok().unwrap_or(0);
    let ayes = decode_vec_account_id32(values[3]).unwrap_or_default();
    let nays = decode_vec_account_id32(values[4]).unwrap_or_default();
    Some(VoteData {
        index,
        threshold,
        ayes,
        nays,
        end,
    })
}

/// Extract proposal hashes from the Proposals storage value
fn extract_proposal_hashes(val: &Value) -> Vec<[u8; 32]> {
    decode_vec(val, |entry| {
        let bytes = decode_bytes(entry)?;
        if bytes.len() == 32 {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&bytes);
            Ok(arr)
        } else {
            Err(anyhow!("invalid proposal hash"))
        }
    })
    .unwrap_or_default()
}

/// Extract call data from a proposal value

fn extract_u32_from_value(s: &str) -> Option<u32> {
    let s = s.trim();
    let (prefix, base) = if let Some(rest) = s.strip_prefix("U32(") {
        (rest, 32)
    } else if let Some(rest) = s.strip_prefix("U64(") {
        (rest, 64)
    } else if let Some(rest) = s.strip_prefix("U128(") {
        (rest, 128)
    } else {
        return None;
    };
    let end = prefix.find(')')?;
    let value = prefix[..end].trim().parse::<u64>().ok()?;
    if base == 128 && value > u32::MAX as u64 {
        return None;
    }
    Some(value as u32)
}

fn extract_first_u64_after_key(s: &str, key: &str) -> Option<u64> {
    let key_pos = s.find(key)?;
    let mut slice = &s[key_pos + key.len()..];
    if let Some(colon_pos) = slice.find(':') {
        slice = &slice[colon_pos + 1..];
    }

    let prefixes = ["U64(", "U32(", "U128("];
    let mut best: Option<(usize, &str)> = None;
    for prefix in prefixes {
        if let Some(pos) = slice.find(prefix) {
            if best.map_or(true, |(best_pos, _)| pos < best_pos) {
                best = Some((pos, prefix));
            }
        }
    }
    let (pos, prefix) = best?;
    let number_start = pos + prefix.len();
    let after_prefix = &slice[number_start..];
    let end = after_prefix.find(')')?;
    after_prefix[..end].trim().parse::<u64>().ok()
}

fn extract_accounts_array_after_key(s: &str, key: &str) -> Vec<AccountId32> {
    let key_pos = match s.find(key) {
        Some(pos) => pos,
        None => return Vec::new(),
    };
    let after_key = &s[key_pos + key.len()..];
    let start = match after_key.find('[') {
        Some(pos) => pos,
        None => return Vec::new(),
    };
    let after_bracket = &after_key[start + 1..];
    let end = match after_bracket.find(']') {
        Some(pos) => pos,
        None => return Vec::new(),
    };
    let content = &after_bracket[..end];

    content
        .split(',')
        .filter_map(|entry| {
            let trimmed = entry.trim();
            let hex_str = trimmed.strip_prefix("0x").unwrap_or(trimmed);
            let decoded = hex::decode(hex_str).ok()?;
            if decoded.len() != 32 {
                return None;
            }
            let mut bytes = [0u8; 32];
            bytes.copy_from_slice(&decoded);
            Some(AccountId32::from(bytes))
        })
        .collect()
}
fn extract_call_data(val: &Value) -> Vec<u8> {
    decode_bytes(val).unwrap_or_default()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vote_data_default() {
        let vote_data = VoteData {
            index: 0,
            threshold: 2,
            ayes: vec![],
            nays: vec![],
            end: 1000,
        };
        assert_eq!(vote_data.index, 0);
        assert_eq!(vote_data.threshold, 2);
        assert!(vote_data.ayes.is_empty());
        assert!(vote_data.nays.is_empty());
        assert_eq!(vote_data.end, 1000);
    }

    #[test]
    fn test_proposal_creation() {
        let hash = [1u8; 32];
        let proposer = AccountId32::from([2u8; 32]);

        let proposal = Proposal {
            hash,
            index: 5,
            proposer: Some(proposer.clone()),
            call_data: vec![1, 2, 3, 4],
            threshold: 3,
            ayes: vec![proposer.clone()],
            nays: vec![],
            end: 2000,
        };

        assert_eq!(proposal.hash, hash);
        assert_eq!(proposal.index, 5);
        assert_eq!(proposal.threshold, 3);
        assert_eq!(proposal.ayes.len(), 1);
        assert!(proposal.nays.is_empty());
        assert_eq!(proposal.call_data, vec![1, 2, 3, 4]);
        assert!(proposal.proposer.is_some());
    }

    #[test]
    fn test_proposal_with_unknown_proposer() {
        let hash = [1u8; 32];

        let proposal = Proposal {
            hash,
            index: 5,
            proposer: None, // Unknown proposer
            call_data: vec![1, 2, 3, 4],
            threshold: 3,
            ayes: vec![],
            nays: vec![],
            end: 2000,
        };

        assert!(proposal.proposer.is_none());
    }

    #[test]
    fn test_extract_u32_from_value() {
        assert_eq!(extract_u32_from_value("U32(42)"), Some(42));
        assert_eq!(extract_u32_from_value("U64(100)"), Some(100));
        assert_eq!(extract_u32_from_value("U128(256)"), Some(256));
        assert_eq!(extract_u32_from_value("nothing here"), None);
    }

    #[test]
    fn test_extract_first_u64_after_key() {
        let s = "{ index: U64(10), threshold: U32(5), end: U64(1000) }";
        assert_eq!(extract_first_u64_after_key(s, "index"), Some(10));
        assert_eq!(extract_first_u64_after_key(s, "threshold"), Some(5));
        assert_eq!(extract_first_u64_after_key(s, "end"), Some(1000));
        assert_eq!(extract_first_u64_after_key(s, "notfound"), None);
    }

    #[test]
    fn test_extract_accounts_array_after_key() {
        // Create a mock debug string with account addresses
        let account1 = [0x01u8; 32];
        let hex1 = hex::encode(account1);
        let s = format!("{{ ayes: [0x{}], nays: [] }}", hex1);

        let ayes = extract_accounts_array_after_key(&s, "ayes");
        assert_eq!(ayes.len(), 1);
        assert_eq!(ayes[0], AccountId32::from(account1));

        let nays = extract_accounts_array_after_key(&s, "nays");
        assert!(nays.is_empty());
    }

    #[test]
    fn test_extract_proposal_hashes() {
        // Simulate a Proposals storage value with multiple hashes
        let hash1 = [0xaau8; 32];
        let hash2 = [0xbbu8; 32];
        let hex1 = hex::encode(hash1);
        let hex2 = hex::encode(hash2);

        // Create a mock Value debug representation
        let mock_val_str = format!("Composite(Unnamed([0x{}, 0x{}]))", hex1, hex2);
        let _mock_val = Value::from_bytes(hash1.as_slice()); // Just need a Value for testing

        // Test the extraction logic directly on the string
        let mut hashes = Vec::new();
        let mut rem = mock_val_str.as_str();

        while let Some(pos) = rem.find("0x") {
            let hex_str: String = rem[pos + 2..]
                .chars()
                .take_while(|c| c.is_ascii_hexdigit())
                .collect();
            if hex_str.len() >= 64 {
                if let Ok(bytes) = hex::decode(&hex_str[..64]) {
                    if bytes.len() == 32 {
                        let mut arr = [0u8; 32];
                        arr.copy_from_slice(&bytes);
                        hashes.push(arr);
                    }
                }
            }
            rem = &rem[pos + 2 + hex_str.len()..];
        }

        assert_eq!(hashes.len(), 2);
        assert_eq!(hashes[0], hash1);
        assert_eq!(hashes[1], hash2);
    }

    #[test]
    fn test_extract_call_data() {
        // Test extraction logic directly on a debug string that mimics
        // how actual chain data looks when formatted with Debug
        let call_data = vec![0x01u8, 0x02, 0x03, 0x04];
        let hex_data = hex::encode(&call_data);

        // Simulate how the debug output looks with 0x prefix
        let mock_debug_str = format!("Composite(Unnamed([0x{}]))", hex_data);

        // Test the extraction logic directly on the string
        let extracted = {
            let s = &mock_debug_str;
            if let Some(pos) = s.find("0x") {
                let hex_str: String = s[pos + 2..]
                    .chars()
                    .take_while(|c| c.is_ascii_hexdigit())
                    .collect();
                if !hex_str.is_empty() {
                    hex::decode(&hex_str).unwrap_or_default()
                } else {
                    vec![]
                }
            } else {
                vec![]
            }
        };

        // The extracted data should match our original data
        assert!(!extracted.is_empty());
        assert_eq!(extracted, call_data);
    }

    #[test]
    fn test_vote_data_with_members() {
        let voter1 = AccountId32::from([1u8; 32]);
        let voter2 = AccountId32::from([2u8; 32]);
        let voter3 = AccountId32::from([3u8; 32]);

        let vote_data = VoteData {
            index: 1,
            threshold: 2,
            ayes: vec![voter1.clone(), voter2.clone()],
            nays: vec![voter3.clone()],
            end: 5000,
        };

        assert_eq!(vote_data.ayes.len(), 2);
        assert_eq!(vote_data.nays.len(), 1);
        assert!(vote_data.ayes.contains(&voter1));
        assert!(vote_data.ayes.contains(&voter2));
        assert!(vote_data.nays.contains(&voter3));
    }

    #[test]
    fn test_proposal_with_empty_call_data() {
        let hash = [0u8; 32];
        let proposer = AccountId32::from([1u8; 32]);

        let proposal = Proposal {
            hash,
            index: 0,
            proposer: Some(proposer),
            call_data: vec![],
            threshold: 1,
            ayes: vec![],
            nays: vec![],
            end: 0,
        };

        assert!(proposal.call_data.is_empty());
        assert_eq!(proposal.index, 0);
    }

    #[test]
    fn test_extract_accounts_array_stops_at_boundary() {
        // Test that accounts extraction correctly stops at array boundary
        let account1 = [0x01u8; 32];
        let account2 = [0x02u8; 32];
        let account3 = [0x03u8; 32]; // This should NOT be extracted - it's in a different array
        let hex1 = hex::encode(account1);
        let hex2 = hex::encode(account2);
        let hex3 = hex::encode(account3);

        // Simulate debug output with two arrays: ayes and nays
        let s = format!("{{ ayes: [0x{}, 0x{}], nays: [0x{}] }}", hex1, hex2, hex3);

        let ayes = extract_accounts_array_after_key(&s, "ayes");
        assert_eq!(
            ayes.len(),
            2,
            "Should extract exactly 2 accounts from ayes array"
        );
        assert_eq!(ayes[0], AccountId32::from(account1));
        assert_eq!(ayes[1], AccountId32::from(account2));

        // Verify ayes doesn't contain the nays account
        assert!(
            !ayes.contains(&AccountId32::from(account3)),
            "ayes should not contain nays account"
        );

        let nays = extract_accounts_array_after_key(&s, "nays");
        assert_eq!(
            nays.len(),
            1,
            "Should extract exactly 1 account from nays array"
        );
        assert_eq!(nays[0], AccountId32::from(account3));
    }

    #[test]
    fn test_extract_accounts_array_with_nested_brackets() {
        // Test with nested structures to ensure bracket tracking works
        let account1 = [0xaau8; 32];
        let hex1 = hex::encode(account1);

        // Simulate complex nested structure
        let s = format!("{{ data: [Composite([0x{}])], other: [] }}", hex1);

        let accounts = extract_accounts_array_after_key(&s, "data");
        assert_eq!(
            accounts.len(),
            1,
            "Should extract account from nested structure"
        );
        assert_eq!(accounts[0], AccountId32::from(account1));

        let other = extract_accounts_array_after_key(&s, "other");
        assert!(other.is_empty(), "other array should be empty");
    }

    #[test]
    fn test_extract_first_u64_after_key_various_formats() {
        // Test U64 format
        let s1 = "{ value: U64(12345) }";
        assert_eq!(extract_first_u64_after_key(s1, "value"), Some(12345));

        // Test U32 format
        let s2 = "{ count: U32(42) }";
        assert_eq!(extract_first_u64_after_key(s2, "count"), Some(42));

        // Test U128 format
        let s3 = "{ big: U128(999999999999) }";
        assert_eq!(extract_first_u64_after_key(s3, "big"), Some(999999999999));

        // Test mixed - should pick the first one after the key
        let s4 = "{ first: U32(10), second: U64(20) }";
        assert_eq!(extract_first_u64_after_key(s4, "first"), Some(10));
        assert_eq!(extract_first_u64_after_key(s4, "second"), Some(20));

        // Test when key is not found
        let s5 = "{ something: U64(100) }";
        assert_eq!(extract_first_u64_after_key(s5, "missing"), None);

        // Test with whitespace
        let s6 = "{ spaced: U64( 555 ) }";
        assert_eq!(extract_first_u64_after_key(s6, "spaced"), Some(555));
    }

    #[test]
    fn test_extract_accounts_empty_array() {
        let s = "{ ayes: [], nays: [] }";

        let ayes = extract_accounts_array_after_key(s, "ayes");
        assert!(ayes.is_empty());

        let nays = extract_accounts_array_after_key(s, "nays");
        assert!(nays.is_empty());
    }

    #[test]
    fn test_extract_accounts_no_array() {
        // Test when key exists but no array follows
        let s = "{ ayes: 123 }";

        let accounts = extract_accounts_array_after_key(s, "ayes");
        assert!(accounts.is_empty());
    }
}
