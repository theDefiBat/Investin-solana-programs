use std::mem::size_of;
use bytemuck::bytes_of;
use fixed::types::U64F64;
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    msg,
    instruction:: {AccountMeta, Instruction},
    program_error::ProgramError,
    program_pack::{Pack, IsInitialized},
    pubkey::Pubkey,
    program::{invoke, invoke_signed},
    clock::Clock,
    sysvar::Sysvar
};

use arrayref::{array_ref, array_refs};

use spl_token::state::{Account, Mint};

use crate::error::FundError;
use crate::instruction::{FundInstruction, Data};
use crate::state::{NUM_TOKENS, MAX_INVESTORS, FundData, InvestorData, TokenInfo, PlatformData, PriceAccount};
use crate::state::Loadable;
use crate::processor::parse_token_account;

use mango::state::{MarginAccount, MangoGroup, AccountFlag};
use mango::state::Loadable as OtherLoadable;
use mango::instruction::{init_margin_account, deposit, withdraw, settle_borrow, place_and_settle};

macro_rules! check {
    ($cond:expr, $err:expr) => {
        if !($cond) {
            return Err(($err).into())
        }
    }
}

pub fn mango_init_margin_account (
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    quantity: u64
) -> Result<(), ProgramError>

{
    const NUM_FIXED: usize = 7;
    let accounts = array_ref![accounts, 0, NUM_FIXED];
    let [
        fund_state_acc,
        manager_acc,
        fund_pda_acc,
        mango_prog_acc,
        mango_group_acc,
        margin_account_acc,
        rent_acc
    ] = accounts;

    let fund_data = FundData::load(fund_state_acc)?;

    check!(manager_acc.is_signer, ProgramError::MissingRequiredSignature);
    
    // check not required as done in Mango
    // check!(margin_account.account_flags != (AccountFlag::Initialized | AccountFlag::MarginAccount).bits(), FundError::)

    invoke_signed(
        &init_margin_account(mango_prog_acc.key, mango_group_acc.key, margin_account_acc.key, fund_pda_acc.key)?,
        &[
            mango_prog_acc.clone(),
            mango_group_acc.clone(),
            margin_account_acc.clone(),
            fund_pda_acc.clone(),
            rent_acc.clone()
        ],
        &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
    )?;
    Ok(())
}

pub fn mango_deposit (
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    quantity: u64
) -> Result<(), ProgramError>

{
    const NUM_FIXED: usize = 10;
    let accounts = array_ref![accounts, 0, NUM_FIXED];
    let [
        fund_state_acc,
        manager_acc,
        fund_pda_acc,
        mango_prog_acc,
        mango_group_acc,
        margin_account_acc,
        token_account_acc,
        vault_acc,
        token_prog_acc,
        clock_acc,
    ] = accounts;

    let mut margin_account = MarginAccount::load_mut(margin_account_acc)?;
    let mut fund_data = FundData::load_mut(fund_state_acc)?;

    check!(manager_acc.is_signer, ProgramError::MissingRequiredSignature);

    invoke_signed(
        &deposit(mango_prog_acc.key, mango_group_acc.key, margin_account_acc.key, fund_pda_acc.key, token_account_acc.key, vault_acc.key, quantity)?,
        &[
            mango_prog_acc.clone(),
            mango_group_acc.clone(),
            margin_account_acc.clone(),
            fund_pda_acc.clone(),
            token_account_acc.clone(),
            vault_acc.clone(),
            token_prog_acc.clone(),
            clock_acc.clone()
        ],
        &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
    )?;

    let token_account = parse_token_account(token_account_acc)?;
    
    for i in 0..NUM_TOKENS {
        if fund_data.tokens[i].mint == token_account.mint {
            fund_data.tokens[i].balance = token_account.amount;
        }
    }
    
    Ok(())
}

pub fn mango_place_and_settle (
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    order: serum_dex::instruction::NewOrderInstructionV3
) -> Result<(), ProgramError>

{

    Ok(())
}

pub fn mango_withdraw_to_fund (
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    quantity: u64
) -> Result<(), ProgramError>

{

    Ok(())
}

pub fn mango_withdraw_investor (
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    quantity: u64
) -> Result<(), ProgramError>

{

    Ok(())
}