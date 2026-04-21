//! bittensor-wallet — wallet management for the bittensor-rs SDK

pub mod keyfile;
pub mod keypair;
pub mod mnemonic;
pub mod ss58;
pub mod wallet;

pub mod prelude {
    pub use crate::keyfile::{KeyfileError, decrypt, encrypt, is_encrypted_nacl};
    pub use crate::keypair::{Keypair, KeypairError};
    pub use crate::mnemonic::{MnemonicError, WordCount};
    pub use crate::ss58::{Ss58Error, decode_ss58, encode_ss58, encode_ss58_address};
    pub use crate::wallet::{Wallet, WalletError};
}
