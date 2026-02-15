# Delegate Queries

Documentation for querying delegate information and voting data from the Bittensor network.

## Overview

Delegates are special accounts that can receive stake delegations from other accounts. They participate in network governance and earn rewards based on their performance and total delegated stake.

## Data Structures

### DelegateInfo

Complete delegate information:

```rust
pub struct DelegateInfo {
    pub delegate_address: AccountId32,
    pub take: f64,
    pub nominators: Vec<(AccountId32, u128)>,
    pub total_stake: u128,
    pub owner: AccountId32,
    pub registrations: Vec<u16>,
    pub validator_permits: Vec<u16>,
    pub return_per_1000: u128,
    pub total_daily_return: u128,
}
```

### DelegateDetails

Additional delegate metadata:

```rust
pub struct DelegateDetails {
    pub name: String,
    pub url: String,
    pub description: String,
    pub image: String,
}
```

## Query Functions

### List All Delegates

Get all registered delegates:

```rust
use bittensor_rs::queries::delegates;
use bittensor_rs::format_rao_as_tao;

// Get all delegate addresses
let delegate_addresses = delegates::get_delegates(&client).await?;

// Get full delegate information for all
let all_delegates = delegates::get_all_delegates_info(&client).await?;
```

### Get Specific Delegate

Query information for a specific delegate:

```rust
// By SS58 address
let delegate_info = delegates::get_delegate_by_ss58(&client, "5F4tQ...").await?;

// By AccountId32
let delegate_info = delegates::get_delegate(&client, &account_id).await?;

match delegate_info {
    Some(info) => {
        println!("Delegate: {}", info.delegate_address.to_ss58check());
        println!("Take: {}%", info.take * 100.0);
        println!("Total stake: {} TAO", format_rao_as_tao(info.total_stake));
        println!("Nominators: {}", info.nominators.len());
    },
    None => println!("Delegate not found"),
}
```

### Query Delegate Take

Get the commission percentage for a delegate:

```rust
let take = delegates::get_delegate_take(&client, &delegate_address).await?;

if let Some(take_percent) = take {
    println!("Delegate take: {}%", take_percent);
}
```

### Query Nominators

Get all accounts delegating to a specific delegate:

```rust
let nominators = delegates::get_nominators_for_delegate(&client, &delegate_address).await?;

for (nominator, stake) in nominators {
    println!("Nominator: {} - Stake: {} TAO", 
        nominator.to_ss58check(), 
        format_rao_as_tao(stake)
    );
}
```

### Check Delegation

Verify if an account is delegating to a specific delegate:

```rust
let is_delegating = delegates::is_delegating(&client, &delegator, &delegate).await?;

if is_delegating {
    println!("Account is delegating to this delegate");
}
```

## Usage Examples

### Display Top Delegates

```rust
async fn display_top_delegates(client: &BittensorClient, top_n: usize) -> Result<()> {
    let mut delegates = delegates::get_all_delegates_info(client).await?;
    
    // Sort by total stake
    delegates.sort_by(|a, b| b.total_stake.cmp(&a.total_stake));
    delegates.truncate(top_n);
    
    println!("Top {} Delegates by Total Stake", top_n);
    println!("{:<6} {:<48} {:>12} {:>6} {:>10}", "Rank", "Address", "Stake (TAO)", "Take%", "Nominators");
    println!("{}", "-".repeat(90));
    
    for (i, delegate) in delegates.iter().enumerate() {
        println!("{:<6} {:<48} {:>12} {:>6.2} {:>10}",
            i + 1,
            delegate.delegate_address.to_ss58check(),
            format_rao_as_tao(delegate.total_stake),
            delegate.take * 100.0,
            delegate.nominators.len()
        );
    }
    
    Ok(())
}
```

### Find Best Return Delegates

```rust
async fn find_best_return_delegates(client: &BittensorClient) -> Result<Vec<DelegateInfo>> {
    let delegates = delegates::get_all_delegates_info(client).await?;
    
    let mut active_delegates: Vec<_> = delegates
        .into_iter()
        .filter(|d| d.total_stake > 1_000_000_000_000) // Min 1000 TAO
        .collect();
    
    // Sort by return per 1000 TAO
    active_delegates.sort_by(|a, b| b.return_per_1000.cmp(&a.return_per_1000));
    
    Ok(active_delegates)
}
```

### Monitor Delegate Performance

```rust
async fn monitor_delegate_performance(
    client: &BittensorClient, 
    delegate_address: &AccountId32
) -> Result<()> {
    let mut last_stake = 0u128;
    
    loop {
        let info = delegates::get_delegate(client, delegate_address).await?
            .ok_or_else(|| anyhow::anyhow!("Delegate not found"))?;
        
        if info.total_stake != last_stake {
            let change = if info.total_stake > last_stake {
                format!("+{}", format_rao_as_tao(info.total_stake - last_stake))
            } else {
                format!("-{}", format_rao_as_tao(last_stake - info.total_stake))
            };
            
            println!("Delegate stake changed: {} TAO (total: {} TAO)",
                change, format_rao_as_tao(info.total_stake)
            );
            
            last_stake = info.total_stake;
        }
        
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
```

### Analyze Delegation Distribution

```rust
async fn analyze_delegation_distribution(client: &BittensorClient) -> Result<()> {
    let delegates = delegates::get_all_delegates_info(client).await?;
    
    let total_delegated: u128 = delegates.iter()
        .map(|d| d.total_stake)
        .sum();
    
    let avg_nominators = delegates.iter()
        .map(|d| d.nominators.len())
        .sum::<usize>() as f64 / delegates.len() as f64;
    
    println!("Delegation Statistics:");
    println!("  Total delegates: {}", delegates.len());
    println!("  Total delegated: {} TAO", format_rao_as_tao(total_delegated));
    println!("  Average nominators per delegate: {:.2}", avg_nominators);
    
    // Find concentration
    let top_10_stake: u128 = delegates.iter()
        .take(10)
        .map(|d| d.total_stake)
        .sum();
    
    let concentration = top_10_stake as f64 / total_delegated as f64 * 100.0;
    println!("  Top 10 delegate concentration: {:.2}%", concentration);
    
    Ok(())
}
```

## Voting and Governance

### Senate Membership

Check if a delegate is in the senate:

```rust
let senate_members = delegates::get_senate_members(&client).await?;

let is_senate_member = senate_members
    .iter()
    .any(|member| member == delegate_address);
```

### Voting History

Query voting records:

```rust
// Get all votes for a proposal
let votes = voting::get_votes(&client, proposal_id).await?;

// Check delegate's vote
let delegate_vote = votes
    .iter()
    .find(|(voter, _)| voter == delegate_address);
```

## Performance Tips

1. Cache delegate information as it changes less frequently than neuron data
2. Use `get_all_delegates_info` and filter locally instead of multiple queries
3. Monitor only stake changes rather than full info for efficiency

## Error Handling

```rust
use bittensor_rs::Error;

match delegates::get_delegate(&client, &address).await {
    Ok(Some(delegate)) => process_delegate(delegate),
    Ok(None) => println!("Not a registered delegate"),
    Err(Error::DecodingError(e)) => println!("Failed to decode: {}", e),
    Err(e) => println!("Query failed: {}", e),
}
```

## Related Documentation

- [Staking Queries](staking.md) - Detailed stake and delegation queries
- [Wallet Queries](wallets.md) - Account balance information
- [Runtime Queries](runtime.md) - Commit-reveal and ownership helpers
- [Metagraph Queries](metagraph.md) - Subnet snapshot data
- [Wallet Queries](wallets.md) - Account balance information
- [Neuron Queries](neurons.md) - Validator information