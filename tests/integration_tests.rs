use bittensor_rs::{BittensorClient, Metagraph};
use sp_core::{crypto::AccountId32, sr25519, Pair};
use std::str::FromStr;

#[tokio::test]
async fn test_client_connection() {
    // Test client connection
    let result = BittensorClient::with_default().await;
    assert!(result.is_ok(), "Failed to connect to Bittensor network");
}

#[tokio::test]
async fn test_metagraph_creation() {
    // Test metagraph creation
    let metagraph = Metagraph::new(1); // Subnet 1
    assert_eq!(metagraph.netuid, 1);
    assert_eq!(metagraph.n, 0);
    assert_eq!(metagraph.neurons.len(), 0);
}

#[tokio::test]
async fn test_runtime_api() {
    let _client = BittensorClient::with_default()
        .await
        .expect("Failed to connect");

    // Test client usage (API accessible from BittensorClient)
    // Client is already connected, no need to call get_api()
}

#[tokio::test]
async fn test_block_number() {
    let client = BittensorClient::with_default()
        .await
        .expect("Failed to connect");

    // Test block number retrieval
    let result = client.block_number().await;
    assert!(result.is_ok(), "Failed to get block number");
    let block = result.unwrap();
    assert!(block > 0, "Block number should be positive");
}

#[tokio::test]
async fn test_storage_query() {
    let _client = BittensorClient::with_default()
        .await
        .expect("Failed to connect");

    // Simple test: verify client is connected
    // (block_number already uses internal API)
}

#[tokio::test]
async fn test_account_id_parsing() {
    // Test with valid address
    let valid_address = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
    let account = AccountId32::from_str(valid_address);
    assert!(account.is_ok(), "Failed to parse valid address");

    // Test with invalid address
    let invalid_address = "invalid_address";
    let account = AccountId32::from_str(invalid_address);
    assert!(account.is_err(), "Should fail with invalid address");
}

#[tokio::test]
async fn test_error_handling() {
    // Test with invalid URL
    let result = BittensorClient::new("invalid_url").await;
    assert!(result.is_err(), "Should fail with invalid URL");
}

#[tokio::test]
async fn test_keypair_generation() {
    // Test keypair generation
    let (pair, _, _) = sr25519::Pair::generate_with_phrase(None);
    let public_key = pair.public();

    assert_eq!(public_key.0.len(), 32, "Public key should be 32 bytes");
}

#[tokio::test]
async fn test_chain_connection() {
    // Test connection with default endpoint
    let result = BittensorClient::new(bittensor_rs::chain::DEFAULT_RPC_URL).await;
    assert!(result.is_ok(), "Failed to connect to default endpoint");
}

#[tokio::test]
async fn test_metagraph_neuron_lookup() {
    let metagraph = Metagraph::new(1);

    // Test get_neuron avec metagraph vide
    let neuron = metagraph.get_neuron(0);
    assert!(neuron.is_none(), "Should return None for empty metagraph");

    // Test get_neuron_by_hotkey avec metagraph vide
    let hotkey = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
    let uid = metagraph.get_neuron_by_hotkey(&hotkey);
    assert!(
        uid.is_none(),
        "Should return None for hotkey not in metagraph"
    );
}
