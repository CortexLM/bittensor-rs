//! Example: Sync and iterate the metagraph for a subnet.
//!
//! Run with: cargo run -p bittensor-examples --example metagraph_sync

use bittensor_chain::prelude::SubtensorClient;
use bittensor_core::config::NetworkConfig;
use bittensor_metagraph::prelude::sync;

#[tokio::main]
async fn main() {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await.expect("connect");

    let netuid = 1u16;

    let metagraph = sync(&client, netuid).await.expect("sync metagraph");

    println!("Metagraph for subnet {netuid}: {} neurons", metagraph.n);
    println!("Total stake: {:.4} TAO", metagraph.stake.sum());

    for neuron in metagraph.neurons() {
        println!(
            "  uid={} hotkey={} stake={:.4} rank={}",
            neuron.uid,
            neuron.hotkey,
            neuron.stake.to_tao(),
            neuron.rank,
        );
    }
}
