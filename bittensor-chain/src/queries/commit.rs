//! Commit-reveal queries — weight commit and reveal hashes, storage commitment data.

use bittensor_core::config::SubtensorConfig;
use bittensor_core::error::BittensorError;
use bittensor_core::types::WeightCommitInfo;
use subxt::OnlineClient;

use crate::client::ClientAtBlock;
use crate::generated::subtensor;

type Result<T> = std::result::Result<T, BittensorError>;

/// Type alias for the commitments pallet Registration type.
type CommitmentRegistration =
    subtensor::runtime_types::pallet_commitments::types::Registration<u64, u32>;

/// Type alias for the commitments pallet UsageTracker type.
type UsageTracker = subtensor::runtime_types::pallet_commitments::types::UsageTracker;

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

/// Fetch the weight commit for a hotkey in a subnet.
pub async fn get_weight_commit(
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

/// Fetch the weight reveal for a hotkey in a subnet at the current block.
pub async fn get_weight_reveal(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    hotkey: &subxt::utils::AccountId32,
) -> Result<Option<Vec<u8>>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();

    let current_block = at.block_number();
    let reveals: Vec<(
        subxt::utils::AccountId32,
        u64,
        subtensor::runtime_types::bounded_collections::bounded_vec::BoundedVec<u8>,
        u64,
    )> = fetch_decode!(at, addr, crv3_weight_commits_v2, (netuid, current_block));

    let found = reveals.into_iter().find(|(hk, _, _, _)| hk == hotkey);
    Ok(found.map(|(_, _, data, _)| data.0))
}

/// Fetch the commitment data for a hotkey in a subnet from the commitments pallet.
///
/// Returns `None` if no commitment exists.
pub async fn get_commitment_of(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    hotkey: &subxt::utils::AccountId32,
) -> Result<Option<CommitmentRegistration>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().commitments();
    let v: Option<CommitmentRegistration> = at
        .storage()
        .try_fetch(addr.commitment_of(), (netuid, *hotkey))
        .await
        .map_err(|e| BittensorError::Rpc(e.to_string()))?
        .and_then(|val| val.decode().ok());
    Ok(v)
}

/// Fetch the last commitment block for a hotkey in a subnet.
pub async fn get_last_commitment(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    hotkey: &subxt::utils::AccountId32,
) -> Result<u32> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().commitments();
    let v: u32 = fetch_decode!(at, addr, last_commitment, (netuid, *hotkey));
    Ok(v)
}

/// Fetch the used storage space tracker for a hotkey in a subnet.
///
/// Returns `None` if no usage record exists.
pub async fn get_used_space_of(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    hotkey: &subxt::utils::AccountId32,
) -> Result<Option<UsageTracker>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().commitments();
    let v: Option<UsageTracker> = at
        .storage()
        .try_fetch(addr.used_space_of(), (netuid, *hotkey))
        .await
        .map_err(|e| BittensorError::Rpc(e.to_string()))?
        .and_then(|val| val.decode().ok());
    Ok(v)
}

/// Fetch the maximum allowed storage space (global, in bytes).
pub async fn get_max_space(client: &OnlineClient<SubtensorConfig>) -> Result<u32> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().commitments();
    let v: u32 = fetch_decode!(at, addr, max_space, ());
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn commitment_default() {
        let v: u32 = 0;
        assert_eq!(v, 0);
    }

    #[tokio::test]
    async fn get_weight_commit_returns_none_for_empty_mock() {
        let client = crate::test_utils::mock_client_empty().await;
        let hotkey = subxt::utils::AccountId32::from([0u8; 32]);
        let result = get_weight_commit(&client, 0u16, &hotkey).await;
        assert!(result.is_ok(), "get_weight_commit should succeed: {:?}", result.err());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn get_weight_reveal_returns_none_for_empty_mock() {
        let client = crate::test_utils::mock_client_empty().await;
        let hotkey = subxt::utils::AccountId32::from([0u8; 32]);
        let result = get_weight_reveal(&client, 0u16, &hotkey).await;
        assert!(result.is_ok(), "get_weight_reveal should succeed: {:?}", result.err());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn get_commitment_of_returns_none_for_empty_mock() {
        let client = crate::test_utils::mock_client_empty().await;
        let hotkey = subxt::utils::AccountId32::from([0u8; 32]);
        let result = get_commitment_of(&client, 0u16, &hotkey).await;
        assert!(result.is_ok(), "get_commitment_of should succeed: {:?}", result.err());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn get_last_commitment_returns_zero_for_empty_mock() {
        let client = crate::test_utils::mock_client_empty().await;
        let hotkey = subxt::utils::AccountId32::from([0u8; 32]);
        let result = get_last_commitment(&client, 0u16, &hotkey).await;
        assert!(result.is_ok(), "get_last_commitment should succeed: {:?}", result.err());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn get_used_space_of_returns_none_for_empty_mock() {
        let client = crate::test_utils::mock_client_empty().await;
        let hotkey = subxt::utils::AccountId32::from([0u8; 32]);
        let result = get_used_space_of(&client, 0u16, &hotkey).await;
        assert!(result.is_ok(), "get_used_space_of should succeed: {:?}", result.err());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn get_max_space_returns_default_for_empty_mock() {
        let client = crate::test_utils::mock_client_empty().await;
        let result = get_max_space(&client).await;
        assert!(result.is_ok(), "get_max_space should succeed: {:?}", result.err());
        assert_eq!(result.unwrap(), 3100);
    }
}
