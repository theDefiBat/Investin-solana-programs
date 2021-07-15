use std::mem::size_of;
use bytemuck::bytes_of;
use fixed::types::U64F64;
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    msg,
    instruction:: {AccountMeta, Instruction},
    log::{sol_log_compute_units},
    program_error::ProgramError,
    program_pack::{Pack, IsInitialized},
    pubkey::Pubkey,
    program::{invoke, invoke_signed},
    clock::Clock,
    sysvar::Sysvar
};

use fixed_macro::types::U64F64;
pub const ONE_U64F64: U64F64 = U64F64!(1);
use serum_dex::matching::{Side, OrderType};
use serum_dex::instruction::{NewOrderInstructionV3, MarketInstruction, SelfTradeBehavior};
use std::num::NonZeroU64;


use arrayref::{array_ref, array_refs};

use spl_token::state::{Account, Mint};

use crate::error::FundError;
use crate::instruction::{FundInstruction, Data};
use crate::state::{NUM_TOKENS, MAX_INVESTORS, FundData, InvestorData, TokenInfo, PlatformData, PriceAccount};
use crate::state::Loadable;
use crate::processor::{ parse_token_account, update_amount_and_performance, get_share};

use mango::state::{MarginAccount, MangoGroup, AccountFlag, NUM_MARKETS};
use mango::state::Loadable as OtherLoadable;
use mango::instruction::{init_margin_account, deposit, withdraw, settle_funds, settle_borrow, place_and_settle, MangoInstruction};
use mango::processor::get_prices;

pub const MANGO_NUM_TOKENS:usize = 5;

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

    let mut fund_data = FundData::load_mut(fund_state_acc)?;

    check!(manager_acc.is_signer, ProgramError::MissingRequiredSignature);
    check!(fund_data.mango_positions[0].margin_account == Pubkey::default(), ProgramError::AccountAlreadyInitialized);
    
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
    fund_data.mango_positions[0].margin_account = *margin_account_acc.key;
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

    // let mut margin_account = MarginAccount::load_mut(margin_account_acc)?;
    let mut fund_data = FundData::load_mut(fund_state_acc)?;

    msg!("fund_data");

    check!(manager_acc.is_signer, ProgramError::MissingRequiredSignature);
    check!(fund_data.mango_positions[0].margin_account == *margin_account_acc.key, ProgramError::InvalidAccountData);
    check!(U64F64::from_num(quantity) <=
        U64F64::from_num(fund_data.total_amount).checked_mul(U64F64!(0.4)).unwrap(), ProgramError::InsufficientFunds
    );
    
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

    msg!("invoke done");

    let token_account = parse_token_account(token_account_acc)?;
    
    for i in 0..NUM_TOKENS {
        if fund_data.tokens[i].mint == token_account.mint {
            fund_data.tokens[i].balance = token_account.amount;
        }
    }
    fund_data.mango_positions[0].is_active = true;
    fund_data.mango_positions[0].trade_amount = quantity;
    Ok(())
}



pub fn mango_place_order (
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    side: u8,
    price: u64,
    quote_size: u64,
    base_size: u64
) -> Result<(), ProgramError>

