use bytemuck::bytes_of;
use fixed::types::I80F48;
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

use std::num::NonZeroU64;


use arrayref::{array_ref, array_refs};

use crate::error::FundError;
use crate::state::{MAX_INVESTORS_WITHDRAW, NUM_MARGIN, FundData, InvestorData};
use crate::state::Loadable;
use crate::processor::{ parse_token_account };

use mango::state::{MangoAccount, MangoGroup, MangoCache, MAX_PAIRS, QUOTE_INDEX};
// use mango::state::Loadable as OtherLoadable;
use mango::instruction::{init_mango_account, deposit, withdraw, place_perp_order, settle_pnl, MangoInstruction};
use mango::matching::{Side, OrderType};
use spl_token::state::Account;

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

pub mod mango_v3_id {
    use solana_program::declare_id;
    #[cfg(feature = "devnet")]
    declare_id!("4skJ85cdxQAFVKbcGgfun8iZPL7BadVYXG3kGEGkufqA");
    #[cfg(not(feature = "devnet"))]
    declare_id!("mv3ekLzLbnVPNxjSKvqBpU3ZeZXPQdEC3bp5MDEBG68");
}

pub fn mango_init_mango_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> Result<(), ProgramError> {
    const NUM_FIXED: usize = 6;
    let accounts = array_ref![accounts, 0, NUM_FIXED];

    let [
        fund_state_ai,
        manager_ai,
        fund_pda_ai,
        mango_prog_ai,
        mango_group_ai,
        mango_account_ai,
    ] = accounts;

    let mut fund_data = FundData::load_mut_checked(fund_state_ai, program_id)?;
    //Check for Mango v3 ID 
    check_eq!(*mango_prog_ai.key, mango_v3_id::ID);
    check!(manager_ai.is_signer, ProgramError::MissingRequiredSignature);
    check_eq!(fund_data.manager_account, *manager_ai.key);
    check_eq!(fund_data.mango_positions.mango_account, Pubkey::default());
    invoke_signed(
        &init_mango_account(mango_prog_ai.key, mango_group_ai.key, mango_account_ai.key, fund_pda_ai.key)?,
        &[
            mango_prog_ai.clone(),
            mango_group_ai.clone(),
            mango_account_ai.clone(),
            fund_pda_ai.clone()
        ],
        &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
        )?;
    fund_data.mango_positions.mango_account = *mango_account_ai.key;
    
    Ok(())
}

