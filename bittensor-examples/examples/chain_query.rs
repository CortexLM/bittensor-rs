//! Example: Connect to the Bittensor Subtensor chain and query neuron info.
//!
//! Run with: cargo run -p bittensor-examples --example chain_query

use bittensor_chain::prelude::SubtensorClient;
use bittensor_chain::queries;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() {
    let client = SubtensorClient::from_config(NetworkConfig::finney())
        .await
        .expect("failed to connect to Subtensor");

    println!("Connected to Subtensor");

    let total_subnets = queries::subnet::get_total_subnets(client.rpc())
        .await
        .expect("failed to query total subnets");
    println!("Total subnets: {total_subnets}");
}
