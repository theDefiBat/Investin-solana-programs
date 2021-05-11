use crate::id;
use solana_program::{
    pubkey::Pubkey,
};

/// Struct wrapping data and providing metadata
// #[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct FundData {

    /// The account allowed to update the data
    pub owner: Pubkey,

    /// The data contained by the account, could be anything serializable
    pub amount: u64,

    // pub flag: bool,
} 