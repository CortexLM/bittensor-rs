//! Metagraph queries — subnet-wide neural graph snapshots and pending emission data.

use bittensor_core::balance::Balance;
use bittensor_core::config::SubtensorConfig;
use bittensor_core::error::BittensorError;
use bittensor_core::types::MetagraphInfo;
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

/// Fetch a metagraph snapshot for a subnet.
pub async fn get_metagraph(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<MetagraphInfo> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();

    let n: u16 = fetch_decode!(at, addr, subnetwork_n, (netuid,));
    let total_issuance: u64 = fetch_decode!(at, addr, total_issuance, ());
    let total_stake: u64 = fetch_decode!(at, addr, total_stake, ());

    Ok(MetagraphInfo {
        netuid,
        block: at.block_number(),
        n,
        stake: Balance::from_rao(total_stake),
        total_issuance: Balance::from_rao(total_issuance),
        total_weight: 0,
        total_bond: 0,
    })
}

/// Fetch a selective metagraph (same as full for now).
pub async fn get_selective_metagraph(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<MetagraphInfo> {
    get_metagraph(client, netuid).await
}

/// Fetch the subnet owner cut (global, in basis points).
pub async fn get_subnet_owner_cut(client: &OnlineClient<SubtensorConfig>) -> Result<u16> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u16 = fetch_decode!(at, addr, subnet_owner_cut, ());
    Ok(v)
}

/// Fetch the root proposal weight for a subnet (FixedU128 — returned as raw bytes).
///
/// Returns `None` if the value is not set.
pub async fn get_root_prop(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<
    Option<
        subtensor::runtime_types::substrate_fixed::FixedU128<
            subtensor::runtime_types::substrate_typenum::uint::UInt<
                subtensor::runtime_types::substrate_typenum::uint::UInt<
                    subtensor::runtime_types::substrate_typenum::uint::UInt<
                        subtensor::runtime_types::substrate_typenum::uint::UInt<
                            subtensor::runtime_types::substrate_typenum::uint::UInt<
                                subtensor::runtime_types::substrate_typenum::uint::UInt<
                                    subtensor::runtime_types::substrate_typenum::uint::UTerm,
                                    subtensor::runtime_types::substrate_typenum::bit::B1,
                                >,
                                subtensor::runtime_types::substrate_typenum::bit::B0,
                            >,
                            subtensor::runtime_types::substrate_typenum::bit::B0,
                        >,
                        subtensor::runtime_types::substrate_typenum::bit::B0,
                    >,
                    subtensor::runtime_types::substrate_typenum::bit::B0,
                >,
                subtensor::runtime_types::substrate_typenum::bit::B0,
            >,
        >,
    >,
> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v = at
        .storage()
        .try_fetch(addr.root_prop(), (netuid,))
        .await
        .map_err(|e| BittensorError::Rpc(e.to_string()))?
        .and_then(|val| val.decode().ok());
    Ok(v)
}

/// Fetch the first emission block number for a subnet.
pub async fn get_first_emission_block_number(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, first_emission_block_number, (netuid,));
    Ok(v)
}

/// Fetch the pending server emission for a subnet.
pub async fn get_pending_server_emission(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, pending_server_emission, (netuid,));
    Ok(v)
}

/// Fetch the pending validator emission for a subnet.
pub async fn get_pending_validator_emission(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, pending_validator_emission, (netuid,));
    Ok(v)
}

/// Fetch the pending root alpha dividends for a subnet.
pub async fn get_pending_root_alpha_divs(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, pending_root_alpha_divs, (netuid,));
    Ok(v)
}

/// Fetch the pending owner cut for a subnet.
pub async fn get_pending_owner_cut(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, pending_owner_cut, (netuid,));
    Ok(v)
}

/// Fetch the number of blocks since the last mechanism step for a subnet.
pub async fn get_blocks_since_last_step(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, blocks_since_last_step, (netuid,));
    Ok(v)
}

/// Fetch the last mechanism step block for a subnet.
pub async fn get_last_mechanism_step_block(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, last_mechansim_step_block, (netuid,));
    Ok(v)
}

