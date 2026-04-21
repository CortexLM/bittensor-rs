//! Example: Create a Bittensor wallet, generate coldkey/hotkey, and display addresses.
//!
//! Run with: cargo run -p bittensor-examples --example wallet_create

use bittensor_wallet::prelude::Wallet;

fn main() {
    let mut wallet = Wallet::new("default");

    let mnemonic = wallet.create_coldkey("my-password").expect("failed to create coldkey");
    println!("Coldkey mnemonic (BACK UP SECURELY): {mnemonic}");

    let coldkey_addr = wallet.get_coldkeypub().expect("failed to read coldkeypub");
    println!("Coldkey SS58 address: {coldkey_addr}");

    let hotkey = wallet.create_hotkey().expect("failed to create hotkey");
    println!("Hotkey SS58 address: {}", hotkey.ss58_address());

    println!("Coldkey path:    {}", wallet.coldkey_path().display());
    println!("Coldkeypub path: {}", wallet.coldkeypub_path().display());
    println!("Hotkey path:     {}", wallet.hotkey_path().display());
}
