//! Runtime/metadata consistency checks
//!
//! Validates that the Finney runtime metadata contains the expected pallets
//! and storage entries that the SDK depends on. These tests ensure that
//! runtime upgrades haven't removed or renamed critical storage items.
//!
//! All tests use `BittensorClient::with_default()` which reads `BITTENSOR_RPC`
//! env var, defaulting to `wss://entrypoint-finney.opentensor.ai:443`.

use bittensor_rs::BittensorClient;

async fn connect_or_skip() -> Option<BittensorClient> {
    match BittensorClient::with_default().await {
        Ok(client) => Some(client),
        Err(err) => {
            eprintln!("Skipping test: unable to connect ({err})");
            None
        }
    }
}

// ============================================================================
// Pallet existence checks
// ============================================================================

#[tokio::test]
async fn test_metadata_has_subtensor_module() {
    let Some(client) = connect_or_skip().await else {
        return;
    };
    let metadata = client.metadata();
    assert!(
        metadata.pallet_by_name("SubtensorModule").is_some(),
        "SubtensorModule pallet must exist in runtime metadata"
    );
}

#[tokio::test]
async fn test_metadata_has_system_pallet() {
    let Some(client) = connect_or_skip().await else {
        return;
    };
    let metadata = client.metadata();
    assert!(
        metadata.pallet_by_name("System").is_some(),
        "System pallet must exist in runtime metadata"
    );
}

#[tokio::test]
async fn test_metadata_has_balances_pallet() {
    let Some(client) = connect_or_skip().await else {
        return;
    };
    let metadata = client.metadata();
    assert!(
        metadata.pallet_by_name("Balances").is_some(),
        "Balances pallet must exist in runtime metadata"
    );
}

#[tokio::test]
async fn test_metadata_has_drand_pallet() {
    let Some(client) = connect_or_skip().await else {
        return;
    };
    let metadata = client.metadata();
    assert!(
        metadata.pallet_by_name("Drand").is_some(),
        "Drand pallet must exist in runtime metadata"
    );
}

#[tokio::test]
async fn test_metadata_has_timestamp_pallet() {
    let Some(client) = connect_or_skip().await else {
        return;
    };
    let metadata = client.metadata();
    assert!(
        metadata.pallet_by_name("Timestamp").is_some(),
        "Timestamp pallet must exist in runtime metadata"
    );
}

// ============================================================================
// SubtensorModule storage entries
// ============================================================================

fn assert_storage_entry_exists(client: &BittensorClient, pallet_name: &str, entry_name: &str) {
    let metadata = client.metadata();
    let pallet = metadata
        .pallet_by_name(pallet_name)
        .unwrap_or_else(|| panic!("Pallet '{}' not found in metadata", pallet_name));
    let storage = pallet
        .storage()
        .unwrap_or_else(|| panic!("Pallet '{}' has no storage", pallet_name));
    assert!(
        storage.entry_by_name(entry_name).is_some(),
        "Storage entry '{}.{}' must exist in runtime metadata",
        pallet_name,
        entry_name
    );
}

#[tokio::test]
async fn test_subtensor_has_tempo_storage() {
    let Some(client) = connect_or_skip().await else {
        return;
    };
    assert_storage_entry_exists(&client, "SubtensorModule", "Tempo");
}

#[tokio::test]
async fn test_subtensor_has_total_networks_storage() {
    let Some(client) = connect_or_skip().await else {
        return;
    };
    assert_storage_entry_exists(&client, "SubtensorModule", "TotalNetworks");
}

#[tokio::test]
async fn test_subtensor_has_networks_added_storage() {
    let Some(client) = connect_or_skip().await else {
        return;
    };
    assert_storage_entry_exists(&client, "SubtensorModule", "NetworksAdded");
}

#[tokio::test]
async fn test_subtensor_has_commit_reveal_weights_enabled() {
    let Some(client) = connect_or_skip().await else {
        return;
    };
    assert_storage_entry_exists(&client, "SubtensorModule", "CommitRevealWeightsEnabled");
}

#[tokio::test]
async fn test_subtensor_has_reveal_period_epochs() {
    let Some(client) = connect_or_skip().await else {
        return;
    };
    assert_storage_entry_exists(&client, "SubtensorModule", "RevealPeriodEpochs");
}

#[tokio::test]
async fn test_subtensor_has_tx_rate_limit() {
    let Some(client) = connect_or_skip().await else {
        return;
    };
    assert_storage_entry_exists(&client, "SubtensorModule", "TxRateLimit");
}

#[tokio::test]
async fn test_subtensor_has_emission_storage() {
    let Some(client) = connect_or_skip().await else {
        return;
    };
    assert_storage_entry_exists(&client, "SubtensorModule", "Emission");
}

#[tokio::test]
async fn test_subtensor_has_subnetwork_n() {
    let Some(client) = connect_or_skip().await else {
        return;
    };
    assert_storage_entry_exists(&client, "SubtensorModule", "SubnetworkN");
}

// ============================================================================
// Drand storage entries
// ============================================================================

#[tokio::test]
async fn test_drand_has_last_stored_round() {
    let Some(client) = connect_or_skip().await else {
        return;
    };
    assert_storage_entry_exists(&client, "Drand", "LastStoredRound");
}

// ============================================================================
// Timestamp storage entries
// ============================================================================

#[tokio::test]
async fn test_timestamp_has_now() {
    let Some(client) = connect_or_skip().await else {
        return;
    };
    assert_storage_entry_exists(&client, "Timestamp", "Now");
}

// ============================================================================
// Live storage value sanity checks
// ============================================================================

#[tokio::test]
async fn test_total_networks_positive() {
    let Some(client) = connect_or_skip().await else {
        return;
    };
    let total = bittensor_rs::queries::subnets::total_subnets(&client)
        .await
        .expect("total_subnets query");
    assert!(total > 0, "Finney should have at least 1 subnet");
}

#[tokio::test]
async fn test_subnet_1_tempo_positive() {
    let Some(client) = connect_or_skip().await else {
        return;
    };
    let tempo = bittensor_rs::queries::subnets::tempo(&client, 1)
        .await
        .expect("tempo query")
        .unwrap_or(0);
    assert!(tempo > 0, "Subnet 1 tempo should be positive on Finney");
}

#[tokio::test]
async fn test_timestamp_reasonable() {
    let Some(client) = connect_or_skip().await else {
        return;
    };
    let ts = bittensor_rs::queries::chain_info::get_timestamp(&client)
        .await
        .expect("timestamp query");
    let jan_2024_ms = 1_704_067_200_000u64;
    assert!(
        ts > jan_2024_ms,
        "On-chain timestamp should be after Jan 2024"
    );
}

#[tokio::test]
async fn test_drand_last_round_positive() {
    let Some(client) = connect_or_skip().await else {
        return;
    };
    let round = bittensor_rs::queries::chain_info::last_drand_round(&client)
        .await
        .expect("drand round query")
        .unwrap_or(0);
    assert!(round > 0, "Drand last stored round should be positive");
}

#[tokio::test]
async fn test_block_number_positive() {
    let Some(client) = connect_or_skip().await else {
        return;
    };
    let block = client.block_number().await.expect("block number");
    assert!(block > 0, "Block number should be positive");
}
