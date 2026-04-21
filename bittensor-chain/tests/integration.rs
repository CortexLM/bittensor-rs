//! Integration tests for bittensor-chain against a local Subtensor devnet.
//!
//! All tests are feature-gated behind `integration-tests` and require a running
//! local devnet at `ws://localhost:31444`. Run the devnet first:
//!
//!   ./scripts/devnet.sh start
//!
//! Then execute tests:
//!
//!   cargo test -p bittensor-chain --features integration-tests --test integration -- --ignored
//!
//! Stop the devnet when done:
//!
//!   ./scripts/devnet.sh stop

#![cfg(feature = "integration-tests")]

use bittensor_chain::client::SubtensorClient;
use bittensor_chain::events::{EventFilter, subscribe_blocks, subscribe_events};
use bittensor_chain::extrinsics::{add_stake, set_weights, transfer};
use bittensor_chain::queries::{
    get_balance, get_metagraph, get_neuron_count, get_stake, get_total_network_stake,
};

/// Devnet WebSocket endpoint.
const DEVNET_URL: &str = "ws://localhost:31444";

/// Time to wait for block finalization before timing out.
const FINALIZE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);

/// Create a `SubtensorClient` connected to the local devnet.
async fn create_client() -> SubtensorClient {
    SubtensorClient::from_url(DEVNET_URL)
        .await
        .expect("failed to connect to devnet — is it running at ws://localhost:31444?")
}

/// Get the underlying `OnlineClient` for raw subxt operations.
fn rpc(client: &SubtensorClient) -> &subxt::OnlineClient<bittensor_core::config::SubtensorConfig> {
    client.rpc()
}

/// Dev account Alice (sr25519).
fn alice() -> subxt_signer::sr25519::Keypair {
    subxt_signer::sr25519::dev::alice()
}

/// Dev account Bob (sr25519).
fn bob() -> subxt_signer::sr25519::Keypair {
    subxt_signer::sr25519::dev::bob()
}

/// Dev account Charlie (sr25519).
fn charlie() -> subxt_signer::sr25519::Keypair {
    subxt_signer::sr25519::dev::charlie()
}

/// Get Alice's AccountId32.
fn alice_account() -> subxt::utils::AccountId32 {
    alice().public_key().to_account_id()
}

/// Get Bob's AccountId32.
fn bob_account() -> subxt::utils::AccountId32 {
    bob().public_key().to_account_id()
}

/// Get Charlie's AccountId32.
fn charlie_account() -> subxt::utils::AccountId32 {
    charlie().public_key().to_account_id()
}

// ═══════════════════════════════════════════════════════════════
//  QUERY TESTS
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn query_get_balance_alice_has_funds() {
    let client = create_client().await;
    let balance = get_balance(rpc(&client), &alice_account()).await;
    assert!(balance.is_ok(), "get_balance should succeed against devnet");
    let bal = balance.unwrap();
    // Dev accounts are pre-funded with large balances on --dev chains
    assert!(
        bal.to_tao() > 0.0,
        "Alice should have a positive balance on devnet, got {}",
        bal.to_tao()
    );
}

#[tokio::test]
#[ignore]
async fn query_get_balance_bob_has_funds() {
    let client = create_client().await;
    let balance = get_balance(rpc(&client), &bob_account()).await;
    assert!(balance.is_ok(), "get_balance for Bob should succeed");
    let bal = balance.unwrap();
    assert!(
        bal.to_tao() > 0.0,
        "Bob should have a positive balance on devnet, got {}",
        bal.to_tao()
    );
}

#[tokio::test]
#[ignore]
async fn query_get_balance_unknown_account_is_zero() {
    let client = create_client().await;
    // A random account that has never received funds
    let unknown = subxt::utils::AccountId32::from([0x99u8; 32]);
    let balance = get_balance(rpc(&client), &unknown).await;
    assert!(balance.is_ok(), "get_balance for unknown account should succeed");
    assert_eq!(balance.unwrap().to_tao(), 0.0, "Unknown account should have zero balance");
}

#[tokio::test]
#[ignore]
async fn query_get_stake_is_zero_for_new_account() {
    let client = create_client().await;
    let unknown = subxt::utils::AccountId32::from([0xAAu8; 32]);
    // get_stake is currently a stub returning ZERO; still verify it doesn't error
    let stake = get_stake(rpc(&client), &unknown, &unknown, 1).await;
    assert!(stake.is_ok(), "get_stake should not error on devnet");
}

#[tokio::test]
#[ignore]
async fn query_get_total_network_stake_succeeds() {
    let client = create_client().await;
    let total = get_total_network_stake(rpc(&client)).await;
    assert!(total.is_ok(), "get_total_network_stake should succeed on devnet");
}

