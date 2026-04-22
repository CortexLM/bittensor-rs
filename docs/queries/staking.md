# Staking Queries

Documentation for querying stake distributions, delegations, and staking operations on the Bittensor network.

## Overview

Staking is central to Bittensor's security and consensus mechanism. These queries provide detailed information about stake distributions, delegations, and validator requirements.

## Query Functions

### Total Stake Queries

#### Get Total Stake

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::staking;
use bittensor_core::balance::Balance;

// Get total stake for a hotkey across all subnets
let total_stake = staking::get_total_stake(&client, &hotkey).await?;

// Get stake for specific subnet
let subnet_stake = staking::get_stake(&client, netuid, &hotkey).await?;
```

#### Get Stake Distribution

```rust
// Get all stakes for a hotkey
let stake_distribution = staking::get_stake_distribution(&client, &hotkey).await?;

for (coldkey, amount) in stake_distribution {
    println!("Staker: {} - Amount: {} TAO", 
        coldkey.to_ss58check(), 
        Balance::from_rao(amount).to_tao()
    );
}
```

### Delegation Queries

#### Check Delegation Status

```rust
// Check if account is delegating
let is_delegating = staking::is_delegating(&client, &coldkey, &hotkey).await?;

// Get delegation amount
let delegation = staking::get_delegation(&client, &coldkey, &hotkey).await?;
```

#### Get All Delegations

```rust
// Get all delegations from a coldkey
let delegations = staking::get_delegations_for_coldkey(&client, &coldkey).await?;

for (hotkey, amount) in delegations {
    println!("Delegating to: {} - Amount: {} TAO",
        hotkey.to_ss58check(),
        Balance::from_rao(amount).to_tao()
    );
}
```

### Validator Requirements

#### Check Validator Eligibility

```rust
// Minimum stake requirement
let min_stake = staking::get_minimum_delegation(&client, netuid).await?;

