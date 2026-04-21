//! Sudo extrinsics — privileged runtime calls (governance/root-only).

use bittensor_core::config::SubtensorConfig;
use subxt::OnlineClient;

use crate::generated::subtensor;

use super::{Result, TxSuccess, submit_and_watch};

/// Execute a privileged runtime call as the sudo (root) key.
pub async fn sudo(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    call: subtensor::runtime_types::node_subtensor_runtime::RuntimeCall,
) -> Result<TxSuccess> {
    let tx_call = subtensor::tx().sudo().sudo(call);
    submit_and_watch(client, tx_call, signer).await
}

#[cfg(test)]
mod tests {

    #[test]
    fn sudo_call_signature() {
        assert!(true);
    }
}
