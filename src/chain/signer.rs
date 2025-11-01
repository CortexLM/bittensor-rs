use sp_core::{sr25519, Pair};
use sp_runtime::{
    traits::{IdentifyAccount, Verify},
    MultiSignature as SpMultiSignature,
};
use subxt::{
    config::substrate::{AccountId32, MultiSignature},
    tx::Signer,
    Config, PolkadotConfig,
};

/// A concrete PairSigner implementation for sr25519::Pair compatible with subxt 0.44
/// This implements the Signer trait required by subxt
#[derive(Clone)]
pub struct PairSigner {
    account_id: <PolkadotConfig as Config>::AccountId,
    signer: sr25519::Pair,
}

impl PairSigner {
    /// Creates a new PairSigner from an sr25519::Pair
    pub fn new(signer: sr25519::Pair) -> Self {
        let account_id =
            <SpMultiSignature as Verify>::Signer::from(Pair::public(&signer)).into_account();
        Self {
            // Convert sp_core::AccountId32 to subxt::config::substrate::AccountId32
            account_id: AccountId32(account_id.into()),
            signer,
        }
    }

    /// Returns the sr25519::Pair used for signing
    pub fn signer(&self) -> &sr25519::Pair {
        &self.signer
    }

    /// Return the account ID
    pub fn account_id(&self) -> &AccountId32 {
        &self.account_id
    }
}

impl Signer<PolkadotConfig> for PairSigner {
    fn account_id(&self) -> <PolkadotConfig as Config>::AccountId {
        self.account_id.clone()
    }

    fn sign(&self, signer_payload: &[u8]) -> <PolkadotConfig as Config>::Signature {
        let signature = Pair::sign(&self.signer, signer_payload);
        MultiSignature::Sr25519(signature.0)
    }
}

/// Type alias for BittensorSigner
pub type BittensorSigner = PairSigner;

/// Create a signer from a keypair
pub fn create_signer(pair: sr25519::Pair) -> BittensorSigner {
    PairSigner::new(pair)
}

/// Create a signer from a seed phrase or key
pub fn signer_from_seed(seed: &str) -> anyhow::Result<BittensorSigner> {
    use sp_core::crypto::Pair as CryptoPair;
    let pair = sr25519::Pair::from_string(seed, None)
        .map_err(|e| anyhow::anyhow!("Failed to create pair from seed: {:?}", e))?;
    Ok(create_signer(pair))
}
