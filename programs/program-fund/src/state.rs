use std::cell::{Ref, RefMut};
use solana_program::pubkey::Pubkey;
use solana_program::account_info::AccountInfo;
use solana_program::program_pack::{IsInitialized, Sealed};
use solana_program::program_error::ProgramError;
use solana_program::clock::UnixTimestamp;
use bytemuck::{from_bytes, from_bytes_mut, Pod, Zeroable};
use fixed::types::U64F64;



pub const NUM_TOKENS:usize = 10;
pub const MAX_INVESTORS:usize = 10;
pub const MAX_INVESTORS_WITHDRAW: usize = 2;

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

macro_rules! impl_loadable {
    ($type_name:ident) => {
        unsafe impl Zeroable for $type_name {}
        unsafe impl Pod for $type_name {}
        impl Loadable for $type_name {}
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
    pub investin_vault: Pubkey,
}
impl_loadable!(PlatformData);

#[repr(C)]
#[derive(Clone, Copy)]
pub struct InvestorData {

    pub is_initialized: bool,
    pub has_withdrawn: bool,
    pub withdrawn_from_margin: bool,
    pub padding: [u8; 5],

    /// Investor wallet address
    pub owner: Pubkey,

    /// The Initial deposit (in USDT tokens)
    pub amount: u64,

    // start performance of investor
    pub start_performance: U64F64,

    /// Amount In Router for multiple investments
    pub amount_in_router: u64,

    // Fund manager wallet key
    pub manager: Pubkey,

    // margin assets owed in USDC tokens
    pub margin_debt: u64,

    // margin position id
    pub margin_position_id: u64,

    // investor assets in tokens
    pub fund_debt: [u64; NUM_TOKENS]   
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
    /// Number of open margin positions
    pub no_of_margin_positions: u8,
    /// Position count
    pub position_count: u16,
  
    /// Minimum Amount
    pub min_amount: u64,

    /// Minimum Return
    pub min_return: U64F64,

    /// Performance Fee Percentage
    pub performance_fee_percentage: U64F64,

    /// Total Amount in fund (in USDC)
    pub total_amount: u64,

    /// Preformance in fund
    pub prev_performance: U64F64,

    /// Amount in Router (in USDC)
    pub amount_in_router: u64,

    /// Performance Fee
    pub performance_fee: U64F64,

    /// Wallet Address of the Manager
    pub manager_account: Pubkey,

    /// Tokens owned
    pub tokens: [TokenInfo; NUM_TOKENS],

    // Store investor state account addresses
    pub investors: [Pubkey; MAX_INVESTORS],

    // margin position info
    pub mango_positions: [MarginInfo; 2]
}
unsafe impl Zeroable for FundData {}
unsafe impl Pod for FundData {}
impl Loadable for FundData {}


#[repr(C)]
#[derive(Clone, Copy)]
pub struct MarginInfo {
    // margin account pubkey to check if the passed acc is correct
    pub margin_account: Pubkey,

    // 0: inactive, 1: deposited, 2: position_open, 3: settled_open, 4: position_closed, 5: settled_close
    pub state: u8, 
    pub margin_index: u8, // token_index for the trade
    pub position_side: u8, // 0 for LONG, 1 for SHORT
    pub padding: [u8; 3],
    pub position_id: u16, // unique id for the position
    
    pub trade_amount: u64, // used for PnL calculation

    pub close_collateral: U64F64,

    pub investor_debt: u64 // updated on every investor withdraw
}

unsafe impl Zeroable for MarginInfo {}
unsafe impl Pod for MarginInfo {}
impl Loadable for MarginInfo {}

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

    // token debt owed to investors
    pub debt: u64

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


