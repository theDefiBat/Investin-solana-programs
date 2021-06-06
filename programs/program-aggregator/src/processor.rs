use std::mem::size_of;
use fixed::types::U64F64;
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    msg,
    program_error::ProgramError,
    program_pack::{Pack, IsInitialized},
    pubkey::Pubkey,
    clock::{Clock, UnixTimestamp},
    sysvar::Sysvar
};

use spl_token::state::{Account, Mint};

use crate::instruction::AggInstruction;
use crate::state::{PriceAccount, PriceInfo};
use crate::state::Loadable;

pub struct Aggregator {}

impl Aggregator {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        data: &[u8]
    ) -> Result<(), ProgramError> {
        msg!("Aggregator Program Entrypoint");
        let instruction = AggInstruction::unpack(data).ok_or(ProgramError::InvalidInstructionData)?;

        // Iterating accounts is safer then indexing
        let accounts_iter = &mut accounts.iter();
    
        let price_account = next_account_info(accounts_iter)?;
    
        msg!("size of account:: {:?}, size of data:: {:?}", size_of::<PriceAccount>(), price_account.data_len());
        let mut price_data = PriceAccount::load_mut(price_account)?;
    
        let clock_sysvar_info = next_account_info(accounts_iter)?;
        let clock = &Clock::from_account_info(clock_sysvar_info)?;
    
        match instruction {
            AggInstruction::AddToken { count } => {
                let mut cnt = count;
                let admin_account = next_account_info(accounts_iter)?;
                if !admin_account.is_signer{
                    return Err(ProgramError::MissingRequiredSignature);
                }
                loop {
                    let mint_account = next_account_info(accounts_iter)?;
                    let pool_coin_account = next_account_info(accounts_iter)?;
                    let pool_pc_account = next_account_info(accounts_iter)?;
                    
                    let mint_data = Mint::unpack(&mint_account.data.borrow())?;
                    let pool_coin_data = parse_token_account(pool_coin_account)?;
                    let pool_pc_data = parse_token_account(pool_pc_account)?;
    
                    if pool_coin_data.mint != *mint_account.key {
                        return Err(ProgramError::InvalidAccountData);
                    }
                    
                    if price_data.count == 0 {
                        price_data.decimals = 6;
                    }
    
                    // panic if token already present
                    find_index(&price_data, pool_coin_data.mint).unwrap_err();
                    
                    let index = price_data.count as usize;
                    price_data.prices[index].token_mint = *mint_account.key;
                    price_data.prices[index].decimals = mint_data.decimals.into();
                    price_data.prices[index].pool_account = *pool_coin_account.key;
                    price_data.prices[index].base_pool_account = *pool_pc_account.key;
    
                    update_price(&mut price_data.prices[index], pool_pc_data.amount, pool_coin_data.amount, clock.unix_timestamp)?;
    
                    price_data.count += 1;
                    cnt -= 1;
                    if cnt == 0 { break; }
                }
            }
            AggInstruction::UpdateTokenPrices { count } => {
                let mut cnt = count;
                loop {
    
                    let pool_coin_account = next_account_info(accounts_iter)?;
                    let pool_pc_account = next_account_info(accounts_iter)?;
                    
                    let pool_coin_data = parse_token_account(pool_coin_account)?;
                    let pool_pc_data = parse_token_account(pool_pc_account)?;
    
                    let index = find_index(&price_data, pool_coin_data.mint)?;
                    if price_data.prices[index].pool_account != *pool_coin_account.key {
                        return Err(ProgramError::InvalidAccountData);
                    }
                    if price_data.prices[index].base_pool_account != *pool_pc_account.key {
                        return Err(ProgramError::InvalidAccountData);
                    }
    
                    update_price(&mut price_data.prices[index], pool_pc_data.amount, pool_coin_data.amount, clock.unix_timestamp)?;
    
                    cnt -= 1;
                    if cnt == 0 { break; }
                }
            }
        }
        Ok(())
    }
}

pub fn find_index (
    price_data: &PriceAccount,
    mint: Pubkey
) -> Result<usize, ProgramError> {
    for i in 0..price_data.count {
        if price_data.prices[i as usize].token_mint == mint {
            return Ok(i as usize)
        }
    }
    return Err(ProgramError::InvalidArgument)
}
pub fn update_price(
    price_info: &mut PriceInfo,
    pc_res: u64,
    coin_res: u64,
    timestamp: UnixTimestamp
) -> Result<(), ProgramError> {
    msg!("update price called");
    let adj_decimals = 6 + (price_info.decimals - 6);

    price_info.token_price = U64F64::to_num(
        U64F64::from_num(pc_res)
        .checked_div(U64F64::from_num(coin_res)).unwrap()
        .checked_mul(U64F64::from_num(10u64.pow(adj_decimals as u32))).unwrap()
    );
    price_info.last_updated = timestamp;

    Ok(())
}

pub fn parse_token_account (account_info: &AccountInfo) -> Result<Account, ProgramError> {
    if account_info.owner != &spl_token::ID {
        msg!("Account not owned by spl-token program");
        return Err(ProgramError::IncorrectProgramId);
    }
    let parsed = Account::unpack(&account_info.try_borrow_data()?)?;
    if !parsed.is_initialized() {
        msg!("Token account not initialized");
        return Err(ProgramError::UninitializedAccount);
    }
    Ok(parsed)
}