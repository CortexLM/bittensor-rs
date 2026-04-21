//! MEV Shield — encrypted extrinsic submission using ML-KEM-768.
//!
//! Fetches the on-chain NextKey (ML-KEM-768 post-quantum public key),
//! encrypts the extrinsic payload, and submits it via
//! `submit_encrypted_extrinsic`.
//!
//! Feature-gated: `#[cfg(feature = "mev-shield")]`

pub mod encrypt;
pub mod submit;

pub use encrypt::{EncryptedPayload, MevShieldEncrypt, MevShieldEncryptError};
pub use submit::{MevShieldSubmit, MevShieldSubmitError};
