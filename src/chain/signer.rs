use sp_core::{sr25519, Pair};
use sp_runtime::{
    traits::{IdentifyAccount, Verify},
    MultiSignature as SpMultiSignature,
};
use std::collections::HashMap;
use std::sync::Arc;
use subxt::{
    config::substrate::{AccountId32, MultiSignature},
    tx::Signer,
    Config, PolkadotConfig,
};
use tokio::sync::Mutex;
use tracing::{debug, warn};

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

/// Nonce tracking for a single account
#[derive(Debug)]
struct AccountNonceState {
    /// Last known on-chain nonce (from account query)
    on_chain_nonce: u64,
    /// Next nonce to use for transactions (local tracking)
    next_nonce: u64,
    /// Nonces currently in-flight (submitted but not confirmed)
    in_flight: std::collections::HashSet<u64>,
    /// Nonces that failed and should be retried
    failed: std::collections::VecDeque<u64>,
    /// Last nonce update timestamp
    last_update: std::time::Instant,
}

impl AccountNonceState {
    fn new(on_chain_nonce: u64) -> Self {
        Self {
            on_chain_nonce,
            next_nonce: on_chain_nonce,
            in_flight: std::collections::HashSet::new(),
            failed: std::collections::VecDeque::new(),
            last_update: std::time::Instant::now(),
        }
    }

    /// Get the next available nonce
    fn get_next_nonce(&mut self) -> u64 {
        // First, check if there are any failed nonces to retry
        while let Some(nonce) = self.failed.pop_front() {
            if !self.in_flight.contains(&nonce) {
                self.in_flight.insert(nonce);
                return nonce;
            }
        }

        // Use the next sequential nonce
        let nonce = self.next_nonce;
        self.in_flight.insert(nonce);
        self.next_nonce += 1;
        nonce
    }

    /// Mark a nonce as confirmed/successful
    fn confirm_nonce(&mut self, nonce: u64) {
        self.in_flight.remove(&nonce);
        // Update on_chain_nonce if this was the expected next one
        if nonce == self.on_chain_nonce {
            self.on_chain_nonce = nonce + 1;
        }
    }

    /// Mark a nonce as failed (for retry)
    fn fail_nonce(&mut self, nonce: u64) {
        self.in_flight.remove(&nonce);
        self.failed.push_back(nonce);
        // Don't increment next_nonce - we'll retry this one
    }

    /// Reset nonce tracking when nonce errors occur
    fn reset(&mut self, new_on_chain_nonce: u64) {
        debug!(
            "Resetting nonce tracking: {} -> {}",
            self.on_chain_nonce, new_on_chain_nonce
        );
        self.on_chain_nonce = new_on_chain_nonce;
        self.next_nonce = new_on_chain_nonce;
        self.in_flight.clear();
        self.failed.clear();
        self.last_update = std::time::Instant::now();
    }

    /// Check if nonce tracking needs refresh (after certain time)
    fn needs_refresh(&self, max_age: std::time::Duration) -> bool {
        self.last_update.elapsed() > max_age
    }
}

/// Thread-safe nonce manager for concurrent transaction signing
///
/// This manager handles:
/// - Sequence tracking for each account
/// - Concurrent transaction nonce allocation
/// - Failed transaction retry with correct nonces
/// - Automatic re-sync when nonce errors occur
#[derive(Debug)]
pub struct NonceManager {
    /// Per-account nonce tracking - using String key since AccountId32 doesn't implement Hash
    accounts: Mutex<HashMap<String, AccountNonceState>>,
    /// Maximum age before refreshing nonce from chain
    max_nonce_age: std::time::Duration,
}

impl NonceManager {
    /// Create a new nonce manager with default settings
    pub fn new() -> Self {
        Self::with_config(std::time::Duration::from_secs(60))
    }

