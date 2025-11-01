use crate::chain::BittensorClient;
use crate::types::ProposalVoteData;
use anyhow::Result;
use sp_core::{crypto::AccountId32, H256};
use subxt::dynamic::Value;

const TRI_PALLET: &str = "Triumvirate";

pub async fn get_vote_data(
    client: &BittensorClient,
    proposal_hash: H256,
) -> Result<Option<ProposalVoteData>> {
    if let Some(val) = client
        .storage_with_keys(
            TRI_PALLET,
            "Voting",
            vec![Value::from_bytes(proposal_hash.as_bytes())],
        )
        .await?
    {
        let s = format!("{:?}", val);
        let index = extract_first_u64_after_key(&s, "index").unwrap_or(0);
        let threshold = extract_first_u64_after_key(&s, "threshold").unwrap_or(0);
        let end = extract_first_u64_after_key(&s, "end").unwrap_or(0);
        let ayes = extract_accounts_array_after_key(&s, "ayes");
        let nays = extract_accounts_array_after_key(&s, "nays");
        return Ok(Some(ProposalVoteData {
            index,
            threshold,
            ayes,
            nays,
            end,
        }));
    }
    Ok(None)
}

fn extract_first_u64_after_key(s: &str, key: &str) -> Option<u64> {
    if let Some(kp) = s.find(key) {
        let subs = &s[kp..];
        if let Some(p) = subs.find("U64(") {
            let aft = &subs[p + 4..];
            if let Some(end) = aft.find(')') {
                return aft[..end].trim().parse::<u64>().ok();
            }
        }
        if let Some(p) = subs.find("U32(") {
            let aft = &subs[p + 4..];
            if let Some(end) = aft.find(')') {
                return aft[..end].trim().parse::<u32>().ok().map(|v| v as u64);
            }
        }
    }
    None
}

fn extract_accounts_array_after_key(s: &str, key: &str) -> Vec<AccountId32> {
    let mut accounts = Vec::new();
    if let Some(kp) = s.find(key) {
        let subs = &s[kp..];
        let mut rem = subs;
        while let Some(pos) = rem.find("0x") {
            let hexstr: String = rem[pos + 2..]
                .chars()
                .take_while(|c| c.is_ascii_hexdigit())
                .collect();
            if hexstr.len() >= 64 {
                if let Ok(bytes) = hex::decode(&hexstr[0..64]) {
                    if bytes.len() == 32 {
                        if let Ok(arr) = <[u8; 32]>::try_from(bytes.as_slice()) {
                            accounts.push(AccountId32::from(arr));
                        }
                    }
                }
            }
            rem = &rem[pos + 2 + hexstr.len()..];
            // Stop when we likely left the array section
            if rem.starts_with("]") {
                break;
            }
        }
    }
    accounts
}
