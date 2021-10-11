use fixed::types::U64F64;

use solana_program::{
    account_info::{AccountInfo, next_account_info},
    program_error::ProgramError,
    program_pack::Pack,
    msg,
    pubkey::Pubkey,
    clock::Clock,
    sysvar::Sysvar
};

use spl_token::state::Mint;

use crate::error::FundError;
use crate::processor::parse_token_account;
use crate::state::{FundData, PlatformData};

macro_rules! check_eq {
    ($x:expr, $y:expr) => {
        if ($x != $y) {
            return Err(FundError::Default.into())
        }
    }
}

pub fn add_token_to_whitelist (
    program_id: &Pubkey,
    accounts: &[AccountInfo]
) -> Result<(), ProgramError> {
    let accounts_iter = &mut accounts.iter();

    let platform_acc = next_account_info(accounts_iter)?;
    let clock_sysvar_info = next_account_info(accounts_iter)?;
    let investin_admin_acc = next_account_info(accounts_iter)?;

    let mut platform_data = PlatformData::load_mut_checked(platform_acc, program_id)?;
    let clock = &Clock::from_account_info(clock_sysvar_info)?;

    check_eq!(investin_admin_acc.is_signer, true); // signer check
    check_eq!(platform_data.investin_admin, *investin_admin_acc.key); // only admin is allowed to add token

    let mint_account = next_account_info(accounts_iter)?;
    let pool_coin_account = next_account_info(accounts_iter)?;
    let pool_pc_account = next_account_info(accounts_iter)?;

    let mint_data = Mint::unpack(&mint_account.data.borrow())?;
    let pool_coin_data = parse_token_account(pool_coin_account)?;
    let pool_pc_data = parse_token_account(pool_pc_account)?;

    check_eq!(pool_coin_data.mint, *mint_account.key);
    check_eq!(pool_pc_data.mint, platform_data.token_list[0].mint); // USDC
    check_eq!(platform_data.get_token_index(mint_account.key), None);

    let index = platform_data.token_count as usize;
    platform_data.token_list[index].mint = *mint_account.key;
    platform_data.token_list[index].decimals = mint_data.decimals as u64;
    platform_data.token_list[index].pool_coin_account = *pool_coin_account.key;
    platform_data.token_list[index].pool_pc_account = *pool_pc_account.key;
    platform_data.token_list[index].pool_price = U64F64::from_num(pool_pc_data.amount)
    .checked_div(U64F64::from_num(pool_coin_data.amount)).unwrap();
    platform_data.token_list[index].last_updated = clock.unix_timestamp;

    platform_data.token_count += 1;

    Ok(())
}

pub fn update_token_prices (
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    count: u8
) -> Result<(), ProgramError> {

    let accounts_iter = &mut accounts.iter();

    let platform_acc = next_account_info(accounts_iter)?;
    let clock_sysvar_info = next_account_info(accounts_iter)?;

    let mut platform_data = PlatformData::load_mut_checked(platform_acc, program_id)?;
    let clock = &Clock::from_account_info(clock_sysvar_info)?;

    for _i in 0..count {
        let pool_coin_account = next_account_info(accounts_iter)?;
        let pool_pc_account = next_account_info(accounts_iter)?;

        let pool_coin_data = parse_token_account(pool_coin_account)?;
        let pool_pc_data = parse_token_account(pool_pc_account)?;

        let index = platform_data.get_token_index(&pool_coin_data.mint).unwrap() as usize;

        check_eq!(platform_data.token_list[index].pool_coin_account, *pool_coin_account.key);
        check_eq!(platform_data.token_list[index].pool_pc_account, *pool_pc_account.key);

        platform_data.token_list[index].pool_price = U64F64::from_num(pool_pc_data.amount)
        .checked_div(U64F64::from_num(pool_coin_data.amount)).unwrap();
        platform_data.token_list[index].last_updated = clock.unix_timestamp;
    }
    Ok(())
}

pub fn add_token_to_fund (
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    index: u8
) -> Result<(), ProgramError> {

    let accounts_iter = &mut accounts.iter();
    let platform_acc = next_account_info(accounts_iter)?;
    let fund_state_acc = next_account_info(accounts_iter)?;
    let mint_acc = next_account_info(accounts_iter)?;
    let vault_acc = next_account_info(accounts_iter)?;

    let platform_data = PlatformData::load_checked(platform_acc, program_id)?;
    let mut fund_data = FundData::load_mut_checked(fund_state_acc, program_id)?;
    
    // checking for Platform WhiteList  
    let token_index = platform_data.get_token_index(mint_acc.key).unwrap(); // should break in case of non-whitelist
    check_eq!(fund_data.get_token_slot(token_index), None);

    let vault_info = parse_token_account(vault_acc)?;
    check_eq!(vault_info.owner, fund_data.fund_pda);
    check_eq!(fund_data.tokens[index as usize].is_active, false);


    fund_data.tokens[index as usize].is_active = true;
    fund_data.tokens[index as usize].index = token_index as u8;
    fund_data.tokens[index as usize].balance = 0;
    fund_data.tokens[index as usize].debt = 0;
    fund_data.tokens[index as usize].vault = *vault_acc.key;
    fund_data.no_of_assets += 1;

    Ok(())
}

pub fn remove_token_from_fund (
    program_id: &Pubkey,
    accounts: &[AccountInfo]
) -> Result<(), ProgramError> {

    let accounts_iter = &mut accounts.iter();
    let platform_acc = next_account_info(accounts_iter)?;
    let fund_state_acc = next_account_info(accounts_iter)?;
    let mint_acc = next_account_info(accounts_iter)?;

    let platform_data = PlatformData::load_checked(platform_acc, program_id)?;
    let mut fund_data = FundData::load_mut_checked(fund_state_acc, program_id)?;

    let token_index = platform_data.get_token_index(mint_acc.key).unwrap();
    let token_slot = fund_data.get_token_slot(token_index).unwrap();

    check_eq!(fund_data.tokens[token_slot].balance, 0);
    check_eq!(fund_data.tokens[token_slot].debt, 0);
    check_eq!((token_index == 0), false); // cant remove USDC

    fund_data.tokens[token_slot].is_active = false;
    fund_data.tokens[token_slot].index = 0;
    fund_data.tokens[token_slot].balance = 0;
    fund_data.tokens[token_slot].debt = 0;
    fund_data.tokens[token_slot].vault = Pubkey::default();
    fund_data.no_of_assets -= 1;
    Ok(())
}