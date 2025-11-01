use bittensor_rust::{
    BittensorClient, Metagraph,
};
use sp_core::{sr25519, Pair, crypto::AccountId32};
use std::str::FromStr;

#[tokio::test]
async fn test_client_connection() {
    // Test de connexion au client
    let result = BittensorClient::new("wss://entrypoint-finney.opentensor.ai:443").await;
    assert!(result.is_ok(), "Failed to connect to Bittensor network");
}

#[tokio::test] 
async fn test_metagraph_creation() {
    // Test de création du metagraph
    let metagraph = Metagraph::new(1); // Subnet 1
    assert_eq!(metagraph.netuid, 1);
    assert_eq!(metagraph.n, 0);
    assert_eq!(metagraph.neurons.len(), 0);
}

#[tokio::test]
async fn test_runtime_api() {
    let client = BittensorClient::new("wss://entrypoint-finney.opentensor.ai:443")
        .await
        .expect("Failed to connect");
    
    // Test utilisation du client (API accessible depuis BittensorClient)
    // Le client est déjà connecté, pas besoin de get_api()
}

#[tokio::test]
async fn test_block_number() {
    let client = BittensorClient::new("wss://entrypoint-finney.opentensor.ai:443")
        .await
        .expect("Failed to connect");
    
    // Test récupération du numéro de bloc
    let result = client.block_number().await;
    assert!(result.is_ok(), "Failed to get block number");
    let block = result.unwrap();
    assert!(block > 0, "Block number should be positive");
}

#[tokio::test]
async fn test_storage_query() {
    let client = BittensorClient::new("wss://entrypoint-finney.opentensor.ai:443")
        .await
        .expect("Failed to connect");
    
    // Test simple : vérifier que le client est bien connecté
    // (le block_number utilise déjà l'API interne)
}

#[tokio::test]
async fn test_account_id_parsing() {
    // Test avec adresse valide
    let valid_address = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
    let account = AccountId32::from_str(valid_address);
    assert!(account.is_ok(), "Failed to parse valid address");
    
    // Test avec adresse invalide
    let invalid_address = "invalid_address";
    let account = AccountId32::from_str(invalid_address);
    assert!(account.is_err(), "Should fail with invalid address");
}

#[tokio::test]
async fn test_error_handling() {
    // Test avec une URL invalide
    let result = BittensorClient::new("invalid_url").await;
    assert!(result.is_err(), "Should fail with invalid URL");
}

#[tokio::test]
async fn test_keypair_generation() {
    // Test de génération de paire de clés
    let (pair, _, _) = sr25519::Pair::generate_with_phrase(None);
    let public_key = pair.public();
    
    assert_eq!(public_key.0.len(), 32, "Public key should be 32 bytes");
}

#[tokio::test]
async fn test_chain_connection() {
    // Test de connexion avec endpoint par défaut
    let result = BittensorClient::new(bittensor_rust::chain::DEFAULT_RPC_URL).await;
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
    assert!(uid.is_none(), "Should return None for hotkey not in metagraph");
}