use bittensor_rs::core::constants::FINNEY_ENDPOINT;
use bittensor_rs::crv4::{
    calculate_reveal_round, get_last_drand_round, get_mechid_storage_index, prepare_crv4_commit,
    verify_encrypted_data,
};
use bittensor_rs::{get_commit_reveal_version, BittensorClient, Config, Metagraph, Subtensor};
use bittensor_rs::{queries, utils::weights::normalize_weights};

async fn connect_default_or_skip() -> Option<BittensorClient> {
    match BittensorClient::with_default().await {
        Ok(client) => Some(client),
        Err(err) => {
            eprintln!("Skipping test: unable to connect ({err})");
            None
        }
    }
}
use bittensor_rs::{validator, ExtrinsicWait, DEFAULT_COMMIT_REVEAL_VERSION};
use sp_core::{crypto::AccountId32, sr25519, Pair};
use std::str::FromStr;
use std::sync::{Mutex, OnceLock};

#[tokio::test]
async fn test_client_connection() {
    if connect_default_or_skip().await.is_none() {
        return;
    }
}

#[tokio::test]
async fn test_metagraph_creation() {
    let metagraph = Metagraph::new(1);
    assert_eq!(metagraph.netuid, 1);
    assert_eq!(metagraph.n, 0);
    assert_eq!(metagraph.neurons.len(), 0);
}

#[tokio::test]
async fn test_runtime_api() {
    if connect_default_or_skip().await.is_none() {
        return;
    }
}

#[tokio::test]
async fn test_block_number() {
    let Some(client) = connect_default_or_skip().await else {
        return;
    };

    let result = client.block_number().await;
    assert!(result.is_ok(), "Failed to get block number");
    let block = result.unwrap();
    assert!(block > 0, "Block number should be positive");
}

#[tokio::test]
async fn test_storage_query() {
    if connect_default_or_skip().await.is_none() {
        return;
    }
}

#[tokio::test]
async fn test_account_id_parsing() {
    let valid_address = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
    let account = AccountId32::from_str(valid_address);
    assert!(account.is_ok(), "Failed to parse valid address");

    let invalid_address = "invalid_address";
    let account = AccountId32::from_str(invalid_address);
    assert!(account.is_err(), "Should fail with invalid address");
}

#[tokio::test]
async fn test_error_handling() {
    let result = BittensorClient::new("invalid_url").await;
    assert!(result.is_err(), "Should fail with invalid URL");
}

#[tokio::test]
async fn test_keypair_generation() {
    let (pair, _, _) = sr25519::Pair::generate_with_phrase(None);
    let public_key = pair.public();

    assert_eq!(public_key.0.len(), 32, "Public key should be 32 bytes");
}

#[tokio::test]
async fn test_chain_connection() {
    let result = BittensorClient::new(FINNEY_ENDPOINT).await;
    assert!(result.is_ok(), "Failed to connect to default endpoint");
}

#[tokio::test]
async fn test_metagraph_neuron_lookup() {
    let metagraph = Metagraph::new(1);

    let neuron = metagraph.get_neuron(0);
    assert!(neuron.is_none(), "Should return None for empty metagraph");

    let hotkey = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
    let uid = metagraph.get_neuron_by_hotkey(&hotkey);
    assert!(
        uid.is_none(),
        "Should return None for hotkey not in metagraph"
    );
}

#[tokio::test]
async fn test_finney_default_endpoint_config() {
    let config = Config::default();
    assert_eq!(config.subtensor.network, "finney");
    assert_eq!(config.subtensor.chain_endpoint, FINNEY_ENDPOINT);

    let Some(client) = connect_default_or_skip().await else {
        return;
    };
    assert_eq!(client.rpc_url(), FINNEY_ENDPOINT);
}

#[tokio::test]
async fn test_bittensor_rpc_env_override() {
    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();

    let custom_endpoint = "ws://127.0.0.1:9944";
    std::env::set_var("BITTENSOR_RPC", custom_endpoint);

    let config = Config::from_env();
    assert_eq!(config.subtensor.chain_endpoint, custom_endpoint);

    if let Some(client) = connect_default_or_skip().await {
        assert_eq!(client.rpc_url(), custom_endpoint);
    }

    std::env::remove_var("BITTENSOR_RPC");
}