pub fn mango_deposit(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    token_slot_index: u8,
    mango_token_index: u8,
    quantity: u64
) -> Result<(), ProgramError> {
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

    let mut fund_data = FundData::load_mut_checked(fund_state_ai, program_id)?;
    let mango_group = MangoGroup::load_checked(mango_group_ai, mango_prog_ai.key)?;
    let mango_account = MangoAccount::load_checked(mango_account_ai, mango_prog_ai.key, mango_group_ai.key)?;
    let mango_cache = MangoCache::load_checked(mango_cache_ai, mango_prog_ai.key, &mango_group)?;

    check_eq!(fund_data.tokens[token_slot_index as usize].vault, *owner_token_account_ai.key); 
    check_eq!(*mango_prog_ai.key, mango_v3_id::ID);
    // check_eq!(mango_group.tokens[mango_token_index].root_bank, )
    if(mango_token_index as usize != QUOTE_INDEX){
        check!(fund_data.mango_positions.deposit_index == mango_token_index || 
            fund_data.mango_positions.deposit_index == QUOTE_INDEX as u8, FundError::InvalidMangoState);
        fund_data.mango_positions.deposit_index = mango_token_index;
    }
    check!(fund_data.is_initialized, ProgramError::InvalidAccountData);
    check!(manager_ai.is_signer, ProgramError::MissingRequiredSignature);
    
    // check_eq!(fund_data.manager_account, *manager_ai.key);
    check!((fund_data.manager_account == *manager_ai.key), FundError::ManagerMismatch);

    // // check fund vault
    // check_eq!(fund_data.vault_key, *owner_token_account_ai.key); 
    
    invoke_signed(
        &deposit(mango_prog_ai.key, mango_group_ai.key, mango_account_ai.key, fund_pda_ai.key,
            mango_cache_ai.key, root_bank_ai.key, node_bank_ai.key, vault_ai.key, owner_token_account_ai.key, quantity)?,
        &[
            mango_prog_ai.clone(),
            mango_group_ai.clone(),
            mango_account_ai.clone(),
            fund_pda_ai.clone(),
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

    let token_info = parse_token_account(owner_token_account_ai)?;

    fund_data.tokens[token_slot_index as usize].balance = token_info.amount;
    check!(fund_data.tokens[token_slot_index as usize].balance >= fund_data.tokens[token_slot_index as usize].debt, ProgramError::InsufficientFunds);

    Ok(())
}

pub fn mango_place_perp_order(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    perp_market_id: u8,
    side: Side,
    quantity: i64
) -> Result<(), ProgramError> {
    const NUM_FIXED: usize = 12;
    let accounts = array_ref![accounts, 0, NUM_FIXED];

    let [
        fund_state_ai,
        manager_ai,
        mango_prog_ai,
        mango_group_ai,     // read
        mango_account_ai,   // write
        fund_pda_ai,           // read, signer
        mango_cache_ai,     // read
        perp_market_ai,     // write
        bids_ai,            // write
        asks_ai,            // write
        event_queue_ai,    // write
        default_ai,
    ] = accounts;

    let fund_data = FundData::load_mut_checked(fund_state_ai, program_id)?;
    //Check for perp_market_id matches derived from ai 
    //Check for perp_market_id already active on fund/add perp_market if markets_active < 4
    //Check if its close on full amount, if yes remove from active perp_markets on funds --> END
    //Base_position + taker_base and quote_position and taker_quote should both be considered
    //Settle PnL to be executed right after place_perp_order...

    //Check for Mango v3 ID 
    check_eq!(*mango_prog_ai.key, mango_v3_id::ID);
    check!(manager_ai.is_signer, ProgramError::MissingRequiredSignature);

    // check_eq!(fund_data.manager_account, *manager_ai.key);
    check!((fund_data.manager_account == *manager_ai.key), FundError::ManagerMismatch);
    
    let open_orders_accs = [Pubkey::default(); MAX_PAIRS];
    invoke_signed(
        &place_perp_order(mango_prog_ai.key,
            mango_group_ai.key, mango_account_ai.key, fund_pda_ai.key,
            mango_cache_ai.key,perp_market_ai.key, bids_ai.key, asks_ai.key, event_queue_ai.key, &open_orders_accs,
            side, 0, quantity, 0, OrderType::Market, true)?,
        &[
            mango_prog_ai.clone(),
            mango_group_ai.clone(),
            mango_account_ai.clone(),
            fund_pda_ai.clone(),
            perp_market_ai.clone(),
            mango_cache_ai.clone(),
            bids_ai.clone(),
            asks_ai.clone(),
            default_ai.clone(),
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
    perp_market_id: u8
) -> Result<(), ProgramError>
{
    const NUM_FIXED: usize = 10;
    let accounts = array_ref![accounts, 0, NUM_FIXED];

    let [
        fund_state_ai,
        manager_ai,
        fund_pda_ai,
        mango_prog_ai,
        mango_group_ai,
        mango_account_a_ai,
        mango_account_b_ai,
        mango_cache_ai,
        root_bank_ai,
        node_bank_ai,
    ] = accounts;
    let mut fund_data = FundData::load_mut_checked(fund_state_ai, program_id)?;
    // check!(manager_ai.is_signer, ProgramError::MissingRequiredSignature);
    // check!((fund_data.manager_account == *manager_ai.key), FundError::ManagerMismatch);
    invoke_signed(
        &settle_pnl(mango_prog_ai.key, mango_group_ai.key, mango_account_a_ai.key, mango_account_a_ai.key, 
            mango_cache_ai.key, root_bank_ai.key, node_bank_ai.key, perp_market_id as usize)?,
        &[
            mango_prog_ai.clone(),
            mango_group_ai.clone(),
            mango_account_a_ai.clone(),
            mango_account_b_ai.clone(),
            mango_cache_ai.clone(),
            root_bank_ai.clone(),
            node_bank_ai.clone()
        ],
        &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
    )?;
    Ok(())
}


pub fn mango_withdraw(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    token_slot_index: u8,
    mango_token_index: u8,
    quantity: u64
) -> Result<(), ProgramError>
{
    const NUM_FIXED: usize = 14;
    let accounts = array_ref![accounts, 0, NUM_FIXED];
    let [
        fund_state_ai,
        manager_ai,
        mango_prog_ai,

        mango_group_ai,     // read
        mango_account_ai,   // write
        fund_pda_ai,           // read
        mango_cache_ai,     // read
        root_bank_ai,       // read
        node_bank_ai,       // write
        vault_ai,           // write
        fund_token_ai,   // write
        signer_ai,          // read
        token_prog_ai,      // read
        default_ai
    ] = accounts;

    let mut fund_data = FundData::load_mut_checked(fund_state_ai, program_id)?;

    check!(fund_data.is_initialized, ProgramError::InvalidAccountData);
    check!(manager_ai.is_signer, ProgramError::MissingRequiredSignature);
    
    //Check for Mango v3 ID 
    check_eq!(*mango_prog_ai.key, mango_v3_id::ID);

    let mango_group = MangoGroup::load_checked(mango_group_ai, mango_prog_ai.key)?;

    // check_eq!(mango_group.tokens[mango_token_index].root_bank, )
    check!((fund_data.manager_account == *manager_ai.key), FundError::ManagerMismatch);
    
    // withdraw USDC from mango account
    let open_orders_accs = [Pubkey::default(); MAX_PAIRS];
    invoke_signed(
        &withdraw(mango_prog_ai.key, mango_group_ai.key, mango_account_ai.key, fund_pda_ai.key,
            mango_cache_ai.key, root_bank_ai.key, node_bank_ai.key, vault_ai.key, fund_token_ai.key,
            signer_ai.key, &open_orders_accs, quantity, false)?,
        &[
            mango_prog_ai.clone(),
            mango_group_ai.clone(),
            mango_account_ai.clone(),
            fund_pda_ai.clone(),
            mango_cache_ai.clone(),
            root_bank_ai.clone(),
            node_bank_ai.clone(),
            vault_ai.clone(),
            fund_token_ai.clone(),
            signer_ai.clone(),
            default_ai.clone(),
            token_prog_ai.clone()
        ],
        &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
    )?;

    msg!("invoke done");

    let dest_info = parse_token_account(fund_token_ai)?;
    check_eq!(dest_info.owner, fund_data.fund_pda);
    let mango_account = MangoAccount::load_checked(mango_account_ai, mango_prog_ai.key, mango_group_ai.key)?;
    if(mango_account.deposits[mango_token_index as usize] == 0){
        fund_data.mango_positions.deposit_index = QUOTE_INDEX as u8;
    }
    // fund_data.tokens[0].balance = parse_token_account(fund_token_ai)?.amount;

    Ok(())
}



// pub fn convert_size_to_lots(
//     spot_market_acc: &AccountInfo,
//     dex_program_id: &Pubkey,
//     size: u64,
//     pc: bool
// ) -> Result <u64, ProgramError> {
//     let market = MarketState::load(spot_market_ai, dex_program_id)?;
//     if pc {
//         Ok(size * market.pc_lot_size / market.coin_lot_size)
//     }
//     else {
//         Ok(size / market.coin_lot_size)
//     }
// }

// fn get_withdraw_lots(
//     spot_market_acc: &AccountInfo,
//     dex_program_id: &Pubkey,
//     size: u64,
//     side: u8,
// ) -> Result <u64, ProgramError> {
//     let market = MarketState::load(spot_market_ai, dex_program_id)?;
//     Ok((size / market.coin_lot_size) + side as u64)
// }

// pub fn get_investor_withdraw_lots(
//     spot_market_acc: &AccountInfo,
//     dex_program_id: &Pubkey,
//     size: u64,
//     pos_size: u64,
//     side: u8
// ) -> Result <u64, ProgramError> {
//     let market = MarketState::load(spot_market_ai, dex_program_id)?;
//     //if size + market.coin_lot_size > pos_size {
//     if (pos_size - size) / market.coin_lot_size == 0 {
//         Ok((size / market.coin_lot_size) + side as u64) // same as manager close case
//     }
//     else {
//         Ok((size / market.coin_lot_size) + 1)
//     }
//     // Ok((size / market.coin_lot_size) + side as u64)
// }

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




