use crate::chain::BittensorClient;
use crate::utils::balance_newtypes::Rao;
use anyhow::Result;
use parity_scale_codec::Encode;
use sp_core::crypto::AccountId32;
use subxt::dynamic::Value;
use subxt::ext::scale_value::{Composite, ValueDef};

/// Get balance for an account
pub async fn get_balance(client: &BittensorClient, account: &AccountId32) -> Result<Rao> {
    client
        .account_balance(account)
        .await
        .map(Rao::from)
        .map_err(|e| anyhow::anyhow!("{}", e))
}

/// Get balances for multiple accounts (batch)
pub async fn get_balances(client: &BittensorClient, accounts: &[AccountId32]) -> Result<Vec<Rao>> {
    let storage = client.api().storage().at_latest().await?;
    let _keys: Vec<_> = accounts
        .iter()
        .map(|acc| {
            subxt::dynamic::storage("System", "Account", vec![Value::from_bytes(acc.encode())])
        })
        .collect();

    let mut out = Vec::with_capacity(accounts.len());
    for acc in accounts.iter() {
        let addr =
            subxt::dynamic::storage("System", "Account", vec![Value::from_bytes(acc.encode())]);
        let res = storage.fetch(&addr).await?;
        if let Some(thunk) = res {
            let value = thunk
                .to_value()
                .map_err(|e| anyhow::anyhow!("decode: {}", e))?
                .remove_context();
            let free = extract_free_balance(&value).unwrap_or(0);
            out.push(Rao::from(free));
        } else {
            out.push(Rao::ZERO);
        }
    }
    Ok(out)
}

/// Get existential deposit
pub async fn get_existential_deposit(client: &BittensorClient) -> Result<Rao> {
    let value = client
        .query_constant("Balances", "ExistentialDeposit")
        .await?
        .ok_or_else(|| anyhow::anyhow!("Unable to retrieve existential deposit amount."))?;

    crate::utils::decoders::decode_u128(&value)
        .map(Rao::from)
        .map_err(|e| anyhow::anyhow!("Failed to decode existential deposit: {}", e))
}

fn extract_free_balance(value: &Value) -> Option<u128> {
    let fields = match &value.value {
        ValueDef::Composite(Composite::Named(fields)) => Some(fields.as_slice()),
        ValueDef::Variant(variant) => match &variant.values {
            Composite::Named(fields) => Some(fields.as_slice()),
            _ => None,
        },
        _ => None,
    }?;

    for (name, val) in fields {
        if name == "data" {
            if let Some(free) = extract_free_balance(val) {
                return Some(free);
            }
        }
        if name == "free" {
            if let Some(amount) = crate::utils::decoders::primitive::extract_u128(val) {
                return Some(amount);
            }
        }
    }
    None
}