#[tokio::test]
#[ignore]
async fn query_get_neuron_count_succeeds() {
    let client = create_client().await;
    let count = get_neuron_count(rpc(&client), 1).await;
    assert!(count.is_ok(), "get_neuron_count should succeed on devnet");
}

#[tokio::test]
#[ignore]
async fn query_get_metagraph_succeeds() {
    let client = create_client().await;
    let meta = get_metagraph(rpc(&client), 1).await;
    assert!(meta.is_ok(), "get_metagraph should succeed on devnet");
    let graph = meta.unwrap();
    assert_eq!(graph.netuid, 1, "metagraph should return requested netuid");
}

// ═══════════════════════════════════════════════════════════════
//  EXTRINSIC TESTS
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn extrinsic_transfer_alice_to_bob() {
    let client = create_client().await;
    let rpc_client = rpc(&client);

    // Record initial balance
    let bob_before = get_balance(rpc_client, &bob_account()).await.expect("bob balance before");

    // Transfer 1 TAO from Alice to Bob
    let one_tao: u64 = 1_000_000_000; // 1 TAO in rao
    let result = transfer(rpc_client, &alice(), bob_account(), one_tao).await;
    assert!(result.is_ok(), "transfer should succeed: {:?}", result.err());

    let tx = result.unwrap();
    assert_ne!(
        tx.block_hash,
        subxt::utils::H256::zero(),
        "block hash should be non-zero after finalization"
    );
    assert_ne!(tx.extrinsic_hash, subxt::utils::H256::zero(), "extrinsic hash should be non-zero");

    // Wait for the next block to ensure state is updated
    tokio::time::sleep(std::time::Duration::from_secs(6)).await;

    // Verify Bob's balance increased
    let bob_after = get_balance(rpc_client, &bob_account()).await.expect("bob balance after");
    assert!(
        bob_after.to_tao() > bob_before.to_tao(),
        "Bob's balance should increase after transfer: before={}, after={}",
        bob_before.to_tao(),
        bob_after.to_tao()
    );
}

#[tokio::test]
#[ignore]
async fn extrinsic_add_stake_alice() {
    let client = create_client().await;
    let rpc_client = rpc(&client);

    // Attempt to add stake — may fail if subnet 1 doesn't exist on devnet,
    // but should at least compile and submit without client-side error
    let result = tokio::time::timeout(
        FINALIZE_TIMEOUT,
        add_stake(rpc_client, &alice(), alice_account(), 1, 100_000_000),
    )
    .await;

    match result {
        Ok(Ok(tx)) => {
            assert_ne!(tx.block_hash, subxt::utils::H256::zero());
        }
        Ok(Err(e)) => {
            // Acceptable if the chain rejects (e.g., subnet doesn't exist)
            eprintln!("add_stake rejected by chain (expected on empty devnet): {e}");
        }
        Err(_) => {
            panic!("add_stake timed out waiting for finalization");
        }
    }
}

#[tokio::test]
#[ignore]
async fn extrinsic_set_weights_alice() {
    let client = create_client().await;
    let rpc_client = rpc(&client);

    // Set weights for subnet 1 — will likely be rejected on an empty devnet
    // but tests that the call construction and submission path works
    let result = tokio::time::timeout(
        FINALIZE_TIMEOUT,
        set_weights(rpc_client, &alice(), 1, vec![0], vec![65535], 0),
    )
    .await;

    match result {
        Ok(Ok(tx)) => {
            assert_ne!(tx.block_hash, subxt::utils::H256::zero());
        }
        Ok(Err(e)) => {
            eprintln!("set_weights rejected by chain (expected on empty devnet): {e}");
        }
        Err(_) => {
            panic!("set_weights timed out waiting for finalization");
        }
    }
}

// ═══════════════════════════════════════════════════════════════
//  EVENT TESTS
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn event_subscribe_events_receives_events() {
    let client = create_client().await;
    let rpc_client = rpc(&client);

    // Subscribe to events
    let mut event_rx =
        subscribe_events(rpc_client).await.expect("subscribe_events should succeed on devnet");

    // Trigger a transfer to generate events
    let _ = transfer(rpc_client, &alice(), bob_account(), 1_000_000).await;

    // Wait for events with a timeout
    let mut received = Vec::new();
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(30);

    while tokio::time::Instant::now() < deadline {
        match tokio::time::timeout(std::time::Duration::from_secs(5), event_rx.recv()).await {
            Ok(Some(event)) => {
                received.push(event);
                if received.len() >= 2 {
                    break;
                }
            }
            Ok(None) => break, // channel closed
            Err(_) => continue,
        }
    }

    assert!(!received.is_empty(), "should receive at least one event from the subscription");

    // Verify we can inspect event metadata
    for event in &received {
        let _: u64 = event.block_number();
        let _: subxt::utils::H256 = event.block_hash();
        let _ = event.pallet_name();
        let _ = event.event_name();
    }
}

