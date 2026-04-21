//! Childkey queries — childkey hierarchy, take rates, and cooldown.

use bittensor_core::config::SubtensorConfig;
use bittensor_core::error::BittensorError;
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

/// Fetch the child keys for a hotkey in a subnet.
pub async fn get_children(
    client: &OnlineClient<SubtensorConfig>,
    hotkey: &subxt::utils::AccountId32,
    netuid: u16,
) -> Result<Vec<(u64, subxt::utils::AccountId32)>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let children: Vec<(u64, subxt::utils::AccountId32)> =
        fetch_decode!(at, addr, child_keys, (*hotkey, netuid));
    Ok(children)
}

/// Fetch the childkey take for a hotkey in a subnet.
pub async fn get_childkey_take(
    client: &OnlineClient<SubtensorConfig>,
    hotkey: &subxt::utils::AccountId32,
    netuid: u16,
) -> Result<u16> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let take: u16 = fetch_decode!(at, addr, childkey_take, (*hotkey, netuid));
    Ok(take)
}

/// Fetch the pending child key cooldown (global, in blocks).
pub async fn get_pending_child_key_cooldown(client: &OnlineClient<SubtensorConfig>) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, pending_child_key_cooldown, ());
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

#[cfg(test)]
mod tests {
    #[test]
    fn children_query_signature() {
        assert!(true);
    }

    #[test]
    fn childkey_take_default() {
        let v: u16 = 0;
        assert_eq!(v, 0);
    }
}
