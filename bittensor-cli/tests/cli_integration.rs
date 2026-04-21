//! CLI integration tests using `assert_cmd` to exercise the `btcli-rs` binary.
//!
//! These tests verify command-line parsing, help output, version output,
//! argument validation, and offline wallet operations. No network
//! connections are required.

use assert_cmd::Command;
use predicates::prelude::*;

// ---------------------------------------------------------------------------
// Basic binary tests
// ---------------------------------------------------------------------------

#[test]
fn main_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("btcli-rs"))
        .stdout(predicate::str::contains("Bittensor CLI"));
}

#[test]
fn version() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.arg("--version").assert().success().stdout(predicate::str::contains("btcli-rs"));
}

#[test]
fn no_subcommand_fails() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required").or(predicate::str::contains("subcommand")));
}

#[test]
fn unknown_command_fails() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized").or(predicate::str::contains("unknown")));
}

// ---------------------------------------------------------------------------
// Top-level subcommand --help tests
// ---------------------------------------------------------------------------

#[test]
fn wallet_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["wallet", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Wallet management commands"));
}

#[test]
fn stake_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["stake", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Stake management commands"));
}

#[test]
fn transfer_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["transfer", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Transfer commands"));
}

#[test]
fn register_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["register", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Registration commands"));
}

#[test]
fn root_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["root", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Root network commands"));
}

#[test]
fn subnet_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["subnet", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Subnet commands"));
}

#[test]
fn delegate_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["delegate", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Delegate commands"));
}

#[test]
fn weights_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["weights", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Weights commands"));
}

#[test]
fn metagraph_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["metagraph", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Metagraph commands"));
}

// ---------------------------------------------------------------------------
// Wallet sub-subcommand --help tests
// ---------------------------------------------------------------------------

#[test]
fn wallet_create_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["wallet", "create", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("no-password"))
        .stdout(predicate::str::contains("password"));
}

#[test]
fn wallet_list_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["wallet", "list", "--help"]).assert().success();
}

#[test]
fn wallet_balance_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["wallet", "balance", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--all"));
}

#[test]
fn wallet_overview_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["wallet", "overview", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--all"));
}

#[test]
fn wallet_transfer_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["wallet", "transfer", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<DEST>"))
        .stdout(predicate::str::contains("<AMOUNT>"));
}

#[test]
fn wallet_swap_coldkey_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["wallet", "swap-coldkey", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<NEW_COLDKEY>"));
}

#[test]
fn wallet_inspect_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["wallet", "inspect", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--password"));
}

#[test]
fn wallet_regen_coldkey_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["wallet", "regen-coldkey", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("mnemonic"))
        .stdout(predicate::str::contains("--yes"));
}

#[test]
fn wallet_regen_coldkeypub_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["wallet", "regen-coldkeypub", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<SS58_ADDRESS>"));
}

#[test]
fn wallet_create_hotkey_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["wallet", "create-hotkey", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--hotkey"))
        .stdout(predicate::str::contains("--seed"));
}

#[test]
fn wallet_regen_hotkey_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["wallet", "regen-hotkey", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("mnemonic"))
        .stdout(predicate::str::contains("--hotkey"));
}

// ---------------------------------------------------------------------------
// Other sub-subcommand --help tests
// ---------------------------------------------------------------------------

#[test]
fn stake_add_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["stake", "add", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--hotkey"))
        .stdout(predicate::str::contains("--netuid"));
}

#[test]
fn stake_remove_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["stake", "remove", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--hotkey"));
}

#[test]
fn stake_list_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["stake", "list", "--help"]).assert().success();
}

#[test]
fn transfer_transfer_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["transfer", "transfer", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<DEST>"))
        .stdout(predicate::str::contains("<AMOUNT>"));
}

#[test]
fn transfer_multiple_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["transfer", "multiple", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--destinations"))
        .stdout(predicate::str::contains("--amounts"));
}

#[test]
fn register_register_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["register", "register", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--netuid"));
}

#[test]
fn register_burned_register_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["register", "burned-register", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--netuid"));
}

#[test]
fn root_set_weights_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["root", "set-weights", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--netuid"));
}

#[test]
fn subnet_create_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["subnet", "create", "--help"]).assert().success();
}

#[test]
fn subnet_list_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["subnet", "list", "--help"]).assert().success();
}

#[test]
fn subnet_info_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["subnet", "info", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--netuid"));
}

#[test]
fn delegate_add_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["delegate", "add", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--hotkey"));
}

