use crate::chain::BittensorClient;
use crate::types::ProposalVoteData;
use crate::utils::decoders::{decode_named_composite, decode_u64, decode_vec_account_id32};
use anyhow::Result;
use sp_core::H256;
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
        let fields = decode_named_composite(&val).unwrap_or_default();

        let index = fields
            .get("index")
            .and_then(|v| decode_u64(v).ok())
            .unwrap_or(0);
        let threshold = fields
            .get("threshold")
            .and_then(|v| decode_u64(v).ok())
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