{
    const NUM_FIXED: usize = 20;
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
        vault_acc,
        signer_acc,
        dex_base_acc,
        dex_quote_acc,
        token_prog_acc,
        rent_acc,
        srm_vault_acc,
    ] = fixed_accs;

    check!(manager_acc.is_signer, ProgramError::MissingRequiredSignature);

    // let mut margin_account = MarginAccount::load_mut(margin_account_acc)?;
    let mut fund_data = FundData::load_mut(fund_state_acc)?;

    // TODO:: check if collateral ratio is greater than 150, otherwise skip
    // TODO::  check margin account valuation and AUM


    msg!("quote_amount:: {:?} ", quote_size);
    msg!("pc_amount:: {:?}", base_size);
    let order = NewOrderInstructionV3 {
        side: match side {
            0 => Side::Bid,
            1 => Side::Ask,
            _ => Side::Bid
        },
        limit_price: NonZeroU64::new(price).unwrap(),
        max_coin_qty: NonZeroU64::new(quote_size).unwrap(),
        max_native_pc_qty_including_fees: NonZeroU64::new(base_size).unwrap(),
        order_type: OrderType::Limit,
        client_order_id: 1,
        self_trade_behavior: SelfTradeBehavior::DecrementTake,
        limit: 65535,
        
    };

    invoke_signed(
        &instruction_place_order(mango_prog_acc.key,
            mango_group_acc.key, fund_pda_acc.key, margin_account_acc.key,
            dex_prog_acc.key,spot_market_acc.key, dex_request_queue_acc.key,
            dex_event_queue_acc.key, bids_acc.key, asks_acc.key, vault_acc.key,
            signer_acc.key, dex_base_acc.key, dex_quote_acc.key, srm_vault_acc.key,
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
            dex_event_queue_acc.clone(), bids_acc.clone(), asks_acc.clone(), vault_acc.clone(),
            signer_acc.clone(), dex_base_acc.clone(), dex_quote_acc.clone(), srm_vault_acc.clone(),
            open_orders_accs[0].clone(), open_orders_accs[1].clone(), open_orders_accs[2].clone(), open_orders_accs[3].clone(),
            oracle_accs[0].clone(), oracle_accs[1].clone(), oracle_accs[2].clone(), oracle_accs[3].clone(),
            clock_acc.clone(), token_prog_acc.clone(), rent_acc.clone()
        ],
        &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
    )?;
    let margin_data = MarginAccount::load(margin_account_acc)?;
    let mango_group_data = MangoGroup::load(mango_group_acc)?;

    let prices = get_prices(&mango_group_data, oracle_accs)?;
    let coll_ratio = margin_data.get_collateral_ratio(&mango_group_data, &prices, open_orders_accs)?;

    // restrict to 2x leverage
    check!(coll_ratio >= U64F64!(1.45), ProgramError::InsufficientFunds);
    Ok(())
}

pub fn mango_settle_funds (
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    settle_amount: u64,
    token_index: usize
) -> Result<(), ProgramError>

{
    const NUM_FIXED: usize = 17;
    let accounts = array_ref![accounts, 0, NUM_FIXED];
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
        open_orders_acc,
        signer_acc,
        dex_base_acc,
        dex_quote_acc,
        base_vault_acc,
        quote_vault_acc,
        dex_signer_acc,
        token_prog_acc,
    ] = accounts;

    // let mut margin_account = MarginAccount::load_mut(margin_account_acc)?;
    let fund_data = FundData::load(fund_state_acc)?;

    check!(manager_acc.is_signer, ProgramError::MissingRequiredSignature);

    invoke_signed(
        &settle_funds(mango_prog_acc.key, mango_group_acc.key, fund_pda_acc.key, margin_account_acc.key,
            dex_prog_acc.key, spot_market_acc.key, open_orders_acc.key, signer_acc.key,
            dex_base_acc.key, dex_quote_acc.key, base_vault_acc.key, quote_vault_acc.key, dex_signer_acc.key)?,
        &[
            mango_prog_acc.clone(), mango_group_acc.clone(), fund_pda_acc.clone(), margin_account_acc.clone(),
            dex_prog_acc.clone(), spot_market_acc.clone(), open_orders_acc.clone(), signer_acc.clone(),
            dex_base_acc.clone(), dex_quote_acc.clone(), base_vault_acc.clone(), quote_vault_acc.clone(),
            dex_signer_acc.clone(), clock_acc.clone(), token_prog_acc.clone()
        ],
        &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
    )?;

    // settle borrows
    invoke_signed(
        &settle_borrow(mango_prog_acc.key, mango_group_acc.key, margin_account_acc.key, fund_pda_acc.key, token_index,
            settle_amount)?,
        &[
            mango_prog_acc.clone(),
            mango_group_acc.clone(),
            margin_account_acc.clone(),
            fund_pda_acc.clone(),
            clock_acc.clone(),
        ],
        &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
    )?;


    Ok(())
}

pub fn mango_close_position (
    program_id: &Pubkey,
    accounts: &[AccountInfo],
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

    let mango_group_data = MangoGroup::load_checked(mango_group_acc, mango_prog_acc.key)?;
    let margin_account_data = MarginAccount::load_checked(mango_prog_acc.key, margin_account_acc, mango_group_acc.key)?;

    //mango_margin_valuation(&mut fund_data, &mango_group_data, &margin_account_data, oracle_accs, open_orders_accs, true)?;

    fund_data.total_amount += fund_data.tokens[0].balance;
    // fund_data.total_amount = U64F64::to_num(U64F64::from_num(fund_data.total_amount)
            // .checked_add(U64F64::from_num(fund_data.prev_performance)).unwrap()
    
    Ok(())
}

