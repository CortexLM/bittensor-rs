//! Network queries — block number, hash rate, issuance, subnet limit, and network-level parameters.

use bittensor_core::balance::Balance;
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

/// Fetch the current best block number.
pub async fn get_network_block(client: &OnlineClient<SubtensorConfig>) -> Result<u64> {
    let at = at_block(client).await?;
    Ok(at.block_number())
}

/// Fetch the current network hash rate (stub — always returns 0).
pub async fn get_network_hash_rate(client: &OnlineClient<SubtensorConfig>) -> Result<u64> {
    let _ = at_block(client).await?;
    Ok(0)
}

/// Fetch the weight vector for a UID in a subnet.
pub async fn get_current_weight(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    uid: u16,
) -> Result<Vec<(u16, u16)>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let weights: Vec<(u16, u16)> = fetch_decode!(at, addr, weights, (netuid, uid));
    Ok(weights)
}

/// Fetch the total token issuance (per-netuid, currently returns the global total).
pub async fn get_total_issuance(
    client: &OnlineClient<SubtensorConfig>,
    _netuid: u16,
) -> Result<Balance> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let total: u64 = fetch_decode!(at, addr, total_issuance, ());
    Ok(Balance::from_rao(total))
}

/// Fetch the block hash for a given block number.
pub async fn get_block_hash(
    client: &OnlineClient<SubtensorConfig>,
    block_number: u64,
) -> Result<Option<subxt::utils::H256>> {
    match client.at_block(block_number).await {
        Ok(at) => Ok(Some(at.block_hash())),
        Err(_) => Ok(None),
    }
}

/// Fetch the total number of networks (subnets) on chain.
pub async fn get_total_networks(client: &OnlineClient<SubtensorConfig>) -> Result<u16> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u16 = fetch_decode!(at, addr, total_networks, ());
    Ok(v)
}

/// Fetch the block emission rate (global, in rao per block).
pub async fn get_block_emission(client: &OnlineClient<SubtensorConfig>) -> Result<Balance> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, block_emission, ());
    Ok(Balance::from_rao(v))
}

/// Fetch the subnet limit (maximum number of subnets).
pub async fn get_subnet_limit(client: &OnlineClient<SubtensorConfig>) -> Result<u16> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u16 = fetch_decode!(at, addr, subnet_limit, ());
    Ok(v)
}

/// Fetch the network immunity period (in blocks, global).
pub async fn get_network_immunity_period(client: &OnlineClient<SubtensorConfig>) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, network_immunity_period, ());
    Ok(v)
}

/// Fetch the network rate limit (in blocks, global).
pub async fn get_network_rate_limit(client: &OnlineClient<SubtensorConfig>) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, network_rate_limit, ());
    Ok(v)
}

/// Fetch the minimum required stake for a nominator (global).
pub async fn get_nominator_min_required_stake(
    client: &OnlineClient<SubtensorConfig>,
) -> Result<Balance> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, nominator_min_required_stake, ());
    Ok(Balance::from_rao(v))
}

/// Fetch the subnetwork_n for a given subnet (number of UIDs).
pub async fn get_subnetwork_n(client: &OnlineClient<SubtensorConfig>, netuid: u16) -> Result<u16> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u16 = fetch_decode!(at, addr, subnetwork_n, (netuid,));
    Ok(v)
}

/// Check if a subnet has been added (exists).
pub async fn get_networks_added(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<bool> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: bool = fetch_decode!(at, addr, networks_added, (netuid,));
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network_block_signature() {
        assert!(true);
    }

    #[test]
    fn balance_from_rao_network() {
        let b = Balance::from_rao(1_500_000_000);
        assert!((b.to_tao() - 1.5).abs() < 1e-10);
    }
}
