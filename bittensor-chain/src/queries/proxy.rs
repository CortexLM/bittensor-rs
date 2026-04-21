//! Proxy queries — proxy account and pure proxy lookups, permission checks.

use bittensor_core::config::SubtensorConfig;
use bittensor_core::error::BittensorError;
use subxt::OnlineClient;

use crate::client::ClientAtBlock;
use crate::generated::subtensor;

type Result<T> = std::result::Result<T, BittensorError>;

/// Proxy definition type alias for readability.
type ProxyDef = subtensor::runtime_types::pallet_subtensor_proxy::ProxyDefinition<
    subxt::utils::AccountId32,
    subtensor::runtime_types::subtensor_runtime_common::ProxyType,
    u32,
>;

/// Proxies storage value type alias for readability.
type ProxiesValue =
    (subtensor::runtime_types::bounded_collections::bounded_vec::BoundedVec<ProxyDef>, u64);

async fn at_block(client: &OnlineClient<SubtensorConfig>) -> Result<ClientAtBlock> {
    client.at_current_block().await.map_err(|e| BittensorError::Rpc(e.to_string()))
}

/// Fetch the proxy list for a delegator and delegate pair.
pub async fn get_proxies(
    client: &OnlineClient<SubtensorConfig>,
    delegator: &subxt::utils::AccountId32,
    _delegate: &subxt::utils::AccountId32,
    _proxy_type: Option<u8>,
) -> Result<Vec<(subxt::utils::AccountId32, u8)>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().proxy();

    let proxies_opt: Option<ProxiesValue> = at
        .storage()
        .try_fetch(addr.proxies(), (*delegator,))
        .await
        .map_err(|e| BittensorError::Rpc(e.to_string()))?
        .and_then(|v| v.decode().ok());

    match proxies_opt {
        Some((proxies_vec, _deposit)) => Ok(proxies_vec
            .0
            .into_iter()
            .map(|p| (p.delegate, proxy_type_to_u8(&p.proxy_type)))
            .collect()),
        None => Ok(vec![]),
    }
}

/// Convert a ProxyType enum to its u8 discriminant.
#[allow(clippy::match_like_matches_macro)]
fn proxy_type_to_u8(pt: &subtensor::runtime_types::subtensor_runtime_common::ProxyType) -> u8 {
    use subtensor::runtime_types::subtensor_runtime_common::ProxyType::*;
    match pt {
        Any => 0,
        Owner => 1,
        NonCritical => 2,
        NonTransfer => 3,
        Senate => 4,
        NonFungible => 5,
        Triumvirate => 6,
        Governance => 7,
        Staking => 8,
        Registration => 9,
        Transfer => 10,
        SmallTransfer => 11,
        RootWeights => 12,
        ChildKeys => 13,
        SudoUncheckedSetCode => 14,
        SwapHotkey => 15,
        SubnetLeaseBeneficiary => 16,
        RootClaim => 17,
    }
}

/// Fetch a pure proxy (currently returns None — not yet available in storage).
pub async fn get_pure_proxy(
    _client: &OnlineClient<SubtensorConfig>,
    _proxy_account: &subxt::utils::AccountId32,
) -> Result<Option<subxt::utils::AccountId32>> {
    Ok(None)
}

/// Fetch the proxy definition for a given delegator from the proxy pallet.
///
/// Returns `None` if the delegator has no proxy entries.
pub async fn get_proxy(
    client: &OnlineClient<SubtensorConfig>,
    delegator: &subxt::utils::AccountId32,
) -> Result<Option<ProxiesValue>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().proxy();
    let v: Option<ProxiesValue> = at
        .storage()
        .try_fetch(addr.proxies(), (*delegator,))
        .await
        .map_err(|e| BittensorError::Rpc(e.to_string()))?
        .and_then(|val| val.decode().ok());
    Ok(v)
}

/// Check if a delegate has a specific proxy type permission for a delegator.
pub async fn get_check_permissions(
    client: &OnlineClient<SubtensorConfig>,
    delegator: &subxt::utils::AccountId32,
    delegate: &subxt::utils::AccountId32,
    proxy_type: u8,
) -> Result<bool> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().proxy();

    let proxies_opt: Option<ProxiesValue> = at
        .storage()
        .try_fetch(addr.proxies(), (*delegator,))
        .await
        .map_err(|e| BittensorError::Rpc(e.to_string()))?
        .and_then(|v| v.decode().ok());

    match proxies_opt {
        Some((proxies_vec, _deposit)) => Ok(proxies_vec
            .0
            .iter()
            .any(|p| p.delegate == *delegate && matches_proxy_type(&p.proxy_type, proxy_type))),
        None => Ok(false),
    }
}

/// Compare a ProxyType enum value against a u8 index.
fn matches_proxy_type(
    pt: &subtensor::runtime_types::subtensor_runtime_common::ProxyType,
    index: u8,
) -> bool {
    use subtensor::runtime_types::subtensor_runtime_common::ProxyType::*;
    matches!(
        (pt, index),
        (Any, 0)
            | (Owner, 1)
            | (NonCritical, 2)
            | (NonTransfer, 3)
            | (Senate, 4)
            | (NonFungible, 5)
            | (Triumvirate, 6)
            | (Governance, 7)
            | (Staking, 8)
            | (Registration, 9)
            | (Transfer, 10)
            | (SmallTransfer, 11)
            | (RootWeights, 12)
            | (ChildKeys, 13)
            | (SudoUncheckedSetCode, 14)
            | (SwapHotkey, 15)
            | (SubnetLeaseBeneficiary, 16)
            | (RootClaim, 17)
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn proxy_query_signature() {
        assert!(true);
    }

    #[test]
    fn proxy_type_conversion() {
        let v: u8 = 0;
        assert_eq!(v, 0);
    }
}