    /// Create a nonce manager with custom configuration
    pub fn with_config(max_nonce_age: std::time::Duration) -> Self {
        Self {
            accounts: Mutex::new(HashMap::new()),
            max_nonce_age,
        }
    }

    /// Generate a key for the account
    fn account_key(account: &AccountId32) -> String {
        format!("{:?}", account.0)
    }

    /// Initialize or update nonce tracking for an account
    pub async fn initialize_account(&self, account: &AccountId32, on_chain_nonce: u64) {
        let mut accounts = self.accounts.lock().await;
        let key = Self::account_key(account);
        accounts.insert(key, AccountNonceState::new(on_chain_nonce));
        debug!(
            "Initialized nonce tracking for {:?} at nonce {}",
            account, on_chain_nonce
        );
    }

    /// Get the next nonce for an account (for concurrent transaction submission)
    ///
    /// This method allocates a nonce for immediate use. The caller must either
    /// confirm or fail the nonce after the transaction completes.
    pub async fn allocate_nonce(&self, account: &AccountId32) -> Option<u64> {
        let mut accounts = self.accounts.lock().await;
        let key = Self::account_key(account);

        if let Some(state) = accounts.get_mut(&key) {
            let nonce = state.get_next_nonce();
            debug!("Allocated nonce {} for {:?}", nonce, account);
            Some(nonce)
        } else {
            warn!("Account {:?} not initialized in nonce manager", account);
            None
        }
    }

    /// Confirm a nonce was successfully used
    pub async fn confirm_nonce(&self, account: &AccountId32, nonce: u64) {
        let mut accounts = self.accounts.lock().await;
        let key = Self::account_key(account);

        if let Some(state) = accounts.get_mut(&key) {
            state.confirm_nonce(nonce);
            debug!("Confirmed nonce {} for {:?}", nonce, account);
        }
    }

    /// Mark a nonce as failed (will be retried)
    pub async fn fail_nonce(&self, account: &AccountId32, nonce: u64) {
        let mut accounts = self.accounts.lock().await;
        let key = Self::account_key(account);

        if let Some(state) = accounts.get_mut(&key) {
            state.fail_nonce(nonce);
            warn!("Marked nonce {} failed for {:?}", nonce, account);
        }
    }

    /// Reset nonce tracking for an account (e.g., after nonce error)
    pub async fn reset_account(&self, account: &AccountId32, on_chain_nonce: u64) {
        let mut accounts = self.accounts.lock().await;
        let key = Self::account_key(account);

        if let Some(state) = accounts.get_mut(&key) {
            state.reset(on_chain_nonce);
        } else {
            accounts.insert(key, AccountNonceState::new(on_chain_nonce));
        }
    }

    /// Check if an account needs nonce refresh
    pub async fn needs_refresh(&self, account: &AccountId32) -> bool {
        let accounts = self.accounts.lock().await;
        let key = Self::account_key(account);

        if let Some(state) = accounts.get(&key) {
            state.needs_refresh(self.max_nonce_age)
        } else {
            true // Needs initialization
        }
    }

    /// Get current nonce state for an account
    pub async fn get_account_state(&self, account: &AccountId32) -> Option<NonceAccountState> {
        let accounts = self.accounts.lock().await;
        let key = Self::account_key(account);

        accounts.get(&key).map(|state| NonceAccountState {
            on_chain_nonce: state.on_chain_nonce,
            next_nonce: state.next_nonce,
            in_flight_count: state.in_flight.len() as u32,
            failed_count: state.failed.len() as u32,
        })
    }

    /// Remove an account from tracking
    pub async fn remove_account(&self, account: &AccountId32) {
        let mut accounts = self.accounts.lock().await;
        let key = Self::account_key(account);
        accounts.remove(&key);
    }

    /// Get all tracked accounts (returns empty list since we can't reconstruct from string keys)
    pub async fn tracked_accounts(&self) -> Vec<AccountId32> {
        // Cannot reconstruct AccountId32 from string representation
        Vec::new()
    }
}

