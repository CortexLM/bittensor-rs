//! Take extrinsics — set delegate take rate.

use bittensor_core::config::SubtensorConfig;
use subxt::OnlineClient;

use crate::generated::subtensor;

use super::{Result, TxSuccess, submit_and_watch};

/// Decrease the delegate take rate (basis points) for a hotkey.
pub async fn decrease_take(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    hotkey: subxt::utils::AccountId32,
    take: u16,
) -> Result<TxSuccess> {
    let call = subtensor::tx().subtensor_module().decrease_take(hotkey, take);
    submit_and_watch(client, call, signer).await
}

/// Increase the delegate take rate (basis points) for a hotkey.
pub async fn increase_take(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    hotkey: subxt::utils::AccountId32,
    take: u16,
) -> Result<TxSuccess> {
    let call = subtensor::tx().subtensor_module().increase_take(hotkey, take);
    submit_and_watch(client, call, signer).await
}

#[cfg(test)]
mod tests {

    use crate::generated::subtensor;

    #[test]
    fn decrease_take_call_construction() {
        let hotkey = subxt::utils::AccountId32::from([1u8; 32]);
        let _call = subtensor::tx().subtensor_module().decrease_take(hotkey, 10u16);
    }

    #[test]
    fn increase_take_call_construction() {
        let hotkey = subxt::utils::AccountId32::from([1u8; 32]);
        let _call = subtensor::tx().subtensor_module().increase_take(hotkey, 18u16);
    }
}
