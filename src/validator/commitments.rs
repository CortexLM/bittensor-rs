use crate::chain::{BittensorClient, BittensorSigner, ExtrinsicWait};
use anyhow::Result;
use subxt::dynamic::Value;

const COMMITMENTS_MODULE: &str = "Commitments";

/// Set a commitment on-chain for a given subnet.
///
/// Commitments pallet dispatch: `set_commitment(netuid, data)`
///
/// # Arguments
/// * `client` — The Bittensor RPC client.
/// * `signer` — The signing keypair.
/// * `netuid` — The subnet ID.
/// * `data` — The commitment data bytes.
/// * `wait_for` — How long to wait for on-chain inclusion.
pub async fn set_commitment(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    data: &[u8],
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let fields_value = Value::named_composite([("info", build_commitment_info(data))]);

    let args = vec![Value::from(netuid), fields_value];

    client
        .submit_extrinsic(COMMITMENTS_MODULE, "set_commitment", args, signer, wait_for)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to set commitment: {}", e))
}

fn build_commitment_info(data: &[u8]) -> Value {
    Value::named_composite([(
        "fields",
        Value::unnamed_composite(vec![Value::named_variant(
            "Raw0",
            [("0", Value::from_bytes(data))],
        )]),
    )])
}
