use crate::chain::{BittensorClient, BittensorSigner, ExtrinsicWait};
use anyhow::Result;
use parity_scale_codec::Encode;
use sp_core::crypto::AccountId32;
use subxt::dynamic::Value;

const SUBTENSOR_MODULE: &str = "SubtensorModule";

/// Register a new subnet (network) on the Bittensor chain.
///
/// Subtensor pallet dispatch: `register_network(hotkey, mechid)`
/// Source: pallets/subtensor/src/subnets/registration.rs
///
/// # Arguments
/// * `client` — The Bittensor RPC client.
/// * `signer` — The signing keypair (coldkey that will own the subnet).
/// * `hotkey` — The hotkey to associate with the new subnet.
/// * `mechid` — The mechanism ID for the subnet (e.g., 0 for default).
/// * `wait_for` — How long to wait for on-chain inclusion.
pub async fn register_network(
    client: &BittensorClient,
    signer: &BittensorSigner,
    hotkey: &AccountId32,
    mechid: u16,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args = vec![Value::from_bytes(hotkey.encode()), Value::from(mechid)];

    client
        .submit_extrinsic(SUBTENSOR_MODULE, "register_network", args, signer, wait_for)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to register network: {}", e))
}

/// Dissolve (remove) a subnet from the chain.
///
/// Subtensor pallet dispatch: `dissolve_network(netuid)`
/// Only the subnet owner can dissolve their subnet.
///
/// # Arguments
/// * `client` — The Bittensor RPC client.
/// * `signer` — The signing keypair (must be the subnet owner).
/// * `netuid` — The subnet ID to dissolve.
/// * `wait_for` — How long to wait for on-chain inclusion.
pub async fn dissolve_network(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args = vec![Value::from(netuid)];

    client
        .submit_extrinsic(SUBTENSOR_MODULE, "dissolve_network", args, signer, wait_for)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to dissolve network: {}", e))
}

/// Set metadata for a subnet.
///
/// Subtensor pallet dispatch: `set_subnet_metadata(netuid, metadata)`
///
/// # Arguments
/// * `client` — The Bittensor RPC client.
/// * `signer` — The signing keypair (must be the subnet owner).
/// * `netuid` — The subnet ID.
/// * `metadata` — Metadata bytes to set for the subnet.
/// * `wait_for` — How long to wait for on-chain inclusion.
pub async fn set_subnet_metadata(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    metadata: &[u8],
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args = vec![Value::from(netuid), Value::from_bytes(metadata)];

    client
        .submit_extrinsic(
            SUBTENSOR_MODULE,
            "set_subnet_metadata",
            args,
            signer,
            wait_for,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to set subnet metadata: {}", e))
}
