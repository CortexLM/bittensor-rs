# Wallet Queries

Documentation for querying wallet balances and account information from the Bittensor network.

## Overview

Wallet queries provide access to account balances, transaction history, and staking information. These queries are essential for monitoring account state and managing funds.

## Query Functions

### Balance Queries

#### Get Account Balance

```rust
use bittensor_rs::queries::wallets;

// Get total balance
let balance = wallets::get_balance(&client, &account_id).await?;
println!("Balance: {} TAO", balance as f64 / 1e9);

// Get detailed balance info
let account_info = wallets::get_account_info(&client, &account_id).await?;
if let Some(info) = account_info {
    println!("Free: {} TAO", info.data.free as f64 / 1e9);
    println!("Reserved: {} TAO", info.data.reserved as f64 / 1e9);
    println!("Frozen: {} TAO", info.data.frozen as f64 / 1e9);
}
```

#### Get Multiple Balances

```rust
// Query multiple accounts
let addresses = vec![account1, account2, account3];
let balances = wallets::get_balances(&client, &addresses).await?;

for (account, balance) in addresses.iter().zip(balances.iter()) {
    println!("{}: {} TAO", 
        account.to_ss58check(), 
        balance.as_ref().map(|b| b as f64 / 1e9).unwrap_or(0.0)
    );
}
```

### Address Conversion

#### SS58 to AccountId32

```rust
use sp_core::crypto::AccountId32;

// From SS58 string
let ss58 = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
let account_id = AccountId32::from_ss58check(ss58)?;

// With custom format
let account_id = AccountId32::from_ss58check_with_version(ss58)?;
```

#### AccountId32 to SS58

```rust
use sp_core::crypto::Ss58Codec;
use bittensor_rs::core::SS58_FORMAT;

// To Bittensor SS58 format
let ss58 = account_id.to_ss58check_with_version(
    sp_core::crypto::Ss58AddressFormat::custom(SS58_FORMAT)
);

// To default format
let ss58 = account_id.to_ss58check();
```

## Staking Queries

### Total Stake

```rust
use bittensor_rs::queries::stakes;

// Get total stake for a hotkey
let total_stake = stakes::get_total_stake(&client, &hotkey).await?;
println!("Total stake: {} TAO", total_stake as f64 / 1e9);

// Get stake for specific subnet
let stake = stakes::get_stake(&client, netuid, &hotkey).await?;
```

### Stake Distribution

```rust
// Get all stakes for a hotkey
let all_stakes = stakes::get_all_stakes_for_hotkey(&client, &hotkey).await?;

for (subnet, stake) in all_stakes {
    println!("Subnet {}: {} TAO", subnet, stake as f64 / 1e9);
}

// Get delegators for a hotkey
let delegators = stakes::get_delegators(&client, &hotkey).await?;
for (delegator, amount) in delegators {
    println!("Delegator {}: {} TAO", 
        delegator.to_ss58check(), 
        amount as f64 / 1e9
    );
}
```

## Usage Examples

### Monitor Account Activity

```rust
async fn monitor_account(client: &BittensorClient, account: &AccountId32) -> Result<()> {
    let mut last_balance = 0u128;
    
    loop {
        let balance = wallets::get_balance(client, account).await?
            .unwrap_or(0);
        
        if balance != last_balance {
            let change = if balance > last_balance {
                format!("+{} TAO", (balance - last_balance) as f64 / 1e9)
            } else {
                format!("-{} TAO", (last_balance - balance) as f64 / 1e9)
            };
            
            println!("Balance changed: {} (new total: {} TAO)", 
                change, balance as f64 / 1e9
            );
            
            last_balance = balance;
        }
        
        tokio::time::sleep(Duration::from_secs(12)).await;
    }
}
```

### Find Rich Accounts

```rust
async fn find_wealthy_accounts(
    client: &BittensorClient, 
    accounts: &[AccountId32], 
    min_balance: u128
) -> Result<Vec<(AccountId32, u128)>> {
    let mut wealthy = Vec::new();
    
    for account in accounts {
        if let Some(balance) = wallets::get_balance(client, account).await? {
            if balance >= min_balance {
                wealthy.push((account.clone(), balance));
            }
        }
    }
    
    wealthy.sort_by(|a, b| b.1.cmp(&a.1));
    Ok(wealthy)
}
```

### Calculate Portfolio Value

```rust
async fn calculate_portfolio(
    client: &BittensorClient,
    accounts: &[(AccountId32, String)]  // (account, name)
) -> Result<()> {
    let mut total = 0u128;
    
    println!("Portfolio Summary");
    println!("{:-<50}", "");
    
    for (account, name) in accounts {
        let balance = wallets::get_balance(client, account).await?
            .unwrap_or(0);
        
        let stake = stakes::get_total_stake(client, account).await?
            .unwrap_or(0);
        
        let account_total = balance + stake;
        total += account_total;
        
        println!("{:<20} {:>15.6} TAO (Balance: {:.6}, Stake: {:.6})",
            name,
            account_total as f64 / 1e9,
            balance as f64 / 1e9,
            stake as f64 / 1e9
        );
    }
    
    println!("{:-<50}", "");
    println!("{:<20} {:>15.6} TAO", "Total", total as f64 / 1e9);
    
    Ok(())
}
```

### Export Account Data

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct AccountExport {
    address: String,
    balance: u128,
    stake: u128,
    delegations: Vec<(String, u128)>,
    timestamp: u64,
}

async fn export_account_data(
    client: &BittensorClient,
    account: &AccountId32
) -> Result<AccountExport> {
    let balance = wallets::get_balance(client, account).await?
        .unwrap_or(0);
    
    let stake = stakes::get_total_stake(client, account).await?
        .unwrap_or(0);
    
    let delegators = stakes::get_delegators(client, account).await?
        .unwrap_or_default();
    
    let delegations: Vec<_> = delegators
        .into_iter()
        .map(|(d, a)| (d.to_ss58check(), a))
        .collect();
    
    let block = client.get_block_number().await?;
    
    Ok(AccountExport {
        address: account.to_ss58check(),
        balance,
        stake,
        delegations,
        timestamp: block,
    })
}
```

## Performance Considerations

1. Batch balance queries when checking multiple accounts
2. Cache balance information for frequently accessed accounts
3. Use event subscriptions for real-time balance monitoring
4. Consider rate limiting for continuous monitoring

## Error Handling

```rust
use bittensor_rs::Error;

match wallets::get_balance(&client, &account).await {
    Ok(Some(balance)) => println!("Balance: {} TAO", balance as f64 / 1e9),
    Ok(None) => println!("Account not found or zero balance"),
    Err(Error::DecodingError(e)) => println!("Failed to decode balance: {}", e),
    Err(e) => println!("Query failed: {}", e),
}
```

## Security Notes

1. Never log or display private keys
2. Use secure storage for sensitive account information
3. Validate SS58 addresses before processing
4. Consider using hardware wallets for high-value accounts

## Related Documentation

- [Staking Queries](staking.md) - Detailed staking information
- [Delegate Queries](delegates.md) - Delegation management
- [Chain Operations](../chain.md) - Transaction submission
