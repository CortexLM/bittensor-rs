//! Serving extrinsics — publish and unpublish axon/prometheus metadata.

use bittensor_core::config::SubtensorConfig;
use subxt::OnlineClient;

use crate::generated::subtensor;

use super::{Result, TxSuccess, submit_and_watch};

/// Parameters for the `serve_axon` extrinsic.
pub struct ServeAxonParams {
    /// Subnet ID.
    pub netuid: u16,
    /// Axon version.
    pub version: u32,
    /// IP address as u128.
    pub ip: u128,
    /// Port number.
    pub port: u16,
    /// IP protocol type (4 = IPv4, 6 = IPv6).
    pub ip_type: u8,
    /// Protocol identifier.
    pub protocol: u8,
    /// Placeholder1 reserved by chain.
    pub placeholder1: u8,
    /// Placeholder2 reserved by chain.
    pub placeholder2: u8,
}

/// Publish axon endpoint metadata on chain so other neurons can discover this node.
pub async fn serve_axon(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    params: ServeAxonParams,
) -> Result<TxSuccess> {
    let call = subtensor::tx().subtensor_module().serve_axon(
        params.netuid,
        params.version,
        params.ip,
        params.port,
        params.ip_type,
        params.protocol,
        params.placeholder1,
        params.placeholder2,
    );
    submit_and_watch(client, call, signer).await
}

/// Parameters for the `set_identity` extrinsic.
pub struct SetIdentityParams {
    /// Display name.
    pub name: Vec<u8>,
    /// URL.
    pub url: Vec<u8>,
    /// GitHub repository.
    pub github_repo: Vec<u8>,
    /// Image / avatar.
    pub image: Vec<u8>,
    /// Discord handle.
    pub discord: Vec<u8>,
    /// Description.
    pub description: Vec<u8>,
    /// Additional info.
    pub additional: Vec<u8>,
}

/// Set the neuron identity metadata (name, URL, description, etc.) on chain.
pub async fn set_identity(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    params: SetIdentityParams,
) -> Result<TxSuccess> {
    let call = subtensor::tx().subtensor_module().set_identity(
        params.name,
        params.url,
        params.github_repo,
        params.image,
        params.discord,
        params.description,
        params.additional,
    );
    submit_and_watch(client, call, signer).await
}

#[cfg(test)]
mod tests {

    use crate::generated::subtensor;

    #[test]
    fn serve_axon_call_construction() {
        let _call = subtensor::tx().subtensor_module().serve_axon(
            1u16,
            1u32,
            2130706433u128,
            8090u16,
            4u8,
            0u8,
            0u8,
            0u8,
        );
    }

    #[test]
    fn set_identity_call_construction() {
        let _call = subtensor::tx().subtensor_module().set_identity(
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
        );
    }
}
