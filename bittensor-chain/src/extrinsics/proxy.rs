//! Proxy extrinsics — add and remove proxy accounts.

use bittensor_core::config::SubtensorConfig;
use subxt::OnlineClient;

use crate::generated::subtensor;

use super::{Result, TxSuccess, submit_and_watch, to_multi_address};

/// Register a delegate account as a proxy for the signer.
pub async fn add_proxy(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    delegate: subxt::utils::AccountId32,
    proxy_type: subtensor::runtime_types::subtensor_runtime_common::ProxyType,
    delay: u32,
) -> Result<TxSuccess> {
    let call = subtensor::tx().proxy().add_proxy(to_multi_address(delegate), proxy_type, delay);
    submit_and_watch(client, call, signer).await
}

/// Remove a previously added proxy delegate.
pub async fn remove_proxy(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    delegate: subxt::utils::AccountId32,
    proxy_type: subtensor::runtime_types::subtensor_runtime_common::ProxyType,
    delay: u32,
) -> Result<TxSuccess> {
    let call = subtensor::tx().proxy().remove_proxy(to_multi_address(delegate), proxy_type, delay);
    submit_and_watch(client, call, signer).await
}

/// Remove all proxy delegates for the signer.
pub async fn remove_proxies(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
) -> Result<TxSuccess> {
    let call = subtensor::tx().proxy().remove_proxies();
    submit_and_watch(client, call, signer).await
}

/// Create a pure (anonymous) proxy account.
pub async fn create_pure_proxy(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    proxy_type: subtensor::runtime_types::subtensor_runtime_common::ProxyType,
    delay: u32,
    index: u16,
) -> Result<TxSuccess> {
    let call = subtensor::tx().proxy().create_pure(proxy_type, delay, index);
    submit_and_watch(client, call, signer).await
}

/// Kill a previously created pure proxy.
pub async fn kill_pure_proxy(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    spawner: subxt::utils::AccountId32,
    proxy_type: subtensor::runtime_types::subtensor_runtime_common::ProxyType,
    index: u16,
    height: u32,
    ext_index: u32,
) -> Result<TxSuccess> {
    let call = subtensor::tx().proxy().kill_pure(
        to_multi_address(spawner),
        proxy_type,
        index,
        height,
        ext_index,
    );
    submit_and_watch(client, call, signer).await
}

/// Execute a proxied call on behalf of the `real` account.
pub async fn proxy(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    real: subxt::utils::AccountId32,
    force_proxy_type: Option<subtensor::runtime_types::subtensor_runtime_common::ProxyType>,
    call: subtensor::runtime_types::node_subtensor_runtime::RuntimeCall,
) -> Result<TxSuccess> {
    let call = subtensor::tx().proxy().proxy(to_multi_address(real), force_proxy_type, call);
    submit_and_watch(client, call, signer).await
}

/// Announce a proxied call that will be executed after the announcement delay.
pub async fn announce(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    real: subxt::utils::AccountId32,
    call_hash: subxt::utils::H256,
) -> Result<TxSuccess> {
    let call = subtensor::tx().proxy().announce(to_multi_address(real), call_hash);
    submit_and_watch(client, call, signer).await
}

/// Execute a previously announced proxied call.
pub async fn proxy_announced(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    delegate: subxt::utils::AccountId32,
    real: subxt::utils::AccountId32,
    force_proxy_type: Option<subtensor::runtime_types::subtensor_runtime_common::ProxyType>,
    call: subtensor::runtime_types::node_subtensor_runtime::RuntimeCall,
) -> Result<TxSuccess> {
    let call = subtensor::tx().proxy().proxy_announced(
        to_multi_address(delegate),
        to_multi_address(real),
        force_proxy_type,
        call,
    );
    submit_and_watch(client, call, signer).await
}

/// Reject a previously made proxy announcement.
pub async fn reject_announcement(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    delegate: subxt::utils::AccountId32,
    call_hash: subxt::utils::H256,
) -> Result<TxSuccess> {
    let call = subtensor::tx().proxy().reject_announcement(to_multi_address(delegate), call_hash);
    submit_and_watch(client, call, signer).await
}

/// Remove a proxy announcement made by the signer.
pub async fn remove_announcement(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    real: subxt::utils::AccountId32,
    call_hash: subxt::utils::H256,
) -> Result<TxSuccess> {
    let call = subtensor::tx().proxy().remove_announcement(to_multi_address(real), call_hash);
    submit_and_watch(client, call, signer).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generated::subtensor;

    #[test]
    fn add_proxy_call_construction() {
        let delegate = subxt::utils::AccountId32::from([3u8; 32]);
        let proxy_type = subtensor::runtime_types::subtensor_runtime_common::ProxyType::Any;
        let _call = subtensor::tx().proxy().add_proxy(to_multi_address(delegate), proxy_type, 0u32);
    }

    #[test]
    fn remove_proxy_call_construction() {
        let delegate = subxt::utils::AccountId32::from([3u8; 32]);
        let proxy_type = subtensor::runtime_types::subtensor_runtime_common::ProxyType::Any;
        let _call =
            subtensor::tx().proxy().remove_proxy(to_multi_address(delegate), proxy_type, 0u32);
    }

    #[test]
    fn remove_proxies_call_construction() {
        let _call = subtensor::tx().proxy().remove_proxies();
    }

    #[test]
    fn announce_call_construction() {
        let real = subxt::utils::AccountId32::from([1u8; 32]);
        let _call =
            subtensor::tx().proxy().announce(to_multi_address(real), subxt::utils::H256::zero());
    }

    #[test]
    fn reject_announcement_call_construction() {
        let delegate = subxt::utils::AccountId32::from([3u8; 32]);
        let _call = subtensor::tx()
            .proxy()
            .reject_announcement(to_multi_address(delegate), subxt::utils::H256::zero());
    }

    #[test]
    fn remove_announcement_call_construction() {
        let real = subxt::utils::AccountId32::from([1u8; 32]);
        let _call = subtensor::tx()
            .proxy()
            .remove_announcement(to_multi_address(real), subxt::utils::H256::zero());
    }

    #[test]
    fn create_pure_proxy_call_construction() {
        let proxy_type = subtensor::runtime_types::subtensor_runtime_common::ProxyType::Any;
        let _call = subtensor::tx().proxy().create_pure(proxy_type, 0u32, 0u16);
    }

    #[test]
    fn kill_pure_proxy_call_construction() {
        let spawner = subxt::utils::AccountId32::from([1u8; 32]);
        let proxy_type = subtensor::runtime_types::subtensor_runtime_common::ProxyType::Any;
        let _call = subtensor::tx().proxy().kill_pure(
            to_multi_address(spawner),
            proxy_type,
            0u16,
            0u32,
            0u32,
        );
    }
}
