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


pub fn mango_init_margin_account (
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
        fund_state_acc,
        manager_acc,
        fund_pda_acc,
        mango_prog_acc,
        mango_group_acc,
        rent_acc
    ] = fixed_accs;

    let mut fund_data = FundData::load_mut_checked(fund_state_acc, program_id)?;

    check!(manager_acc.is_signer, ProgramError::MissingRequiredSignature);
    check_eq!(fund_data.manager_account, *manager_acc.key);


    for i in 0..NUM_MARGIN {
        check_eq!(fund_data.mango_positions[i].margin_account, Pubkey::default());
        invoke_signed(
            &init_margin_account(mango_prog_acc.key, mango_group_acc.key, margin_accs[i].key, fund_pda_acc.key)?,
            &[
                mango_prog_acc.clone(),
                mango_group_acc.clone(),
                margin_accs[i].clone(),
                fund_pda_acc.clone(),
                rent_acc.clone()
            ],
            &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
        )?;
        fund_data.mango_positions[i].margin_account = *margin_accs[i].key;
        fund_data.mango_positions[i].state = 0; // inactive state
    }
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

    let mut fund_data = FundData::load_mut_checked(fund_state_acc, program_id)?;
    check!(fund_data.is_initialized, ProgramError::InvalidAccountData);

    let index = fund_data.get_margin_index(margin_account_acc.key).unwrap();

    check!(fund_data.mango_positions[index].state == 0, FundError::InvalidMangoState); // has to be inactive for deposit
    check!(manager_acc.is_signer, ProgramError::MissingRequiredSignature);
    check_eq!(fund_data.manager_account, *manager_acc.key);

    let mut amount_in_margin = 0;
    if fund_data.no_of_margin_positions != 0 { // there's an active position
        if index == 0 {
            amount_in_margin = fund_data.mango_positions[1].trade_amount;
        }
        else {
            amount_in_margin = fund_data.mango_positions[0].trade_amount;
        }
    }
    let avail_amount = fund_data.total_amount.checked_mul(U64F64!(0.4)).unwrap()
        .checked_sub(U64F64::from_num(amount_in_margin)).unwrap();
    
    check!(U64F64::from_num(quantity) <= avail_amount, ProgramError::InsufficientFunds);
    
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

    check_eq!(fund_data.tokens[0].vault, *token_account_acc.key); 

    fund_data.tokens[0].balance = token_account.amount;
    fund_data.mango_positions[index].trade_amount = quantity;
    fund_data.mango_positions[index].state = 1; // set state to deposited
    Ok(())
}

