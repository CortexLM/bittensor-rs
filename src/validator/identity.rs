use crate::chain::{BittensorClient, BittensorSigner, ExtrinsicWait};
use anyhow::Result;
use subxt::dynamic::Value;

const SUBTENSOR_MODULE: &str = "SubtensorModule";

/// Set on-chain identity for the signer's account.
///
/// Subtensor pallet dispatch: `set_identity(name, url, description, image, discord, additional)`
///
/// # Arguments
/// * `client` — The Bittensor RPC client.
/// * `signer` — The signing keypair.
/// * `name` — Display name.
/// * `url` — URL associated with the identity.
/// * `description` — Short description.
/// * `image` — Image URL or hash.
/// * `discord` — Discord handle.
/// * `additional` — Additional metadata bytes.
/// * `wait_for` — How long to wait for on-chain inclusion.
#[allow(clippy::too_many_arguments)]
pub async fn set_identity(
    client: &BittensorClient,
    signer: &BittensorSigner,
    name: &str,
    url: &str,
    description: &str,
    image: &str,
    discord: &str,
    additional: &str,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args = vec![
        Value::from_bytes(name.as_bytes()),
        Value::from_bytes(url.as_bytes()),
        Value::from_bytes(description.as_bytes()),
        Value::from_bytes(image.as_bytes()),
        Value::from_bytes(discord.as_bytes()),
        Value::from_bytes(additional.as_bytes()),
    ];

    client
        .submit_extrinsic(SUBTENSOR_MODULE, "set_identity", args, signer, wait_for)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to set identity: {}", e))
}

/// Set identity for a subnet.
///
/// Subtensor pallet dispatch: `set_subnet_identity(netuid, subnet_name, github_repo, subnet_contact)`
///
/// # Arguments
/// * `client` — The Bittensor RPC client.
/// * `signer` — The signing keypair (must be subnet owner).
/// * `netuid` — The subnet ID.
/// * `subnet_name` — Display name for the subnet.
/// * `github_repo` — GitHub repository URL.
/// * `subnet_contact` — Contact information.
/// * `wait_for` — How long to wait for on-chain inclusion.
pub async fn set_subnet_identity(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    subnet_name: &str,
    github_repo: &str,
    subnet_contact: &str,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args = vec![
        Value::from(netuid),
        Value::from_bytes(subnet_name.as_bytes()),
        Value::from_bytes(github_repo.as_bytes()),
        Value::from_bytes(subnet_contact.as_bytes()),
    ];

    client
        .submit_extrinsic(
            SUBTENSOR_MODULE,
            "set_subnet_identity",
            args,
            signer,
            wait_for,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to set subnet identity: {}", e))
}

/// Clear the on-chain identity for the signer's account.
///
/// Subtensor pallet dispatch: `clear_identity()`
///
/// # Arguments
/// * `client` — The Bittensor RPC client.
/// * `signer` — The signing keypair.
/// * `wait_for` — How long to wait for on-chain inclusion.
pub async fn clear_identity(
    client: &BittensorClient,
    signer: &BittensorSigner,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let args: Vec<Value> = vec![];

    client
        .submit_extrinsic(SUBTENSOR_MODULE, "clear_identity", args, signer, wait_for)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to clear identity: {}", e))
}
