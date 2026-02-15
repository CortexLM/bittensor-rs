use serde::{Deserialize, Serialize};
use sp_core::crypto::AccountId32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalVoteData {
    pub index: u32,
    pub threshold: u32,
    #[serde(with = "crate::utils::ss58::serde_account_vec")]
    pub ayes: Vec<AccountId32>,
    #[serde(with = "crate::utils::ss58::serde_account_vec")]
    pub nays: Vec<AccountId32>,
    pub end: u32,
}