pub fn mango_open_position (
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    side: u8,
    price: u64,
    trade_size: u64
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


    // let mut margin_account = MarginAccount::load_mut(margin_account_acc)?;
    let mut fund_data = FundData::load_mut_checked(fund_state_acc, program_id)?;
    let index = fund_data.get_margin_index(margin_account_acc.key).unwrap();

    check!(manager_acc.is_signer, ProgramError::MissingRequiredSignature);
    check_eq!(fund_data.manager_account, *manager_acc.key);
    check!(fund_data.mango_positions[index].state == 1, FundError::InvalidMangoState); // has to be deposited
    
    let coin_lots = convert_size_to_lots(spot_market_acc, dex_prog_acc.key, trade_size, false)?;
    msg!("coin_lots:: {:?} ", coin_lots);

    let pc_qty = convert_size_to_lots(spot_market_acc, dex_prog_acc.key, trade_size * price, true)?;
    msg!("pc_qty:: {:?}", pc_qty);
    let fee_rate:U64F64 = U64F64!(0.0022); // fee_bps = 22; BASE

    let exact_fee: u64 = U64F64::to_num(fee_rate.checked_mul(U64F64::from_num(pc_qty)).unwrap());

    let pc_qty_including_fees = pc_qty + exact_fee;
    msg!("pc_qty:: {:?}", pc_qty_including_fees);

    let order_side = serum_dex::matching::Side::try_from_primitive(side.try_into().unwrap()).unwrap();

    //here 
    let order = NewOrderInstructionV3 {
        side: order_side,
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
    let margin_data = MarginAccount::load(margin_account_acc)?;
    let mango_group_data = MangoGroup::load(mango_group_acc)?;

    let prices = get_prices(&mango_group_data, oracle_accs)?;
    let coll_ratio = margin_data.get_collateral_ratio(&mango_group_data, &prices, open_orders_accs)?;

    // restrict to 2x leverage
    check!(coll_ratio >= U64F64!(1.4), ProgramError::InsufficientFunds);

    let token_index = mango_group_data.get_market_index(spot_market_acc.key).unwrap();

    fund_data.no_of_margin_positions += 1;
    fund_data.position_count += 1;
    fund_data.mango_positions[index].fund_share = U64F64!(1);
    fund_data.mango_positions[index].share_ratio = U64F64!(1);
    fund_data.mango_positions[index].margin_index = u8::try_from(token_index).unwrap();
    fund_data.mango_positions[index].state = 2; // change state to open_position
    fund_data.mango_positions[index].position_side = side;
    fund_data.mango_positions[index].position_id = fund_data.position_count;

    Ok(())
}

pub fn mango_close_position (
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    price: u64,
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

    let mut fund_data = FundData::load_mut_checked(fund_state_acc, program_id)?;
    let index = fund_data.get_margin_index(margin_account_acc.key).unwrap();

    check!(manager_acc.is_signer, ProgramError::MissingRequiredSignature);
    check_eq!(fund_data.manager_account, *manager_acc.key);
    check!(fund_data.mango_positions[index].state == 3, FundError::InvalidMangoState); // has to be settled_open

    let side = fund_data.mango_positions[index].position_side;
    let token_index = fund_data.mango_positions[index].margin_index as usize;
    let close_amount = get_close_amount(margin_account_acc, mango_group_acc, side, token_index)?;

    //let coin_lots = convert_size_to_lots(spot_market_acc, dex_prog_acc.key, close_amount, false)?;
    let coin_lots = get_withdraw_lots(spot_market_acc, dex_prog_acc.key, close_amount, side)?;
    //let coin_lots = convert_size_to_lots(spot_market_acc, dex_prog_acc.key, close_amount, false)?;
    if coin_lots == 0 {
        // allow for close when the trades have been settled by investors
        //fund_data.no_of_margin_positions -= 1;
        fund_data.mango_positions[index].state = 4; // change to close_position
        return Ok(())
    }
    msg!("coin_lots : {:?}", coin_lots);

    let pc_qty = convert_size_to_lots(spot_market_acc, dex_prog_acc.key, close_amount * price, true)?;
    let fee_rate:U64F64 = U64F64!(0.0022); // fee_bps = 22; BASE
    let exact_fee: u64 = U64F64::to_num(fee_rate.checked_mul(U64F64::from_num(pc_qty)).unwrap());
    let pc_qty_including_fees = pc_qty + exact_fee;

    let side = serum_dex::matching::Side::try_from_primitive(side.try_into().unwrap()).unwrap();
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

    //fund_data.no_of_margin_positions -= 1;
    fund_data.mango_positions[index].state = 4; // change to close_position

    Ok(())
}

pub fn mango_settle_position (
    program_id: &Pubkey,
    accounts: &[AccountInfo],
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
    let mut fund_data = FundData::load_mut_checked(fund_state_acc, program_id)?;
    let index = fund_data.get_margin_index(margin_account_acc.key).unwrap();

    check!(fund_data.mango_positions[index].state == 2 || fund_data.mango_positions[index].state == 4, FundError::InvalidMangoState);
    check!(manager_acc.is_signer, ProgramError::MissingRequiredSignature);
    check_eq!(fund_data.manager_account, *manager_acc.key);

    
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
    let token_index;
    let settle_amount;

    if fund_data.mango_positions[index].position_side == 0 { // for LONG settle USDC
        token_index = NUM_MARKETS;
        settle_amount = get_settle_amount(margin_account_acc, mango_group_acc, NUM_MARKETS)?;
    }
    else { // for SHORT settle token
        token_index = fund_data.mango_positions[index].margin_index as usize;
        settle_amount = get_settle_amount(margin_account_acc, mango_group_acc, token_index)?;
    }
    msg!("settle_amount :{:?} for token: {:?}", settle_amount, token_index);
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

    fund_data.mango_positions[index].state += 1; // increement state
    let margin_data = MarginAccount::load(margin_account_acc)?;
    // sanity check
    check!(*open_orders_acc.key == margin_data.open_orders[fund_data.mango_positions[index].margin_index as usize],
        ProgramError::InvalidAccountData);
    Ok(())
}

pub fn mango_withdraw_fund (
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> Result<(), ProgramError>
{
    const NUM_FIXED: usize = 11;
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

    let quantity = get_withdraw_amount(&mut fund_data, margin_account_acc, mango_group_acc,open_orders_accs, oracle_accs, index)?;
 
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

    if fund_data.mango_positions[index].debtors == 0 { // no debtors, close full
        fund_data.mango_positions[index].fund_share = U64F64!(0);
        fund_data.mango_positions[index].share_ratio = U64F64!(1);
        fund_data.mango_positions[index].position_side = 0;
        fund_data.mango_positions[index].position_id = 0;
        fund_data.mango_positions[index].margin_index = 0;
        fund_data.mango_positions[index].trade_amount = 0;

        fund_data.mango_positions[index].state = 0; // set state to inactive for new positions
        fund_data.no_of_margin_positions -= 1;
    }
    else { // active debtors
        fund_data.mango_positions[index].share_ratio = fund_data.mango_positions[index].share_ratio
        .checked_sub(fund_data.mango_positions[index].fund_share).unwrap();
        fund_data.mango_positions[index].fund_share = U64F64!(0);

        fund_data.mango_positions[index].state = 6; // set state to stale until tx complete

    }
    
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

    if fund_data.mango_positions[index].state == 6 { // stale state
        return Ok(())
    }

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
    fund_data.mango_positions[index].debtors = fund_data.mango_positions[index].debtors.checked_sub(1).unwrap();

    // close if last investor in position has withdrawn
    if last_investor || (fund_data.mango_positions[index].debtors == 0) {
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
    fund_data: &FundData,
    margin_acc: &AccountInfo,
    mango_group_acc: &AccountInfo,
    open_orders_accs: &[AccountInfo; 4],
    oracle_accs: &[AccountInfo; 4],
    pos_index: usize
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
    .checked_mul(fund_data.mango_positions[pos_index].fund_share / fund_data.mango_positions[pos_index].share_ratio).unwrap()
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

// pub fn update_investor_debts(
//     fund_data: &FundData,
//     investor_accs: &[AccountInfo],
//     withdraw_amount: u64,
//     index: usize
// ) -> Result<(u64, U64F64), ProgramError> {
    
//     let mut debts: u64 = 0;
//     let mut debts_share = U64F64!(0);

//     for i in 0..MAX_INVESTORS_WITHDRAW {
//         if *investor_accs[i].key == Pubkey::default() {
//             continue;
//         }
//         let mut investor_data = InvestorData::load_mut(&investor_accs[i])?;
//         if investor_data.margin_position_id[index] == fund_data.mango_positions[index].position_id as u64 {
//             // update
//             let debt_valuation: u64 = U64F64::to_num(U64F64::from_num(withdraw_amount)
//             .checked_mul(investor_data.margin_debt[index] / fund_data.mango_positions[index].share_ratio).unwrap());
//             debts += debt_valuation;
//             debts_share += investor_data.margin_debt[index] / fund_data.mango_positions[index].share_ratio;

//             // update investor debts; add to USDC debt
//             investor_data.margin_debt[index] = U64F64!(0);
//             investor_data.token_debts[0] += debt_valuation;
//             investor_data.has_withdrawn = true;
//             investor_data.withdrawn_from_margin = false;
//             investor_data.margin_position_id[index] = 0; // remove position id
//         }
//     }
//     Ok((debts, debts_share))
// }

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