pub fn mango_withdraw_place_order (
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    order: serum_dex::instruction::NewOrderInstructionV3
) -> Result<(), ProgramError> {

    const NUM_FIXED: usize = 21;
    let accounts = array_ref![accounts, 0, NUM_FIXED + 2 * NUM_MARKETS];
    let (
        fixed_accs,
        open_orders_accs,
        oracle_accs,
    ) = array_refs![accounts, NUM_FIXED, NUM_MARKETS, NUM_MARKETS];

    let [
        fund_state_acc,
        investor_state_acc,
        investor_acc,
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
        vault_acc,
        signer_acc,
        dex_base_acc,
        dex_quote_acc,
        token_prog_acc,
        rent_acc,
        srm_vault_acc,
    ] = fixed_accs;

    let mut fund_data = FundData::load_mut(fund_state_acc)?;

    let mut m_order = order;

    check!(investor_acc.is_signer, ProgramError::MissingRequiredSignature);

    fill_order_withdraw(&mut fund_data, investor_state_acc, margin_account_acc, mango_group_acc, spot_market_acc, open_orders_accs, &mut m_order)?;

    invoke_signed(
        &instruction_place_order(mango_prog_acc.key,
            mango_group_acc.key, fund_pda_acc.key, margin_account_acc.key,
            dex_prog_acc.key,spot_market_acc.key, dex_request_queue_acc.key,
            dex_event_queue_acc.key, bids_acc.key, asks_acc.key, vault_acc.key,
            signer_acc.key, dex_base_acc.key, dex_quote_acc.key, srm_vault_acc.key,
            &[*open_orders_accs[0].key, *open_orders_accs[1].key, *open_orders_accs[2].key, *open_orders_accs[3].key],
            &[*oracle_accs[0].key, *oracle_accs[1].key, *oracle_accs[2].key, *oracle_accs[3].key],
            m_order)?,
        &[
            mango_prog_acc.clone(),
            mango_group_acc.clone(),
            margin_account_acc.clone(),
            fund_pda_acc.clone(),
            dex_prog_acc.clone(),
            spot_market_acc.clone(),
            dex_request_queue_acc.clone(),
            dex_event_queue_acc.clone(), bids_acc.clone(), asks_acc.clone(), vault_acc.clone(),
            signer_acc.clone(), dex_base_acc.clone(), dex_quote_acc.clone(), srm_vault_acc.clone(),
            open_orders_accs[0].clone(), open_orders_accs[1].clone(), open_orders_accs[2].clone(), open_orders_accs[3].clone(),
            oracle_accs[0].clone(), oracle_accs[1].clone(), oracle_accs[2].clone(), oracle_accs[3].clone(),
            clock_acc.clone(), token_prog_acc.clone(), rent_acc.clone()
        ],
        &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
    )?;
    Ok(())

}

pub fn fill_order_withdraw (
    fund_data: &mut FundData,
    inv_acc: &AccountInfo,
    margin_acc: &AccountInfo,
    mango_group_acc: &AccountInfo,
    spot_market_acc: &AccountInfo,
    open_orders_accs: &[AccountInfo; 4],
    order: &mut serum_dex::instruction::NewOrderInstructionV3
) -> Result<(), ProgramError> {
    let mut investor_data = InvestorData::load_mut(inv_acc)?;
    
    // check!(investor_data.owner == *inv_acc.key, FundError::InvestorMismatch);

    let inv_share = get_share(fund_data, &mut investor_data)?;

    let margin_data = MarginAccount::load(&margin_acc)?;
    let mango_group = MangoGroup::load(&mango_group_acc)?;
    
    let market_i = mango_group.get_market_index(spot_market_acc.key).unwrap();

    //TODO: add base quantity

    // msg!("max_coin_qty")
    match order.side {
        Side::Bid => {
            let liabs = margin_data.get_liabs(&mango_group)?;
            order.max_coin_qty = NonZeroU64::new(
                U64F64::to_num(inv_share.checked_mul(liabs[market_i]).unwrap()
                .checked_div(U64F64::from_num(10u64.pow(2))).unwrap())).ok_or_else(|| 0
            )?;
        },
        Side::Ask => {
            let assets = margin_data.get_assets(&mango_group, open_orders_accs)?;
            order.max_coin_qty = NonZeroU64::new(
                U64F64::to_num(inv_share.checked_mul(assets[market_i]).unwrap()
                .checked_div(U64F64::from_num(10u64.pow(2))).unwrap())).ok_or_else(|| 0
            )?;
        }
    };
    
    Ok(())
}

