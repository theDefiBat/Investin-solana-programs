use std::cell::{Ref, RefMut};
use std::mem::size_of;
use solana_program::pubkey::Pubkey;
use solana_program::account_info::AccountInfo;
use solana_program::program_pack::{IsInitialized, Sealed};
use solana_program::program_error::ProgramError;
use solana_program::clock::UnixTimestamp;
use solana_program::msg;
use bytemuck::{from_bytes, from_bytes_mut, Pod, Zeroable};
use fixed::types::U64F64;
use crate::error::FundError;

pub const NUM_TOKENS:usize = 8;
pub const MAX_TOKENS:usize = 50;
pub const MAX_INVESTORS:usize = 10;
pub const MAX_INVESTORS_WITHDRAW: usize = 2;
pub const NUM_MARGIN: usize = 2;
pub const NUM_PERP: usize = 4;

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

macro_rules! check_eq {
    ($x:expr, $y:expr) => {
        if ($x != $y) {
            return Err(FundError::Default.into())
        }
    }
}

/// Struct wrapping data and providing metadata
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlatformData {

    pub is_initialized: bool,
    // version info
    pub version: u8,
    // router nonce for signing
    pub router_nonce: u8,
    // keep track of active funds
    pub no_of_active_funds: u8,
    // running count of tokens in whitelist
    pub token_count: u8,
    pub padding: [u8; 3],

    // PDA of router
    pub router: Pubkey,

    // Investin admin
    pub investin_admin: Pubkey,

    // vault for protocol fee
    pub investin_vault: Pubkey,

    pub token_list: [TokenInfo; MAX_TOKENS]
}
impl_loadable!(PlatformData);

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FundData {

    pub is_initialized: bool,
    /// Number of Active Investments in fund
    pub number_of_active_investments: u8,
    /// Total Number of investments in fund
    pub no_of_investments: u8,
    // nonce to sign transactions
    pub signer_nonce: u8,
    /// Number of open margin positions
    pub no_of_margin_positions: u8,
    /// Number of active tokens
    pub no_of_assets: u8,
    /// Position count
    pub position_count: u16,

    /// version info
    pub version: u8,
    pub padding: [u8; 7],
  
    /// Minimum Amount
    pub min_amount: u64,

    /// Minimum Return
    pub min_return: U64F64,

    /// Performance Fee Percentage
    pub performance_fee_percentage: U64F64,

    /// Total Amount in fund (in USDC)
    pub total_amount: U64F64,

    /// Preformance in fund
    pub prev_performance: U64F64,

    /// Amount in Router (in USDC)
    pub amount_in_router: u64,

    /// Performance Fee
    pub performance_fee: U64F64,

    /// Wallet Address of the Manager
    pub manager_account: Pubkey,

    /// Fund PDA
    pub fund_pda: Pubkey,

    /// Tokens owned
    pub tokens: [TokenSlot; NUM_TOKENS],

    // Store investor state account addresses
    pub investors: [Pubkey; MAX_INVESTORS],

    // margin position info
    pub mango_positions: [MarginInfo; 2],

    // padding for future use
    pub xpadding: [u8; 32]
}
impl_loadable!(FundData);

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FundDataNew {

    pub is_initialized: bool,
    /// Number of Active Investments in fund
    pub number_of_active_investments: u8,
    /// Total Number of investments in fund
    pub no_of_investments: u8,
    // nonce to sign transactions
    pub signer_nonce: u8,
    /// Number of open margin positions
    pub no_of_margin_positions: u8,
    /// Number of active tokens
    pub no_of_assets: u8,
    /// Position count
    pub position_count: u16,

    /// version info
    pub version: u8,
    pub padding: [u8; 7],

    /// Minimum Amount
    pub min_amount: u64,

    /// Minimum Return
    pub min_return: U64F64,

    /// Performance Fee Percentage
    pub performance_fee_percentage: U64F64,

    /// Total Amount in fund (in USDC)
    pub total_amount: U64F64,

    /// Preformance in fund
    pub prev_performance: U64F64,

    /// Amount in Router (in USDC)
    pub amount_in_router: u64,

    /// Performance Fee
    pub performance_fee: U64F64,

    /// Wallet Address of the Manager
    pub manager_account: Pubkey,

    /// Fund PDA
    pub fund_pda: Pubkey,

     /// Tokens owned
     pub tokens: [TokenSlot; NUM_TOKENS],

     // Store investor state account addresses
     pub investors: [Pubkey; MAX_INVESTORS],
 
     // mango position info
     pub mango_positions: MangoInfo,

     pub margin_update_padding: [u8; 80], //80 Bytes for Depr. MarginInfo Size
 
     // padding for future use
     pub xpadding: [u8; 32]
}
impl_loadable!(FundData);

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TokenSlot {
    // state vars
    pub is_active: bool,
    pub index: [u8; 3],
    pub mux: u8,
    pub padding: [u8; 3],

    // token balances & debts
    pub balance: u64,
    pub debt: u64,

    // token vault account
    pub vault: Pubkey,
}
impl_loadable!(TokenSlot);


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

    // margin percentage
    pub margin_debt: [U64F64; NUM_MARGIN],

    // margin position id
    pub margin_position_id: [u64; NUM_MARGIN],

    // investor assets in tokens
    pub token_indexes: [u8; NUM_TOKENS],
    pub token_debts: [u64; NUM_TOKENS],

    // padding for future use
    pub xpadding: [u8; 32]
}
impl_loadable!(InvestorData);

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
    
    pub trade_amount: u64, // 8 for PnL calculation

    pub fund_share: U64F64,
    pub share_ratio: U64F64
}
impl_loadable!(MarginInfo);

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MangoInfo {
    // margin account pubkey to check if the passed acc is correct
    pub mango_account: Pubkey, 
    pub perp_markets: [u8; 4],
    pub deposit_index: u8,
    pub markets_active: u8,
    pub deposits_active: u8,
    pub xpadding: u8,

    pub investor_debts: [u64; 2], // cumulative investor debts for each deposit token 
    pub padding: [u8; 24]
}
impl_loadable!(MangoInfo);

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

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TokenInfo {
    // mint address of the token
    pub mint: Pubkey,
    // decimals for token, not used can be used later
    pub decimals: u64,
    
    // poolCoinTokenAccount for pool price
    pub pool_coin_account: Pubkey,
    // poolPcTokenAccount for pool price
    pub pool_pc_account: Pubkey,
    // price of token
    pub pool_price: U64F64,
    // last updated timestamp
    pub last_updated: UnixTimestamp,

    pub token_id: u8, // 0 -> raydium, 1-> orca, 2 -> strategy

    pub pc_index: u8, // 0 -> USDC (index of the pc i.e base's tokenInfo in platformState)
    // padding for future use
    pub padding: [u8; 6],
}
impl_loadable!(TokenInfo);


