use crate::chain::BittensorClient;
use crate::types::ChainIdentity;
use anyhow::Result;
use subxt::dynamic::Value;
use parity_scale_codec::Encode;

const SUBTENSOR_MODULE: &str = "SubtensorModule";

pub async fn query_identity(client: &BittensorClient, coldkey: &sp_core::crypto::AccountId32) -> Result<Option<ChainIdentity>> {
    if let Some(val) = client
        .storage_with_keys(SUBTENSOR_MODULE, "IdentitiesV2", vec![Value::from_bytes(&coldkey.encode())])
        .await?
    {
        let fields = decode_identity_map(&val);
        return Ok(Some(ChainIdentity { fields }));
    }
    Ok(None)
}

fn decode_identity_map(value: &Value) -> std::collections::HashMap<String, String> {
    // Use proper SCALE decoding for identity data
    crate::utils::scale_decode::decode_identity_map(value).unwrap_or_default()
}
