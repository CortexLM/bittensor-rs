// Test file to verify Python-Rust keyfile compatibility
use bittensor_rs::wallet::{Keyfile, Keypair};
use std::path::Path;

fn main() {
    // Test 1: Read Python-encrypted keyfile with Rust
    println!("=== Test 1: Python encrypted keyfile with Rust ===");

    let python_keyfile_path = Path::new("/tmp/python_test_keyfile");

    if python_keyfile_path.exists() {
        let keyfile = Keyfile::new(python_keyfile_path);

        println!("Is encrypted: {}", keyfile.is_encrypted());

        match keyfile.get_keypair(Some("testpass123")) {
            Ok(kp) => {
                println!("Successfully decrypted Python keyfile!");
                println!("SS58 Address: {}", kp.ss58_address());
                println!("Public Key: 0x{}", hex::encode(kp.public_key()));
            }
            Err(e) => {
                println!("Failed to decrypt Python keyfile: {}", e);
            }
        }
    } else {
        println!("Python keyfile not found at /tmp/python_test_keyfile");
    }

    // Test 2: Read Python unencrypted keyfile with Rust
    println!("\n=== Test 2: Python unencrypted keyfile with Rust ===");

    let python_unenc_path = Path::new("/tmp/python_test_keyfile_unenc");

    if python_unenc_path.exists() {
        let keyfile = Keyfile::new(python_unenc_path);

        println!("Is encrypted: {}", keyfile.is_encrypted());

        match keyfile.get_keypair(None) {
            Ok(kp) => {
                println!("Successfully read Python JSON keyfile!");
                println!("SS58 Address: {}", kp.ss58_address());
                println!("Public Key: 0x{}", hex::encode(kp.public_key()));
            }
            Err(e) => {
                println!("Failed to read Python JSON keyfile: {}", e);
            }
        }
    } else {
        println!("Python unencrypted keyfile not found");
    }

    // Test 3: Create Rust keyfile and verify Python can read it
    println!("\n=== Test 3: Rust encrypted keyfile format ===");

    let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let kp = Keypair::from_mnemonic(mnemonic, None).unwrap();

    println!("Rust Keypair:");
    println!("  SS58 Address: {}", kp.ss58_address());
    println!("  Public Key: 0x{}", hex::encode(kp.public_key()));

    // Expected from Python:
    // SS58: 5EPCUjPxiHAcNooYipQFWr9NmmXJKpNG5RhcntXwbtUySrgH
    // Public Key: 66933bd1f37070ef87bd1198af3dacceb095237f803f3d32b173e6b425ed7972

    // Verify they match
    let expected_ss58 = "5EPCUjPxiHAcNooYipQFWr9NmmXJKpNG5RhcntXwbtUySrgH";
    let expected_pubkey = "66933bd1f37070ef87bd1198af3dacceb095237f803f3d32b173e6b425ed7972";

    println!("\n=== Verification ===");
    println!("SS58 match: {}", kp.ss58_address() == expected_ss58);
    println!(
        "Public key match: {}",
        hex::encode(kp.public_key()) == expected_pubkey
    );

    // Test 4: Create Rust encrypted keyfile
    println!("\n=== Test 4: Create Rust encrypted keyfile for Python ===");

    let rust_keyfile_path = Path::new("/tmp/rust_test_keyfile");
    let mut keyfile = Keyfile::new(rust_keyfile_path);
    keyfile
        .set_keypair(kp.clone(), Some("rustpass123"), true)
        .unwrap();

    println!("Rust keyfile created at /tmp/rust_test_keyfile");
    println!("Password: rustpass123");
}
