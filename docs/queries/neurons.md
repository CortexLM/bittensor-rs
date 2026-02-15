# Neuron Queries

Comprehensive documentation for querying neuron information from the Bittensor network.

## Overview

Neurons are the fundamental units of the Bittensor network. Each neuron has a unique identifier (UID) within a subnet and contains information about stake, weights, trust scores, and network endpoints.

## Data Structures

### NeuronInfo

The complete neuron information structure:

```rust
pub struct NeuronInfo {
    pub uid: u64,
    pub netuid: u16,
    pub hotkey: AccountId32,
    pub coldkey: AccountId32,
    pub stake: u128,
    pub stake_dict: HashMap<AccountId32, u128>,
    pub total_stake: u128,
    pub rank: f64,
    pub trust: f64,
    pub consensus: f64,
    pub validator_trust: f64,
    pub incentive: f64,
    pub emission: f64,
    pub dividends: f64,
    pub active: bool,
    pub last_update: u64,
    pub validator_permit: bool,
    pub version: u64,
    pub weights: Vec<(u64, u64)>,
    pub bonds: Vec<(u64, u64)>,
    pub pruning_score: u64,
    pub prometheus_info: Option<PrometheusInfo>,
    pub axon_info: Option<AxonInfo>,
}
```

### NeuronInfoLite

A lightweight version with essential fields:

```rust
pub struct NeuronInfoLite {
    pub uid: u64,
    pub netuid: u16,
    pub hotkey: AccountId32,
    pub coldkey: AccountId32,
    pub stake: u128,
    pub rank: f64,
    pub trust: f64,
    pub consensus: f64,
    pub incentive: f64,
    pub dividends: f64,
    pub emission: f64,
    pub active: bool,
    pub validator_permit: bool,
    pub last_update: u64,
    pub prometheus_info: Option<PrometheusInfo>,
    pub axon_info: Option<AxonInfo>,
    pub is_null: bool,
}
```

## Query Functions

### Get All Neurons

Retrieve all neurons for a subnet:

```rust
use bittensor_rs::queries::neurons;

// Get all neurons at latest block
let neurons = neurons::neurons(&client, netuid, None).await?;

// Get neurons at specific block
let block = 1234567;
let neurons = neurons::neurons(&client, netuid, Some(block)).await?;
```

### Get Single Neuron

Query a specific neuron by UID:

```rust
let neuron = neurons::get_neuron(&client, netuid, uid).await?;

match neuron {
    Some(info) => println!("Neuron {} found", info.uid),
    None => println!("Neuron not found"),
}
```

### Bulk Queries (Optimized)

For efficient retrieval of all neurons:

```rust
use bittensor_rs::queries::neurons_bulk;

// Fetches all neuron data in parallel
let all_neurons = neurons_bulk::neurons_bulk(&client, netuid, None).await?;
```

### Lightweight Queries

Get essential neuron information only:

```rust
// Single neuron lite
let neuron_lite = neurons::get_neuron_lite(&client, netuid, uid).await?;

// All neurons lite
let neurons_lite = neurons::neurons_lite(&client, netuid, None).await?;
```

## Storage Queries

### Individual Components

Query specific neuron properties:

```rust
use bittensor_rs::queries::neurons_storage;

// Get hotkey for a UID
let hotkey = neurons_storage::get_hotkey(&client, netuid, uid).await?;

// Get coldkey (owner)
let coldkey = neurons_storage::get_coldkey(&client, netuid, hotkey).await?;

// Get stake amount
let stake = neurons_storage::get_stake(&client, netuid, hotkey).await?;

// Get neuron metrics
let rank = neurons_storage::get_rank(&client, netuid, uid).await?;
let trust = neurons_storage::get_trust(&client, netuid, uid).await?;
let consensus = neurons_storage::get_consensus(&client, netuid, uid).await?;
```

### Vector Storage

Many metrics are stored as vectors indexed by UID:

```rust
// Get all ranks for a subnet
let ranks = neurons_storage::get_ranks(&client, netuid).await?;

// Get all emissions
let emissions = neurons_storage::get_emissions(&client, netuid).await?;
```

## Usage Examples

### Display Neuron Information

```rust
use bittensor_rs::format_rao_as_tao;

async fn display_neuron_info(client: &BittensorClient, netuid: u16, uid: u64) -> Result<()> {
    let neuron = neurons::get_neuron(client, netuid, uid).await?
        .ok_or_else(|| anyhow::anyhow!("Neuron not found"))?;
    
    println!("Neuron {} Information:", uid);
    println!("  Hotkey: {}", neuron.hotkey.to_ss58check());
    println!("  Coldkey: {}", neuron.coldkey.to_ss58check());
    println!("  Stake: {} TAO", format_rao_as_tao(neuron.stake));
    println!("  Rank: {:.4}", neuron.rank);
    println!("  Trust: {:.4}", neuron.trust);
    println!("  Consensus: {:.4}", neuron.consensus);
    println!("  Incentive: {:.4}", neuron.incentive);
    println!("  Emission: {:.2} RAO/block", neuron.emission);
    
    if let Some(axon) = &neuron.axon_info {
        println!("  Axon: {}:{}", axon.ip, axon.port);
    }
    
    Ok(())
}
```

### Find Top Validators

```rust
async fn get_top_validators(client: &BittensorClient, netuid: u16, top_n: usize) -> Result<Vec<NeuronInfo>> {
    let neurons = neurons::neurons(client, netuid, None).await?;
    
    let mut validators: Vec<_> = neurons
        .into_iter()
        .filter(|n| n.validator_permit)
        .collect();
    
    validators.sort_by(|a, b| b.stake.cmp(&a.stake));
    validators.truncate(top_n);
    
    Ok(validators)
}
```

### Monitor Neuron Changes

```rust
async fn monitor_neuron_updates(client: &BittensorClient, netuid: u16, uid: u64) -> Result<()> {
    let mut last_update = 0u64;
    
    loop {
        let neuron = neurons::get_neuron(client, netuid, uid).await?
            .ok_or_else(|| anyhow::anyhow!("Neuron not found"))?;
        
        if neuron.last_update > last_update {
            println!("Neuron {} updated at block {}", uid, neuron.last_update);
            last_update = neuron.last_update;
        }
        
        tokio::time::sleep(Duration::from_secs(12)).await;
    }
}
```

## Performance Tips

1. Use `neurons_bulk` for fetching all neurons - it's significantly faster than individual queries
2. Cache neuron data locally if querying frequently
3. Use `NeuronInfoLite` when full neuron information isn't needed
4. Query specific storage items directly for minimal data transfer

## Error Handling

Common errors and their handling:

```rust
use bittensor_rs::Error;

match neurons::get_neuron(&client, netuid, uid).await {
    Ok(Some(neuron)) => process_neuron(neuron),
    Ok(None) => println!("Neuron does not exist"),
    Err(Error::StorageNotFound) => println!("Storage entry not found"),
    Err(Error::DecodingError(e)) => println!("Failed to decode: {}", e),
    Err(e) => println!("Query failed: {}", e),
}
```

## Related Documentation

- [Subnet Queries](subnets.md) - Query subnet configuration
- [Staking Queries](staking.md) - Detailed stake information
- [Type Definitions](../types.md) - Complete structure definitions
