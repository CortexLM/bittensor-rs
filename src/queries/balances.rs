use crate::chain::BittensorClient;
use anyhow::Result;
use parity_scale_codec::Encode;
use sp_core::crypto::AccountId32;
use subxt::dynamic::Value;

/// Get balance for an account
pub async fn get_balance(client: &BittensorClient, account: &AccountId32) -> Result<u128> {
    client
        .account_balance(account)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))
}

/// Get balances for multiple accounts (batch)
pub async fn get_balances(client: &BittensorClient, accounts: &[AccountId32]) -> Result<Vec<u128>> {
    let storage = client.api().storage().at_latest().await?;
    let _keys: Vec<_> = accounts
        .iter()
        .map(|acc| {
            subxt::dynamic::storage("System", "Account", vec![Value::from_bytes(&acc.encode())])
        })
        .collect();

    // No fetch_many in current subxt version; perform sequential fetches
    let mut out = Vec::with_capacity(accounts.len());
    for acc in accounts.iter() {
        let addr =
            subxt::dynamic::storage("System", "Account", vec![Value::from_bytes(&acc.encode())]);
        let res = storage.fetch(&addr).await?;
        if let Some(thunk) = res {
            let value = thunk
                .to_value()
                .map_err(|e| anyhow::anyhow!("decode: {}", e))?
                .remove_context();
            let s = format!("{:?}", value);
            let mut free: u128 = 0;
            if let Some(pos) = s.find("free") {
                let after = &s[pos + 4..];
                let trimmed = after.trim_start_matches(':').trim_start();
                let mut num = String::new();
                for ch in trimmed.chars() {
                    if ch.is_ascii_digit() {
                        num.push(ch);
                    } else {
                        break;
                    }
                }
                if !num.is_empty() {
                    free = num.parse::<u128>().unwrap_or(0);
                }
            }
            out.push(free);
        } else {
            out.push(0);
        }
    }
    Ok(out)
}

/// Get existential deposit
pub async fn get_existential_deposit(client: &BittensorClient) -> Result<u128> {
    // Query constant from metadata (same as Bittensor Python)
    let value = client
        .query_constant("Balances", "ExistentialDeposit")
        .await?
        .ok_or_else(|| anyhow::anyhow!("Unable to retrieve existential deposit amount."))?;

    // Decode the constant value as u128
    crate::utils::decoders::decode_u128(&value)
        .map_err(|e| anyhow::anyhow!("Failed to decode existential deposit: {}", e))
}