pub fn mango_withdraw_investor (
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    token_index: usize
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
        inv_state_acc,
        investor_acc,
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

    check!(investor_acc.is_signer, ProgramError::MissingRequiredSignature);

    let mut fund_data = FundData::load_mut(fund_state_acc)?;
    let mut investor_data = InvestorData::load_mut(inv_state_acc)?;
    
    let ( withdraw_amount, settle_amount ) = get_withdraw_amounts(&mut fund_data, &mut investor_data, margin_account_acc, mango_group_acc, open_orders_accs, oracle_accs, token_index)?;

    msg!("withdraw_amount {:?}", withdraw_amount);
    msg!("settle_amount {:?}", settle_amount);

    // settle borrows
    invoke_signed(
        &settle_borrow(mango_prog_acc.key, mango_group_acc.key, margin_account_acc.key, fund_pda_acc.key, token_index,
            settle_amount)?,
        &[
            mango_prog_acc.clone(),
            mango_group_acc.clone(),
            margin_account_acc.clone(),
            fund_pda_acc.clone(),
            clock_acc.clone(),
        ],
        &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
    )?;

    invoke_signed(
        &withdraw(mango_prog_acc.key, mango_group_acc.key, margin_account_acc.key, fund_pda_acc.key, token_account_acc.key, vault_acc.key, signer_acc.key,
            &[*open_orders_accs[0].key, *open_orders_accs[1].key, *open_orders_accs[2].key, *open_orders_accs[3].key],
            &[*oracle_accs[0].key, *oracle_accs[1].key, *oracle_accs[2].key, *oracle_accs[3].key],
            withdraw_amount)?,
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


    Ok(())
}

pub fn get_withdraw_amounts (
    fund_data: &mut FundData,
    inv_data: &mut InvestorData,
    margin_acc: &AccountInfo,
    mango_group_acc: &AccountInfo,
    open_orders_accs: &[AccountInfo; 4],
    oracle_accs: &[AccountInfo; 4],
    token_index: usize
) -> Result<(u64, u64), ProgramError> {
    
    let margin_data = MarginAccount::load(&margin_acc)?;
    let mango_group = MangoGroup::load(&mango_group_acc)?;

    let prices = get_prices(&mango_group, oracle_accs)?;
    let equity = margin_data.get_equity(&mango_group, &prices, open_orders_accs)?;

    let inv_share = get_share(fund_data, inv_data)?;
    let withdraw_amount = U64F64::to_num(inv_share.checked_mul(equity).unwrap());
    let settle_amount = U64F64::to_num(margin_data.deposits[token_index] * mango_group.indexes[token_index].deposit);

    check!(margin_data.deposits[token_index] > 100, ProgramError::InvalidArgument);
    check!(margin_data.borrows[token_index] > 100, ProgramError::InvalidArgument);
    
    return Ok((withdraw_amount, settle_amount))
}
pub fn instruction_place_and_settle(
    program_id: &Pubkey,
    mango_group_pk: &Pubkey,
    owner_pk: &Pubkey,
    margin_account_pk: &Pubkey,
    dex_prog_id: &Pubkey,
    spot_market_pk: &Pubkey,
    dex_request_queue_pk: &Pubkey,
    dex_event_queue_pk: &Pubkey,
    bids_pk: &Pubkey,
    asks_pk: &Pubkey,
    base_vault_pk: &Pubkey,
    quote_vault_pk: &Pubkey,
    signer_pk: &Pubkey,
    dex_base_pk: &Pubkey,
    dex_quote_pk: &Pubkey,
    srm_vault_pk: &Pubkey,
    dex_signer_pk: &Pubkey,
    open_orders_pks: &[Pubkey],
    oracle_pks: &[Pubkey],
    order: serum_dex::instruction::NewOrderInstructionV3
) -> Result<Instruction, ProgramError> {

    let mut accounts = vec![
        AccountMeta::new(*mango_group_pk, false),
        AccountMeta::new_readonly(*owner_pk, true),
        AccountMeta::new(*margin_account_pk, false),
        AccountMeta::new_readonly(solana_program::sysvar::clock::ID, false),
        AccountMeta::new_readonly(*dex_prog_id, false),
        AccountMeta::new(*spot_market_pk, false),
        AccountMeta::new(*dex_request_queue_pk, false),
        AccountMeta::new(*dex_event_queue_pk, false),
        AccountMeta::new(*bids_pk, false),
        AccountMeta::new(*asks_pk, false),
        AccountMeta::new(*base_vault_pk, false),
        AccountMeta::new(*quote_vault_pk, false),
        AccountMeta::new_readonly(*signer_pk, false),
        AccountMeta::new(*dex_base_pk, false),
        AccountMeta::new(*dex_quote_pk, false),
        AccountMeta::new_readonly(spl_token::ID, false),
        AccountMeta::new_readonly(solana_program::sysvar::rent::ID, false),
        AccountMeta::new(*srm_vault_pk, false),
        AccountMeta::new_readonly(*dex_signer_pk, false),
    ];


    accounts.extend(open_orders_pks.iter().map(
        |pk| 
        if *pk == Pubkey::default(){
            AccountMeta::new_readonly(*pk, false)
        } else {
            AccountMeta::new(*pk, false)
        })
    );
    accounts.extend(oracle_pks.iter().map(
        |pk| AccountMeta::new_readonly(*pk, false))
    );

    let instr = MangoInstruction::PlaceAndSettle { order };
    let data = instr.pack();
    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data
    })
}

