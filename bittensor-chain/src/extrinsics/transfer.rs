use bittensor_core::config::SubtensorConfig;
use subxt::OnlineClient;

use crate::generated::subtensor;

use super::{Result, TxSuccess, submit_and_watch, to_multi_address};

/// Transfer balance to a destination account, allowing the sender account to be reaped.
pub async fn transfer(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    dest: subxt::utils::AccountId32,
    value: u64,
) -> Result<TxSuccess> {
    let call = subtensor::tx().balances().transfer_allow_death(to_multi_address(dest), value);
    submit_and_watch(client, call, signer).await
}

/// Transfer balance to a destination account, keeping the sender account alive.
pub async fn transfer_keep_alive(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    dest: subxt::utils::AccountId32,
    value: u64,
) -> Result<TxSuccess> {
    let call = subtensor::tx().balances().transfer_keep_alive(to_multi_address(dest), value);
    submit_and_watch(client, call, signer).await
}

/// Transfer the entire available balance to a destination account.
pub async fn transfer_all(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    dest: subxt::utils::AccountId32,
    keep_alive: bool,
) -> Result<TxSuccess> {
    let call = subtensor::tx().balances().transfer_all(to_multi_address(dest), keep_alive);
    submit_and_watch(client, call, signer).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generated::subtensor;

    #[test]
    fn transfer_call_construction() {
        let dest = subxt::utils::AccountId32::from([2u8; 32]);
        let _call = subtensor::tx()
            .balances()
            .transfer_allow_death(to_multi_address(dest), 1_000_000_000u64);
    }

    #[test]
    fn transfer_keep_alive_call_construction() {
        let dest = subxt::utils::AccountId32::from([2u8; 32]);
        let _call = subtensor::tx()
            .balances()
            .transfer_keep_alive(to_multi_address(dest), 1_000_000_000u64);
    }

    #[test]
    fn transfer_all_call_construction() {
        let dest = subxt::utils::AccountId32::from([2u8; 32]);
        let _call = subtensor::tx().balances().transfer_all(to_multi_address(dest), false);
    }
}