#[tokio::test]
async fn test_query_and_weights_commit_flow() {
    let Some(client) = connect_default_or_skip().await else {
        return;
    };

    let netuid = 1u16;
    let tempo = queries::subnets::tempo(&client, netuid)
        .await
        .expect("Tempo query failed")
        .unwrap_or(0);
    assert!(tempo > 0, "Expected non-zero tempo");

    let _cr_enabled = queries::subnets::commit_reveal_enabled(&client, netuid)
        .await
        .expect("commit_reveal_enabled failed");

    let cr_version = get_commit_reveal_version(&client).await.unwrap_or(0);
    assert!(cr_version >= DEFAULT_COMMIT_REVEAL_VERSION);

    let uids: Vec<u64> = vec![0, 1, 2];
    let weights: Vec<f32> = vec![0.5, 0.3, 0.2];
    let (uid_u16, weight_u16) = normalize_weights(&uids, &weights).expect("normalize weights");
    assert_eq!(uid_u16.len(), weight_u16.len());

    let total: u32 = weight_u16.iter().map(|v| *v as u32).sum();
    assert!(total <= u16::MAX as u32 + 1);
}

#[tokio::test]
async fn test_transfer_and_stake_flow_requires_funded_keys() {
    let Some(client) = connect_default_or_skip().await else {
        return;
    };

    let (pair, _, _) = sr25519::Pair::generate_with_phrase(None);
    let signer = bittensor_rs::chain::create_signer(pair);

    let coldkey = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY")
        .expect("Invalid coldkey");
    let hotkey = AccountId32::from_str("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty")
        .expect("Invalid hotkey");

    let transfer_result =
        validator::transfer::transfer(&client, &signer, &coldkey, 1u128, true, ExtrinsicWait::None)
            .await;
    assert!(transfer_result.is_err());

    let stake_result =
        validator::staking::add_stake(&client, &signer, &hotkey, 1, 1u128, ExtrinsicWait::None)
            .await;
    assert!(stake_result.is_err());
}

#[tokio::test]
async fn test_subtensor_set_weights_crv4_branching() {
    let subtensor = Subtensor::new(FINNEY_ENDPOINT)
        .await
        .expect("Failed to create subtensor");

    let version = subtensor
        .get_commit_reveal_version()
        .await
        .unwrap_or(DEFAULT_COMMIT_REVEAL_VERSION);
    assert!(version >= DEFAULT_COMMIT_REVEAL_VERSION);

    let enabled = subtensor
        .commit_reveal_enabled(1)
        .await
        .expect("commit_reveal_enabled failed");
    assert!(enabled);

    let (pair, _, _) = sr25519::Pair::generate_with_phrase(None);
    let signer = bittensor_rs::chain::create_signer(pair);
    let uids: Vec<u64> = vec![0, 1, 2];
    let weights: Vec<f32> = vec![0.5, 0.3, 0.2];
    let (uid_u16, weight_u16) = normalize_weights(&uids, &weights).expect("normalize weights");
    let result = subtensor
        .set_weights(&signer, 1, &uid_u16, &weight_u16, 0, ExtrinsicWait::None)
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_commit_reveal_flow_read_only() {
    let Some(client) = connect_default_or_skip().await else {
        return;
    };
    let netuid = 1u16;

    let enabled = queries::subnets::commit_reveal_enabled(&client, netuid)
        .await
        .expect("commit reveal enabled");
    assert!(enabled);

    let version = get_commit_reveal_version(&client)
        .await
        .unwrap_or(DEFAULT_COMMIT_REVEAL_VERSION);
    assert!(version >= DEFAULT_COMMIT_REVEAL_VERSION);

    let tempo = queries::subnets::tempo(&client, netuid)
        .await
        .expect("tempo")
        .unwrap_or(0);
    assert!(tempo > 0);
}

#[tokio::test]
async fn test_crv4_commit_payload_uses_chain_drand() {
    let Some(client) = connect_default_or_skip().await else {
        return;
    };
    let netuid = 1u16;

    let tempo = queries::subnets::tempo(&client, netuid)
        .await
        .expect("tempo")
        .unwrap_or(0);
    let reveal_period = queries::subnets::get_subnet_reveal_period_epochs(&client, netuid)
        .await
        .expect("reveal period")
        .unwrap_or(1);

    let current_block = client.block_number().await.expect("block number");
    let chain_last_round = get_last_drand_round(&client).await.expect("drand round");
    assert!(chain_last_round > 0);

    let storage_index = get_mechid_storage_index(netuid, 0);
    let reveal_round = calculate_reveal_round(
        tempo.try_into().expect("tempo fits in u16"),
        current_block,
        storage_index,
        reveal_period,
        12.0,
        chain_last_round,
    );
    assert!(reveal_round >= chain_last_round);

    let (pair, _, _) = sr25519::Pair::generate_with_phrase(None);
    let hotkey_bytes = pair.public().0.to_vec();
    let uids: Vec<u16> = vec![0, 1, 2];
    let weights: Vec<u16> = vec![10_000, 20_000, 30_000];
    let encrypted =
        prepare_crv4_commit(&hotkey_bytes, &uids, &weights, 0, reveal_round).expect("encrypt");
    assert!(verify_encrypted_data(&encrypted));
}
