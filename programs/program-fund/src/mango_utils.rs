use bytemuck::bytes_of;
use fixed::types::U64F64;
use solana_program::{
    account_info::AccountInfo,
    msg,
    instruction:: {AccountMeta, Instruction},
    program_error::ProgramError,
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
use crate::state::{NUM_TOKENS, MAX_INVESTORS_WITHDRAW, FundData, InvestorData};
use crate::state::Loadable;
use crate::processor::{ parse_token_account, get_equity_and_coll_ratio};

use mango::state::{MarginAccount, MangoGroup, NUM_MARKETS};
use mango::state::Loadable as OtherLoadable;
use mango::instruction::{init_margin_account, deposit, withdraw, settle_funds, settle_borrow, MangoInstruction};
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
    fund_data.mango_positions[0].state = 0; // inactive state
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
    check!(fund_data.mango_positions[0].state == 0, FundError::InvalidMangoState); // has to be inactive for deposit
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
    fund_data.mango_positions[0].trade_amount = quantity;
    fund_data.mango_positions[0].state = 1; // set state to deposited
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

    check!(manager_acc.is_signer, ProgramError::MissingRequiredSignature);

    // let mut margin_account = MarginAccount::load_mut(margin_account_acc)?;
    let mut fund_data = FundData::load_mut(fund_state_acc)?;

    check!(fund_data.mango_positions[0].state == 1, FundError::InvalidMangoState); // has to be deposited
    
    let coin_lots = convert_size_to_lots(spot_market_acc, dex_prog_acc.key, trade_size, false)?;
    msg!("coin_lots:: {:?} ", coin_lots);

    let pc_qty = convert_size_to_lots(spot_market_acc, dex_prog_acc.key, trade_size * price, true)?;
    msg!("pc_qty:: {:?}", pc_qty);
    let fee_rate:U64F64 = U64F64!(0.0022); // fee_bps = 22; BASE

    let exact_fee: u64 = U64F64::to_num(fee_rate.checked_mul(U64F64::from_num(pc_qty)).unwrap());

    let pc_qty_including_fees = pc_qty + exact_fee;
    msg!("pc_qty:: {:?}", pc_qty_including_fees);

    let order_side = serum_dex::matching::Side::try_from_primitive(side.try_into().unwrap()).unwrap();

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
    check!(coll_ratio >= U64F64!(1.45), ProgramError::InsufficientFunds);

    let token_index = mango_group_data.get_market_index(spot_market_acc.key).unwrap();

    fund_data.no_of_margin_positions += 1;
    fund_data.position_count += 1;
    fund_data.mango_positions[0].margin_index = u8::try_from(token_index).unwrap();
    fund_data.mango_positions[0].state = 2; // change state to open_position
    fund_data.mango_positions[0].position_side = side;
    fund_data.mango_positions[0].position_id = fund_data.position_count;

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

    check!(manager_acc.is_signer, ProgramError::MissingRequiredSignature);

    let mut fund_data = FundData::load_mut(fund_state_acc)?;

    check!(fund_data.mango_positions[0].state == 3, FundError::InvalidMangoState); // has to be settled_open

    let side = fund_data.mango_positions[0].position_side;
    let token_index = fund_data.mango_positions[0].margin_index as usize;
    let close_amount = get_close_amount(margin_account_acc, mango_group_acc, side, token_index)?;

    msg!("close amount:: {:?}", close_amount);
    let coin_lots = convert_size_to_lots(spot_market_acc, dex_prog_acc.key, close_amount, false)?;
    msg!("coin_lots:: {:?} ", coin_lots);

    let pc_qty = convert_size_to_lots(spot_market_acc, dex_prog_acc.key, close_amount * price, true)?;
    let fee_rate:U64F64 = U64F64!(0.0022); // fee_bps = 22; BASE
    let exact_fee: u64 = U64F64::to_num(fee_rate.checked_mul(U64F64::from_num(pc_qty)).unwrap());
    let pc_qty_including_fees = pc_qty + exact_fee;

    msg!("pc_qty:: {:?} ", pc_qty_including_fees);

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
    // let margin_data = MarginAccount::load(margin_account_acc)?;
    // let mango_group_data = MangoGroup::load(mango_group_acc)?;

    // let prices = get_prices(&mango_group_data, oracle_accs)?;
    // let coll_ratio = margin_data.get_collateral_ratio(&mango_group_data, &prices, open_orders_accs)?;

    // // close leverage
    // check!(coll_ratio >= U64F64!(1.20), ProgramError::InsufficientFunds);


    fund_data.no_of_margin_positions -= 1;
    fund_data.mango_positions[0].state = 4; // change to close_position
    // fund_data.mango_positions[0].investor_debt = U64F64::to_num(coll_ratio.
    //     checked_mul(U64F64::from_num(fund_data.mango_positions[0].investor_debt)).unwrap()
    // ); // update investor_debts
    //fund_data.mango_positions[0].close_collateral = coll_ratio; // save coll_ratio of close

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
    let mut fund_data = FundData::load_mut(fund_state_acc)?;
    check!(fund_data.mango_positions[0].state == 2 || fund_data.mango_positions[0].state == 4, FundError::InvalidMangoState);
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
    let token_index;
    let settle_amount;

    if fund_data.mango_positions[0].position_side == 0 { // for LONG settle USDC
        token_index = NUM_MARKETS;
        settle_amount = get_settle_amount(margin_account_acc, mango_group_acc, NUM_MARKETS)?;
    }
    else { // for SHORT settle token
        token_index = fund_data.mango_positions[0].margin_index as usize;
        settle_amount = get_settle_amount(margin_account_acc, mango_group_acc, token_index)?;
    }
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

    fund_data.mango_positions[0].state += 1; // increement state
    let margin_data = MarginAccount::load(margin_account_acc)?;
    // sanity check
    check!(*open_orders_acc.key == margin_data.open_orders[fund_data.mango_positions[0].margin_index as usize],
        ProgramError::InvalidAccountData);
    Ok(())
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

    check!(manager_acc.is_signer, ProgramError::MissingRequiredSignature);

    let mut fund_data = FundData::load_mut(fund_state_acc)?;

    // position has to be settled_close
    check!(fund_data.mango_positions[0].state == 5, FundError::InvalidMangoState);

    let quantity = get_withdraw_amount(margin_account_acc, mango_group_acc,open_orders_accs, oracle_accs)?;
 
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

    let token_account = parse_token_account(token_account_acc)?;

    msg!("quantity to withdraw:: {:?}", quantity);

    check!(token_account.mint == fund_data.tokens[0].mint, FundError::InvalidTokenAccount);
    check!(*token_account_acc.key == fund_data.tokens[0].vault, FundError::InvalidTokenAccount);
    check!(fund_data.mango_positions[0].investor_debt < quantity, FundError::InvalidAmount);

    // update investor state account debts
    let investor_debts = update_investor_debts(investor_accs, fund_data.mango_positions[0].close_collateral,
        fund_data.mango_positions[0].position_id)?;

    // all investors should be passed
    check!(investor_debts == fund_data.mango_positions[0].investor_debt, ProgramError::InvalidAccountData);

    fund_data.tokens[0].balance = token_account.amount;
    // add margin_debt to USDC debt
    fund_data.tokens[0].debt = fund_data.mango_positions[0].investor_debt;
    fund_data.mango_positions[0].investor_debt = 0;
    fund_data.mango_positions[0].position_side = 0;
    fund_data.mango_positions[0].position_id = 0;
    fund_data.mango_positions[0].margin_index = 0;
    fund_data.mango_positions[0].trade_amount = 0;
    fund_data.mango_positions[0].close_collateral = U64F64!(0);

    fund_data.mango_positions[0].state = 0; // set state to inactive for new positions
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

    let mut fund_data = FundData::load_mut(fund_state_acc)?;
    let mut investor_data = InvestorData::load_mut(investor_state_acc)?;

    check!(investor_data.owner == *investor_acc.key, ProgramError::MissingRequiredSignature);
    check!(investor_acc.is_signer, ProgramError::MissingRequiredSignature);
    check!(investor_data.margin_position_id != 0, ProgramError::InvalidAccountData);
    check!(investor_data.margin_debt > 0, ProgramError::InvalidAccountData);
    check!(!investor_data.withdrawn_from_margin, ProgramError::InvalidAccountData);
    check!(investor_data.margin_position_id == fund_data.mango_positions[0].position_id as u64, ProgramError::InvalidAccountData);
    check!(fund_data.mango_positions[0].state != 0, ProgramError::InvalidAccountData);

    let order = fill_order_investor_withdraw(&mut fund_data, &mut investor_data, margin_account_acc,
        mango_group_acc, spot_market_acc, dex_prog_acc, open_orders_accs, oracle_accs, price)?;

    // set investor variable, no double orders
    investor_data.withdrawn_from_margin = true;

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

pub fn fill_order_investor_withdraw (
    fund_data: &mut FundData,
    investor_data: &mut InvestorData,
    margin_acc: &AccountInfo,
    mango_group_acc: &AccountInfo,
    spot_market_acc: &AccountInfo,
    dex_prog_acc: &AccountInfo,
    open_orders_accs: &[AccountInfo; 4],
    oracle_accs: &[AccountInfo; 4],
    price: u64
) -> Result<serum_dex::instruction::NewOrderInstructionV3, ProgramError> {
    
    let margin_data = MarginAccount::load(&margin_acc)?;
    let mango_group = MangoGroup::load(&mango_group_acc)?;

    let prices = get_prices(&mango_group, oracle_accs)?;
    let coll_ratio = margin_data.get_collateral_ratio(&mango_group, &prices, open_orders_accs)?;
    
    let market = MarketState::load(spot_market_acc, dex_prog_acc.key)?;

    let pc_qty: u64 = U64F64::to_num(coll_ratio.
        checked_mul(U64F64::from_num(investor_data.margin_debt)).unwrap()
    );
    let fee_rate = U64F64::from_num(((22 as u128) << 64) / 10_000);
    let exact_fee: u64 = U64F64::to_num(fee_rate.checked_mul(U64F64::from_num(pc_qty)).unwrap());
    let pc_qty_including_fees = pc_qty + exact_fee;
    let pc_lots = convert_size_to_lots(spot_market_acc, dex_prog_acc.key, pc_qty_including_fees, true)?;
    msg!("pc_lots:: {:?}", pc_lots);

    let coin_lots = pc_lots / market.coin_lot_size / price;
    msg!("coin_lots:: {:?}", coin_lots);

    
    let order = NewOrderInstructionV3 {
        side: match fund_data.mango_positions[0].position_side { // reverse of open position
            0 => Side::Ask,
            1 => Side::Bid,
            _ => Side::Ask
        },
        limit_price: NonZeroU64::new(price).unwrap(),
        max_coin_qty: NonZeroU64::new(coin_lots).unwrap(),
        max_native_pc_qty_including_fees: NonZeroU64::new(pc_lots).unwrap(),
        order_type: OrderType::ImmediateOrCancel,
        client_order_id: 1,
        self_trade_behavior: SelfTradeBehavior::AbortTransaction,
        limit: 65535,
    };
    Ok(order)
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

    let mut fund_data = FundData::load_mut(fund_state_acc)?;
    let mut investor_data = InvestorData::load_mut(investor_state_acc)?;

    check!(investor_acc.is_signer, ProgramError::MissingRequiredSignature);
    check!(investor_data.owner == *investor_acc.key, ProgramError::MissingRequiredSignature);
    check!(investor_data.margin_debt > 0, ProgramError::InvalidAccountData);
    check!(investor_data.margin_position_id == fund_data.mango_positions[0].position_id as u64, ProgramError::InvalidAccountData);
    check!(investor_data.withdrawn_from_margin, ProgramError::InvalidAccountData);
    check!(fund_data.mango_positions[0].state != 0, ProgramError::InvalidAccountData);

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
        margin_account_acc, mango_group_acc, oracle_acc, open_orders_acc)?;
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

    let mut fund_data = FundData::load_mut(fund_state_acc)?;
    let mut investor_data = InvestorData::load_mut(investor_state_acc)?;

    check!(investor_acc.is_signer, ProgramError::MissingRequiredSignature);
    check!(investor_data.owner == *investor_acc.key, ProgramError::MissingRequiredSignature);
    check!(investor_data.margin_debt > 0, ProgramError::InvalidAccountData);
    check!(investor_data.margin_position_id == fund_data.mango_positions[0].position_id as u64, ProgramError::InvalidAccountData);
    check!(investor_data.withdrawn_from_margin, ProgramError::InvalidAccountData);

    // position has to be settled_close
    let quantity = get_investor_withdraw_amount(&investor_data,
        margin_account_acc, mango_group_acc, oracle_accs, open_orders_accs)?;

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

    fund_data.mango_positions[0].investor_debt -= investor_data.margin_debt; // subtract from fund debt
    investor_data.margin_debt = 0; // zero out investor_debt
    investor_data.margin_position_id = 0; // not in investment queue

    Ok(())
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

    // let withdraw_amount = mango_group.indexes[NUM_MARKETS].deposit
    // .checked_mul(margin_data.deposits[NUM_MARKETS]).unwrap();
    let withdraw_amount = equity.checked_div(U64F64!(1.25)).unwrap();

    let withdraw_amt = mango_group.indexes[0].deposit
    .checked_mul(margin_data.deposits[0]).unwrap();
    msg!("deposits BTC : {:?}", margin_data.deposits[0]);
    msg!("borrows BTC : {:?}", margin_data.borrows[0]);

    msg!("withdraw_amount btc {:?}", withdraw_amt);

    msg!("deposits USDC : {:?}", margin_data.deposits[NUM_MARKETS]);
    msg!("borrows USDC {:?}", margin_data.borrows[NUM_MARKETS]);

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
    msg!("close_amount:: {:?}", close_amount);
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
    open_orders_acc: &AccountInfo
) -> Result<(u64, usize), ProgramError> {

    let margin_data = MarginAccount::load(&margin_acc)?;
    let mango_group = MangoGroup::load(&mango_group_acc)?;

    let token_index = fund_data.mango_positions[0].margin_index as usize;
    let (_equity, coll) = get_equity_and_coll_ratio(token_index, &mango_group, &margin_data, oracle_acc, open_orders_acc)?;

    let inv_debt = coll.checked_mul(U64F64::from_num(investor_data.margin_debt)).unwrap();
    if fund_data.mango_positions[0].position_side == 0 { // for LONG settle USDC
        let deposit_amount = mango_group.indexes[NUM_MARKETS].deposit
        .checked_mul(margin_data.deposits[NUM_MARKETS]).unwrap();
        let settle_amount = deposit_amount.checked_sub(inv_debt).unwrap();
        Ok((U64F64::to_num(settle_amount), NUM_MARKETS))
    }
    else { // for SHORT settle full borrows of that token
        let token_index = fund_data.mango_positions[0].margin_index as usize;
        let deposit_amount = mango_group.indexes[NUM_MARKETS].deposit
        .checked_mul(margin_data.deposits[NUM_MARKETS]).unwrap();
        Ok((U64F64::to_num(deposit_amount), token_index))
    }
}

