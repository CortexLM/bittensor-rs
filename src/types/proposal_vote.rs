use serde::{Deserialize, Serialize};
use sp_core::crypto::AccountId32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalVoteData {
    pub index: u64,
    pub threshold: u64,
    pub ayes: Vec<AccountId32>,
    pub nays: Vec<AccountId32>,
    pub end: u64,
}
