use parity_scale_codec::Encode;
use crate::chain::{BittensorClient, BittensorSigner, ExtrinsicWait};
use anyhow::Result;
use sp_core::crypto::AccountId32;
use subxt::dynamic::Value;

const BALANCES_MODULE: &str = "Balances";

/// Transfer TAO to another account
pub async fn transfer(
    client: &BittensorClient,
    signer: &BittensorSigner,
    dest: &AccountId32,
    amount: u128,
    keep_alive: bool,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let dest_bytes = dest.encode();
    let dest_value = Value::from_bytes(&dest_bytes);
    
    // Choose function based on keep_alive
    let function = if keep_alive { "transfer_keep_alive" } else { "transfer" };
    
    let args = vec![
        dest_value,
        Value::u128(amount),
    ];
    
    client
        .submit_extrinsic(BALANCES_MODULE, function, args, signer, wait_for)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to transfer: {}", e))
}

/// Transfer stake from one hotkey to another
pub async fn transfer_stake(
    client: &BittensorClient,
    signer: &BittensorSigner,
    from_hotkey: &AccountId32,
    to_hotkey: &AccountId32,
    amount: u128,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    use crate::validator::staking::move_stake;
    
    // transfer_stake is equivalent to move_stake
    move_stake(client, signer, from_hotkey, to_hotkey, amount, wait_for).await
}