#[test]
fn delegate_remove_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["delegate", "remove", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--hotkey"));
}

#[test]
fn delegate_list_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["delegate", "list", "--help"]).assert().success();
}

#[test]
fn weights_set_weights_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["weights", "set-weights", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--netuid"));
}

#[test]
fn weights_get_weights_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["weights", "get-weights", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--netuid"));
}

#[test]
fn metagraph_show_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["metagraph", "show", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--netuid"));
}

#[test]
fn metagraph_sync_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["metagraph", "sync", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--netuid"));
}

// ---------------------------------------------------------------------------
// Global flag tests
// ---------------------------------------------------------------------------

#[test]
fn network_flag_in_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.arg("--help").assert().success().stdout(predicate::str::contains("--network"));
}

#[test]
fn wallet_name_flag_in_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.arg("--help").assert().success().stdout(predicate::str::contains("--wallet.name"));
}

#[test]
fn wallet_path_flag_in_help() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.arg("--help").assert().success().stdout(predicate::str::contains("--wallet.path"));
}

#[test]
fn invalid_network_fails() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["--network", "invalidnet", "wallet", "list"]).assert().failure().stderr(
        predicate::str::contains("unknown network").or(predicate::str::contains("invalid")),
    );
}

// ---------------------------------------------------------------------------
// Offline wallet command tests (using temp directories)
// ---------------------------------------------------------------------------

#[test]
fn wallet_create_with_no_password() {
    let dir = tempfile::TempDir::new().unwrap();
    let dir_path = dir.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args([
        "--wallet.path",
        dir_path,
        "--wallet.name",
        "test-wallet",
        "wallet",
        "create",
        "--no-password",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("Wallet created"))
    .stdout(predicate::str::contains("test-wallet"))
    .stdout(predicate::str::contains("IMPORTANT"));

    // Verify key files exist
    let wallet_dir = dir.path().join("test-wallet");
    assert!(wallet_dir.join("coldkey").exists(), "coldkey file should exist");
    assert!(wallet_dir.join("coldkeypub").exists(), "coldkeypub file should exist");
    assert!(wallet_dir.join("hotkeys").join("default").exists(), "default hotkey should exist");
}

#[test]
fn wallet_create_with_password() {
    let dir = tempfile::TempDir::new().unwrap();
    let dir_path = dir.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args([
        "--wallet.path",
        dir_path,
        "--wallet.name",
        "pwd-wallet",
        "wallet",
        "create",
        "--password",
        "testpass123",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("Wallet created"));

    let wallet_dir = dir.path().join("pwd-wallet");
    assert!(wallet_dir.join("coldkey").exists());
    assert!(wallet_dir.join("coldkeypub").exists());
}

#[test]
fn wallet_list_empty() {
    let dir = tempfile::TempDir::new().unwrap();
    let dir_path = dir.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["--wallet.path", dir_path, "--wallet.name", "default", "wallet", "list"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("No wallets found").or(predicate::str::contains("Wallets")),
        );
}

#[test]
fn wallet_list_after_create() {
    let dir = tempfile::TempDir::new().unwrap();
    let dir_path = dir.path().to_str().unwrap();

    // Create a wallet first
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args([
        "--wallet.path",
        dir_path,
        "--wallet.name",
        "list-test",
        "wallet",
        "create",
        "--no-password",
    ])
    .assert()
    .success();

    // Now list should show it
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["--wallet.path", dir_path, "--wallet.name", "default", "wallet", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list-test"));
}

#[test]
fn wallet_list_multiple_wallets() {
    let dir = tempfile::TempDir::new().unwrap();
    let dir_path = dir.path().to_str().unwrap();

    // Create two wallets
    for name in &["alpha", "beta"] {
        let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
        cmd.args([
            "--wallet.path",
            dir_path,
            "--wallet.name",
            name,
            "wallet",
            "create",
            "--no-password",
        ])
        .assert()
        .success();
    }

    // List should find both
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["--wallet.path", dir_path, "--wallet.name", "default", "wallet", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("alpha"))
        .stdout(predicate::str::contains("beta"));
}

#[test]
fn wallet_show_after_create() {
    let dir = tempfile::TempDir::new().unwrap();
    let dir_path = dir.path().to_str().unwrap();

    // Create a wallet
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args([
        "--wallet.path",
        dir_path,
        "--wallet.name",
        "show-wallet",
        "wallet",
        "create",
        "--no-password",
    ])
    .assert()
    .success();

    // Show should display wallet details
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["--wallet.path", dir_path, "--wallet.name", "show-wallet", "wallet", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("show-wallet"))
        .stdout(predicate::str::contains("Coldkey SS58"));
}

#[test]
fn wallet_inspect_after_create() {
    let dir = tempfile::TempDir::new().unwrap();
    let dir_path = dir.path().to_str().unwrap();

    // Create a wallet
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args([
        "--wallet.path",
        dir_path,
        "--wallet.name",
        "inspect-wallet",
        "wallet",
        "create",
        "--no-password",
    ])
    .assert()
    .success();

    // Inspect without password should succeed (graceful degradation)
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["--wallet.path", dir_path, "--wallet.name", "inspect-wallet", "wallet", "inspect"])
        .assert()
        .success()
        .stdout(predicate::str::contains("inspect-wallet"));
}

#[test]
fn wallet_create_hotkey_seed() {
    let dir = tempfile::TempDir::new().unwrap();
    let dir_path = dir.path().to_str().unwrap();

    // Create wallet first
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args([
        "--wallet.path",
        dir_path,
        "--wallet.name",
        "hotkey-wallet",
        "wallet",
        "create",
        "--no-password",
    ])
    .assert()
    .success();

    // Create a seed-based hotkey
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args([
        "--wallet.path",
        dir_path,
        "--wallet.name",
        "hotkey-wallet",
        "wallet",
        "create-hotkey",
        "--hotkey",
        "validator",
        "--seed",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("Hotkey created"));

    let hotkey_path = dir.path().join("hotkey-wallet").join("hotkeys").join("validator");
    assert!(hotkey_path.exists(), "hotkey file should exist");
}

// ---------------------------------------------------------------------------
// Argument validation tests
// ---------------------------------------------------------------------------

#[test]
fn wallet_transfer_missing_args() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["wallet", "transfer"]).assert().failure();
}

#[test]
fn wallet_swap_coldkey_missing_args() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["wallet", "swap-coldkey"]).assert().failure();
}