impl PlatformData {
    pub fn load_mut_checked<'a>(
        account: &'a AccountInfo,
        program_id: &Pubkey
    ) -> Result<RefMut<'a, Self>, ProgramError> {

        msg!("Platform data:: {:?}, {:?}", account.data_len(), size_of::<Self>());
        check_eq!(account.data_len(), size_of::<Self>());
        msg!("size check done");
        check_eq!(account.owner, program_id);

        let data = Self::load_mut(account)?;
        Ok(data)
    }
    pub fn load_checked<'a>(
        account: &'a AccountInfo,
        program_id: &Pubkey
    ) -> Result<Ref<'a, Self>, ProgramError> {
        check_eq!(account.data_len(), size_of::<Self>());  // TODO not necessary check
        check_eq!(account.owner, program_id);

        let data = Self::load(account)?;
        Ok(data)
    }
    pub fn get_token_index(&self, mint_pk: &Pubkey, token_id: u8) -> Option<usize> {
        self.token_list.iter().position(|token| ((token.mint == *mint_pk) && (token.token_id == token_id)))
    }
    pub fn get_token_index_by_coin(&self, pool_coin_account: &Pubkey) -> Option<usize> {
        self.token_list.iter().position(|token| token.pool_coin_account == *pool_coin_account)
    }
}

impl FundData {
    pub fn load_mut_checked<'a>(
        account: &'a AccountInfo,
        program_id: &Pubkey
    ) -> Result<RefMut<'a, Self>, ProgramError> {

        check_eq!(account.data_len(), size_of::<Self>());
        check_eq!(account.owner, program_id);

        let data = Self::load_mut(account)?;
        Ok(data)
    }
    pub fn load_checked<'a>(
        account: &'a AccountInfo,
        program_id: &Pubkey
    ) -> Result<Ref<'a, Self>, ProgramError> {
        check_eq!(account.data_len(), size_of::<Self>());  // TODO not necessary check
        check_eq!(account.owner, program_id);

        let data = Self::load(account)?;
        Ok(data)
    }
    pub fn get_token_slot(&self, index: usize, mux: usize) -> Option<usize> {
        self.tokens.iter().position(|token| token.index[mux] as usize == index)
    }
    pub fn get_margin_index(&self, margin_account_pk: &Pubkey) -> Option<usize> {
        self.mango_positions.iter().position(|pos| pos.margin_account == *margin_account_pk)
    }
    pub fn get_investor_index(&self, inv_state_pk: &Pubkey) -> Option<usize> {
        self.investors.iter().position(|pos| *pos == *inv_state_pk)
    }
}

impl FundDataNew {
    pub fn load_mut_checked<'a>(
        account: &'a AccountInfo,
        program_id: &Pubkey
    ) -> Result<RefMut<'a, Self>, ProgramError> {

        check_eq!(account.data_len(), size_of::<Self>());
        check_eq!(account.owner, program_id);

        let data = Self::load_mut(account)?;
        Ok(data)
    }
    pub fn load_checked<'a>(
        account: &'a AccountInfo,
        program_id: &Pubkey
    ) -> Result<Ref<'a, Self>, ProgramError> {
        check_eq!(account.data_len(), size_of::<Self>());  // TODO not necessary check
        check_eq!(account.owner, program_id);

        let data = Self::load(account)?;
        Ok(data)
    }
    pub fn get_token_slot(&self, index: usize, mux: usize) -> Option<usize> {
        self.tokens.iter().position(|token| token.index[mux] as usize == index)
    }
    pub fn get_mango_perp_index(&self, mango_perp_index: u8) -> Option<usize> {
        self.mango_positions.perp_markets.iter().position(|pmid| *pmid == mango_perp_index)
    }
    pub fn get_investor_index(&self, inv_state_pk: &Pubkey) -> Option<usize> {
        self.investors.iter().position(|pos| *pos == *inv_state_pk)
    }
}

impl InvestorData {
    pub fn load_mut_checked<'a>(
        account: &'a AccountInfo,
        program_id: &Pubkey
    ) -> Result<RefMut<'a, Self>, ProgramError> {

        check_eq!(account.data_len(), size_of::<Self>());
        check_eq!(account.owner, program_id);

        let data = Self::load_mut(account)?;
        Ok(data)
    }
    pub fn load_checked<'a>(
        account: &'a AccountInfo,
        program_id: &Pubkey
    ) -> Result<Ref<'a, Self>, ProgramError> {
        check_eq!(account.data_len(), size_of::<Self>());  // TODO not necessary check
        check_eq!(account.owner, program_id);

        let data = Self::load(account)?;
        Ok(data)
    }
}
