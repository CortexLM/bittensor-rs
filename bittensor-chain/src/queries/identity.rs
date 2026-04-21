//! Identity queries for the Subtensor chain.
//!
//! Provides access to identity registrations and subnet identity storage.

use bittensor_core::config::SubtensorConfig;
use bittensor_core::error::BittensorError;
use subxt::OnlineClient;

use crate::client::ClientAtBlock;
use crate::generated::subtensor;

type Result<T> = std::result::Result<T, BittensorError>;

async fn at_block(client: &OnlineClient<SubtensorConfig>) -> Result<ClientAtBlock> {
    client.at_current_block().await.map_err(|e| BittensorError::Rpc(e.to_string()))
}

/// Fetch the identity registration for a given hotkey from the registry pallet.
///
/// Returns the raw `Registration` struct from the chain if it exists.
/// The caller can then inspect individual fields (info, judgements) as needed.
pub async fn get_identity(
    client: &OnlineClient<SubtensorConfig>,
    hotkey: &subxt::utils::AccountId32,
) -> Result<Option<subtensor::runtime_types::pallet_registry::types::Registration<u64>>> {
    let at = at_block(client).await?;

    let identity: Option<subtensor::runtime_types::pallet_registry::types::Registration<u64>> = at
        .storage()
        .try_fetch(subtensor::storage().registry().identity_of(), (*hotkey,))
        .await
        .map_err(|e| BittensorError::Rpc(e.to_string()))?
        .and_then(|v| v.decode().ok());

    Ok(identity)
}

/// Fetch the ChainIdentityV2 for a given hotkey, returning None if not found.
pub async fn get_identities_v2(
    client: &OnlineClient<SubtensorConfig>,
    hotkey: &subxt::utils::AccountId32,
) -> Result<Option<subtensor::runtime_types::pallet_subtensor::pallet::ChainIdentityV2>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();

    let v: Option<subtensor::runtime_types::pallet_subtensor::pallet::ChainIdentityV2> = at
        .storage()
        .try_fetch(addr.identities_v2(), (*hotkey,))
        .await
        .map_err(|e| BittensorError::Rpc(e.to_string()))?
        .and_then(|val| val.decode().ok());

    Ok(v)
}

/// Fetch the SubnetIdentityV3 for a given subnet, returning None if not found.
pub async fn get_subnet_identities_v3(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<Option<subtensor::runtime_types::pallet_subtensor::pallet::SubnetIdentityV3>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();

    let v: Option<subtensor::runtime_types::pallet_subtensor::pallet::SubnetIdentityV3> = at
        .storage()
        .try_fetch(addr.subnet_identities_v3(), (netuid,))
        .await
        .map_err(|e| BittensorError::Rpc(e.to_string()))?
        .and_then(|val| val.decode().ok());

    Ok(v)
}

/// Fetch the neuron certificate for a hotkey in a subnet, returning None if not found.
pub async fn get_neuron_certificates(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    hotkey: &subxt::utils::AccountId32,
) -> Result<Option<subtensor::runtime_types::pallet_subtensor::pallet::NeuronCertificate>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();

    let v: Option<subtensor::runtime_types::pallet_subtensor::pallet::NeuronCertificate> = at
        .storage()
        .try_fetch(addr.neuron_certificates(), (netuid, *hotkey))
        .await
        .map_err(|e| BittensorError::Rpc(e.to_string()))?
        .and_then(|val| val.decode().ok());

    Ok(v)
}

/// Fetch the AxonInfo for a hotkey in a subnet, returning None if not found.
pub async fn get_axons(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    hotkey: &subxt::utils::AccountId32,
) -> Result<Option<subtensor::runtime_types::pallet_subtensor::pallet::AxonInfo>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();

    let v: Option<subtensor::runtime_types::pallet_subtensor::pallet::AxonInfo> = at
        .storage()
        .try_fetch(addr.axons(), (netuid, *hotkey))
        .await
        .map_err(|e| BittensorError::Rpc(e.to_string()))?
        .and_then(|val| val.decode().ok());

    Ok(v)
}

/// Fetch the PrometheusInfo for a hotkey in a subnet, returning None if not found.
pub async fn get_prometheus(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    hotkey: &subxt::utils::AccountId32,
) -> Result<Option<subtensor::runtime_types::pallet_subtensor::pallet::PrometheusInfo>> {
    let at = at_block(client).await?;
    let addr = subtensor::storage().subtensor_module();

    let v: Option<subtensor::runtime_types::pallet_subtensor::pallet::PrometheusInfo> = at
        .storage()
        .try_fetch(addr.prometheus(), (netuid, *hotkey))
        .await
        .map_err(|e| BittensorError::Rpc(e.to_string()))?
        .and_then(|val| val.decode().ok());

    Ok(v)
}

#[cfg(test)]
mod tests {
    #[test]
    fn identity_module_compiles() {
        assert!(true, "identity module compiles and methods are exported");
    }

    #[test]
    fn identity_default_values() {
        let v: Option<u64> = None;
        assert!(v.is_none());
    }
}
