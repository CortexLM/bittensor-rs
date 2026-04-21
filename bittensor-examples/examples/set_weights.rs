//! Example: Set validator weights on a subnet.
//!
//! Run with: cargo run -p bittensor-examples --example set_weights
//!
//! NOTE: This submits a real transaction. Use testnet or a local node.

use bittensor_chain::prelude::{SubtensorClient, set_weights};
use bittensor_core::config::NetworkConfig;
use bittensor_core::error::BittensorError;
use subxt_signer::sr25519::dev;

#[tokio::main]
async fn main() -> Result<(), BittensorError> {
    let client = SubtensorClient::from_config(NetworkConfig::test()).await?;

    let signer = dev::alice();
    let netuid = 1u16;

    // Weight UIDs and corresponding weight values (u16 scale, max = 65535)
    let dests = vec![0u16, 1, 2];
    let weights = vec![32768u16, 16384, 16384]; // 50%, 25%, 25%
    let version_key = 0u64;

    println!("Setting weights on subnet {netuid}...");

    let result = set_weights(client.rpc(), &signer, netuid, dests, weights, version_key).await?;

    println!("Weights set in block {:?}", result.block_hash);
    println!("Extrinsic hash: {:?}", result.extrinsic_hash);

    Ok(())
}
