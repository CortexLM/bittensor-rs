//! Extrinsic (transaction) submission methods for the Bittensor Subtensor chain.
//!
//! Each method follows the subxt 0.50.0 pattern:
//! 1. Construct a typed call via `subtensor::tx().pallet().method(params)`
//! 2. Submit with `client.tx().sign_and_submit_then_watch_default(&call, &signer)`
//! 3. Wait for finalization and extract block hash / extrinsic hash

pub mod children;
pub mod coldkey_swap;
pub mod proxy;
pub mod registration;
pub mod root;
pub mod serving;
pub mod staking;
pub mod sudo;
pub mod take;
pub mod transfer;
pub mod weights;

pub use children::*;
pub use coldkey_swap::*;
pub use proxy::*;
pub use registration::*;
pub use root::*;
pub use serving::*;
pub use staking::*;
pub use sudo::*;
pub use take::*;
pub use transfer::*;
pub use weights::*;

use bittensor_core::config::SubtensorConfig;
use bittensor_core::error::BittensorError;
use subxt::OnlineClient;

type Result<T> = std::result::Result<T, BittensorError>;

/// Result of a successfully finalized extrinsic.
#[derive(Debug, Clone)]
pub struct TxSuccess {
    /// Hash of the block that included the extrinsic.
    pub block_hash: subxt::utils::H256,
    /// Hash of the extrinsic itself.
    pub extrinsic_hash: subxt::utils::H256,
}

async fn submit_and_watch(
    client: &OnlineClient<SubtensorConfig>,
    call: impl subxt::tx::Payload,
    signer: &subxt_signer::sr25519::Keypair,
) -> Result<TxSuccess> {
    let mut tx = client.tx().await.map_err(|e| BittensorError::Rpc(e.to_string()))?;

    let watch = tx
        .sign_and_submit_then_watch_default(&call, signer)
        .await
        .map_err(|e| BittensorError::Transaction(e.to_string()))?;

    let ext_hash = watch.extrinsic_hash();

    let in_block =
        watch.wait_for_finalized().await.map_err(|e| BittensorError::Transaction(e.to_string()))?;

    in_block.wait_for_success().await.map_err(|e| BittensorError::Transaction(e.to_string()))?;

    Ok(TxSuccess { block_hash: in_block.block_hash(), extrinsic_hash: ext_hash })
}

/// Convert an `AccountId32` into a `MultiAddress::Id` variant,
/// which is the address format required by most Subtensor extrinsics.
pub(crate) fn to_multi_address(
    account: subxt::utils::AccountId32,
) -> subxt::utils::MultiAddress<subxt::utils::AccountId32, ()> {
    subxt::utils::MultiAddress::Id(account)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tx_success_construction() {
        let success = TxSuccess {
            block_hash: subxt::utils::H256::zero(),
            extrinsic_hash: subxt::utils::H256::zero(),
        };
        assert_eq!(success.block_hash, subxt::utils::H256::zero());
        assert_eq!(success.extrinsic_hash, subxt::utils::H256::zero());
    }

    #[test]
    fn to_multi_address_works() {
        let account = subxt::utils::AccountId32::from([0u8; 32]);
        let addr = to_multi_address(account.clone());
        match addr {
            subxt::utils::MultiAddress::Id(id) => assert_eq!(id, account),
            _ => panic!("expected Id variant"),
        }
    }
}
