//! Example: Transfer TAO from one coldkey to another on the Subtensor chain.
//!
//! Run with: cargo run -p bittensor-examples --example transfer
//!
//! NOTE: This submits a real transaction. Use testnet or a local node.

use bittensor_chain::prelude::{SubtensorClient, transfer};
use bittensor_core::config::NetworkConfig;
use bittensor_core::error::BittensorError;
use subxt_signer::sr25519::dev;

#[tokio::main]
async fn main() -> Result<(), BittensorError> {
    let client = SubtensorClient::from_config(NetworkConfig::test()).await?;

    // Use dev keys for demonstration (NEVER in production)
    let signer = dev::alice();
    let dest = dev::bob().public_key();

    let amount_rao = 1_000_000_000u64; // 1 TAO in rao

    println!("Transferring 1 TAO to Bob...");

    let result = transfer(client.rpc(), &signer, dest.into(), amount_rao).await?;

    println!("Transaction included in block {:?}", result.block_hash);
    println!("Extrinsic hash: {:?}", result.extrinsic_hash);

    Ok(())
}
