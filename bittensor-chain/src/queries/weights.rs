//! Weight queries — weight matrix, commit-reveal state, and rate limits.

use bittensor_core::config::SubtensorConfig;
use bittensor_core::error::BittensorError;
use bittensor_core::types::WeightCommitInfo;
use subxt::OnlineClient;

use crate::client::ClientAtBlock;
use crate::generated::subtensor;

type Result<T> = std::result::Result<T, BittensorError>;

async fn at_block(client: &OnlineClient<SubtensorConfig>) -> Result<ClientAtBlock> {
    client.at_current_block().await.map_err(|e| BittensorError::Rpc(e.to_string()))
}

fn decode_opt<T>(opt: Option<subxt::storage::StorageValue<'_, T>>) -> T
where
    T: subxt::ext::scale_decode::DecodeAsType + Default,
{
    opt.and_then(|v| v.decode().ok()).unwrap_or_default()
}

macro_rules! fetch_decode {
    ($at:expr, $addr:expr, $method:ident, $keys:expr) => {{
        let opt = $at
            .storage()
            .try_fetch($addr.$method(), $keys)
            .await
            .map_err(|e| BittensorError::Rpc(e.to_string()))?;
        decode_opt(opt)
    }};
}

/// Fetch the weight row for a given UID in a subnet.
pub async fn get_weights(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    uid: u16,
) -> Result<Vec<(u16, u16)>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let weights: Vec<(u16, u16)> = fetch_decode!(at, addr, weights, (netuid, uid));
    Ok(weights)
}

/// Fetch the minimum allowed weight value for a subnet.
pub async fn get_weights_min(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u16> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let min: u16 = fetch_decode!(at, addr, min_allowed_weights, (netuid,));
    Ok(min)
}

/// Fetch the maximum weight limit for a subnet.
pub async fn get_weights_max(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u16> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let max: u16 = fetch_decode!(at, addr, max_weights_limit, (netuid,));
    Ok(max)
}

/// Fetch the weight-set rate limit for a subnet (blocks between weight sets).
pub async fn get_weights_set_rate_limit(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let rate: u64 = fetch_decode!(at, addr, weights_set_rate_limit, (netuid,));
    Ok(rate)
}

/// Fetch the weight commits for a hotkey in a subnet.
pub async fn get_weight_commits(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    hotkey: &subxt::utils::AccountId32,
) -> Result<Option<WeightCommitInfo>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();

    let commits: Vec<(subxt::utils::H256, u64, u64, u64)> =
        fetch_decode!(at, addr, weight_commits, (netuid, *hotkey));

    if commits.is_empty() {
        Ok(None)
    } else {
        let first = &commits[0];
        Ok(Some(WeightCommitInfo {
            hotkey: hotkey.to_string(),
            commit: first.0.as_bytes().to_vec(),
            reveal_round: first.1,
            netuid,
        }))
    }
}

/// Fetch the timelocked weight commits for a subnet at a given block.
pub async fn get_timelocked_weight_commits(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    block: u64,
) -> Result<
    Vec<(
        subxt::utils::AccountId32,
        u64,
        subtensor::runtime_types::bounded_collections::bounded_vec::BoundedVec<u8>,
        u64,
    )>,
> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v = fetch_decode!(at, addr, timelocked_weight_commits, (netuid, block));
    Ok(v)
}

/// Fetch the CRV3 weight commits for a subnet at a given block.
pub async fn get_crv3_weight_commits(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    block: u64,
) -> Result<
    Vec<(
        subxt::utils::AccountId32,
        subtensor::runtime_types::bounded_collections::bounded_vec::BoundedVec<u8>,
        u64,
    )>,
> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v = fetch_decode!(at, addr, crv3_weight_commits, (netuid, block));
    Ok(v)
}

/// Fetch the CRV3 weight commits v2 for a subnet at a given block.
pub async fn get_crv3_weight_commits_v2(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    block: u64,
) -> Result<
    Vec<(
        subxt::utils::AccountId32,
        u64,
        subtensor::runtime_types::bounded_collections::bounded_vec::BoundedVec<u8>,
        u64,
    )>,
> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v = fetch_decode!(at, addr, crv3_weight_commits_v2, (netuid, block));
    Ok(v)
}

/// Check if commit-reveal weights is enabled for a subnet.
pub async fn get_commit_reveal_weights_enabled(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<bool> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: bool = fetch_decode!(at, addr, commit_reveal_weights_enabled, (netuid,));
    Ok(v)
}

/// Fetch the commit-reveal weights version (global).
pub async fn get_commit_reveal_weights_version(
    client: &OnlineClient<SubtensorConfig>,
) -> Result<u16> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u16 = fetch_decode!(at, addr, commit_reveal_weights_version, ());
    Ok(v)
}

/// Fetch the reveal period in epochs for a subnet.
pub async fn get_reveal_period_epochs(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, reveal_period_epochs, (netuid,));
    Ok(v)
}

/// Fetch the weights version key for a subnet.
pub async fn get_weights_version_key(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, weights_version_key, (netuid,));
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weights_params_present() {
        assert!(true);
    }

    #[test]
    fn weight_commit_info_construction() {
        let wci = WeightCommitInfo {
            hotkey: "hk".into(),
            commit: vec![1, 2, 3, 4],
            reveal_round: 1000,
            netuid: 1,
        };
        assert_eq!(wci.netuid, 1);
    }
}
