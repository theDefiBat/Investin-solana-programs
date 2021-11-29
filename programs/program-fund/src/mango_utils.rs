use bytemuck::bytes_of;
use fixed::types::U64F64;
use solana_program::{
    account_info::AccountInfo,
    msg,
    instruction:: {AccountMeta, Instruction},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    program::invoke_signed,
};
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;
use std::convert::TryInto;

use fixed_macro::types::U64F64;
pub const ONE_U64F64: U64F64 = U64F64!(1);
use serum_dex::matching::{Side, OrderType};
use serum_dex::state::MarketState;

use serum_dex::instruction::{NewOrderInstructionV3, SelfTradeBehavior};
use std::num::NonZeroU64;


use arrayref::{array_ref, array_refs};

use crate::error::FundError;
use crate::state::{MAX_INVESTORS_WITHDRAW, NUM_MARGIN, FundData, InvestorData};
use crate::state::Loadable;
use crate::processor::{ parse_token_account, get_margin_valuation};

use mango::state::{MarginAccount, MangoGroup, NUM_MARKETS};
use mango::state::Loadable as OtherLoadable;
use mango::instruction::{init_margin_account, deposit, withdraw, settle_funds, settle_borrow, MangoInstruction};
use mango::processor::get_prices;
use spl_token::state::Account;

pub const MANGO_NUM_TOKENS:usize = 5;

macro_rules! check {
    ($cond:expr, $err:expr) => {
        if !($cond) {
            return Err(($err).into())
        }
    }
}

macro_rules! check_eq {
    ($x:expr, $y:expr) => {
        if ($x != $y) {
            return Err(FundError::Default.into())
        }
    }
}


