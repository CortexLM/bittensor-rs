use bittensor_core::config::SubtensorConfig;
use subxt::OnlineClient;

use crate::generated::subtensor;

use super::{Result, TxSuccess, submit_and_watch};

/// Set weight assignments for a subnet (direct, non-commit-reveal).
pub async fn set_weights(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    netuid: u16,
    dests: Vec<u16>,
    weights: Vec<u16>,
    version_key: u64,
) -> Result<TxSuccess> {
    let call = subtensor::tx().subtensor_module().set_weights(netuid, dests, weights, version_key);
    submit_and_watch(client, call, signer).await
}

/// Commit a hash of weight assignments for later reveal (commit-reveal v1).
pub async fn commit_weights(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    netuid: u16,
    commit_hash: subxt::utils::H256,
) -> Result<TxSuccess> {
    let call = subtensor::tx().subtensor_module().commit_weights(netuid, commit_hash);
    submit_and_watch(client, call, signer).await
}

/// Reveal previously committed weight assignments (commit-reveal v1).
pub async fn reveal_weights(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    netuid: u16,
    uids: Vec<u16>,
    values: Vec<u16>,
    salt: Vec<u16>,
    version_key: u64,
) -> Result<TxSuccess> {
    let call =
        subtensor::tx().subtensor_module().reveal_weights(netuid, uids, values, salt, version_key);
    submit_and_watch(client, call, signer).await
}

/// Commit time-locked weight assignments with a scheduled reveal round (commit-reveal v2).
pub async fn commit_timelocked_weights(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    netuid: u16,
    commit: subtensor::runtime_types::bounded_collections::bounded_vec::BoundedVec<u8>,
    reveal_round: u64,
    commit_reveal_version: u16,
) -> Result<TxSuccess> {
    let call = subtensor::tx().subtensor_module().commit_timelocked_weights(
        netuid,
        commit,
        reveal_round,
        commit_reveal_version,
    );
    submit_and_watch(client, call, signer).await
}

#[cfg(test)]
mod tests {

    use crate::generated::subtensor;

    #[test]
    fn set_weights_call_construction() {
        let _call = subtensor::tx().subtensor_module().set_weights(
            1u16,
            vec![0, 1, 2],
            vec![100, 200, 300],
            1u64,
        );
    }

    #[test]
    fn commit_weights_call_construction() {
        let _call =
            subtensor::tx().subtensor_module().commit_weights(1u16, subxt::utils::H256::zero());
    }

    #[test]
    fn reveal_weights_call_construction() {
        let _call = subtensor::tx().subtensor_module().reveal_weights(
            1u16,
            vec![0, 1],
            vec![100, 200],
            vec![42, 43],
            1u64,
        );
    }
}
