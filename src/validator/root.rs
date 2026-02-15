use crate::chain::{BittensorClient, BittensorSigner, ExtrinsicWait};
use anyhow::Result;
use sp_core::crypto::AccountId32;

const ROOT_NETUID: u16 = 0;

/// Register on root network (netuid 0)
pub async fn root_register(
    client: &BittensorClient,
    signer: &BittensorSigner,
    hotkey: &AccountId32,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    // Root register is just register on netuid 0
    crate::validator::registration::register(client, signer, ROOT_NETUID, hotkey, wait_for).await
}

/// Set root weights (weights on root network)
pub async fn root_set_weights(
    client: &BittensorClient,
    signer: &BittensorSigner,
    uids: &[u16],
    weights: &[u16],
    version_key: u64,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    crate::validator::weights::set_weights(
        client,
        signer,
        ROOT_NETUID,
        uids,
        weights,
        version_key,
        wait_for,
    )
    .await
}
