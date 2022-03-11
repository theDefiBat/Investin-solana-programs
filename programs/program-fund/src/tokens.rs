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
use std::cell::{Ref, RefMut};
use bytemuck::{ from_bytes };


use spl_token::state::Mint;

use crate::error::FundError;
use crate::processor::parse_token_account;
use crate::state::{FundAccount, FundData, PlatformData, AmmInfo, Loadable};

macro_rules! check_eq {
    ($x:expr, $y:expr) => {
        if ($x != $y) {
            return Err(FundError::Default.into())
        }
    }
}

macro_rules! check {
    ($cond:expr, $err:expr) => {
        if !($cond) {
            return Err(($err).into())
        }
    }
}

pub fn add_token_to_whitelist (
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    token_id: u8,
    pc_index: u8
) -> Result<(), ProgramError> {
    let accounts_iter = &mut accounts.iter();

    let platform_acc = next_account_info(accounts_iter)?;
    let clock_sysvar_info = next_account_info(accounts_iter)?;
    let investin_admin_acc = next_account_info(accounts_iter)?;

    let mut platform_data = PlatformData::load_mut_checked(platform_acc, program_id)?;
    let clock = &Clock::from_account_info(clock_sysvar_info)?;

    check_eq!(investin_admin_acc.is_signer, true); // signer check
    check_eq!(platform_data.investin_admin, *investin_admin_acc.key); // only admin is allowed to add token

    // token id check => 0 for Raydium and 1 for Orca for now!
    check!(token_id < 2, ProgramError::InvalidArgument);
    
    //later can keep if else condition 
    // check!(pc_index < 2, ProgramError::InvalidArgument);


    let mint_account = next_account_info(accounts_iter)?;
    let pool_coin_account = next_account_info(accounts_iter)?;
    let pool_pc_account = next_account_info(accounts_iter)?;

    let mint_data = Mint::unpack(&mint_account.data.borrow())?;
    let pool_coin_data = parse_token_account(pool_coin_account)?;
    let pool_pc_data = parse_token_account(pool_pc_account)?;

    check_eq!(pool_coin_data.mint, *mint_account.key);
    check_eq!(pool_pc_data.mint, platform_data.token_list[pc_index as usize].mint); // 0 -> USDC... pc should be whitlisted first which is refrenced here
    check_eq!(platform_data.token_list[pc_index as usize].pc_index, 0); // pc should either be USDC itself or have a USDC pair
    check_eq!(platform_data.get_token_index(mint_account.key, token_id), None);

    let index = platform_data.token_count as usize;
    platform_data.token_list[index].mint = *mint_account.key;
    platform_data.token_list[index].token_id = token_id;
    platform_data.token_list[index].decimals = mint_data.decimals as u64;
    platform_data.token_list[index].pool_coin_account = *pool_coin_account.key;
    platform_data.token_list[index].pool_pc_account = *pool_pc_account.key;
    platform_data.token_list[index].pc_index = pc_index;
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

        let index = platform_data.get_token_index_by_coin(pool_coin_account.key).unwrap() as usize;

        check_eq!(platform_data.token_list[index].pool_coin_account, *pool_coin_account.key);
        check_eq!(platform_data.token_list[index].pool_pc_account, *pool_pc_account.key);

        let mux = platform_data.token_list[index].token_id;
        msg!("MUX: {:?}", mux);
        if mux == 0 {
            let amm_open_orders_account = next_account_info(accounts_iter)?;
            let amm_info_account = next_account_info(accounts_iter)?;
            let amm_open_order_data = load_open_orders(amm_open_orders_account)?;
            let amm_info_data = AmmInfo::load(amm_info_account)?;
            platform_data.token_list[index].pool_price = U64F64::from_num(pool_pc_data.amount + amm_open_order_data.native_pc_total - amm_info_data.need_take_pnl_pc)
            .checked_div(U64F64::from_num(pool_coin_data.amount + amm_open_order_data.native_coin_total - amm_info_data.need_take_pnl_coin)).unwrap();
        } else {
            platform_data.token_list[index].pool_price = U64F64::from_num(pool_pc_data.amount)
            .checked_div(U64F64::from_num(pool_coin_data.amount)).unwrap();
        }
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
    let fund_account_acc = next_account_info(accounts_iter)?;
    let mint_acc = next_account_info(accounts_iter)?;
    let vault_acc = next_account_info(accounts_iter)?;

    let platform_data = PlatformData::load_checked(platform_acc, program_id)?;
    let mut fund_data = FundAccount::load_mut_checked(fund_account_acc, program_id)?;

    // if invalid fund_state_acc
    // although other signers cannot chnage some others fundState so error will be thrown
    // still be better if we add checks (will need to pass manager acc)
    // check!(manager_ai.is_signer, ProgramError::MissingRequiredSignature);
    // check_eq!(fund_data.manager_account, *manager_ai.key);

    let vault_info = parse_token_account(vault_acc)?;
    check_eq!(vault_info.owner, fund_data.fund_pda);
    check_eq!(fund_data.tokens[index as usize].is_active, false);

    let token_index_1 = platform_data.get_token_index(mint_acc.key, 0);
    let token_index_2 = platform_data.get_token_index(mint_acc.key, 1);
    
    // both indexes cant be None
    check!(((token_index_1 != None) || (token_index_2 != None)), ProgramError::InvalidAccountData);

    //what happens when it is both on ray and Orca MUX=??
    if token_index_1 != None {
        fund_data.tokens[index as usize].mux = 0;
        fund_data.tokens[index as usize].index[0] = token_index_1.unwrap() as u8;
    }
    else {
        fund_data.tokens[index as usize].index[0] = 255; // Max u8
    }

    if token_index_2 != None {
        fund_data.tokens[index as usize].mux = 1;
        fund_data.tokens[index as usize].index[1] = token_index_2.unwrap() as u8;
    }
    else {
        fund_data.tokens[index as usize].index[1] = 255;
    }

    fund_data.tokens[index as usize].is_active = true;    
    fund_data.tokens[index as usize].balance = 0;
    fund_data.tokens[index as usize].debt = 0;
    fund_data.tokens[index as usize].vault = *vault_acc.key;
    fund_data.no_of_assets += 1;

    Ok(())
}

