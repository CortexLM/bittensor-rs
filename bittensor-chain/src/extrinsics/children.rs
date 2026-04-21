//! Children extrinsics — set children and childkey take rates.

use bittensor_core::config::SubtensorConfig;
use subxt::OnlineClient;

use crate::generated::subtensor;

use super::{Result, TxSuccess, submit_and_watch};

/// Set the child hotkeys and their proportional weights for a parent hotkey in a subnet.
pub async fn set_children(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    hotkey: subxt::utils::AccountId32,
    netuid: u16,
    children: Vec<(u64, subxt::utils::AccountId32)>,
) -> Result<TxSuccess> {
    let call = subtensor::tx().subtensor_module().set_children(hotkey, netuid, children);
    submit_and_watch(client, call, signer).await
}

/// Set the take rate (basis points) for a childkey in a subnet.
pub async fn set_childkey_take(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    hotkey: subxt::utils::AccountId32,
    netuid: u16,
    take: u16,
) -> Result<TxSuccess> {
    let call = subtensor::tx().subtensor_module().set_childkey_take(hotkey, netuid, take);
    submit_and_watch(client, call, signer).await
}

#[cfg(test)]
mod tests {

    use crate::generated::subtensor;

    #[test]
    fn set_children_call_construction() {
        let hotkey = subxt::utils::AccountId32::from([1u8; 32]);
        let _call = subtensor::tx().subtensor_module().set_children(
            hotkey,
            1u16,
            vec![(100u64, subxt::utils::AccountId32::from([2u8; 32]))],
        );
    }

    #[test]
    fn set_childkey_take_call_construction() {
        let hotkey = subxt::utils::AccountId32::from([1u8; 32]);
        let _call = subtensor::tx().subtensor_module().set_childkey_take(hotkey, 1u16, 18u16);
    }
}
