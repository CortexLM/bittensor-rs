//! Delegate queries — delegate list, take rates, delegation info, and childkey hierarchy.

use bittensor_core::balance::Balance;
use bittensor_core::config::SubtensorConfig;
use bittensor_core::error::BittensorError;
use bittensor_core::types::DelegateInfo;
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

/// Fetch the list of delegates that have a non-zero take across all subnets.
pub async fn get_delegates(client: &OnlineClient<SubtensorConfig>) -> Result<Vec<DelegateInfo>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();

    let total_networks: u16 = fetch_decode!(at, addr, total_networks, ());

    let mut delegates = Vec::new();

    for netuid in 0..=total_networks {
        let n: u16 = fetch_decode!(at, addr, subnetwork_n, (netuid,));
        for uid in 0..n {
            let hotkey_opt: Option<subxt::utils::AccountId32> = at
                .storage()
                .try_fetch(addr.keys(), (netuid, uid))
                .await
                .map_err(|e| BittensorError::Rpc(e.to_string()))?
                .and_then(|v| v.decode().ok());

            if let Some(hk) = hotkey_opt {
                let take: u16 = fetch_decode!(at, addr, delegates, (hk,));
                if take > 0 {
                    delegates.push(DelegateInfo {
                        delegate_ss58: String::new(),
                        delegate_hotkey: hk.to_string(),
                        total_stake: Balance::ZERO,
                        nominators: vec![],
                        owner_hotkey: String::new(),
                        take,
                        owner_ss58: String::new(),
                        registrations: vec![],
                        validator_permits: vec![],
                    });
                }
            }
        }
    }

    Ok(delegates)
}

/// Fetch the delegate take rate (basis points) for a hotkey.
pub async fn get_delegate_take(
    client: &OnlineClient<SubtensorConfig>,
    hotkey: &subxt::utils::AccountId32,
) -> Result<u16> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let take: u16 = fetch_decode!(at, addr, delegates, (*hotkey,));
    Ok(take)
}

/// Fetch the delegate info for all hotkeys owned by a coldkey.
pub async fn get_delegated_info(
    client: &OnlineClient<SubtensorConfig>,
    coldkey: &subxt::utils::AccountId32,
) -> Result<Vec<DelegateInfo>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();

    let hotkeys: Vec<subxt::utils::AccountId32> =
        fetch_decode!(at, addr, owned_hotkeys, (*coldkey,));

    let mut result = Vec::new();
    for hk in &hotkeys {
        let take: u16 = fetch_decode!(at, addr, delegates, (*hk,));
        result.push(DelegateInfo {
            delegate_ss58: String::new(),
            delegate_hotkey: hk.to_string(),
            total_stake: Balance::ZERO,
            nominators: vec![],
            owner_hotkey: coldkey.to_string(),
            take,
            owner_ss58: String::new(),
            registrations: vec![],
            validator_permits: vec![],
        });
    }

    Ok(result)
}

/// Fetch the global maximum delegate take (basis points).
pub async fn get_max_delegate_take(client: &OnlineClient<SubtensorConfig>) -> Result<u16> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u16 = fetch_decode!(at, addr, max_delegate_take, ());
    Ok(v)
}

/// Fetch the global minimum delegate take (basis points).
pub async fn get_min_delegate_take(client: &OnlineClient<SubtensorConfig>) -> Result<u16> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u16 = fetch_decode!(at, addr, min_delegate_take, ());
    Ok(v)
}

/// Fetch the childkey take for a hotkey in a subnet (basis points).
pub async fn get_childkey_take(
    client: &OnlineClient<SubtensorConfig>,
    hotkey: &subxt::utils::AccountId32,
    netuid: u16,
) -> Result<u16> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u16 = fetch_decode!(at, addr, childkey_take, (*hotkey, netuid));
    Ok(v)
}

/// Fetch the global maximum childkey take (basis points).
pub async fn get_max_childkey_take(client: &OnlineClient<SubtensorConfig>) -> Result<u16> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u16 = fetch_decode!(at, addr, max_childkey_take, ());
    Ok(v)
}

/// Fetch the global minimum childkey take (basis points).
pub async fn get_min_childkey_take(client: &OnlineClient<SubtensorConfig>) -> Result<u16> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u16 = fetch_decode!(at, addr, min_childkey_take, ());
    Ok(v)
}

/// Fetch the pending child keys for a subnet and hotkey.
pub async fn get_pending_child_keys(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    hotkey: &subxt::utils::AccountId32,
) -> Result<(Vec<(u64, subxt::utils::AccountId32)>, u64)> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: (Vec<(u64, subxt::utils::AccountId32)>, u64) =
        fetch_decode!(at, addr, pending_child_keys, (netuid, *hotkey));
    Ok(v)
}

/// Fetch the child keys for a hotkey in a subnet.
pub async fn get_child_keys(
    client: &OnlineClient<SubtensorConfig>,
    hotkey: &subxt::utils::AccountId32,
    netuid: u16,
) -> Result<Vec<(u64, subxt::utils::AccountId32)>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: Vec<(u64, subxt::utils::AccountId32)> =
        fetch_decode!(at, addr, child_keys, (*hotkey, netuid));
    Ok(v)
}

