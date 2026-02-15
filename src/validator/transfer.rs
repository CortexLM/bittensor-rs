use crate::chain::{BittensorClient, BittensorSigner, ExtrinsicWait};
use anyhow::Result;
use parity_scale_codec::Encode;
use sp_core::crypto::AccountId32;
use subxt::dynamic::Value;

const BALANCES_MODULE: &str = "Balances";
const SUBTENSOR_MODULE: &str = "SubtensorModule";

/// Transfer TAO to another account
pub async fn transfer(
    client: &BittensorClient,
    signer: &BittensorSigner,
    dest: &AccountId32,
    amount: u128,
    keep_alive: bool,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    if amount == 0 {
        return Err(anyhow::anyhow!("Transfer amount must be greater than zero"));
    }

    let dest_bytes = dest.encode();
    let dest_value = Value::from_bytes(&dest_bytes);

    // Choose function based on keep_alive
    let function = if keep_alive {
        "transfer_keep_alive"
    } else {
        "transfer"
    };

    let args = vec![dest_value, Value::u128(amount)];

    client
        .submit_extrinsic(BALANCES_MODULE, function, args, signer, wait_for)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to transfer: {}", e))
}

/// Transfer stake from one coldkey to another, optionally across subnets
/// Subtensor expects: (destination_coldkey, hotkey, origin_netuid, destination_netuid, alpha_amount)
#[allow(clippy::too_many_arguments)]
pub async fn transfer_stake(
    client: &BittensorClient,
    signer: &BittensorSigner,
    destination_coldkey: &AccountId32,
    hotkey: &AccountId32,
    origin_netuid: u16,
    destination_netuid: u16,
    amount: u128,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    if amount == 0 {
        return Err(anyhow::anyhow!(
            "Stake transfer amount must be greater than zero"
        ));
    }

    let args = vec![
        Value::from_bytes(destination_coldkey.encode()),
        Value::from_bytes(hotkey.encode()),
        Value::u128(origin_netuid as u128),
        Value::u128(destination_netuid as u128),
        Value::u128(amount),
    ];

    client
        .submit_extrinsic(SUBTENSOR_MODULE, "transfer_stake", args, signer, wait_for)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to transfer stake: {}", e))
}
