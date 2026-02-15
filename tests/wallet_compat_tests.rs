//! Wallet compatibility tests
//!
//! Validates:
//! - Deterministic key derivation from well-known mnemonic matches Python SDK
//! - SS58 format 42 (Bittensor) is used consistently
//! - Keypair::from_seed determinism
//! - Keypair::from_uri("//Alice") produces a known, stable address
//! - Sign/verify roundtrip works correctly

use bittensor_rs::wallet::{Keypair, Mnemonic, BITTENSOR_SS58_FORMAT};

// ============================================================================
// Deterministic mnemonic derivation (Python SDK parity)
// ============================================================================

#[test]
fn test_mnemonic_abandon_produces_known_ss58() {
    let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let kp = Keypair::from_mnemonic(mnemonic, None).unwrap();

    let expected_ss58 = "5EPCUjPxiHAcNooYipQFWr9NmmXJKpNG5RhcntXwbtUySrgH";
    assert_eq!(
        kp.ss58_address(),
        expected_ss58,
        "SS58 address from well-known mnemonic must match Python SDK"
    );
}

#[test]
fn test_mnemonic_abandon_produces_known_pubkey() {
    let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let kp = Keypair::from_mnemonic(mnemonic, None).unwrap();

    let expected_pubkey = "66933bd1f37070ef87bd1198af3dacceb095237f803f3d32b173e6b425ed7972";
    assert_eq!(
        hex::encode(kp.public_key()),
        expected_pubkey,
        "Public key from well-known mnemonic must match Python SDK"
    );
}

#[test]
fn test_mnemonic_derivation_is_deterministic() {
    let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let kp1 = Keypair::from_mnemonic(mnemonic, None).unwrap();
    let kp2 = Keypair::from_mnemonic(mnemonic, None).unwrap();

    assert_eq!(kp1.public_key(), kp2.public_key());
    assert_eq!(kp1.ss58_address(), kp2.ss58_address());
}

#[test]
fn test_mnemonic_with_password_differs() {
    let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let kp_no_pass = Keypair::from_mnemonic(mnemonic, None).unwrap();
    let kp_with_pass = Keypair::from_mnemonic(mnemonic, Some("password")).unwrap();

    assert_ne!(
        kp_no_pass.public_key(),
        kp_with_pass.public_key(),
        "Password should change derived key"
    );
}

// ============================================================================
// SS58 format 42 verification
// ============================================================================

#[test]
fn test_ss58_format_is_42() {
    assert_eq!(BITTENSOR_SS58_FORMAT, 42);
}

#[test]
fn test_all_addresses_start_with_5() {
    let kp_gen = Keypair::generate();
    assert!(
        kp_gen.ss58_address().starts_with('5'),
        "SS58 format 42 addresses should start with '5'"
    );

    let kp_seed = Keypair::from_seed(&[0u8; 32]).unwrap();
    assert!(kp_seed.ss58_address().starts_with('5'));

    let kp_alice = Keypair::from_uri("//Alice").unwrap();
    assert!(kp_alice.ss58_address().starts_with('5'));
}

// ============================================================================
// from_seed determinism
// ============================================================================

#[test]
fn test_from_seed_zero_deterministic() {
    let kp1 = Keypair::from_seed(&[0u8; 32]).unwrap();
    let kp2 = Keypair::from_seed(&[0u8; 32]).unwrap();

    assert_eq!(kp1.public_key(), kp2.public_key());
    assert_eq!(kp1.ss58_address(), kp2.ss58_address());
}

#[test]
fn test_from_seed_different_seeds_differ() {
    let kp1 = Keypair::from_seed(&[0u8; 32]).unwrap();
    let kp2 = Keypair::from_seed(&[1u8; 32]).unwrap();

    assert_ne!(kp1.public_key(), kp2.public_key());
}

#[test]
fn test_from_seed_invalid_length_rejected() {
    assert!(Keypair::from_seed(&[0u8; 16]).is_err());
    assert!(Keypair::from_seed(&[0u8; 64]).is_err());
    assert!(Keypair::from_seed(&[]).is_err());
}

// ============================================================================
// from_uri determinism
// ============================================================================

#[test]
fn test_alice_uri_deterministic() {
    let kp1 = Keypair::from_uri("//Alice").unwrap();
    let kp2 = Keypair::from_uri("//Alice").unwrap();

    assert_eq!(kp1.public_key(), kp2.public_key());
    assert_eq!(kp1.ss58_address(), kp2.ss58_address());
}

#[test]
fn test_alice_and_bob_differ() {
    let alice = Keypair::from_uri("//Alice").unwrap();
    let bob = Keypair::from_uri("//Bob").unwrap();

    assert_ne!(alice.public_key(), bob.public_key());
    assert_ne!(alice.ss58_address(), bob.ss58_address());
}

// ============================================================================
// Sign/verify roundtrip
// ============================================================================

#[test]
fn test_sign_verify_roundtrip() {
    let kp = Keypair::generate();
    let message = b"Hello, Bittensor!";

    let signature = kp.sign(message);
    assert_eq!(signature.len(), 64);
    assert!(kp.verify(message, &signature));
}

#[test]
fn test_sign_verify_wrong_message_fails() {
    let kp = Keypair::generate();
    let signature = kp.sign(b"correct message");
    assert!(!kp.verify(b"wrong message", &signature));
}

#[test]
fn test_sign_verify_wrong_key_fails() {
    let kp1 = Keypair::generate();
    let kp2 = Keypair::generate();
    let signature = kp1.sign(b"test");
    assert!(!kp2.verify(b"test", &signature));
}

#[test]
fn test_sign_verify_with_public_key() {
    let kp = Keypair::generate();
    let message = b"verify with public key";
    let signature = kp.sign(message);

    assert!(Keypair::verify_with_public(
        message,
        &signature,
        kp.public_key()
    ));
}

#[test]
fn test_sign_verify_empty_message() {
    let kp = Keypair::generate();
    let signature = kp.sign(b"");
    assert!(kp.verify(b"", &signature));
}

#[test]
fn test_sign_verify_large_message() {
    let kp = Keypair::generate();
    let message = vec![0xABu8; 10_000];
    let signature = kp.sign(&message);
    assert!(kp.verify(&message, &signature));
}

// ============================================================================
// Mnemonic validation
// ============================================================================

#[test]
fn test_mnemonic_validate_valid_phrase() {
    let valid = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    assert!(Mnemonic::validate(valid));
}

#[test]
fn test_mnemonic_validate_invalid_phrase() {
    assert!(!Mnemonic::validate("not a valid mnemonic"));
    assert!(!Mnemonic::validate(""));
    assert!(!Mnemonic::validate("abandon"));
}

#[test]
fn test_mnemonic_generate_is_valid() {
    let m = Mnemonic::generate();
    assert!(Mnemonic::validate(m.phrase()));
    let words: Vec<&str> = m.phrase().split_whitespace().collect();
    assert_eq!(words.len(), 12);
}

// ============================================================================
// Keypair bytes roundtrip
// ============================================================================

#[test]
fn test_keypair_bytes_roundtrip() {
    let original = Keypair::generate();
    let bytes = original.to_bytes();
    let restored = Keypair::from_bytes(&bytes).unwrap();

    assert_eq!(original.public_key(), restored.public_key());
    assert_eq!(original.ss58_address(), restored.ss58_address());

    let message = b"roundtrip test";
    let sig = original.sign(message);
    assert!(restored.verify(message, &sig));
}