pub fn remove_token_from_fund (
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    index: u8
) -> Result<(), ProgramError> {

    let accounts_iter = &mut accounts.iter();
    let platform_acc = next_account_info(accounts_iter)?;
    let fund_state_acc = next_account_info(accounts_iter)?;
    let mint_acc = next_account_info(accounts_iter)?;

    let platform_data = PlatformData::load_checked(platform_acc, program_id)?;
    let mut fund_data = FundAccount::load_mut_checked(fund_state_acc, program_id)?;

    // if invalid fund_state_acc
    // although other signers cannot chnage some others fundState so error will be thrown
    // still be better if we add checks (will need to pass manager acc)
    // check!(manager_ai.is_signer, ProgramError::MissingRequiredSignature);
    // check_eq!(fund_data.manager_account, *manager_ai.key);

    let token_slot = index as usize;
    let mux = fund_data.tokens[token_slot].mux as usize;

    check!(fund_data.tokens[token_slot].balance<=10, ProgramError::InsufficientFunds);
    check_eq!(fund_data.tokens[token_slot].debt, 0);
    // check_eq!(fund_data.tokens[token_slot].is_on_mango, 0);
    check_eq!((fund_data.tokens[token_slot].index[mux] == 0), false); // cant remove USDC

    fund_data.tokens[token_slot].is_active = false;
    fund_data.tokens[token_slot].index[0] = 0;
    fund_data.tokens[token_slot].index[1] = 0;
    fund_data.tokens[token_slot].mux = 0;

    fund_data.tokens[token_slot].balance = 0;
    fund_data.tokens[token_slot].debt = 0;
    fund_data.tokens[token_slot].vault = Pubkey::default();
    fund_data.no_of_assets -= 1;
    Ok(())
}


fn strip_dex_padding<'a>(acc: &'a AccountInfo) -> Result<Ref<'a, [u8]>, ProgramError> {
    check!(acc.data_len() >= 12, ProgramError::InvalidArgument);
    let unpadded_data: Ref<[u8]> = Ref::map(acc.try_borrow_data()?, |data| {
        let data_len = data.len() - 12;
        let (_, rest) = data.split_at(5);
        let (mid, _) = rest.split_at(data_len);
        mid
    });
    Ok(unpadded_data)
}

pub fn load_open_orders<'a>(
    acc: &'a AccountInfo,
) -> Result<Ref<'a, serum_dex::state::OpenOrders>, ProgramError> {
    Ok(Ref::map(strip_dex_padding(acc)?, from_bytes))
}




