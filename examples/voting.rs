use anyhow::Result;
use bittensor_rs::chain::BittensorClient;

#[tokio::main]
async fn main() -> Result<()> {
    let client = BittensorClient::with_default().await?;

    // Try to find one Voting entry
    let storage = client.api().storage().at_latest().await?;
    let base = subxt::dynamic::storage("Triumvirate", "Voting", vec![]);
    let mut iter = storage.iter(base).await?;
    let mut printed = false;
    while let Some(item) = iter.next().await {
        let kv = item?;
        let value = kv.value.to_value()?.remove_context();
        println!("found_voting_entry={:?}", value);
        printed = true;
        break;
    }
    if !printed {
        println!("no voting entries found");
    }
    Ok(())
}
