# Subnet Queries

Documentation for querying subnet information and configuration from the Bittensor network.

## Overview

Subnets are isolated environments within Bittensor where neurons interact. Each subnet has its own configuration, hyperparameters, and registration requirements.

## Data Structures

### SubnetInfo

Complete subnet information:

```rust
pub struct SubnetInfo {
    pub netuid: u16,
    pub network_n: u64,
    pub network_modality: u16,
    pub network_hotkey_cost: u128,
    pub network_coldkey_cost: u128,
    pub subnetwork_n: u64,
    pub max_n: u64,
    pub blocks_since_epoch: u64,
    pub tempo: u16,
    pub network_registration_allowed: bool,
    pub network_pow_allowed: bool,
    pub network_immunity_period: u16,
    pub emission_ratio: u16,
    pub max_weight_limit: f32,
    pub min_difficulty: u64,
    pub max_difficulty: u64,
    pub difficulty: u64,
    pub commit_reveal_period: u16,
    pub commit_reveal_enabled: bool,
    pub alpha_values: (u16, u16),
    pub liquid_alpha_enabled: bool,
}
```

### SubnetParameters

Subnet hyperparameters:

```rust
pub struct SubnetHyperparameters {
    pub rho: u16,
    pub kappa: u16,
    pub immunity_period: u16,
    pub min_allowed_weights: u16,
    pub max_weight_limit: f32,
    pub tempo: u16,
    pub min_difficulty: u64,
    pub max_difficulty: u64,
    pub weights_version_key: u64,
    pub weights_rate_limit: u64,
    pub adjustment_interval: u16,
    pub activity_cutoff: u16,
    pub registration_allowed: bool,
    pub target_regs_per_interval: u16,
    pub min_burn: u128,
    pub max_burn: u128,
    pub bonds_moving_avg: u64,
    pub max_regs_per_block: u16,
    pub serving_rate_limit: u64,
    pub max_validators: u16,
    pub adjustment_alpha: u64,
    pub difficulty: u64,
    pub commit_reveal_period: u16,
    pub commit_reveal_enabled: bool,
    pub alpha_values: (u16, u16),
    pub liquid_alpha_enabled: bool,
}
```

## Query Functions

### List All Subnets

Get all registered subnet IDs:

```rust
use bittensor_rs::queries::subnets;

// Get all subnet IDs
let subnet_ids = subnets::get_subnets(&client).await?;

// Get total count
let total_subnets = subnets::total_subnets(&client).await?;
```

### Get Subnet Info

Query detailed information for a specific subnet:

```rust
// Get subnet info
let subnet_info = subnets::get_subnet_info(&client, netuid).await?;

match subnet_info {
    Some(info) => {
        println!("Subnet {} has {} neurons", info.netuid, info.subnetwork_n);
        println!("Tempo: {} blocks", info.tempo);
        println!("Registration allowed: {}", info.network_registration_allowed);
    },
    None => println!("Subnet not found"),
}
```

### Query Hyperparameters

Get subnet hyperparameters:

```rust
let hyperparams = subnets::get_subnet_hyperparameters(&client, netuid).await?;

if let Some(params) = hyperparams {
    println!("Rho: {}", params.rho);
    println!("Kappa: {}", params.kappa);
    println!("Max validators: {}", params.max_validators);
    println!("Weights rate limit: {}", params.weights_rate_limit);
}
```

### Individual Parameter Queries

Query specific subnet parameters:

```rust
// Network size
let n = subnets::get_network_n(&client, netuid).await?;

// Registration status
let registration_allowed = subnets::get_network_registration_allowed(&client, netuid).await?;

// Tempo (blocks per epoch)
let tempo = subnets::get_tempo(&client, netuid).await?;

// Emission ratio
let emission_ratio = subnets::get_emission_ratio(&client, netuid).await?;

// Difficulty
let difficulty = subnets::get_difficulty(&client, netuid).await?;

// Immunity period
let immunity_period = subnets::get_immunity_period(&client, netuid).await?;

// Max allowed UIDs
let max_n = subnets::get_max_n(&client, netuid).await?;
```

