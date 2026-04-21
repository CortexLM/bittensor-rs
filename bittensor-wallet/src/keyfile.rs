use sodiumoxide::crypto::pwhash::argon2i13;
use sodiumoxide::crypto::secretbox;

const NACL_PREFIX: &[u8] = b"$NACL";
const NACL_SALT: &[u8] = b"\x13q\x83\xdf\xf1Z\t\xbc\x9c\x90\xb5Q\x879\xe9\xb1";

#[derive(Debug, thiserror::Error)]
pub enum KeyfileError {
    #[error("Invalid encryption: {0}")]
    InvalidEncryption(String),
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("Key derivation failed")]
    KeyDerivationFailed,
}

fn derive_key(password: &[u8]) -> Result<secretbox::Key, KeyfileError> {
    let salt = argon2i13::Salt::from_slice(NACL_SALT)
        .ok_or_else(|| KeyfileError::InvalidEncryption("Invalid NaCl salt".into()))?;

    let mut key_bytes = [0u8; secretbox::KEYBYTES];
    argon2i13::derive_key(
        &mut key_bytes,
        password,
        &salt,
        argon2i13::OPSLIMIT_SENSITIVE,
        argon2i13::MEMLIMIT_SENSITIVE,
    )
    .map_err(|_| KeyfileError::KeyDerivationFailed)?;

    Ok(secretbox::Key(key_bytes))
}

/// Check whether the given data begins with the `$NACL` prefix,
/// indicating it was encrypted with NaCl secretbox.
pub fn is_encrypted_nacl(data: &[u8]) -> bool {
    data.starts_with(NACL_PREFIX)
}

/// Encrypts the given data using NaCl secretbox (XSalsa20-Poly1305) with a key
/// derived from the password via Argon2i. The output is prefixed with `$NACL`
/// followed by the nonce and ciphertext.
pub fn encrypt(data: &[u8], password: &[u8]) -> Result<Vec<u8>, KeyfileError> {
    let key = derive_key(password)?;
    let nonce = secretbox::gen_nonce();
    let ciphertext = secretbox::seal(data, &nonce, &key);

    let mut result = NACL_PREFIX.to_vec();
    result.extend_from_slice(&nonce.0);
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

/// Decrypts data that was encrypted with [`encrypt`]. Expects the `$NACL` prefix,
/// followed by the 24-byte nonce and the ciphertext. Returns an error if the
/// password is wrong or the data is corrupted.
pub fn decrypt(encrypted: &[u8], password: &[u8]) -> Result<Vec<u8>, KeyfileError> {
    if !is_encrypted_nacl(encrypted) {
        return Err(KeyfileError::InvalidEncryption(
            "Data does not start with $NACL prefix".into(),
        ));
    }

    let data = &encrypted[NACL_PREFIX.len()..];

    if data.len() < secretbox::NONCEBYTES {
        return Err(KeyfileError::InvalidEncryption("Data too short for nonce".into()));
    }

    let nonce = secretbox::Nonce::from_slice(&data[..secretbox::NONCEBYTES])
        .ok_or_else(|| KeyfileError::InvalidEncryption("Invalid nonce".into()))?;

    let ciphertext = &data[secretbox::NONCEBYTES..];
    let key = derive_key(password)?;

    secretbox::open(ciphertext, &nonce, &key)
        .map_err(|_| KeyfileError::DecryptionFailed("Wrong password or corrupted data".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_encrypt_decrypt() {
        let password = b"testpassword123";
        let plaintext = b"hello bittensor world";
        let encrypted = encrypt(plaintext, password).expect("encrypt failed");
        let decrypted = decrypt(&encrypted, password).expect("decrypt failed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn encrypted_data_has_nacl_prefix() {
        let encrypted = encrypt(b"test", b"pass").expect("encrypt failed");
        assert!(is_encrypted_nacl(&encrypted));
        assert_eq!(&encrypted[..5], b"$NACL");
    }

    #[test]
    fn wrong_password_fails() {
        let encrypted = encrypt(b"secret data", b"correct_pass").expect("encrypt failed");
        let result = decrypt(&encrypted, b"wrong_pass");
        assert!(result.is_err());
    }

    #[test]
    fn invalid_prefix_fails() {
        let result = decrypt(b"invalid_prefix_data_here______", b"pass");
        assert!(result.is_err());
    }

    #[test]
    fn decrypt_python_created_coldkey() {
        let meta_json = include_str!("../scripts/test_coldkey_meta.json");
        let meta: serde_json::Value =
            serde_json::from_str(meta_json).expect("Failed to parse test_coldkey_meta.json");

        let coldkey_path = meta["coldkey_path"].as_str().expect("missing coldkey_path in meta");
        let password = meta["password"].as_str().expect("missing password in meta");
        let expected_secret_hex =
            meta["secret_key_hex"].as_str().expect("missing secret_key_hex in meta");

        let encrypted_data = std::fs::read(coldkey_path).expect("Failed to read coldkey file");
        let decrypted = decrypt(&encrypted_data, password.as_bytes())
            .expect("Failed to decrypt Python-created coldkey");

        let decrypted_str = String::from_utf8(decrypted).expect("not utf-8");
        assert!(
            decrypted_str.contains(expected_secret_hex),
            "Decrypted data does not contain expected secret key\nGot: {decrypted_str}\nExpected to contain: {expected_secret_hex}"
        );
    }

    #[test]
    fn argon2i_params_match_libsodium() {
        assert_eq!(argon2i13::OPSLIMIT_SENSITIVE.0, 8);
        assert_eq!(argon2i13::MEMLIMIT_SENSITIVE.0, 536870912);
    }

    #[test]
    fn salt_matches_btwallet() {
        let expected_salt: &[u8] = b"\x13q\x83\xdf\xf1Z\t\xbc\x9c\x90\xb5Q\x879\xe9\xb1";
        assert_eq!(NACL_SALT, expected_salt);
    }
}
