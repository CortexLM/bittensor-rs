use crate::chain::{BittensorClient, BittensorSigner, ExtrinsicWait};
use anyhow::Result;
use parity_scale_codec::Encode;
use sp_core::crypto::AccountId32;
use subxt::dynamic::Value;

const SUBTENSOR_MODULE: &str = "SubtensorModule";
const REGISTER_FUNCTION: &str = "register";
const BURNED_REGISTER_FUNCTION: &str = "burned_register";

/// Register a neuron on a subnet
pub async fn register(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args = vec![Value::u128(netuid as u128)];

    let tx_hash = client
        .submit_extrinsic(SUBTENSOR_MODULE, REGISTER_FUNCTION, args, signer, wait_for)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to register: {}", e))?;

    Ok(tx_hash)
}

/// Register using burned TAO (registration cost)
pub async fn burned_register(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args = vec![Value::u128(netuid as u128)];

    let tx_hash = client
        .submit_extrinsic(
            SUBTENSOR_MODULE,
            BURNED_REGISTER_FUNCTION,
            args,
            signer,
            wait_for,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to burned register: {}", e))?;

    Ok(tx_hash)
}

/// Check if a hotkey is registered on a subnet
pub async fn is_registered(
    client: &BittensorClient,
    netuid: u16,
    hotkey: &AccountId32,
) -> Result<bool> {
    // Query the Uids storage to check if hotkey is registered
    let keys = vec![
        Value::u128(netuid as u128),
        Value::from_bytes(hotkey.encode()),
    ];

    let uid_data = client
        .storage_with_keys(SUBTENSOR_MODULE, "Uids", keys)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to query registration: {}", e))?;

    match uid_data {
        Some(data) => {
            // Decode UID from Value - if we get a UID back, the hotkey is registered
            use crate::utils::decoders::decode_u64;
            match decode_u64(&data) {
                Ok(_uid) => Ok(true),
                Err(_) => Ok(false),
            }
        }
        None => Ok(false),
    }
}