### Registration Requirements

Query subnet registration costs and requirements:

```rust
// Burn requirement
let burn_cost = subnets::get_burn(&client, netuid).await?;

// Registration allowed
let reg_allowed = subnets::get_network_registration_allowed(&client, netuid).await?;

// POW allowed
let pow_allowed = subnets::get_network_pow_allowed(&client, netuid).await?;

// Max registrations per block
let max_regs = subnets::get_max_registrations_per_block(&client, netuid).await?;
```

## Usage Examples

### Display Subnet Overview

```rust
async fn display_subnet_overview(client: &BittensorClient, netuid: u16) -> Result<()> {
    let info = subnets::get_subnet_info(client, netuid).await?
        .ok_or_else(|| anyhow::anyhow!("Subnet not found"))?;
    
    println!("Subnet {} Overview", netuid);
    println!("  Neurons: {}/{}", info.subnetwork_n, info.max_n);
    println!("  Tempo: {} blocks", info.tempo);
    println!("  Emission ratio: {}", info.emission_ratio);
    println!("  Difficulty: {}", info.difficulty);
    
    if info.network_registration_allowed {
        println!("  Registration: Open (burn: {} RAO)", info.network_coldkey_cost);
    } else {
        println!("  Registration: Closed");
    }
    
    if info.commit_reveal_enabled {
        println!("  Commit-reveal period: {} blocks", info.commit_reveal_period);
    }
    
    Ok(())
}
```

### Find Open Subnets

```rust
async fn find_open_subnets(client: &BittensorClient) -> Result<Vec<u16>> {
    let all_subnets = subnets::get_subnets(client).await?;
    let mut open_subnets = Vec::new();
    
    for netuid in all_subnets {
        if let Ok(Some(true)) = subnets::get_network_registration_allowed(client, netuid).await {
            open_subnets.push(netuid);
        }
    }
    
    Ok(open_subnets)
}
```

### Monitor Subnet Activity

```rust
async fn monitor_subnet_growth(client: &BittensorClient, netuid: u16) -> Result<()> {
    let mut last_n = 0u64;
    
    loop {
        let current_n = subnets::get_network_n(client, netuid).await?
            .ok_or_else(|| anyhow::anyhow!("Subnet not found"))?;
        
        if current_n > last_n {
            println!("Subnet {} grew: {} -> {} neurons", netuid, last_n, current_n);
            last_n = current_n;
        } else if current_n < last_n {
            println!("Subnet {} shrank: {} -> {} neurons", netuid, last_n, current_n);
            last_n = current_n;
        }
        
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
```

### Compare Subnet Parameters

```rust
async fn compare_subnet_parameters(client: &BittensorClient, netuids: Vec<u16>) -> Result<()> {
    println!("Subnet | Neurons | Tempo | Max Validators | Immunity | Difficulty");
    println!("-------|---------|-------|----------------|----------|------------");
    
    for netuid in netuids {
        if let Some(params) = subnets::get_subnet_hyperparameters(client, netuid).await? {
            if let Some(n) = subnets::get_network_n(client, netuid).await? {
                println!("{:6} | {:7} | {:5} | {:14} | {:8} | {:10}",
                    netuid, n, params.tempo, params.max_validators,
                    params.immunity_period, params.difficulty
                );
            }
        }
    }
    
    Ok(())
}
```

## Performance Considerations

1. Cache subnet parameters as they change infrequently
2. Use batch queries when checking multiple subnets
3. Monitor specific parameters rather than full info when possible

## Error Handling

```rust
use bittensor_rs::Error;

match subnets::get_subnet_info(&client, netuid).await {
    Ok(Some(info)) => process_subnet_info(info),
    Ok(None) => println!("Subnet {} does not exist", netuid),
    Err(Error::StorageNotFound) => println!("Storage not initialized"),
    Err(e) => println!("Query failed: {}", e),
}
```

## Related Documentation

- [Neuron Queries](neurons.md) - Query neurons within subnets
- [Staking Queries](staking.md) - Subnet stake distribution
- [Delegate Queries](delegates.md) - Subnet delegation information