pub fn mango_init_mango_account (
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> Result<(), ProgramError>
{
    const NUM_FIXED: usize = 6;
    let accounts = array_ref![accounts, 0, NUM_FIXED + NUM_MARGIN];
    let (
        fixed_accs,
        margin_accs
    ) = array_refs![accounts, NUM_FIXED, NUM_MARGIN];

    let [
        fund_state_ai,
        manager_ai,
        fund_pda_ai,
        mango_prog_ai,
        mango_group_ai,
        mango_account_ai,
    ] = fixed_accs;

    let mut fund_data = FundData::load_mut_checked(fund_state_acc, program_id)?;
    //TODO --what if liquidtated once
    check!(manager_ai.is_signer, ProgramError::MissingRequiredSignature);
    check_eq!(fund_data.manager_account, *manager_acc.key);
    check_eq!(fund_data.mango_positions.mango_account, Pubkey::default());
    invoke_signed(
        &init_mango_account(mango_prog_ai.key, mango_group_ai.key, mango_account_ai.key, fund_pda_acc.key)?,
        &[
            mango_prog_ai.clone(),
            mango_group_ai.clone(),
            mango_account_ai.clone(),
            fund_pda_acc.clone()
        ],
        &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
        )?;
    fund_data.mango_positions.mango_account = *mango_account_ai.key;
    
    Ok(())
}

pub fn mango_deposit (
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    token_index: u8,
    quantity: u64
) -> Result<(), ProgramError>
{
    const NUM_FIXED: usize = 12;
    let accounts = array_ref![accounts, 0, NUM_FIXED];
    let [
        fund_state_ai,
        manager_ai, // or delegate
        fund_pda_ai,
        mango_prog_ai,
        mango_group_ai,         // read
        mango_account_ai,       // write
        mango_cache_ai,         // read
        root_bank_ai,           // read
        node_bank_ai,           // write
        vault_ai,               // write
        token_prog_ai,          // read
        owner_token_account_ai, // write
    ] = accounts;

    let mut fund_data = FundData::load_mut_checked(fund_state_acc, program_id)?;
    //TODO Check for USDC or Active deposit token index OR ELSE check for availabel deposit slot
    //TODO check token_index passed matches the corresponding accounts
    check!(token_index == 0 || )
    check!(fund_data.is_initialized, ProgramError::InvalidAccountData);
    check!(manager_acc.is_signer, ProgramError::MissingRequiredSignature);
    
    // check_eq!(fund_data.manager_account, *manager_acc.key);
    check!((fund_data.manager_account == *manager_acc.key), FundError::ManagerMismatch);

    // check fund vault
    check_eq!(fund_data.vault_key, *owner_token_account_ai.key); 
    
    invoke_signed(
        &deposit(mango_prog_ai.key, mango_group_ai.key, mango_account_ai.key, fund_pda_acc.key,
            mango_cache_ai.key, root_bank_ai.key, node_bank_ai.key, vault_ai.key, owner_token_account_ai.key, quantity)?,
        &[
            mango_prog_ai.clone(),
            mango_group_ai.clone(),
            mango_account_ai.clone(),
            fund_pda_acc.clone(),
            mango_cache_ai.clone(),
            root_bank_ai.clone(),
            node_bank_ai.clone(),
            vault_ai.clone(),
            owner_token_account_ai.clone(),
            token_prog_ai.clone()
        ],
        &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
    )?;

    msg!("invoke done");

    fund_data.vault_balance = parse_token_account(owner_token_account_ai)?.amount;
    //TODO get mint from token_index and get token slot from mint and update tokenSolot acccordingly is_on_mango *if not USDC
    //check if balance is > debt on token_slot >>Similar to Swap
    Ok(())
}

pub fn mango_place_perp_order (
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    perp_market_id: u8,
    side: Side,
    quantity: i64
) -> Result<(), ProgramError>

{
    const NUM_FIXED: usize = 12;
    let accounts = array_ref![accounts, 0, NUM_FIXED];

    let [
        fund_state_acc,
        manager_acc,
        mango_prog_ai,
        mango_group_ai,     // read
        mango_account_ai,   // write
        fund_pda_acc,           // read, signer
        mango_cache_ai,     // read
        perp_market_ai,     // write
        bids_ai,            // write
        asks_ai,            // write
        event_queue_ai,    // write
        default_acc,
    ] = accounts;

    let fund_data = FundData::load_mut_checked(fund_state_acc, program_id)?;
    //Check for perp_market_id matches derived from ai 
    //Check for perp_market_id already active on fund/add perp_market if markets_active < 4
    //Check if its close on full amount, if yes remove from active perp_markets on funds --> END
    //Base_position + taker_base and quote_position and taker_quote should both be considered
    //Settle PnL to be executed right after place_perp_order...

    check!(manager_acc.is_signer, ProgramError::MissingRequiredSignature);

    // check_eq!(fund_data.manager_account, *manager_acc.key);
    check!((fund_data.manager_account == *manager_acc.key), FundError::ManagerMismatch);
    
    let open_orders_accs = [Pubkey::default(); MAX_PAIRS];
    invoke_signed(
        &place_perp_order(mango_prog_ai.key,
            mango_group_ai.key, mango_account_ai.key, fund_pda_acc.key,
            mango_cache_ai.key,perp_market_ai.key, bids_ai.key, asks_ai.key, event_queue_ai.key, &open_orders_accs,
            side, price, quantity, client_order_id, order_type, true)?,
        &[
            mango_prog_ai.clone(),
            mango_group_ai.clone(),
            mango_account_ai.clone(),
            fund_pda_acc.clone(),
            perp_market_ai.clone(),
            mango_cache_ai.clone(),
            bids_ai.clone(),
            asks_ai.clone(),
            default_acc.clone(),
            event_queue_ai.clone()
        ],
        &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
    )?;
    Ok(())
}

//TODO::Update!!!
pub fn mango_settle_pnl(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> Result<(), ProgramError>
{
    const NUM_FIXED: usize = 9;
    let accounts = array_ref![accounts, 0, NUM_FIXED];

    let [
        fund_state_ai,
        manager_ai,
        fund_pda_ai,
        mango_prog_ai,
        mango_group_ai,
        mango_account_ai,  
    ]

}

pub fn mango_withdraw_fund (
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> Result<(), ProgramError>
{
    const NUM_FIXED: usize = 11;
    let accounts = array_ref![accounts, 0, NUM_FIXED + 2 * NUM_MARKETS + MAX_INVESTORS_WITHDRAW];
    let (
        fixed_accs,
        open_orders_accs,
        oracle_accs,
        investor_accs
    ) = array_refs![accounts, NUM_FIXED, NUM_MARKETS, NUM_MARKETS, MAX_INVESTORS_WITHDRAW];

    let [
        fund_state_acc,
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

    let mut fund_data = FundData::load_mut_checked(fund_state_acc, program_id)?;
    let index = fund_data.get_margin_index(margin_account_acc.key).unwrap();

    check!(manager_acc.is_signer, ProgramError::MissingRequiredSignature);
    check_eq!(fund_data.manager_account, *manager_acc.key);

    // position has to be settled_close
    check!(fund_data.mango_positions[index].state == 5, FundError::InvalidMangoState);

    let quantity = get_withdraw_amount(margin_account_acc, mango_group_acc,open_orders_accs, oracle_accs)?;
 
    msg!("withdraw_quantity:: {:?}", quantity);

    let pre_amount = Account::unpack(&token_account_acc.try_borrow_data()?)?.amount;

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

    check!(*token_account_acc.key == fund_data.tokens[0].vault, FundError::InvalidTokenAccount);
    let post_amount = Account::unpack(&token_account_acc.try_borrow_data()?)?.amount;
    let withdraw_amount = post_amount - pre_amount;
    check!(quantity == withdraw_amount, ProgramError::InsufficientFunds);

    // update investor state account debts
    let (debts, debts_share) = update_investor_debts(&fund_data, investor_accs, withdraw_amount, index)?;

    // msg!("debts_share settled:: {:?}", debts_share);
    // msg!("fund_share:: {:?}", fund_data.mango_positions[index].fund_share);
    // msg!("share_ratio:: {:?}", fund_data.mango_positions[index].share_ratio);
    
    // all investors should be passed
    //check!(fund_data.mango_positions[0].fund_share.checked_add(debts_share).unwrap() > U64F64!(0.9999),
    //ProgramError::InvalidAccountData);
    
    // add margin_debt to USDC debt
    fund_data.tokens[0].balance = post_amount;
    fund_data.tokens[0].debt += debts;
    check!(fund_data.tokens[0].balance >= fund_data.tokens[0].debt, FundError::InvalidAmount);

    fund_data.mango_positions[index].fund_share = U64F64!(0);
    fund_data.mango_positions[index].share_ratio = U64F64!(1);
    fund_data.mango_positions[index].position_side = 0;
    fund_data.mango_positions[index].position_id = 0;
    fund_data.mango_positions[index].margin_index = 0;
    fund_data.mango_positions[index].trade_amount = 0;

    fund_data.mango_positions[index].state = 0; // set state to inactive for new positions
    Ok(())
}

pub fn mango_withdraw_investor_place_order (
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    price: u64
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

    let mut fund_data = FundData::load_mut_checked(fund_state_acc, program_id)?;
    let mut investor_data = InvestorData::load_mut_checked(investor_state_acc, program_id)?;
    let index = fund_data.get_margin_index(margin_account_acc.key).unwrap();

    check!(investor_data.owner == *investor_acc.key, ProgramError::MissingRequiredSignature);
    check!(investor_acc.is_signer, ProgramError::MissingRequiredSignature);

    check!(investor_data.margin_position_id[index] != 0, ProgramError::InvalidAccountData);
    check!(investor_data.margin_debt[index] > 0, ProgramError::InvalidAccountData);
    check_eq!(investor_data.margin_position_id[index], fund_data.mango_positions[index].position_id as u64);
    check!(fund_data.mango_positions[index].state != 0, ProgramError::InvalidAccountData);

    let token_index = fund_data.mango_positions[index].margin_index as usize;
    let (place_amount, position_amount) = get_investor_place_amount(&mut fund_data, &mut investor_data, 
        margin_account_acc, mango_group_acc, token_index, index)?;

    //msg!("place amount:: {:?}", place_amount);
    let coin_lots = get_investor_withdraw_lots(spot_market_acc, dex_prog_acc.key, place_amount, position_amount, fund_data.mango_positions[index].position_side)?;

    //msg!("coin_lots:: {:?}", coin_lots);

    let pc_qty = convert_size_to_lots(spot_market_acc, dex_prog_acc.key, place_amount * price, true)?;
    let fee_rate:U64F64 = U64F64!(0.0022); // fee_bps = 22; BASE
    let exact_fee: u64 = U64F64::to_num(fee_rate.checked_mul(U64F64::from_num(pc_qty)).unwrap());
    let pc_qty_including_fees = pc_qty + exact_fee;

    let side = serum_dex::matching::Side::try_from_primitive(fund_data.mango_positions[index].position_side.try_into().unwrap()).unwrap();
    let order = NewOrderInstructionV3 {
        side: match side { // reverse of open position
            Side::Bid => Side::Ask,
            Side::Ask => Side::Bid,
        },
        limit_price: NonZeroU64::new(price).unwrap(),
        max_coin_qty: NonZeroU64::new(coin_lots).unwrap(),
        max_native_pc_qty_including_fees: NonZeroU64::new(pc_qty_including_fees).unwrap(),
        order_type: OrderType::ImmediateOrCancel,
        client_order_id: 1,
        self_trade_behavior: SelfTradeBehavior::AbortTransaction,
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
    Ok(())

}

pub fn get_investor_place_amount (
    fund_data: &mut FundData,
    investor_data: &mut InvestorData,
    margin_acc: &AccountInfo,
    mango_group_acc: &AccountInfo,
    token_index: usize,
    pos_index: usize
) -> Result<(u64, u64), ProgramError> {
    
    let margin_data = MarginAccount::load(&margin_acc)?;
    let mango_group = MangoGroup::load(&mango_group_acc)?;

    let inv_share = investor_data.margin_debt[pos_index] / fund_data.mango_positions[pos_index].share_ratio;
    // msg!("investor share:: {:?}", inv_share);
    let mut place_amount;
    let position_amount;

    if fund_data.mango_positions[pos_index].position_side == 0 { // LONG
        position_amount = mango_group.indexes[token_index].deposit
        .checked_mul(margin_data.deposits[token_index]).unwrap();
        place_amount = position_amount.checked_mul(inv_share).unwrap();
    }
    else { // SHORT
        position_amount = mango_group.indexes[token_index].borrow
        .checked_mul(margin_data.borrows[token_index]).unwrap();
        place_amount = position_amount.checked_mul(inv_share).unwrap();
    }
    // place_amount = place_amount.checked_add(U64F64!(1)).unwrap();

    Ok((U64F64::to_num(place_amount), U64F64::to_num(position_amount)))
}

pub fn mango_withdraw_investor_settle (
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> Result<(), ProgramError> {
    const NUM_FIXED: usize = 19;
    let accounts = array_ref![accounts, 0, NUM_FIXED];
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
        open_orders_acc,
        signer_acc,
        dex_base_acc,
        dex_quote_acc,
        base_vault_acc,
        quote_vault_acc,
        dex_signer_acc,
        token_prog_acc,
        oracle_acc,
    ] = accounts;

    let fund_data = FundData::load_checked(fund_state_acc, program_id)?;
    let investor_data = InvestorData::load_checked(investor_state_acc, program_id)?;
    let index = fund_data.get_margin_index(margin_account_acc.key).unwrap();

    check!(investor_acc.is_signer, ProgramError::MissingRequiredSignature);
    check!(investor_data.owner == *investor_acc.key, ProgramError::MissingRequiredSignature);

    check!(investor_data.margin_debt[index] > 0, ProgramError::InvalidAccountData);
    check_eq!(investor_data.margin_position_id[index], fund_data.mango_positions[index].position_id as u64);
    //check!(investor_data.withdrawn_from_margin, ProgramError::InvalidAccountData);
    check!(fund_data.mango_positions[index].state != 0, ProgramError::InvalidAccountData);

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
    let (settle_amount, token_index) = get_investor_settle_amount(&fund_data, &investor_data,
        margin_account_acc, mango_group_acc, oracle_acc, open_orders_acc, index)?;
    msg!("settling borrows: {:?} for token: {:?}", settle_amount, token_index);
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
    msg!("done with settle borrows");
    Ok(())
}

pub fn mango_withdraw_investor (
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
        investor_state_acc,
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

    let mut fund_data = FundData::load_mut_checked(fund_state_acc, program_id)?;
    let mut investor_data = InvestorData::load_mut_checked(investor_state_acc, program_id)?;
    let index = fund_data.get_margin_index(margin_account_acc.key).unwrap();

    check!(investor_acc.is_signer, ProgramError::MissingRequiredSignature);
    check!(investor_data.owner == *investor_acc.key, ProgramError::MissingRequiredSignature);
    check!(investor_data.margin_debt[index] > 0, ProgramError::InvalidAccountData);
    check_eq!(investor_data.margin_position_id[index], fund_data.mango_positions[index].position_id as u64);
    //check!(investor_data.withdrawn_from_margin, ProgramError::InvalidAccountData);

    msg!("checks passed");
    // position has to be settled_close
    let (quantity, last_investor) = get_investor_withdraw_amount(&fund_data, &investor_data,
        margin_account_acc, mango_group_acc, open_orders_accs, oracle_accs, index)?;

    msg!("withdraw_quantity:: {:?}", quantity);
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

    fund_data.mango_positions[index].share_ratio = fund_data.mango_positions[index].share_ratio
    .checked_sub(investor_data.margin_debt[index]).unwrap();
    investor_data.margin_debt[index] = U64F64!(0); // zero out investor_debt
    investor_data.margin_position_id[index] = 0; // not in investment queue
    investor_data.withdrawn_from_margin = false;

    // close if last investor in position has withdrawn
    if last_investor {
        fund_data.mango_positions[index].fund_share = U64F64!(0);
        fund_data.mango_positions[index].share_ratio = U64F64!(1);
        fund_data.mango_positions[index].position_side = 0;
        fund_data.mango_positions[index].position_id = 0;
        fund_data.mango_positions[index].margin_index = 0;
        fund_data.mango_positions[index].trade_amount = 0;
        fund_data.mango_positions[index].state = 0; // set state to inactive for new positions
        fund_data.no_of_margin_positions -= 1;
    }

    Ok(())
}

pub fn get_coll_ratio (
    margin_acc: &AccountInfo,
    mango_group_acc: &AccountInfo,
    oracle_accs: &[AccountInfo; 4],
    open_orders_accs: &[AccountInfo; 4],
) -> Result<U64F64, ProgramError> {

    let margin_data = MarginAccount::load(&margin_acc)?;
    let mango_group = MangoGroup::load(&mango_group_acc)?;

    let prices = get_prices(&mango_group, oracle_accs)?;
    let coll_ratio = margin_data.get_collateral_ratio(&mango_group, &prices, open_orders_accs)?;

    Ok(coll_ratio)
}

pub fn get_withdraw_amount (
    margin_acc: &AccountInfo,
    mango_group_acc: &AccountInfo,
    open_orders_accs: &[AccountInfo; 4],
    oracle_accs: &[AccountInfo; 4],
) -> Result<u64, ProgramError> {
    
    let margin_data = MarginAccount::load(&margin_acc)?;
    let mango_group = MangoGroup::load(&mango_group_acc)?;

    let prices = get_prices(&mango_group, oracle_accs)?;
    let equity = margin_data.get_equity(&mango_group, &prices, open_orders_accs)?;

    msg!("deposits USDC : {:?}", margin_data.deposits[NUM_MARKETS]);
    msg!("borrows USDC {:?}", margin_data.borrows[NUM_MARKETS]);

    msg!("deposits SRM : {:?}", margin_data.deposits[3]);
    msg!("borrows SSRM : {:?}", margin_data.borrows[3]);


    // make dust amount as 0.1 USDC to account for borrows
    let dust_amount = U64F64::from_num(10u64.pow((mango_group.mint_decimals[NUM_MARKETS] - 1) as u32));
    let withdraw_amount = mango_group.indexes[NUM_MARKETS].deposit
    .checked_mul(margin_data.deposits[NUM_MARKETS]).unwrap()
    .checked_sub(dust_amount).unwrap();

    msg!("equity : {:?}", equity);
    msg!("withdraw amount USDC: {:?}", withdraw_amount);

    // balances should be settled (10 is the DUST threshold)
    //check!(equity - withdraw_amount < 10, FundError::InvalidAmount);

    return Ok(U64F64::to_num(withdraw_amount))
}

pub fn get_close_amount (
    margin_acc: &AccountInfo,
    mango_group_acc: &AccountInfo,
    side: u8,
    token_index: usize
) -> Result<u64, ProgramError> {
    
    let margin_data = MarginAccount::load(&margin_acc)?;
    let mango_group = MangoGroup::load(&mango_group_acc)?;  
    let mut close_amount;

    if side == 0 { // LONG
        close_amount = mango_group.indexes[token_index].deposit
        .checked_mul(margin_data.deposits[token_index]).unwrap();
        // .checked_sub(U64F64!(100)).unwrap();
    }
    else { // SHORT
        close_amount = mango_group.indexes[token_index].borrow
        .checked_mul(margin_data.borrows[token_index]).unwrap();
        // .checked_add(U64F64!(100)).unwrap();
    }
    close_amount = close_amount.checked_add(U64F64!(1)).unwrap();
    return Ok(U64F64::to_num(close_amount)) // account for rounding
}

pub fn get_settle_amount (
    margin_acc: &AccountInfo,
    mango_group_acc: &AccountInfo,
    token_index: usize
) -> Result<u64, ProgramError> {

    let margin_data = MarginAccount::load(&margin_acc)?;
    let mango_group = MangoGroup::load(&mango_group_acc)?;
    let settle_amount = mango_group.indexes[token_index].deposit
    .checked_mul(margin_data.deposits[token_index]).unwrap();
    Ok(U64F64::to_num(settle_amount))
}

pub fn get_investor_settle_amount (
    fund_data: &FundData,
    investor_data: &InvestorData,
    margin_acc: &AccountInfo,
    mango_group_acc: &AccountInfo,
    oracle_acc: &AccountInfo,
    open_orders_acc: &AccountInfo,
    index: usize
) -> Result<(u64, usize), ProgramError> {

    let margin_data = MarginAccount::load(&margin_acc)?;
    let mango_group = MangoGroup::load(&mango_group_acc)?;

    let token_index = fund_data.mango_positions[index].margin_index as usize;
    let equity = get_margin_valuation(token_index, &mango_group, &margin_data, oracle_acc, open_orders_acc)?;

    let dust_amount = U64F64::from_num(10u64.pow((mango_group.mint_decimals[NUM_MARKETS] - 1) as u32));

    let inv_debt = equity.checked_mul(investor_data.margin_debt[index] / fund_data.mango_positions[index].share_ratio).unwrap();
    //.checked_sub(dust_amount).unwrap();
    if fund_data.mango_positions[index].position_side == 0 { // for LONG settle USDC
        // settle only if there are USDC borrows
        if margin_data.borrows[NUM_MARKETS] < 1 {
            Ok((0, NUM_MARKETS))
        }
        else {
            let deposit_amount = mango_group.indexes[NUM_MARKETS].deposit
            .checked_mul(margin_data.deposits[NUM_MARKETS]).unwrap();
            let settle_amount = deposit_amount.checked_sub(inv_debt).unwrap();
            Ok((U64F64::to_num(settle_amount), NUM_MARKETS))
        }
    }
    else { // for SHORT settle full borrows of that token
        let token_index = fund_data.mango_positions[index].margin_index as usize;
        let deposit_amount = mango_group.indexes[token_index].deposit
        .checked_mul(margin_data.deposits[token_index]).unwrap();
        Ok((U64F64::to_num(deposit_amount), token_index))
    }
}

pub fn get_investor_withdraw_amount (
    fund_data: &FundData,
    investor_data: &InvestorData,
    margin_acc: &AccountInfo,
    mango_group_acc: &AccountInfo,
    open_orders_accs: &[AccountInfo; 4],
    oracle_accs: &[AccountInfo; 4],
    index: usize
) -> Result<(u64, bool), ProgramError> {

    let margin_data = MarginAccount::load(&margin_acc)?;
    let mango_group = MangoGroup::load(&mango_group_acc)?;

    let prices = get_prices(&mango_group, oracle_accs)?;
    let equity = margin_data.get_equity(&mango_group, &prices, open_orders_accs)?;

    let dust_amount = U64F64::from_num(10u64.pow((mango_group.mint_decimals[NUM_MARKETS] - 1) as u32));
    let withdraw_amount = equity.checked_mul(investor_data.margin_debt[index] / fund_data.mango_positions[index].share_ratio).unwrap()
    .checked_sub(dust_amount).unwrap();
    let deposit_amount = mango_group.indexes[NUM_MARKETS].deposit
    .checked_mul(margin_data.deposits[NUM_MARKETS]).unwrap();

    // if last investor, then close position
    let equity_threshold = U64F64::from_num(10u64.pow(mango_group.mint_decimals[NUM_MARKETS] as u32));
    let last_investor = equity.checked_sub(withdraw_amount).unwrap() < equity_threshold;
    // sanity check
    check!(withdraw_amount <= deposit_amount, ProgramError::InsufficientFunds);
    Ok((U64F64::to_num(withdraw_amount), last_investor))
}

pub fn convert_size_to_lots(
    spot_market_acc: &AccountInfo,
    dex_program_id: &Pubkey,
    size: u64,
    pc: bool
) -> Result <u64, ProgramError> {
    let market = MarketState::load(spot_market_acc, dex_program_id)?;
    if pc {
        Ok(size * market.pc_lot_size / market.coin_lot_size)
    }
    else {
        Ok(size / market.coin_lot_size)
    }
}

fn get_withdraw_lots(
    spot_market_acc: &AccountInfo,
    dex_program_id: &Pubkey,
    size: u64,
    side: u8,
) -> Result <u64, ProgramError> {
    let market = MarketState::load(spot_market_acc, dex_program_id)?;
    Ok((size / market.coin_lot_size) + side as u64)
}

pub fn get_investor_withdraw_lots(
    spot_market_acc: &AccountInfo,
    dex_program_id: &Pubkey,
    size: u64,
    pos_size: u64,
    side: u8
) -> Result <u64, ProgramError> {
    let market = MarketState::load(spot_market_acc, dex_program_id)?;
    //if size + market.coin_lot_size > pos_size {
    if (pos_size - size) / market.coin_lot_size == 0 {
        Ok((size / market.coin_lot_size) + side as u64) // same as manager close case
    }
    else {
        Ok((size / market.coin_lot_size) + 1)
    }
    // Ok((size / market.coin_lot_size) + side as u64)
}

pub fn update_investor_debts(
    fund_data: &FundData,
    investor_accs: &[AccountInfo],
    withdraw_amount: u64,
    index: usize
) -> Result<(u64, U64F64), ProgramError> {
    
    let mut debts: u64 = 0;
    let mut debts_share = U64F64!(0);

    for i in 0..MAX_INVESTORS_WITHDRAW {
        if *investor_accs[i].key == Pubkey::default() {
            continue;
        }
        let mut investor_data = InvestorData::load_mut(&investor_accs[i])?;
        if investor_data.margin_position_id[index] == fund_data.mango_positions[index].position_id as u64 {
            // update
            let debt_valuation: u64 = U64F64::to_num(U64F64::from_num(withdraw_amount)
            .checked_mul(investor_data.margin_debt[index] / fund_data.mango_positions[index].share_ratio).unwrap());
            debts += debt_valuation;
            debts_share += investor_data.margin_debt[index] / fund_data.mango_positions[index].share_ratio;

            // update investor debts; add to USDC debt
            investor_data.margin_debt[index] = U64F64!(0);
            investor_data.token_debts[0] += debt_valuation;
            investor_data.has_withdrawn = true;
            investor_data.withdrawn_from_margin = false;
            investor_data.margin_position_id[index] = 0; // remove position id
        }
    }
    Ok((debts, debts_share))
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
