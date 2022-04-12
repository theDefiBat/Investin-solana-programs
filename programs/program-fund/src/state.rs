use std::cell::{Ref, RefMut};
use std::mem::size_of;
use mango::matching::Side;
use solana_program::pubkey::Pubkey;
use solana_program::account_info::AccountInfo;
use solana_program::program_pack::{IsInitialized, Sealed};
use solana_program::program_error::ProgramError;
use solana_program::clock::UnixTimestamp;
use solana_program::msg;
use bytemuck::{from_bytes, from_bytes_mut, Pod, Zeroable};
use fixed::types::U64F64;
use fixed::types::I80F48;
use crate::error::FundError;

pub const NUM_TOKENS:usize = 8;
pub const MAX_TOKENS:usize = 50;
pub const MAX_INVESTORS:usize = 10;
pub const MAX_INVESTORS_WITHDRAW: usize = 2;
pub const NUM_MARGIN: usize = 2;
pub const NUM_PERP: usize = 3;
pub const MAX_LIMIT_ORDERS:usize = 2;

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
    pub padding: u8,
    pub total_v3_funds: u16,

    // PDA of router
    pub router: Pubkey,

    // Investin admin
    pub investin_admin: Pubkey,

    // vault for protocol fee
    pub investin_vault: Pubkey,

    pub token_list: [TokenInfo; MAX_TOKENS]
}
impl_loadable!(PlatformData);

/// Struct wrapping data and providing metadata
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AmmInfo {

    pub padding: [u64; 24],

    pub need_take_pnl_coin: u64,
    pub need_take_pnl_pc: u64,

    pub paddingx: [u64; 68],

}
impl_loadable!(AmmInfo);

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FundAccount {

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
    pub is_private: bool,
    pub fund_v3_index: u16,
    pub padding: [u8; 4],

    /// Minimum Amount
    pub min_amount: u64,

    /// Minimum Return
    // pub min_return: U64F64,
    pub mr_padding: [u8; 16],

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

     pub guard: SwapGuard,

     pub limit_orders : [LimitOrderInfo; MAX_LIMIT_ORDERS], // 48 each = 96 
     
    //  pub margin_update_padding: [u8; 24], //80 Bytes for Depr. MarginInfo Size

    pub migration_additonal_padding: [u8; 1952] // 2024 + 24 - 96 =  1
}
impl_loadable!(FundAccount);

#[repr(C)]
#[derive(Clone, Copy, Debug)]
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
#[derive(Clone, Copy, Debug)]
pub struct SwapGuard {
    pub is_active: bool,
    pub is_split: bool,
    pub hop: u8,
    pub count: u8,
    pub token_in_slot: u8,
    pub token_out_slot: u8,
    pub padding: [u8; 2],
    pub triggered_at: UnixTimestamp,
    pub input_value: U64F64,
    pub ex_padding: [u8; 40],
    pub amount_in: u64,
    pub min_amount_out: u64,
}
impl_loadable!(SwapGuard);

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct InvestorData {

    pub is_initialized: bool,
    pub has_withdrawn: bool,
    pub withdrawn_from_margin: bool,
    pub padding: [u8; 5],

    /// Investor wallet address
    pub owner: Pubkey,

    /// The Initial deposit (in USDC tokens)
    pub amount: u64,

    // start performance of investor
    pub start_performance: U64F64,

    /// Amount In Router for multiple investments
    pub amount_in_router: u64,

    // Fund manager wallet key
    pub manager: Pubkey,

    // TODO Debt in Depost Tokens on Mango
    pub margin_debt: [U64F64; NUM_MARGIN], 

    // margin position id
    pub margin_position_id: [u64; NUM_MARGIN],

    // investor assets in tokens
    pub token_indexes: [u8; NUM_TOKENS],
    pub token_debts: [u64; NUM_TOKENS],

    pub share : U64F64,
    // padding for future use
    pub xpadding: [u8; 16] 
}
impl_loadable!(InvestorData);

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MangoInfo {
    // margin account pubkey to check if the passed acc is correct
    pub mango_account: Pubkey, 
    pub perp_markets: [u8; 3], // default u8::MAX
    pub perp_padding: u8,
    pub deposit_index: u8,
    pub markets_active: u8,
    pub deposits_active: u8,
    pub xpadding: u8,

    pub investor_debts: [u64; 2], // cumulative investor debts for each deposit token 
    pub padding: [u8; 24]
}
impl_loadable!(MangoInfo);

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct LimitOrderInfo { 
    // PerpOrderInfo for Limit Orders [ 1 + 8 + 8 + 8 + 8 + 8 + 1 + 1 + 1 + 1 ] = 45 bytes
    pub price: i64,
    pub max_base_quantity: i64,
    pub max_quote_quantity: i64,
    pub client_order_id: u64, // 0 = means inActive
    pub expiry_timestamp: u64,
    pub is_repost_processing: bool,
    pub perp_market_id: u8,
    pub side: Side,
    pub reduce_only: bool,
    pub limit: u8,
    pub padding :[u8;3],
}
impl_loadable!(LimitOrderInfo);

impl Sealed for InvestorData {}
impl IsInitialized for InvestorData {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}


impl Sealed for FundAccount {}
impl IsInitialized for FundAccount {
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

        // msg!("Platform data:: {:?}, {:?}", account.data_len(), size_of::<Self>());
        check_eq!(account.data_len(), size_of::<Self>());
        // msg!("size check done");
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


impl FundAccount {
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
    pub fn find_slot_by_client_id(&self, client_order_id: u64) -> Option<usize> {
        self.limit_orders.iter().position(|limitOrderInfo| (*limitOrderInfo).client_order_id == client_order_id)
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