impl Default for NonceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Nonce state for an account (public view)
#[derive(Debug, Clone, Copy)]
pub struct NonceAccountState {
    pub on_chain_nonce: u64,
    pub next_nonce: u64,
    pub in_flight_count: u32,
    pub failed_count: u32,
}

/// Signer with integrated nonce management for concurrent transactions
#[derive(Clone)]
pub struct ManagedSigner {
    signer: PairSigner,
    nonce_manager: Arc<NonceManager>,
}

impl ManagedSigner {
    /// Create a new managed signer from a pair
    pub fn new(pair: sr25519::Pair) -> Self {
        Self::from_pair_and_manager(pair, Arc::new(NonceManager::new()))
    }

    /// Create from a pair and existing nonce manager
    pub fn from_pair_and_manager(pair: sr25519::Pair, nonce_manager: Arc<NonceManager>) -> Self {
        let signer = PairSigner::new(pair);
        Self {
            signer,
            nonce_manager,
        }
    }

    /// Get the underlying pair signer
    pub fn signer(&self) -> &PairSigner {
        &self.signer
    }

    /// Get the account ID
    pub fn account_id(&self) -> &AccountId32 {
        self.signer.account_id()
    }

    /// Get the nonce manager
    pub fn nonce_manager(&self) -> &Arc<NonceManager> {
        &self.nonce_manager
    }

    /// Initialize nonce tracking for this signer
    pub async fn initialize_nonce(&self, on_chain_nonce: u64) {
        self.nonce_manager
            .initialize_account(self.account_id(), on_chain_nonce)
            .await;
    }

    /// Allocate a nonce for transaction submission
    pub async fn allocate_nonce(&self) -> Option<u64> {
        self.nonce_manager.allocate_nonce(self.account_id()).await
    }

    /// Confirm successful nonce usage
    pub async fn confirm_nonce(&self, nonce: u64) {
        self.nonce_manager
            .confirm_nonce(self.account_id(), nonce)
            .await;
    }

    /// Mark nonce as failed for retry
    pub async fn fail_nonce(&self, nonce: u64) {
        self.nonce_manager
            .fail_nonce(self.account_id(), nonce)
            .await;
    }

    /// Reset nonce tracking
    pub async fn reset_nonce(&self, on_chain_nonce: u64) {
        self.nonce_manager
            .reset_account(self.account_id(), on_chain_nonce)
            .await;
    }

    /// Check if nonce needs refresh
    pub async fn needs_refresh(&self) -> bool {
        self.nonce_manager.needs_refresh(self.account_id()).await
    }
}

impl Signer<PolkadotConfig> for ManagedSigner {
    fn account_id(&self) -> <PolkadotConfig as Config>::AccountId {
        self.signer.account_id().clone()
    }

    fn sign(&self, signer_payload: &[u8]) -> <PolkadotConfig as Config>::Signature {
        self.signer.sign(signer_payload)
    }
}

/// Shared nonce manager for multiple signers
#[derive(Clone)]
pub struct SharedNonceManager {
    inner: Arc<NonceManager>,
}

impl SharedNonceManager {
    /// Create a new shared nonce manager
    pub fn new() -> Self {
        Self {
            inner: Arc::new(NonceManager::new()),
        }
    }

    /// Create with custom config
    pub fn with_config(max_nonce_age: std::time::Duration) -> Self {
        Self {
            inner: Arc::new(NonceManager::with_config(max_nonce_age)),
        }
    }

    /// Get the inner nonce manager
    pub fn inner(&self) -> &Arc<NonceManager> {
        &self.inner
    }

    /// Create a managed signer from a seed using this shared manager
    pub fn create_managed_signer(&self, seed: &str) -> anyhow::Result<ManagedSigner> {
        use sp_core::crypto::Pair as CryptoPair;
        let pair = sr25519::Pair::from_string(seed, None)
            .map_err(|e| anyhow::anyhow!("Failed to create pair from seed: {:?}", e))?;
        Ok(ManagedSigner::from_pair_and_manager(
            pair,
            self.inner.clone(),
        ))
    }