#[test]
fn wallet_regen_coldkey_missing_mnemonic() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["wallet", "regen-coldkey"]).assert().failure();
}

#[test]
fn wallet_regen_coldkeypub_missing_address() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["wallet", "regen-coldkeypub"]).assert().failure();
}

#[test]
fn wallet_regen_hotkey_missing_mnemonic() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["wallet", "regen-hotkey"]).assert().failure();
}

#[test]
fn stake_add_missing_args() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["stake", "add"]).assert().failure();
}

#[test]
fn stake_remove_missing_args() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["stake", "remove"]).assert().failure();
}

#[test]
fn transfer_transfer_missing_args() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["transfer", "transfer"]).assert().failure();
}

#[test]
fn transfer_multiple_missing_args() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["transfer", "multiple"]).assert().failure();
}

#[test]
fn register_register_missing_netuid() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["register", "register"]).assert().failure();
}

#[test]
fn subnet_info_missing_netuid() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["subnet", "info"]).assert().failure();
}

#[test]
fn weights_set_weights_missing_args() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["weights", "set-weights"]).assert().failure();
}

#[test]
fn metagraph_show_missing_netuid() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["metagraph", "show"]).assert().failure();
}

// ---------------------------------------------------------------------------
// Help text content verification
// ---------------------------------------------------------------------------

#[test]
fn main_help_lists_all_subcommands() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("wallet"))
        .stdout(predicate::str::contains("stake"))
        .stdout(predicate::str::contains("transfer"))
        .stdout(predicate::str::contains("register"))
        .stdout(predicate::str::contains("root"))
        .stdout(predicate::str::contains("subnet"))
        .stdout(predicate::str::contains("delegate"))
        .stdout(predicate::str::contains("weights"))
        .stdout(predicate::str::contains("metagraph"));
}

#[test]
fn wallet_help_lists_all_subcommands() {
    let mut cmd = Command::cargo_bin("btcli-rs").unwrap();
    cmd.args(["wallet", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("show"))
        .stdout(predicate::str::contains("balance"))
        .stdout(predicate::str::contains("overview"))
        .stdout(predicate::str::contains("transfer"))
        .stdout(predicate::str::contains("swap-coldkey"))
        .stdout(predicate::str::contains("inspect"))
        .stdout(predicate::str::contains("regen-coldkey"))
        .stdout(predicate::str::contains("regen-coldkeypub"))
        .stdout(predicate::str::contains("create-hotkey"))
        .stdout(predicate::str::contains("regen-hotkey"));
}
