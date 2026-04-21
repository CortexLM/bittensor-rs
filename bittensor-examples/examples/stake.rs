//! Example: Stake TAO to a hotkey on the Subtensor chain.
//!
//! Run with: cargo run -p bittensor-examples --example stake
//!
//! NOTE: This submits a real transaction. Use testnet or a local node.

use bittensor_chain::prelude::{SubtensorClient, add_stake};
use bittensor_core::config::NetworkConfig;
use bittensor_core::error::BittensorError;
use subxt_signer::sr25519::dev;

#[tokio::main]
async fn main() -> Result<(), BittensorError> {
    let client = SubtensorClient::from_config(NetworkConfig::test()).await?;

    let signer = dev::alice();
    let hotkey = dev::bob().public_key();
    let netuid = 1u16;
    let amount_rao = 1_000_000_000u64; // 1 TAO

    println!("Staking 1 TAO to Bob's hotkey on subnet {netuid}...");

    let result = add_stake(client.rpc(), &signer, hotkey.into(), netuid, amount_rao).await?;

    println!("Stake tx in block {:?}", result.block_hash);
    println!("Extrinsic hash: {:?}", result.extrinsic_hash);

    Ok(())
}
