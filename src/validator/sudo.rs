use crate::chain::{BittensorClient, BittensorSigner, ExtrinsicWait};
use crate::utils::balance_newtypes::Rao;
use anyhow::Result;
use parity_scale_codec::Encode;
use sp_core::crypto::AccountId32;
use subxt::dynamic::Value;

const SUDO_MODULE: &str = "Sudo";
const ADMIN_UTILS: &str = "AdminUtils";

/// Force-set the balance of an account (sudo only).
///
/// AdminUtils pallet dispatch: `sudo_set_balance(who, amount)`
/// This requires the signer to have sudo privileges on the chain.
///
/// # Arguments
/// * `client` — The Bittensor RPC client.
/// * `signer` — The signing keypair (must have sudo privileges).
/// * `who` — The account whose balance to set.
/// * `amount` — The balance to set **in RAO** (1 TAO = 1e9 RAO).
/// * `wait_for` — How long to wait for on-chain inclusion.
pub async fn force_set_balance(
    client: &BittensorClient,
    signer: &BittensorSigner,
    who: &AccountId32,
    amount: Rao,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args = vec![
        Value::from_bytes(who.encode()),
        Value::u128(amount.as_u128()),
    ];

    client
        .submit_extrinsic(ADMIN_UTILS, "sudo_set_balance", args, signer, wait_for)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to force set balance: {}", e))
}

/// Execute an arbitrary call via sudo.
///
/// Sudo pallet dispatch: `sudo(call)`
/// The `call` parameter should be SCALE-encoded call data.
///
/// # Arguments
/// * `client` — The Bittensor RPC client.
/// * `signer` — The signing keypair (must have sudo privileges).
/// * `call` — SCALE-encoded call data to execute with sudo privileges.
/// * `wait_for` — How long to wait for on-chain inclusion.
pub async fn sudo(
    client: &BittensorClient,
    signer: &BittensorSigner,
    call: Vec<u8>,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args = vec![Value::from_bytes(&call)];

    client
        .submit_extrinsic(SUDO_MODULE, "sudo", args, signer, wait_for)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to execute sudo call: {}", e))
}
