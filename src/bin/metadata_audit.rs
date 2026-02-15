use anyhow::Result;
use hex::encode as hex_encode;
use subxt::{dynamic::Value, PolkadotConfig};

#[allow(deprecated)]
#[tokio::main]
async fn main() -> Result<()> {
    let endpoint = std::env::var("BITTENSOR_RPC")
        .unwrap_or_else(|_| "wss://entrypoint-finney.opentensor.ai:443".to_string());
    println!("Connecting to {endpoint}");

    let api = subxt::OnlineClient::<PolkadotConfig>::from_url(&endpoint).await?;
    let metadata = api.metadata();
    let metadata_hash = metadata.hasher().hash();
    println!("metadata_hash: 0x{}", hex_encode(metadata_hash));

    let runtime_version = api.runtime_version();
    println!("spec_version: {}", runtime_version.spec_version);
    println!(
        "transaction_version: {}",
        runtime_version.transaction_version
    );

    let pallets = [
        "SubtensorModule",
        "Commitments",
        "Drand",
        "System",
        "Balances",
    ];
    for pallet_name in pallets {
        let Some(pallet) = metadata.pallet_by_name(pallet_name) else {
            println!("pallet {pallet_name} not found");
            continue;
        };
        println!("\nPallet: {pallet_name}");
        println!("index: {}", pallet.index());
        if let Some(calls) = pallet.call_variants() {
            for call_variant in calls {
                let variant_index = call_variant.index();
                let call_name = call_variant.name();
                print!("  call {call_name} index {} args:", variant_index);
                for field in call_variant.fields() {
                    let field_name = field.name().map_or("_", |name| name.as_str());
                    print!(" {}:{}", field_name, field.ty().id());
                }
                println!();
            }
        }
        if let Some(storage) = pallet.storage() {
            for entry in storage.entries() {
                let entry_name = entry.name();
                let entry_ty = entry.entry_type();
                let key_ty = entry_ty.key_ty();
                let value_ty = entry_ty.value_ty();
                print!("  storage {entry_name} key:{:?} value:{}", key_ty, value_ty);
                println!();
            }
        }
    }

    let drand_round = api
        .storage()
        .at_latest()
        .await?
        .fetch(&subxt::dynamic::storage("Drand", "LastStoredRound", vec![]))
        .await?;
    if let Some(round) = drand_round {
        let round_val = round.to_value()?.remove_context();
        println!("Drand.LastStoredRound value sample: {round_val:?}");
    }

    let commit_reveal_enabled = api
        .storage()
        .at_latest()
        .await?
        .fetch(&subxt::dynamic::storage(
            "SubtensorModule",
            "CommitRevealWeightsEnabled",
            vec![Value::from(1u16)],
        ))
        .await?;
    if let Some(enabled) = commit_reveal_enabled {
        let enabled_val = enabled.to_value()?.remove_context();
        println!("CommitRevealWeightsEnabled sample (netuid=1): {enabled_val:?}");
    }

    let commit_reveal_version = api
        .storage()
        .at_latest()
        .await?
        .fetch(&subxt::dynamic::storage(
            "SubtensorModule",
            "CommitRevealWeightsVersion",
            vec![],
        ))
        .await?;
    if let Some(version) = commit_reveal_version {
        let version_val = version.to_value()?.remove_context();
        println!("CommitRevealWeightsVersion sample: {version_val:?}");
    }

    let reveal_period = api
        .storage()
        .at_latest()
        .await?
        .fetch(&subxt::dynamic::storage(
            "SubtensorModule",
            "RevealPeriodEpochs",
            vec![Value::from(1u16)],
        ))
        .await?;
    if let Some(period) = reveal_period {
        let period_val = period.to_value()?.remove_context();
        println!("RevealPeriodEpochs sample (netuid=1): {period_val:?}");
    }

    Ok(())
}
