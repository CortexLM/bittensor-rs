//! Root extrinsics — set root network weights.

use bittensor_core::config::SubtensorConfig;
use subxt::OnlineClient;

use crate::generated::subtensor;

use super::{Result, TxSuccess, submit_and_watch};

/// Register a hotkey on the root network (subnet 0).
pub async fn root_register(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    hotkey: subxt::utils::AccountId32,
) -> Result<TxSuccess> {
    let call = subtensor::tx().subtensor_module().root_register(hotkey);
    submit_and_watch(client, call, signer).await
}

/// Claim root network membership for the given subnets.
pub async fn claim_root(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    subnets: Vec<u16>,
) -> Result<TxSuccess> {
    let call = subtensor::tx().subtensor_module().claim_root(subnets);
    submit_and_watch(client, call, signer).await
}

#[cfg(test)]
mod tests {

    use crate::generated::subtensor;

    #[test]
    fn root_register_call_construction() {
        let hotkey = subxt::utils::AccountId32::from([1u8; 32]);
        let _call = subtensor::tx().subtensor_module().root_register(hotkey);
    }

    #[test]
    fn claim_root_call_construction() {
        let _call = subtensor::tx().subtensor_module().claim_root(vec![1u16, 2u16]);
    }
}
