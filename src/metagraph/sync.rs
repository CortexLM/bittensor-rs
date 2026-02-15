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

    let mut neurons_list = neurons_list;
    neurons_list.sort_by_key(|n| n.uid);

    metagraph.n = neurons_list.len() as u64;
    metagraph.hotkeys = Vec::with_capacity(neurons_list.len());
    metagraph.coldkeys = Vec::with_capacity(neurons_list.len());
    metagraph.validator_permit = Vec::with_capacity(neurons_list.len());
    metagraph.active = Vec::with_capacity(neurons_list.len());

    for neuron in neurons_list {
        metagraph.hotkeys.push(neuron.hotkey.clone());
        metagraph.coldkeys.push(neuron.coldkey.clone());
        metagraph.validator_permit.push(neuron.validator_permit);
        metagraph.active.push(neuron.active);

        metagraph.neurons.insert(neuron.uid, neuron.clone());

        // Extract axon info if available
        if let Some(ref axon) = neuron.axon_info {
            metagraph.axons.insert(neuron.uid, axon.clone());
        }
    }

    Ok(metagraph)
}

// Helper functions use implementations from neurons module

/// Fetch axon information from storage
#[allow(dead_code)]
async fn fetch_axon_info(
    client: &BittensorClient,
    module: &str,
    entry: &str,
    keys: Vec<Value>,
) -> Option<AxonInfo> {
    // Use the implementation from neurons module
    use crate::queries::neurons::fetch_axon_info as fetch_axon_impl;
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
    // Use the implementation from neurons module
    use crate::queries::neurons::fetch_prometheus_info as fetch_prometheus_impl;
    fetch_prometheus_impl(client, module, entry, keys).await
}
