use crate::chain::{BittensorClient, BittensorSigner, ExtrinsicWait};
use anyhow::Result;
use parity_scale_codec::Encode;
use sp_core::crypto::AccountId32;
use subxt::dynamic::Value;

const SUBTENSOR_MODULE: &str = "SubtensorModule";

/// Schedule a coldkey swap to a new coldkey.
///
/// Subtensor pallet dispatch: `schedule_coldkey_swap(new_coldkey)`
/// Matches Python SDK's `subtensor.schedule_coldkey_swap()`.
///
/// # Arguments
/// * `client` — The Bittensor RPC client.
/// * `signer` — The signing keypair (current coldkey).
/// * `new_coldkey` — The new coldkey to swap to.
/// * `wait_for` — How long to wait for on-chain inclusion.
pub async fn schedule_coldkey_swap(
    client: &BittensorClient,
    signer: &BittensorSigner,
    new_coldkey: &AccountId32,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args = vec![Value::from_bytes(new_coldkey.encode())];

    client
        .submit_extrinsic(
            SUBTENSOR_MODULE,
            "schedule_coldkey_swap",
            args,
            signer,
            wait_for,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to schedule coldkey swap: {}", e))
}

/// Schedule a hotkey swap from an old hotkey to a new hotkey.
///
/// Subtensor pallet dispatch: `schedule_hotkey_swap(old_hotkey, new_hotkey)`
///
/// # Arguments
/// * `client` — The Bittensor RPC client.
/// * `signer` — The signing keypair (coldkey that owns the hotkey).
/// * `old_hotkey` — The current hotkey to swap from.
/// * `new_hotkey` — The new hotkey to swap to.
/// * `wait_for` — How long to wait for on-chain inclusion.
pub async fn schedule_hotkey_swap(
    client: &BittensorClient,
    signer: &BittensorSigner,
    old_hotkey: &AccountId32,
    new_hotkey: &AccountId32,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args = vec![
        Value::from_bytes(old_hotkey.encode()),
        Value::from_bytes(new_hotkey.encode()),
    ];

    client
        .submit_extrinsic(
            SUBTENSOR_MODULE,
            "schedule_hotkey_swap",
            args,
            signer,
            wait_for,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to schedule hotkey swap: {}", e))
}

/// Schedule the dissolution of a network.
///
/// Subtensor pallet dispatch: `schedule_dissolve_network(netuid)`
///
/// # Arguments
/// * `client` — The Bittensor RPC client.
/// * `signer` — The signing keypair (must be the subnet owner).
/// * `netuid` — The subnet ID to schedule for dissolution.
/// * `wait_for` — How long to wait for on-chain inclusion.
pub async fn schedule_dissolve_network(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args = vec![Value::from(netuid)];

    client
        .submit_extrinsic(
            SUBTENSOR_MODULE,
            "schedule_dissolve_network",
            args,
            signer,
            wait_for,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to schedule dissolve network: {}", e))
}