// Check if hotkey meets requirements
let stake = staking::get_stake(&client, netuid, &hotkey).await?;
let is_eligible = stake >= min_stake;
```

## Advanced Queries

### Stake History

```rust
// Get stake changes over time
async fn analyze_stake_history(
    client: &SubtensorClient,
    hotkey: &AccountId32,
    blocks: Vec<u64>
) -> Result<Vec<(u64, u128)>> {
    let mut history = Vec::new();
    
    for block in blocks {
        let stake = staking::get_stake_at_block(
            client, 
            hotkey, 
            Some(block)
        ).await?;
        
        history.push((block, stake));
    }
    
    Ok(history)
}
```

### Top Stakers Analysis

```rust
async fn get_top_stakers(
    client: &SubtensorClient,
    netuid: u16,
    top_n: usize
) -> Result<Vec<(AccountId32, u128)>> {
    use bittensor_chain::queries::neuron;
    let neurons = neuron::neurons(client, netuid, None).await?;
    
    let mut stakes: Vec<_> = neurons
        .iter()
        .map(|n| (n.hotkey.clone(), n.stake))
        .collect();
    
    stakes.sort_by(|a, b| b.1.cmp(&a.1));
    stakes.truncate(top_n);
    
    Ok(stakes)
}
```

## Usage Examples

### Monitor Staking Changes

```rust
async fn monitor_stake_changes(
    client: &SubtensorClient,
    hotkey: &AccountId32,
    netuid: u16
) -> Result<()> {
    let mut last_stake = 0u128;
    
    loop {
        let current_stake = staking::get_stake(client, netuid, hotkey).await?
            .unwrap_or(0);
        
        if current_stake != last_stake {
            let change = if current_stake > last_stake {
                format!("+{}", Balance::from_rao(current_stake - last_stake).to_tao())
            } else {
                format!("-{}", Balance::from_rao(last_stake - current_stake).to_tao())
            };
            
            println!("Stake changed: {} TAO (total: {} TAO)",
                change, Balance::from_rao(current_stake).to_tao()
            );
            
            last_stake = current_stake;
        }
        
        tokio::time::sleep(Duration::from_secs(12)).await;
    }
}
```

### Calculate Staking Yields

```rust
async fn calculate_staking_yield(
    client: &SubtensorClient,
    hotkey: &AccountId32,
    netuid: u16,
    period_blocks: u64
) -> Result<f64> {
    let current_block = client.get_block_number().await?;
    let start_block = current_block.saturating_sub(period_blocks);
    
    // Get neuron info
    use bittensor_chain::queries::neuron;
    let neurons = neuron::neurons(client, netuid, None).await?;
    let neuron = neurons.iter()
        .find(|n| n.hotkey == *hotkey)
        .ok_or_else(|| anyhow::anyhow!("Neuron not found"))?;
    
    // Calculate yield from emissions
    let emission_per_block = neuron.emission;
    let total_emissions = emission_per_block.saturating_mul(period_blocks as u128);
    let stake = Balance::from_rao(neuron.stake).to_tao();
    let stake_value: f64 = stake.parse().unwrap_or(0.0);
    
    if stake_value > 0.0 {
        let total_emissions_tao = total_emissions as f64 / 1e9f64;
        let apy = (total_emissions_tao / stake_value)
            * (365.0 * 24.0 * 3600.0 / 12.0)
            / period_blocks as f64;
        Ok(apy * 100.0)
    } else {
        Ok(0.0)
    }
}
```

### Delegation Analysis

```rust
async fn analyze_delegation_distribution(
    client: &SubtensorClient,
    delegate: &AccountId32
) -> Result<()> {
    let delegations = staking::get_stake_distribution(client, delegate).await?;
    
    let total: u128 = delegations.values().sum();
    let count = delegations.len();
    let average = total / count as u128;
    
    println!("Delegation Analysis for {}", delegate.to_ss58check());
    println!("Total delegated: {} TAO", Balance::from_rao(total).to_tao());
    println!("Number of delegators: {}", count);
    println!("Average delegation: {} TAO", Balance::from_rao(average).to_tao());
    
    // Find concentration
    let mut sorted: Vec<_> = delegations.values().copied().collect();
    sorted.sort_by(|a, b| b.cmp(a));
    
    let top_10_sum: u128 = sorted.iter().take(10).sum();
    let concentration = (top_10_sum as f64 / total as f64) * 100.0;
    
    println!("Top 10 delegator concentration: {:.2}%", concentration);
    
    Ok(())
}
```

### Stake Migration Tracking

```rust
async fn track_stake_migrations(
    client: &SubtensorClient,
    addresses: Vec<AccountId32>,
    interval: Duration
) -> Result<()> {
    let mut last_stakes: HashMap<AccountId32, u128> = HashMap::new();
    
    // Initialize
    for addr in &addresses {
        let stake = staking::get_total_stake(client, addr).await?.unwrap_or(0);
        last_stakes.insert(addr.clone(), stake);
    }
    
    loop {
        tokio::time::sleep(interval).await;
        
        for addr in &addresses {
            let current = staking::get_total_stake(client, addr).await?.unwrap_or(0);
            let last = last_stakes.get(addr).copied().unwrap_or(0);
            
            if current != last {
                let change = current as i128 - last as i128;
                println!("{}: {} TAO", 
                    addr.to_ss58check(),
                    if change > 0 {
                        format!("+{}", Balance::from_rao(change as u128).to_tao())
                    } else {
                        format!("-{}", Balance::from_rao((-change) as u128).to_tao())
                    }
                );
                
                last_stakes.insert(addr.clone(), current);
            }
        }
    }
}
```

## Performance Optimization

### Caching Strategies

```rust
use std::time::{Duration, Instant};

struct StakeCache {
    cache: HashMap<(u16, AccountId32), (u128, Instant)>,
    ttl: Duration,
}

impl StakeCache {
    fn new(ttl: Duration) -> Self {
        Self {
            cache: HashMap::new(),
            ttl,
        }
    }
    
    async fn get_stake(
        &mut self,
        client: &SubtensorClient,
        netuid: u16,
        hotkey: &AccountId32
    ) -> Result<u128> {
        let key = (netuid, hotkey.clone());
        
        if let Some((stake, timestamp)) = self.cache.get(&key) {
            if timestamp.elapsed() < self.ttl {
                return Ok(*stake);
            }
        }
        
        let stake = staking::get_stake(client, netuid, hotkey).await?
            .unwrap_or(0);
        
        self.cache.insert(key, (stake, Instant::now()));
        Ok(stake)
    }
}
```

## Error Handling

```rust
use bittensor_core::error::BittensorError;

match staking::get_stake(&client, netuid, &hotkey).await {
    Ok(Some(stake)) => println!("Stake: {} TAO", Balance::from_rao(stake).to_tao()),
    Ok(None) => println!("No stake found"),
    Err(BittensorError::StorageNotFound) => println!("Storage not initialized"),
    Err(e) => println!("Query error: {}", e),
}
```

## Related Documentation

- [Neuron Queries](neurons.md) - Neuron stake information
- [Delegate Queries](delegates.md) - Delegation management
- [Wallet Queries](wallets.md) - Account balances
- [Validator Operations](../validator.md) - Staking transactions