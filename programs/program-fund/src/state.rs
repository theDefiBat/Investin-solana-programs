use std::cell::{Ref, RefMut};
use solana_program::pubkey::Pubkey;
use solana_program::account_info::AccountInfo;
use solana_program::program_pack::{IsInitialized, Sealed};
use solana_program::program_error::ProgramError;
use solana_program::clock::UnixTimestamp;
use bytemuck::{from_bytes, from_bytes_mut, Pod, Zeroable};

pub const NUM_TOKENS:usize = 10;
pub const MAX_INVESTORS:usize = 10;
pub const MAX_FUNDS:usize = 200;

pub trait Loadable: Pod {
    fn load_mut<'a>(account: &'a AccountInfo) -> Result<RefMut<'a, Self>, ProgramError> {
        Ok(RefMut::map(account.try_borrow_mut_data()?, |data| from_bytes_mut(data)))
    }
    fn load<'a>(account: &'a AccountInfo) -> Result<Ref<'a, Self>, ProgramError> {
        Ok(Ref::map(account.try_borrow_data()?, |data| from_bytes(data)))
    }
    fn load_from_bytes(data: &[u8]) -> Result<&Self, ProgramError> {
        Ok(from_bytes(data))
    }
}

/// Struct wrapping data and providing metadata
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlatformData {

    pub is_initialized: bool,
    // router nonce for signing
    pub router_nonce: u8,
    // keep track of active funds
    pub no_of_active_funds: u8,
    pub padding: [u8; 5],

    // PDA of router
    pub router: Pubkey,

    // Investin admin
    pub investin_admin: Pubkey,

    // vault for protocol fee
    //pub investin_vault: Pubkey,

    // Fund managers list
    pub fund_managers: [Pubkey; MAX_FUNDS]
}
unsafe impl Zeroable for PlatformData {}
unsafe impl Pod for PlatformData {}
impl Loadable for PlatformData {}


#[repr(C)]
#[derive(Clone, Copy)]
pub struct InvestorData {

    pub is_initialized: bool,
    pub padding: [u8; 7],

    /// Investor wallet address
    pub owner: Pubkey,

    /// The Initial deposit (in USDT tokens)
    pub amount: u64,

    /// The performance of the fund at the time of investment
    pub start_performance: u64,

    /// Amount In Router for multiple investments
    pub amount_in_router: u64,

    // Fund manager
    pub manager: Pubkey,

}
unsafe impl Zeroable for InvestorData {}
unsafe impl Pod for InvestorData {}
impl Loadable for InvestorData {}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FundData {

    pub is_initialized: bool,
    // decimals
    pub decimals: u8,
    /// Number of Active Investments in fund
    pub number_of_active_investments: u8,
    /// Total Number of investments in fund
    pub no_of_investments: u8,
    // nonce to sign transactions
    pub signer_nonce: u8,
    // padding
    pub padding: [u8; 3],

    /// Minimum Amount
    pub min_amount: u64,

    /// Minimum Return
    pub min_return: u64,

    /// Performance Fee Percentage
    pub performance_fee_percentage: u64,

    /// Total Amount in fund
    pub total_amount: u64,

    /// Preformance in fund
    pub prev_performance: u64,

    /// Amount in Router
    pub amount_in_router: u64,

    /// Performance Fee
    pub performance_fee: u64,

    /// Wallet Address of the Manager
    pub manager_account: Pubkey,

    /// Tokens owned
    pub tokens: [TokenInfo; NUM_TOKENS],

    // Store investor state account addresses
    pub investors: [Pubkey; MAX_INVESTORS]
}
unsafe impl Zeroable for FundData {}
unsafe impl Pod for FundData {}
impl Loadable for FundData {}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TokenInfo {
    // Token Mint
    pub mint: Pubkey,

    // decimals (u64 for packing)
    pub decimals: u64,

    // Token Account Address
    pub vault: Pubkey,

    // Updated balance of token
    pub balance: u64,

}
unsafe impl Zeroable for TokenInfo {}
unsafe impl Pod for TokenInfo {}

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

impl Sealed for PlatformData {}
impl IsInitialized for PlatformData {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

pub const MAX_TOKENS:usize = 50;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PriceInfo {
    // mint address of the token
    pub token_mint: Pubkey,

    pub pool_account: Pubkey,

    pub base_pool_account: Pubkey,

    // decimals for token
    pub decimals: u64,

    // price of token
    pub token_price: u64,

    // last updated timestamp
    pub last_updated: UnixTimestamp,

}
unsafe impl Zeroable for PriceInfo{}
unsafe impl Pod for PriceInfo {}

/// Define the type of state stored in accounts
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PriceAccount {
    /// number of tokens
    pub count: u32,

    /// decimals
    pub decimals: u32,

    /// token price info
    pub prices: [PriceInfo; MAX_TOKENS]
}
unsafe impl Zeroable for PriceAccount {}
unsafe impl Pod for PriceAccount {}
impl Loadable for PriceAccount {}



