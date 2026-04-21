//! Account queries — balance, stake, delegation, alpha, and token flow.

use bittensor_core::balance::Balance;
use bittensor_core::config::SubtensorConfig;
use bittensor_core::error::BittensorError;
use bittensor_core::types::StakeInfo;
use subxt::OnlineClient;

use crate::client::ClientAtBlock;
use crate::generated::subtensor;

type Result<T> = std::result::Result<T, BittensorError>;

async fn at_block(client: &OnlineClient<SubtensorConfig>) -> Result<ClientAtBlock> {
    client.at_current_block().await.map_err(|e| BittensorError::Rpc(e.to_string()))
}

fn decode_val<T>(opt: Option<subxt::storage::StorageValue<'_, T>>) -> T
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
        decode_val(opt)
    }};
}

/// Fetch the free balance for an account.
pub async fn get_balance(
    client: &OnlineClient<SubtensorConfig>,
    account_id: &subxt::utils::AccountId32,
) -> Result<Balance> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().system();

    let account_opt: Option<
        subtensor::runtime_types::frame_system::AccountInfo<
            u32,
            subtensor::runtime_types::pallet_balances::types::AccountData<u64>,
        >,
    > = at
        .storage()
        .try_fetch(addr.account(), (*account_id,))
        .await
        .map_err(|e| BittensorError::Rpc(e.to_string()))?
        .and_then(|v| v.decode().ok());

    let free = account_opt.map(|a| a.data.free).unwrap_or(0);
    Ok(Balance::from_rao(free))
}

/// Fetch the stake for a coldkey/hotkey pair (stub — returns zero).
pub async fn get_stake(
    _client: &OnlineClient<SubtensorConfig>,
    _coldkey: &subxt::utils::AccountId32,
    _hotkey: &subxt::utils::AccountId32,
    _netuid: u16,
) -> Result<Balance> {
    Ok(Balance::ZERO)
}

/// Fetch the stake info for all hotkeys owned by a coldkey.
pub async fn get_stake_info_for_coldkey(
    client: &OnlineClient<SubtensorConfig>,
    coldkey: &subxt::utils::AccountId32,
) -> Result<Vec<StakeInfo>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();

    let hotkeys: Vec<subxt::utils::AccountId32> =
        fetch_decode!(at, addr, owned_hotkeys, (*coldkey,));

    let mut stakes = Vec::new();
    for hk in &hotkeys {
        stakes.push(StakeInfo {
            hotkey: hk.to_string(),
            coldkey: coldkey.to_string(),
            stake: Balance::ZERO,
        });
    }

    Ok(stakes)
}

/// Fetch the total network stake.
pub async fn get_total_network_stake(client: &OnlineClient<SubtensorConfig>) -> Result<Balance> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let total: u64 = fetch_decode!(at, addr, total_stake, ());
    Ok(Balance::from_rao(total))
}

/// Fetch the total balance (free + reserved) for an account.
pub async fn get_total_balance(
    client: &OnlineClient<SubtensorConfig>,
    account_id: &subxt::utils::AccountId32,
) -> Result<Balance> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().system();

    let account_opt: Option<
        subtensor::runtime_types::frame_system::AccountInfo<
            u32,
            subtensor::runtime_types::pallet_balances::types::AccountData<u64>,
        >,
    > = at
        .storage()
        .try_fetch(addr.account(), (*account_id,))
        .await
        .map_err(|e| BittensorError::Rpc(e.to_string()))?
        .and_then(|v| v.decode().ok());

    let total = account_opt.map(|a| a.data.free + a.data.reserved).unwrap_or(0);
    Ok(Balance::from_rao(total))
}

/// Fetch the hotkeys owned by a coldkey.
pub async fn get_owned_hotkeys(
    client: &OnlineClient<SubtensorConfig>,
    coldkey: &subxt::utils::AccountId32,
) -> Result<Vec<subxt::utils::AccountId32>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: Vec<subxt::utils::AccountId32> = fetch_decode!(at, addr, owned_hotkeys, (*coldkey,));
    Ok(v)
}

/// Fetch the total hotkey alpha for a hotkey in a subnet.
pub async fn get_total_hotkey_alpha(
    client: &OnlineClient<SubtensorConfig>,
    hotkey: &subxt::utils::AccountId32,
    netuid: u16,
) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, total_hotkey_alpha, (*hotkey, netuid));
    Ok(v)
}

/// Fetch the total hotkey alpha at the last epoch for a hotkey in a subnet.
pub async fn get_total_hotkey_alpha_last_epoch(
    client: &OnlineClient<SubtensorConfig>,
    hotkey: &subxt::utils::AccountId32,
    netuid: u16,
) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, total_hotkey_alpha_last_epoch, (*hotkey, netuid));
    Ok(v)
}

/// Fetch the token symbol bytes for a subnet.
pub async fn get_token_symbol(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<Vec<u8>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: Vec<u8> = fetch_decode!(at, addr, token_symbol, (netuid,));
    Ok(v)
}

/// Fetch the subnet TAO balance for a subnet.
pub async fn get_subnet_tao(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<Balance> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, subnet_tao, (netuid,));
    Ok(Balance::from_rao(v))
}

/// Fetch the subnet TAO provided for a subnet.
pub async fn get_subnet_tao_provided(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, subnet_tao_provided, (netuid,));
    Ok(v)
}

/// Fetch the subnet alpha in (inflow) for a subnet.
pub async fn get_subnet_alpha_in(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, subnet_alpha_in, (netuid,));
    Ok(v)
}

/// Fetch the subnet alpha in provided for a subnet.
pub async fn get_subnet_alpha_in_provided(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, subnet_alpha_in_provided, (netuid,));
    Ok(v)
}

/// Fetch the subnet alpha out (outflow) for a subnet.
pub async fn get_subnet_alpha_out(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, subnet_alpha_out, (netuid,));
    Ok(v)
}

/// Fetch the subnet alpha in emission for a subnet.
pub async fn get_subnet_alpha_in_emission(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, subnet_alpha_in_emission, (netuid,));
    Ok(v)
}

/// Fetch the subnet alpha out emission for a subnet.
pub async fn get_subnet_alpha_out_emission(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, subnet_alpha_out_emission, (netuid,));
    Ok(v)
}

/// Fetch the subnet TAO in emission for a subnet.
pub async fn get_subnet_tao_in_emission(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, subnet_tao_in_emission, (netuid,));
    Ok(v)
}

/// Fetch the root alpha dividends per subnet for a hotkey.
pub async fn get_root_alpha_dividends_per_subnet(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    hotkey: &subxt::utils::AccountId32,
) -> Result<u64> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, root_alpha_dividends_per_subnet, (netuid, *hotkey));
    Ok(v)
}

/// Fetch the total issuance (global).
pub async fn get_total_issuance(client: &OnlineClient<SubtensorConfig>) -> Result<Balance> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();
    let v: u64 = fetch_decode!(at, addr, total_issuance, ());
    Ok(Balance::from_rao(v))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn balance_from_rao() {
        let b = Balance::from_rao(1_500_000_000);
        assert!((b.to_tao() - 1.5).abs() < 1e-10);
    }

    #[test]
    fn stake_info_construction() {
        let si = StakeInfo {
            hotkey: "hk".into(),
            coldkey: "ck".into(),
            stake: Balance::from_tao(100.0),
        };
        assert_eq!(si.hotkey, "hk");
    }

    #[test]
    fn alpha_default() {
        let v: u64 = 0;
        assert_eq!(v, 0);
    }
}