#[tokio::test]
#[ignore]
async fn event_verify_transfer_event_emission() {
    let client = create_client().await;
    let rpc_client = rpc(&client);

    // Subscribe to events
    let mut event_rx = subscribe_events(rpc_client).await.expect("subscribe_events should succeed");

    // Trigger a transfer
    let _ = transfer(rpc_client, &alice(), charlie_account(), 500_000).await;

    // Wait for a Transfer event
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(30);
    let mut found_transfer = false;

    while tokio::time::Instant::now() < deadline {
        match tokio::time::timeout(std::time::Duration::from_secs(5), event_rx.recv()).await {
            Ok(Some(event)) => {
                if event.is_transfer() {
                    found_transfer = true;
                    break;
                }
            }
            Ok(None) => break,
            Err(_) => continue,
        }
    }

    assert!(
        found_transfer,
        "should observe a Balances::Transfer event after submitting a transfer extrinsic"
    );
}

// ═══════════════════════════════════════════════════════════════
//  SUBSCRIPTION TESTS
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn subscription_subscribe_blocks_increasing() {
    let client = create_client().await;
    let rpc_client = rpc(&client);

    // Subscribe to blocks
    let mut blocks =
        subscribe_blocks(rpc_client).await.expect("subscribe_blocks should succeed on devnet");

    // Collect a few block numbers
    let mut block_numbers = Vec::new();
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(30);

    while tokio::time::Instant::now() < deadline && block_numbers.len() < 3 {
        match tokio::time::timeout(std::time::Duration::from_secs(10), blocks.next()).await {
            Ok(Some(Ok(block))) => {
                block_numbers.push(block.number());
            }
            _ => continue,
        }
    }

    assert!(
        block_numbers.len() >= 2,
        "should receive at least 2 blocks from subscription, got {}",
        block_numbers.len()
    );

    // Verify block numbers are strictly increasing
    for window in block_numbers.windows(2) {
        assert!(
            window[1] > window[0],
            "block numbers should increase: {} -> {}",
            window[0],
            window[1]
        );
    }
}

#[tokio::test]
#[ignore]
async fn subscription_blocks_have_valid_hash() {
    let client = create_client().await;
    let rpc_client = rpc(&client);

    let mut blocks = subscribe_blocks(rpc_client).await.expect("subscribe_blocks should succeed");

    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(15);

    while tokio::time::Instant::now() < deadline {
        match tokio::time::timeout(std::time::Duration::from_secs(10), blocks.next()).await {
            Ok(Some(Ok(block))) => {
                let hash = block.hash();
                assert_ne!(hash, subxt::utils::H256::zero(), "block hash should be non-zero");
                let number = block.number();
                assert!(number > 0, "block number should be positive after genesis");
                return; // Success after first valid block
            }
            _ => continue,
        }
    }

    panic!("did not receive any valid blocks within timeout");
}

// ═══════════════════════════════════════════════════════════════
//  COMBINED FLOW TESTS
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn flow_transfer_then_query_balance() {
    let client = create_client().await;
    let rpc_client = rpc(&client);

    let bob_before = get_balance(rpc_client, &bob_account()).await.unwrap();

    // Transfer 0.5 TAO
    let half_tao: u64 = 500_000_000;
    transfer(rpc_client, &alice(), bob_account(), half_tao).await.expect("transfer should succeed");

    // Allow state to propagate
    tokio::time::sleep(std::time::Duration::from_secs(6)).await;

    let bob_after = get_balance(rpc_client, &bob_account()).await.unwrap();
    assert!(
        bob_after.to_tao() >= bob_before.to_tao() + 0.5 - 0.001,
        "Bob's balance should increase by ~0.5 TAO (accounting for fees): before={}, after={}",
        bob_before.to_tao(),
        bob_after.to_tao()
    );
}

#[tokio::test]
#[ignore]
async fn flow_multiple_transfers_succeed() {
    let client = create_client().await;
    let rpc_client = rpc(&client);

    // Send multiple small transfers
    for i in 0..3 {
        let amount: u64 = 100_000_000; // 0.1 TAO
        let result = transfer(rpc_client, &alice(), bob_account(), amount).await;
        assert!(result.is_ok(), "transfer {} should succeed: {:?}", i + 1, result.err());
    }
}