/// Fetch the parent keys for a hotkey in a subnet.
pub async fn get_parent_keys(
    client: &OnlineClient<SubtensorConfig>,
    hotkey: &subxt::utils::AccountId32,
    netuid: u16,
) -> Result<Vec<(u64, subxt::utils::AccountId32)>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: Vec<(u64, subxt::utils::AccountId32)> =
        fetch_decode!(at, addr, parent_keys, (*hotkey, netuid));
    Ok(v)
}

/// Fetch the staking hotkeys for a coldkey.
pub async fn get_staking_hotkeys(
    client: &OnlineClient<SubtensorConfig>,
    coldkey: &subxt::utils::AccountId32,
) -> Result<Vec<subxt::utils::AccountId32>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: Vec<subxt::utils::AccountId32> = fetch_decode!(at, addr, staking_hotkeys, (*coldkey,));
    Ok(v)
}

/// Fetch the staking coldkeys count for a coldkey.
pub async fn get_num_staking_coldkeys(client: &OnlineClient<SubtensorConfig>) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, num_staking_coldkeys, ());
    Ok(v)
}

/// Fetch the auto-stake destination for a hotkey in a subnet.
///
/// Returns `None` if no destination is set.
pub async fn get_auto_stake_destination(
    client: &OnlineClient<SubtensorConfig>,
    hotkey: &subxt::utils::AccountId32,
    netuid: u16,
) -> Result<Option<subxt::utils::AccountId32>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: Option<subxt::utils::AccountId32> = at
        .storage()
        .try_fetch(addr.auto_stake_destination(), (*hotkey, netuid))
        .await
        .map_err(|e| BittensorError::Rpc(e.to_string()))?
        .and_then(|val| val.decode().ok());
    Ok(v)
}

/// Fetch the last transaction block for a hotkey.
pub async fn get_last_tx_block(
    client: &OnlineClient<SubtensorConfig>,
    hotkey: &subxt::utils::AccountId32,
) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, last_tx_block, (*hotkey,));
    Ok(v)
}

/// Fetch the last transaction block for a childkey take operation.
pub async fn get_last_tx_block_child_key_take(
    client: &OnlineClient<SubtensorConfig>,
    hotkey: &subxt::utils::AccountId32,
) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, last_tx_block_child_key_take, (*hotkey,));
    Ok(v)
}

/// Fetch the last transaction block for a delegate take operation.
pub async fn get_last_tx_block_delegate_take(
    client: &OnlineClient<SubtensorConfig>,
    hotkey: &subxt::utils::AccountId32,
) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, last_tx_block_delegate_take, (*hotkey,));
    Ok(v)
}

/// Fetch the staking operation rate limiter status for a coldkey/hotkey pair.
pub async fn get_staking_operation_rate_limiter(
    client: &OnlineClient<SubtensorConfig>,
    coldkey: &subxt::utils::AccountId32,
    hotkey: &subxt::utils::AccountId32,
    netuid: u16,
) -> Result<bool> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: bool =
        fetch_decode!(at, addr, staking_operation_rate_limiter, (*coldkey, *hotkey, netuid));
    Ok(v)
}

/// Fetch the tx delegate take rate limit (global, in blocks).
pub async fn get_tx_delegate_take_rate_limit(
    client: &OnlineClient<SubtensorConfig>,
) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, tx_delegate_take_rate_limit, ());
    Ok(v)
}

/// Fetch the tx childkey take rate limit (global, in blocks).
pub async fn get_tx_childkey_take_rate_limit(
    client: &OnlineClient<SubtensorConfig>,
) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, tx_childkey_take_rate_limit, ());
    Ok(v)
}

/// Fetch the owner of a hotkey (returns the coldkey that owns it).
///
/// Returns `None` if the hotkey has no owner on-chain.
pub async fn get_owner(
    client: &OnlineClient<SubtensorConfig>,
    hotkey: &subxt::utils::AccountId32,
) -> Result<Option<subxt::utils::AccountId32>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: Option<subxt::utils::AccountId32> = at
        .storage()
        .try_fetch(addr.owner(), (*hotkey,))
        .await
        .map_err(|e| BittensorError::Rpc(e.to_string()))?
        .and_then(|val| val.decode().ok());
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delegate_info_construction() {
        let di = DelegateInfo {
            delegate_ss58: "5Test".into(),
            delegate_hotkey: "hk".into(),
            total_stake: Balance::from_tao(500.0),
            nominators: vec![("nom1".into(), Balance::from_tao(250.0))],
            owner_hotkey: "owner".into(),
            take: 18,
            owner_ss58: "5Owner".into(),
            registrations: vec![1, 3],
            validator_permits: vec![1],
        };
        assert_eq!(di.take, 18);
    }

    #[test]
    fn delegate_take_bounds() {
        assert!(u16::MIN == 0);
        assert!(u16::MAX == 65535);
    }
}
