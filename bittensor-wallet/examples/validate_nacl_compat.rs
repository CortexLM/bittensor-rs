use bittensor_wallet::keyfile;
use std::fs;
use std::process;

fn main() {
    let meta_path = concat!(env!("CARGO_MANIFEST_DIR"), "/scripts/test_coldkey_meta.json");
    let meta_json = fs::read_to_string(meta_path).unwrap_or_else(|e| {
        eprintln!("Failed to read meta file {meta_path}: {e}");
        process::exit(1);
    });
    let meta: serde_json::Value = serde_json::from_str(&meta_json).unwrap_or_else(|e| {
        eprintln!("Failed to parse meta JSON: {e}");
        process::exit(1);
    });

    let coldkey_path = meta["coldkey_path"].as_str().expect("missing coldkey_path");
    let password = meta["password"].as_str().expect("missing password");
    let expected_secret_hex = meta["secret_key_hex"].as_str().expect("missing secret_key_hex");

    println!("=== NaCl Coldkey Compatibility Validation ===\n");
    println!("Coldkey path: {coldkey_path}");
    println!("Password:     {password}");
    println!("Expected key: {expected_secret_hex}\n");

    let encrypted_data = fs::read(coldkey_path).unwrap_or_else(|e| {
        eprintln!("Failed to read coldkey: {e}");
        process::exit(1);
    });

    println!(
        "Encrypted data ({} bytes), prefix: {:?}",
        encrypted_data.len(),
        &encrypted_data[..5.min(encrypted_data.len())]
    );

    if !keyfile::is_encrypted_nacl(&encrypted_data) {
        eprintln!("ERROR: Data does not have $NACL prefix!");
        process::exit(1);
    }
    println!("✓ $NACL prefix confirmed\n");

    let decrypted = keyfile::decrypt(&encrypted_data, password.as_bytes()).unwrap_or_else(|e| {
        eprintln!("Decryption FAILED: {e}");
        process::exit(1);
    });

    let decrypted_str = String::from_utf8(decrypted.clone()).expect("decrypted data is not UTF-8");
    println!("Decrypted JSON ({} bytes):", decrypted.len());
    println!("{decrypted_str}\n");

    if decrypted_str.contains(expected_secret_hex) {
        println!("✓ Secret key match CONFIRMED: {expected_secret_hex}");
        println!("\n=== Python → Rust NaCl compatibility: PASSED ===");
    } else {
        eprintln!("✗ Secret key NOT found in decrypted data!");
        process::exit(1);
    }
}