    /// Create a managed signer from an existing pair
    pub fn create_managed_signer_from_pair(&self, pair: sr25519::Pair) -> ManagedSigner {
        ManagedSigner::from_pair_and_manager(pair, self.inner.clone())
    }
}

impl Default for SharedNonceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_account() -> AccountId32 {
        AccountId32([0u8; 32])
    }

    #[tokio::test]
    async fn test_nonce_manager_initialization() {
        let manager = NonceManager::new();
        let account = test_account();

        manager.initialize_account(&account, 5).await;

        let state = manager.get_account_state(&account).await.unwrap();
        assert_eq!(state.on_chain_nonce, 5);
        assert_eq!(state.next_nonce, 5);
    }

    #[tokio::test]
    async fn test_nonce_allocation() {
        let manager = NonceManager::new();
        let account = test_account();

        manager.initialize_account(&account, 5).await;

        // Allocate sequential nonces
        assert_eq!(manager.allocate_nonce(&account).await, Some(5));
        assert_eq!(manager.allocate_nonce(&account).await, Some(6));
        assert_eq!(manager.allocate_nonce(&account).await, Some(7));

        let state = manager.get_account_state(&account).await.unwrap();
        assert_eq!(state.in_flight_count, 3);
    }

    #[tokio::test]
    async fn test_nonce_confirmation() {
        let manager = NonceManager::new();
        let account = test_account();

        manager.initialize_account(&account, 5).await;

        let nonce = manager.allocate_nonce(&account).await.unwrap();
        assert_eq!(nonce, 5);

        manager.confirm_nonce(&account, nonce).await;

        let state = manager.get_account_state(&account).await.unwrap();
        assert_eq!(state.on_chain_nonce, 6);
        assert_eq!(state.in_flight_count, 0);
    }

    #[tokio::test]
    async fn test_nonce_failure_and_retry() {
        let manager = NonceManager::new();
        let account = test_account();

        manager.initialize_account(&account, 5).await;

        // Allocate some nonces
        let nonce1 = manager.allocate_nonce(&account).await.unwrap(); // 5
        let _nonce2 = manager.allocate_nonce(&account).await.unwrap(); // 6

        // Fail nonce 5
        manager.fail_nonce(&account, nonce1).await;

        // Next allocation should return 5 (retry)
        let retry_nonce = manager.allocate_nonce(&account).await.unwrap();
        assert_eq!(retry_nonce, 5);

        // Then continue with 7
        let nonce3 = manager.allocate_nonce(&account).await.unwrap();
        assert_eq!(nonce3, 7);

        let state = manager.get_account_state(&account).await.unwrap();
        assert_eq!(state.failed_count, 0); // All failed should be allocated
        assert_eq!(state.in_flight_count, 3); // 5, 6, 7
    }

    #[tokio::test]
    async fn test_nonce_reset() {
        let manager = NonceManager::new();
        let account = test_account();

        manager.initialize_account(&account, 5).await;
        manager.allocate_nonce(&account).await; // 5
        manager.allocate_nonce(&account).await; // 6

        // Simulate nonce error - chain says we're at nonce 10
        manager.reset_account(&account, 10).await;

        let state = manager.get_account_state(&account).await.unwrap();
        assert_eq!(state.on_chain_nonce, 10);
        assert_eq!(state.next_nonce, 10);
        assert_eq!(state.in_flight_count, 0);

        // New allocations start from 10
        assert_eq!(manager.allocate_nonce(&account).await, Some(10));
    }

    #[tokio::test]
    async fn test_account_not_initialized() {
        let manager = NonceManager::new();
        let account = test_account();

        // Should return None for uninitialized account
        assert_eq!(manager.allocate_nonce(&account).await, None);
    }
}
