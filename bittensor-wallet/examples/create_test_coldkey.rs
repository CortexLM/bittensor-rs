use bittensor_wallet::keyfile;
use std::fs;
use std::os::unix::fs::PermissionsExt;

fn main() {
    let password = "rust_to_python_test";
    let secret_key_hex = "0000000000000000000000000000000000000000000000000000000000000002";

    let payload = serde_json::json!({
        "accountId": format!("0x{secret_key_hex}"),
        "publicKey": format!("0x{secret_key_hex}"),
        "secretPhrase": null,
        "secretSeed": format!("0x{secret_key_hex}"),
        "ss58Address": null,
    });
    let plaintext = serde_json::to_string(&payload).expect("serialize failed").into_bytes();

    println!("=== Creating Rust-encrypted Coldkey ===\n");
    println!("Password:    {password}");
    println!("Secret key:  {secret_key_hex}");
    println!("Plaintext:   {} bytes", plaintext.len());

    let encrypted = keyfile::encrypt(&plaintext, password.as_bytes()).expect("encrypt failed");
    println!("Encrypted:   {} bytes", encrypted.len());
    println!("Prefix:      {:?}", &encrypted[..5]);

    let out_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/scripts");
    let coldkey_path = format!("{out_dir}/rust_coldkey");
    fs::write(&coldkey_path, &encrypted).expect("write failed");
    fs::set_permissions(&coldkey_path, fs::Permissions::from_mode(0o600)).expect("chmod failed");

    let meta = serde_json::json!({
        "password": password,
        "secret_key_hex": secret_key_hex,
        "coldkey_path": coldkey_path,
        "plaintext_hex": hex::encode(&plaintext),
    });
    let meta_path = format!("{out_dir}/rust_coldkey_meta.json");
    fs::write(&meta_path, serde_json::to_string_pretty(&meta).expect("serialize meta failed"))
        .expect("write meta failed");

    println!("\nWritten: {coldkey_path}");
    println!("Meta:    {meta_path}");
    println!("\nVerify with: python3 scripts/verify_rust_coldkey.py");
}
