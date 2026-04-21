//! Coldkey swap extrinsics — transfer coldkey ownership.

use bittensor_core::config::SubtensorConfig;
use subxt::OnlineClient;

use crate::generated::subtensor;

use super::{Result, TxSuccess, submit_and_watch};

/// Announce the intent to swap the coldkey ownership to a new coldkey (identified by hash).
pub async fn announce_coldkey_swap(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    new_coldkey_hash: subxt::utils::H256,
) -> Result<TxSuccess> {
    let call = subtensor::tx().subtensor_module().announce_coldkey_swap(new_coldkey_hash);
    submit_and_watch(client, call, signer).await
}

/// Dispute a previously announced coldkey swap.
pub async fn dispute_coldkey_swap(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
) -> Result<TxSuccess> {
    let call = subtensor::tx().subtensor_module().dispute_coldkey_swap();
    submit_and_watch(client, call, signer).await
}

/// Finalize a coldkey swap using the previously announced new coldkey account.
pub async fn swap_coldkey_announced(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    new_coldkey: subxt::utils::AccountId32,
) -> Result<TxSuccess> {
    let call = subtensor::tx().subtensor_module().swap_coldkey_announced(new_coldkey);
    submit_and_watch(client, call, signer).await
}

#[cfg(test)]
mod tests {

    use crate::generated::subtensor;

    #[test]
    fn announce_coldkey_swap_call_construction() {
        let _call =
            subtensor::tx().subtensor_module().announce_coldkey_swap(subxt::utils::H256::zero());
    }

    #[test]
    fn dispute_coldkey_swap_call_construction() {
        let _call = subtensor::tx().subtensor_module().dispute_coldkey_swap();
    }

    #[test]
    fn swap_coldkey_announced_call_construction() {
        let new_coldkey = subxt::utils::AccountId32::from([5u8; 32]);
        let _call = subtensor::tx().subtensor_module().swap_coldkey_announced(new_coldkey);
    }
}
