use std::cell::{Ref, RefMut};
use solana_program::{
    account_info::{AccountInfo},
    program_error::ProgramError,
    pubkey::Pubkey,
    clock::UnixTimestamp,
};
use bytemuck::{from_bytes, from_bytes_mut, Pod, Zeroable};

pub const MAX_TOKENS:usize = 50;

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

