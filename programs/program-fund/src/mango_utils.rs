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
use crate::processor::{ parse_token_account, update_amount_and_performance};

use mango::state::{MarginAccount, MangoGroup, AccountFlag, NUM_MARKETS};
use mango::state::Loadable as OtherLoadable;
use mango::instruction::{init_margin_account, deposit, withdraw, settle_borrow, place_and_settle};
use mango::processor::get_prices;

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
    const NUM_FIXED: usize = 22;
    let accounts = array_ref![accounts, 0, NUM_FIXED + 2 * NUM_MARKETS];
    let (
        fixed_accs,
        open_orders_accs,
        oracle_accs,
    ) = array_refs![accounts, NUM_FIXED, NUM_MARKETS, NUM_MARKETS];

    let [
        fund_state_acc,
        manager_acc,
        fund_pda_acc,
        mango_prog_acc,
        mango_group_acc,
        margin_account_acc,
        clock_acc,
        dex_prog_acc,
        spot_market_acc,
        dex_request_queue_acc,
        dex_event_queue_acc,
        bids_acc,
        asks_acc,
        base_vault_acc,
        quote_vault_acc,
        signer_acc,
        dex_base_acc,
        dex_quote_acc,
        token_prog_acc,
        rent_acc,
        srm_vault_acc,
        dex_signer_acc
    ] = fixed_accs;

    check!(manager_acc.is_signer, ProgramError::MissingRequiredSignature);

    let mut margin_account = MarginAccount::load_mut(margin_account_acc)?;
    let mut fund_data = FundData::load_mut(fund_state_acc)?;

    // TODO:: check if leverage <= 2
    // TODO::  check margin account valuation and AUM
    let price = order.limit_price;
    let size = order.max_coin_qty;

    invoke_signed(
        &place_and_settle(mango_prog_acc.key,
            mango_group_acc.key, fund_pda_acc.key, margin_account_acc.key,
            dex_prog_acc.key,spot_market_acc.key, dex_request_queue_acc.key,
            dex_event_queue_acc.key, bids_acc.key, asks_acc.key, base_vault_acc.key,
            quote_vault_acc.key, signer_acc.key, dex_base_acc.key, dex_quote_acc.key,
            srm_vault_acc.key, dex_signer_acc.key,
            &[*open_orders_accs[0].key, *open_orders_accs[1].key, *open_orders_accs[2].key, *open_orders_accs[3].key],
            &[*oracle_accs[0].key, *oracle_accs[1].key, *oracle_accs[2].key, *oracle_accs[3].key],
            order)?,
        &[
            mango_prog_acc.clone(),
            mango_group_acc.clone(),
            margin_account_acc.clone(),
            fund_pda_acc.clone(),
            dex_prog_acc.clone(),
            spot_market_acc.clone(),
            dex_request_queue_acc.clone(),
            dex_event_queue_acc.clone(), bids_acc.clone(), asks_acc.clone(), base_vault_acc.clone(),
            quote_vault_acc.clone(), signer_acc.clone(), dex_base_acc.clone(), dex_quote_acc.clone(),
            srm_vault_acc.clone(), dex_signer_acc.clone(),
            open_orders_accs[0].clone(), open_orders_accs[1].clone(), open_orders_accs[2].clone(), open_orders_accs[3].clone(),
            oracle_accs[0].clone(), oracle_accs[1].clone(), oracle_accs[2].clone(), oracle_accs[3].clone(),
            clock_acc.clone(), token_prog_acc.clone(), rent_acc.clone()
        ],
        &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
    )?;
    Ok(())
}

pub fn mango_withdraw_to_fund (
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    quantity: u64
) -> Result<(), ProgramError>

{
    const NUM_FIXED: usize = 12;
    let accounts = array_ref![accounts, 0, NUM_FIXED + 2 * NUM_MARKETS];
    let (
        fixed_accs,
        open_orders_accs,
        oracle_accs,
    ) = array_refs![accounts, NUM_FIXED, NUM_MARKETS, NUM_MARKETS];

    let [
        fund_state_acc,
        price_acc,
        manager_acc,
        fund_pda_acc,
        mango_prog_acc,
        mango_group_acc,
        margin_account_acc,
        token_account_acc,
        vault_acc,
        signer_acc,
        token_prog_acc,
        clock_acc
    ] = fixed_accs;

    check!(manager_acc.is_signer, ProgramError::MissingRequiredSignature);

    let mut fund_data = FundData::load_mut(fund_state_acc)?;
    let mango_group_data = MangoGroup::load_checked(mango_group_acc, mango_prog_acc.key)?;
    let margin_account_data = MarginAccount::load_checked(mango_prog_acc.key, margin_account_acc, mango_group_acc.key)?;

    invoke_signed(
        &withdraw(mango_prog_acc.key, mango_group_acc.key, margin_account_acc.key, fund_pda_acc.key, token_account_acc.key, vault_acc.key, signer_acc.key,
            &[*open_orders_accs[0].key, *open_orders_accs[1].key, *open_orders_accs[2].key, *open_orders_accs[3].key],
            &[*oracle_accs[0].key, *oracle_accs[1].key, *oracle_accs[2].key, *oracle_accs[3].key],
            quantity)?,
        &[
            mango_prog_acc.clone(),
            mango_group_acc.clone(),
            margin_account_acc.clone(),
            fund_pda_acc.clone(),
            token_account_acc.clone(),
            vault_acc.clone(),
            signer_acc.clone(),
            token_prog_acc.clone(),
            clock_acc.clone(),
            open_orders_accs[0].clone(), open_orders_accs[1].clone(), open_orders_accs[2].clone(), open_orders_accs[3].clone(),
            oracle_accs[0].clone(), oracle_accs[1].clone(), oracle_accs[2].clone(), oracle_accs[3].clone(),
        ],
        &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
    )?;

    let token_account = parse_token_account(token_account_acc)?;

    check!(token_account.mint == fund_data.tokens[0].mint, FundError::InvalidTokenAccount);
    fund_data.tokens[0].balance = token_account.amount;

    mango_margin_valuation(&mut fund_data, &mango_group_data, &margin_account_data, oracle_accs, open_orders_accs, true)?;
    
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

pub fn mango_margin_valuation (
    fund_data: &mut FundData,
    mango_group: &MangoGroup,
    margin_account: &MarginAccount,
    oracle_accs: &[AccountInfo],
    open_orders_acc: &[AccountInfo; 4],
    update_perf: bool
) -> Result<(), ProgramError> {

    let prices = get_prices(mango_group, oracle_accs)?;
    let val = margin_account.get_equity(mango_group, &prices, open_orders_acc)?;
    let fund_val = val.checked_mul(U64F64::from_num(10u64.pow(fund_data.decimals as u32))).unwrap();
    
    if update_perf {
        let mut perf = U64F64::from_num(fund_data.prev_performance);
        // only case where performance is not updated:
        // when no investments and no performance fee for manager
        if fund_data.number_of_active_investments != 0 || fund_data.performance_fee != 0 {
            perf = fund_val.checked_div(U64F64::from_num(fund_data.total_amount)).unwrap()
            .checked_mul(U64F64::from_num(fund_data.prev_performance)).unwrap();
        }
        // adjust for manager performance fee
        fund_data.performance_fee = U64F64::to_num(U64F64::from_num(perf)
            .checked_div(U64F64::from_num(fund_data.prev_performance)).unwrap()
            .checked_mul(U64F64::from_num(fund_data.performance_fee)).unwrap());
        fund_data.prev_performance = U64F64::to_num(perf);
    }

    fund_data.total_amount = U64F64::to_num(fund_val);

    Ok(())
}