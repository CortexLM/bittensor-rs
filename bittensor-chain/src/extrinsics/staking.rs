//! Staking extrinsics — add, remove, move, swap, and transfer stake.

use bittensor_core::config::SubtensorConfig;
use subxt::OnlineClient;

use crate::generated::subtensor;

use super::{Result, TxSuccess, submit_and_watch};

/// Add stake to a hotkey in a subnet.
pub async fn add_stake(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    hotkey: subxt::utils::AccountId32,
    netuid: u16,
    amount_staked: u64,
) -> Result<TxSuccess> {
    let call = subtensor::tx().subtensor_module().add_stake(hotkey, netuid, amount_staked);
    submit_and_watch(client, call, signer).await
}

/// Add stake to multiple hotkeys in a subnet, one transaction per hotkey.
pub async fn add_stake_multiple(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    hotkeys: Vec<subxt::utils::AccountId32>,
    netuid: u16,
    amounts: Vec<u64>,
) -> Result<Vec<TxSuccess>> {
    let mut results = Vec::with_capacity(hotkeys.len());
    for (hotkey, amount_staked) in hotkeys.into_iter().zip(amounts.into_iter()) {
        let call = subtensor::tx().subtensor_module().add_stake(hotkey, netuid, amount_staked);
        results.push(submit_and_watch(client, call, signer).await?);
    }
    Ok(results)
}

/// Remove stake from a hotkey in a subnet.
pub async fn remove_stake(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    hotkey: subxt::utils::AccountId32,
    netuid: u16,
    amount_unstaked: u64,
) -> Result<TxSuccess> {
    let call = subtensor::tx().subtensor_module().remove_stake(hotkey, netuid, amount_unstaked);
    submit_and_watch(client, call, signer).await
}

/// Unstake all stake from a hotkey across all subnets.
pub async fn unstake_all(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    hotkey: subxt::utils::AccountId32,
) -> Result<TxSuccess> {
    let call = subtensor::tx().subtensor_module().unstake_all(hotkey);
    submit_and_watch(client, call, signer).await
}

/// Remove stake from multiple hotkeys in a subnet, one transaction per hotkey.
pub async fn unstake_multiple(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    hotkeys: Vec<subxt::utils::AccountId32>,
    netuid: u16,
    amounts: Vec<u64>,
) -> Result<Vec<TxSuccess>> {
    let mut results = Vec::with_capacity(hotkeys.len());
    for (hotkey, amount_unstaked) in hotkeys.into_iter().zip(amounts.into_iter()) {
        let call = subtensor::tx().subtensor_module().remove_stake(hotkey, netuid, amount_unstaked);
        results.push(submit_and_watch(client, call, signer).await?);
    }
    Ok(results)
}

/// Move stake from one hotkey/subnet pair to another.
pub async fn move_stake(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    origin_hotkey: subxt::utils::AccountId32,
    destination_hotkey: subxt::utils::AccountId32,
    origin_netuid: u16,
    destination_netuid: u16,
    alpha_amount: u64,
) -> Result<TxSuccess> {
    let call = subtensor::tx().subtensor_module().move_stake(
        origin_hotkey,
        destination_hotkey,
        origin_netuid,
        destination_netuid,
        alpha_amount,
    );
    submit_and_watch(client, call, signer).await
}

/// Swap stake between two subnets for the same hotkey.
pub async fn swap_stake(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    hotkey: subxt::utils::AccountId32,
    origin_netuid: u16,
    destination_netuid: u16,
    alpha_amount: u64,
) -> Result<TxSuccess> {
    let call = subtensor::tx().subtensor_module().swap_stake(
        hotkey,
        origin_netuid,
        destination_netuid,
        alpha_amount,
    );
    submit_and_watch(client, call, signer).await
}

/// Transfer stake from one coldkey to another across subnets.
pub async fn transfer_stake(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    destination_coldkey: subxt::utils::AccountId32,
    hotkey: subxt::utils::AccountId32,
    origin_netuid: u16,
    destination_netuid: u16,
    alpha_amount: u64,
) -> Result<TxSuccess> {
    let call = subtensor::tx().subtensor_module().transfer_stake(
        destination_coldkey,
        hotkey,
        origin_netuid,
        destination_netuid,
        alpha_amount,
    );
    submit_and_watch(client, call, signer).await
}

/// Enable auto-staking for a coldkey/hotkey pair in a subnet.
pub async fn set_auto_stake(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    netuid: u16,
    hotkey: subxt::utils::AccountId32,
) -> Result<TxSuccess> {
    let call = subtensor::tx().subtensor_module().set_coldkey_auto_stake_hotkey(netuid, hotkey);
    submit_and_watch(client, call, signer).await
}

#[cfg(test)]
mod tests {

    use crate::generated::subtensor;

    #[test]
    fn add_stake_call_construction() {
        let hotkey = subxt::utils::AccountId32::from([1u8; 32]);
        let _call = subtensor::tx().subtensor_module().add_stake(hotkey, 1u16, 1_000_000_000u64);
    }

    #[test]
    fn remove_stake_call_construction() {
        let hotkey = subxt::utils::AccountId32::from([2u8; 32]);
        let _call = subtensor::tx().subtensor_module().remove_stake(hotkey, 1u16, 500_000_000u64);
    }

    #[test]
    fn unstake_all_call_construction() {
        let hotkey = subxt::utils::AccountId32::from([2u8; 32]);
        let _call = subtensor::tx().subtensor_module().unstake_all(hotkey);
    }

    #[test]
    fn move_stake_call_construction() {
        let origin = subxt::utils::AccountId32::from([1u8; 32]);
        let dest = subxt::utils::AccountId32::from([2u8; 32]);
        let _call = subtensor::tx().subtensor_module().move_stake(
            origin,
            dest,
            1u16,
            2u16,
            1_000_000_000u64,
        );
    }

    #[test]
    fn swap_stake_call_construction() {
        let hotkey = subxt::utils::AccountId32::from([1u8; 32]);
        let _call =
            subtensor::tx().subtensor_module().swap_stake(hotkey, 1u16, 2u16, 1_000_000_000u64);
    }

    #[test]
    fn transfer_stake_call_construction() {
        let coldkey = subxt::utils::AccountId32::from([3u8; 32]);
        let hotkey = subxt::utils::AccountId32::from([1u8; 32]);
        let _call = subtensor::tx().subtensor_module().transfer_stake(
            coldkey,
            hotkey,
            1u16,
            2u16,
            1_000_000_000u64,
        );
    }

    #[test]
    fn add_stake_multiple_call_construction() {
        let hotkey = subxt::utils::AccountId32::from([1u8; 32]);
        let _call = subtensor::tx().subtensor_module().add_stake(hotkey, 1u16, 1_000_000_000u64);
    }

    #[test]
    fn unstake_multiple_call_construction() {
        let hotkey = subxt::utils::AccountId32::from([2u8; 32]);
        let _call = subtensor::tx().subtensor_module().remove_stake(hotkey, 1u16, 500_000_000u64);
    }

    #[test]
    fn set_auto_stake_call_construction() {
        let hotkey = subxt::utils::AccountId32::from([1u8; 32]);
        let _call = subtensor::tx().subtensor_module().set_coldkey_auto_stake_hotkey(1u16, hotkey);
    }
}