pub fn get_investor_withdraw_amount (
    investor_data: &InvestorData,
    margin_acc: &AccountInfo,
    mango_group_acc: &AccountInfo,
    open_orders_accs: &[AccountInfo; 4],
    oracle_accs: &[AccountInfo; 4],
) -> Result<u64, ProgramError> {

    let margin_data = MarginAccount::load(&margin_acc)?;
    let mango_group = MangoGroup::load(&mango_group_acc)?;

    let prices = get_prices(&mango_group, oracle_accs)?;
    let coll_ratio = margin_data.get_collateral_ratio(&mango_group, &prices, open_orders_accs)?;

    let withdraw_amount = coll_ratio.checked_mul(U64F64::from_num(investor_data.margin_debt)).unwrap();
    let deposit_amount = mango_group.indexes[NUM_MARKETS].deposit
    .checked_mul(margin_data.deposits[NUM_MARKETS]).unwrap();

    // sanity check
    check!(withdraw_amount <= deposit_amount, ProgramError::InsufficientFunds);
    Ok(U64F64::to_num(withdraw_amount))
}

pub fn convert_size_to_lots(
    spot_market_acc: &AccountInfo,
    dex_program_id: &Pubkey,
    size: u64,
    pc: bool
) -> Result <u64, ProgramError> {
    let market = MarketState::load(spot_market_acc, dex_program_id)?;
    if pc {
        Ok(size / market.pc_lot_size)
    }
    else {
        Ok(size / market.coin_lot_size)
    }
}

pub fn update_investor_debts(
    investor_accs: &[AccountInfo],
    close_collateral: U64F64,
    position_id: u16
) -> Result<u64, ProgramError> {
    
    let mut debts = 0;
    for i in 0..MAX_INVESTORS_WITHDRAW {
        if *investor_accs[i].key == Pubkey::default() {
            continue;
        }
        let mut investor_data = InvestorData::load_mut(&investor_accs[i])?;
        if investor_data.margin_position_id == position_id as u64 {
            // update 
            investor_data.margin_debt = U64F64::to_num(close_collateral.
                checked_mul(U64F64::from_num(investor_data.margin_debt)).unwrap()
            );
            investor_data.margin_position_id = 0; // remove position id
            debts += investor_data.margin_debt;
        }
    }
    Ok(debts)
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