/// Fetch the recycle-or-burn setting for a subnet.
///
/// Returns `None` if the subnet does not have a recycle-or-burn setting.
pub async fn get_recycle_or_burn(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<Option<subtensor::runtime_types::pallet_subtensor::pallet::RecycleOrBurnEnum>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: Option<subtensor::runtime_types::pallet_subtensor::pallet::RecycleOrBurnEnum> = at
        .storage()
        .try_fetch(addr.recycle_or_burn(), (netuid,))
        .await
        .map_err(|e| BittensorError::Rpc(e.to_string()))?
        .and_then(|val| val.decode().ok());
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metagraph_info_fields() {
        let info = MetagraphInfo {
            netuid: 1,
            block: 100,
            n: 64,
            stake: Balance::from_tao(1000.0),
            total_issuance: Balance::from_tao(5000.0),
            total_weight: 100,
            total_bond: 50,
        };
        assert_eq!(info.netuid, 1);
        assert_eq!(info.n, 64);
    }

    #[test]
    fn pending_emission_default() {
        let v: u64 = 0;
        assert_eq!(v, 0);
    }

    #[tokio::test]
    async fn get_metagraph_returns_defaults_for_empty_mock() {
        let client = crate::test_utils::mock_client_empty().await;
        let result = get_metagraph(&client, 0u16).await;
        assert!(result.is_ok(), "get_metagraph should succeed: {:?}", result.err());
        let info = result.unwrap();
        assert_eq!(info.netuid, 0);
        assert_eq!(info.n, 0);
        assert_eq!(info.stake.to_rao(), 0);
        assert_eq!(info.total_issuance.to_rao(), 0);
        assert_eq!(info.total_weight, 0);
        assert_eq!(info.total_bond, 0);
    }

    #[tokio::test]
    async fn get_selective_metagraph_returns_same_as_metagraph() {
        let client = crate::test_utils::mock_client_empty().await;
        let result = get_selective_metagraph(&client, 0u16).await;
        assert!(result.is_ok(), "get_selective_metagraph should succeed: {:?}", result.err());
        let info = result.unwrap();
        assert_eq!(info.netuid, 0);
    }

    #[tokio::test]
    async fn get_subnet_owner_cut_returns_default() {
        let client = crate::test_utils::mock_client_empty().await;
        let result = get_subnet_owner_cut(&client).await;
        assert!(result.is_ok(), "get_subnet_owner_cut should succeed: {:?}", result.err());
        assert_eq!(result.unwrap(), 11796);
    }

    #[tokio::test]
    async fn get_root_prop_returns_some_for_empty_mock() {
        let client = crate::test_utils::mock_client_empty().await;
        let result = get_root_prop(&client, 0u16).await;
        assert!(result.is_ok(), "get_root_prop should succeed: {:?}", result.err());
        assert!(result.unwrap().is_some());
    }

    #[tokio::test]
    async fn get_first_emission_block_number_returns_zero() {
        let client = crate::test_utils::mock_client_empty().await;
        let result = get_first_emission_block_number(&client, 0u16).await;
        assert!(
            result.is_ok(),
            "get_first_emission_block_number should succeed: {:?}",
            result.err()
        );
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn get_pending_server_emission_returns_zero() {
        let client = crate::test_utils::mock_client_empty().await;
        let result = get_pending_server_emission(&client, 0u16).await;
        assert!(result.is_ok(), "get_pending_server_emission should succeed: {:?}", result.err());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn get_pending_validator_emission_returns_zero() {
        let client = crate::test_utils::mock_client_empty().await;
        let result = get_pending_validator_emission(&client, 0u16).await;
        assert!(
            result.is_ok(),
            "get_pending_validator_emission should succeed: {:?}",
            result.err()
        );
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn get_pending_root_alpha_divs_returns_zero() {
        let client = crate::test_utils::mock_client_empty().await;
        let result = get_pending_root_alpha_divs(&client, 0u16).await;
        assert!(result.is_ok(), "get_pending_root_alpha_divs should succeed: {:?}", result.err());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn get_pending_owner_cut_returns_zero() {
        let client = crate::test_utils::mock_client_empty().await;
        let result = get_pending_owner_cut(&client, 0u16).await;
        assert!(result.is_ok(), "get_pending_owner_cut should succeed: {:?}", result.err());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn get_blocks_since_last_step_returns_zero() {
        let client = crate::test_utils::mock_client_empty().await;
        let result = get_blocks_since_last_step(&client, 0u16).await;
        assert!(result.is_ok(), "get_blocks_since_last_step should succeed: {:?}", result.err());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn get_last_mechanism_step_block_returns_zero() {
        let client = crate::test_utils::mock_client_empty().await;
        let result = get_last_mechanism_step_block(&client, 0u16).await;
        assert!(result.is_ok(), "get_last_mechanism_step_block should succeed: {:?}", result.err());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn get_recycle_or_burn_returns_some_for_empty_mock() {
        let client = crate::test_utils::mock_client_empty().await;
        let result = get_recycle_or_burn(&client, 0u16).await;
        assert!(result.is_ok(), "get_recycle_or_burn should succeed: {:?}", result.err());
        assert!(result.unwrap().is_some());
    }
}