pub fn instruction_place_order(
    program_id: &Pubkey,
    mango_group_pk: &Pubkey,
    owner_pk: &Pubkey,
    margin_account_pk: &Pubkey,
    dex_prog_id: &Pubkey,
    spot_market_pk: &Pubkey,
    dex_request_queue_pk: &Pubkey,
    dex_event_queue_pk: &Pubkey,
    bids_pk: &Pubkey,
    asks_pk: &Pubkey,
    vault_pk: &Pubkey,
    signer_pk: &Pubkey,
    dex_base_pk: &Pubkey,
    dex_quote_pk: &Pubkey,
    srm_vault_pk: &Pubkey,
    open_orders_pks: &[Pubkey],
    oracle_pks: &[Pubkey],
    order: serum_dex::instruction::NewOrderInstructionV3
) -> Result<Instruction, ProgramError> {

    let mut accounts = vec![
        AccountMeta::new(*mango_group_pk, false),
        AccountMeta::new_readonly(*owner_pk, true),
        AccountMeta::new(*margin_account_pk, false),
        AccountMeta::new_readonly(solana_program::sysvar::clock::ID, false),
        AccountMeta::new_readonly(*dex_prog_id, false),
        AccountMeta::new(*spot_market_pk, false),
        AccountMeta::new(*dex_request_queue_pk, false),
        AccountMeta::new(*dex_event_queue_pk, false),
        AccountMeta::new(*bids_pk, false),
        AccountMeta::new(*asks_pk, false),
        AccountMeta::new(*vault_pk, false),
        AccountMeta::new_readonly(*signer_pk, false),
        AccountMeta::new(*dex_base_pk, false),
        AccountMeta::new(*dex_quote_pk, false),
        AccountMeta::new_readonly(spl_token::ID, false),
        AccountMeta::new_readonly(solana_program::sysvar::rent::ID, false),
        AccountMeta::new(*srm_vault_pk, false),
    ];

    accounts.extend(open_orders_pks.iter().map(
        |pk| 
        if *pk == Pubkey::default(){
            AccountMeta::new_readonly(*pk, false)
        } else {
            AccountMeta::new(*pk, false)
        })
    );
    accounts.extend(oracle_pks.iter().map(
        |pk| AccountMeta::new_readonly(*pk, false))
    );

    let instr = MangoInstruction::PlaceOrder { order };
    let data = instr.pack();
    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data
    })
}
