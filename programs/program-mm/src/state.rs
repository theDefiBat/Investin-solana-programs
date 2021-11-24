use std::cell::{Ref, RefMut};
use std::mem::size_of;
use solana_program::pubkey::Pubkey;
use solana_program::account_info::AccountInfo;
use solana_program::program_pack::{IsInitialized, Sealed};
use solana_program::program_error::ProgramError;
use bytemuck::{from_bytes, from_bytes_mut, Pod, Zeroable};
use fixed::types::U64F64;
use crate::error::FundError;

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


#[repr(C)]
#[derive(Clone, Copy)]
pub struct FundData {

    pub is_initialized: bool,
    pub signer_nonce: u8,
    pub perp_market_indexes: [u8; 4],
    pub markets_active: u8;
    pub padding: u8,
    pub no_of_investments: u32,

    /// Minimum Amount
    pub min_amount: u64,

    /// Minimum Return
    pub min_return: U64F64,

    /// Performance Fee Percentage
    pub performance_fee_percentage: U64F64,

    /// Fund AUM
    pub total_amount: U64F64,

    /// Preformance in fund
    pub prev_performance: U64F64,

    /// Performance Fee
    pub performance_fee: U64F64,

    /// Fund Deposits
    pub deposits: u64,

    /// Vault balance
    pub vault_balance: u64,

    /// Wallet Address of the Manager
    pub manager_account: Pubkey,

    /// Fund PDA
    pub fund_pda: Pubkey,

    /// Vault account key
    pub vault_key: Pubkey,

    /// Fund Mngo Vault
    pub mngo_vault_key: Pubkey,

    /// Mango account for the fund
    pub mango_account: Pubkey,

    // /// Mango per share accrued
    // pub mngo_per_share: U64F64,

    // /// Mango due to Manager
    // pub mngo_manager: u64,

    // /// Mango Accrued in Mango Account
    // pub mngo_accrued: u64,

    // // Delegate for Manager to call place/cancel
    // pub delegate: Pubkey,

    // // all time Mngo accrual
    // pub total_mngo_accrued: u64
}
impl_loadable!(FundData);


#[repr(C)]
#[derive(Clone, Copy)]
pub struct InvestorData {

    pub is_initialized: bool,
    pub has_withdrawn: bool,
    pub withdrawn_from_margin: bool,
    pub padding: [u8; 5],

    /// The Initial deposit (in USDC tokens)
    pub amount: u64,

    // start performance of investor
    pub start_performance: U64F64,

    // // mngo reward debt
    // pub mngo_debt: U64F64,

    /// Investor wallet address
    pub owner: Pubkey,

    // Fund manager wallet key
    pub manager: Pubkey
}
impl_loadable!(InvestorData);


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
        check_eq!(account.data_len(), size_of::<Self>());
        check_eq!(account.owner, program_id);

        let data = Self::load(account)?;
        Ok(data)
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
        check_eq!(account.data_len(), size_of::<Self>());
        check_eq!(account.owner, program_id);

        let data = Self::load(account)?;
        Ok(data)
    }
}
