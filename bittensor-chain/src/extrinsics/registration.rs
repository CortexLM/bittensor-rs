//! Registration extrinsics — burned registration and recycled registration.

use bittensor_core::config::SubtensorConfig;
use subxt::OnlineClient;

use crate::generated::subtensor;

use super::{Result, TxSuccess, submit_and_watch};

/// Parameters for the `register` extrinsic.
pub struct RegisterParams {
    /// Subnet ID.
    pub netuid: u16,
    /// Block number used in POW seal.
    pub block_number: u64,
    /// Nonce used in POW seal.
    pub nonce: u64,
    /// POW seal work bytes.
    pub work: Vec<u8>,
    /// Hotkey account to register.
    pub hotkey: subxt::utils::AccountId32,
    /// Coldkey account paying for registration.
    pub coldkey: subxt::utils::AccountId32,
}

/// Register a hotkey on a subnet using proof-of-work parameters.
pub async fn register(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    params: RegisterParams,
) -> Result<TxSuccess> {
    let call = subtensor::tx().subtensor_module().register(
        params.netuid,
        params.block_number,
        params.nonce,
        params.work,
        params.hotkey,
        params.coldkey,
    );
    submit_and_watch(client, call, signer).await
}

/// Register a hotkey on a subnet by burning TAO.
pub async fn burned_register(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    netuid: u16,
    hotkey: subxt::utils::AccountId32,
) -> Result<TxSuccess> {
    let call = subtensor::tx().subtensor_module().burned_register(netuid, hotkey);
    submit_and_watch(client, call, signer).await
}

/// Register a new subnet on the network.
pub async fn register_subnet(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    hotkey: subxt::utils::AccountId32,
) -> Result<TxSuccess> {
    let call = subtensor::tx().subtensor_module().register_network(hotkey);
    submit_and_watch(client, call, signer).await
}

/// Parameters for the `set_subnet_identity` extrinsic.
pub struct SetSubnetIdentityParams {
    /// Subnet ID.
    pub netuid: u16,
    /// Subnet name.
    pub subnet_name: Vec<u8>,
    /// GitHub repository URL.
    pub github_repo: Vec<u8>,
    /// Contact information.
    pub subnet_contact: Vec<u8>,
    /// Subnet URL.
    pub subnet_url: Vec<u8>,
    /// Discord handle.
    pub discord: Vec<u8>,
    /// Description of the subnet.
    pub description: Vec<u8>,
    /// Logo URL.
    pub logo_url: Vec<u8>,
    /// Additional information.
    pub additional: Vec<u8>,
}

/// Set the identity metadata for a subnet (name, GitHub repo, URL, etc.).
pub async fn set_subnet_identity(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    params: SetSubnetIdentityParams,
) -> Result<TxSuccess> {
    let call = subtensor::tx().subtensor_module().set_subnet_identity(
        params.netuid,
        params.subnet_name,
        params.github_repo,
        params.subnet_contact,
        params.subnet_url,
        params.discord,
        params.description,
        params.logo_url,
        params.additional,
    );
    submit_and_watch(client, call, signer).await
}

#[cfg(test)]
mod tests {

    use crate::generated::subtensor;

    #[test]
    fn register_call_construction() {
        let hotkey = subxt::utils::AccountId32::from([1u8; 32]);
        let coldkey = subxt::utils::AccountId32::from([2u8; 32]);
        let _call = subtensor::tx().subtensor_module().register(
            1u16,
            100u64,
            0u64,
            vec![0u8; 32],
            hotkey,
            coldkey,
        );
    }

    #[test]
    fn burned_register_call_construction() {
        let hotkey = subxt::utils::AccountId32::from([1u8; 32]);
        let _call = subtensor::tx().subtensor_module().burned_register(1u16, hotkey);
    }

    #[test]
    fn register_network_call_construction() {
        let hotkey = subxt::utils::AccountId32::from([1u8; 32]);
        let _call = subtensor::tx().subtensor_module().register_network(hotkey);
    }
}
