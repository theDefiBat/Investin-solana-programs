use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::pubkey::Pubkey;
use solana_program::program_pack::{IsInitialized, Sealed};


pub const NUM_TOKENS:usize = 3;

/// Struct wrapping data and providing metadata
/// Struct wrapping data and providing metadata
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct InvestorData {
    pub is_initialized: bool,

    /// Investor wallet address
    pub owner: Pubkey,

    /// The Initial deposit (in USDT tokens)
    pub amount: u64,

    /// The performance of the fund at the time of investment
    pub start_performance: u64,

    // Fund manager
    pub manager: Pubkey,

    // nonce to sign transactions
    pub signer_nonce: u8
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct FundData {
    pub is_initialized: bool,

    /// Wallet Address of the Manager
    pub manager_account: Pubkey,

    // nonce to sign transactions
    pub signer_nonce: u8,

    /// Minimum Amount
    pub min_amount: u64,

    /// Minimum Return
    pub min_return: u64,

    /// Performance Fee Percentage
    pub performance_fee_percentage: u64,

    /// Total Amount in fund
    pub total_amount: u64,

    // decimals
    pub decimals: u8,

    /// Preformance in fund
    pub prev_performance: u64,

    /// Number of Active Investments in fund
    pub number_of_active_investments: u64,

    /// Amount in Router
    pub amount_in_router: u64,

    /// Tokens owned
    pub tokens: [TokenInfo; NUM_TOKENS]
}

#[derive(Clone, Debug, Default, Copy, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct TokenInfo {
    // Token Mint
    pub mint: Pubkey,

    // decimals
    pub decimals: u8,

    // Token Account Address
    pub vault: Pubkey,

    // Updated balance of token
    pub balance: u64,

}

impl Sealed for InvestorData {}
impl IsInitialized for InvestorData {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Sealed for FundData {}
impl IsInitialized for FundData {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}


