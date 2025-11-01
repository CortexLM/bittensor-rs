use crate::chain::BittensorClient;
use crate::metagraph::Metagraph;
use crate::queries::neurons;
use crate::types::{AxonInfo, PrometheusInfo};
use anyhow::{Context, Result};
use subxt::dynamic::Value;

/// Synchronize metagraph data from the chain
pub async fn sync_metagraph(client: &BittensorClient, netuid: u16) -> Result<Metagraph> {
    let mut metagraph = Metagraph::new(netuid);

    // Get current block
    metagraph.block = client.block_number().await?;

    // Use runtime API to get all neurons at once (more efficient)
    let neurons_list = neurons::neurons(client, netuid, None)
        .await
        .context("Failed to query neurons via runtime API")?;

    for neuron in neurons_list {
        metagraph.neurons.insert(neuron.uid, neuron.clone());

        // Extract axon info if available
        if let Some(ref axon) = neuron.axon_info {
            metagraph.axons.insert(neuron.uid, axon.clone());
        }
    }

    metagraph.n = metagraph.neurons.len() as u64;

    Ok(metagraph)
}

// Helper functions removed - now using neurons_storage module functions

/// Fetch axon information from storage
#[allow(dead_code)]
async fn fetch_axon_info(
    client: &BittensorClient,
    module: &str,
    entry: &str,
    keys: Vec<Value>,
) -> Option<AxonInfo> {
    // Use the implementation from neurons_storage module
    use crate::queries::neurons_storage::fetch_axon_info as fetch_axon_impl;
    fetch_axon_impl(client, module, entry, keys).await
}

/// Fetch prometheus information from storage  
#[allow(dead_code)]
async fn fetch_prometheus_info(
    client: &BittensorClient,
    module: &str,
    entry: &str,
    keys: Vec<Value>,
) -> Option<PrometheusInfo> {
    // Use the implementation from neurons_storage module
    use crate::queries::neurons_storage::fetch_prometheus_info as fetch_prometheus_impl;
    fetch_prometheus_impl(client, module, entry, keys).await
}
