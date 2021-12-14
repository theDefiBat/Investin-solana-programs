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
    rent::Rent,
};
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::cell::RefMut;

use fixed_macro::types::U64F64;
pub const ONE_U64F64: U64F64 = U64F64!(1);
pub const ZERO_U64F64: U64F64 = U64F64!(0);

use std::num::NonZeroU64;


use arrayref::{array_ref, array_refs};

use crate::error::FundError;
use crate::state::{MAX_INVESTORS_WITHDRAW, NUM_MARGIN, FundData, InvestorData};
use crate::state::Loadable;
use crate::processor::{ parse_token_account };

use mango::state::{MangoAccount, MangoGroup, MangoCache, MAX_PAIRS, QUOTE_INDEX};
// use mango::state::Loadable as OtherLoadable;
use mango::instruction::{deposit, withdraw, place_perp_order, settle_pnl, MangoInstruction};
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

pub fn init_mango_account(
    program_id: &Pubkey,
    mango_group_pk: &Pubkey,
    mango_account_pk: &Pubkey,
    owner_pk: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let accounts = vec![
        AccountMeta::new_readonly(*mango_group_pk, false),
        AccountMeta::new(*mango_account_pk, false),
        AccountMeta::new_readonly(*owner_pk, true),
    ];

    let instr = MangoInstruction::InitMangoAccount;
    let data = instr.pack();
    Ok(Instruction { program_id: *program_id, accounts, data })
}

pub mod mango_v3_id {
    use solana_program::declare_id;
    // #[cfg(feature = "devnet")]
    // declare_id!("4skJ85cdxQAFVKbcGgfun8iZPL7BadVYXG3kGEGkufqA");
    // #[cfg(not(feature = "devnet"))]
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
    
    msg!("Loaded DATA");
    check_eq!(fund_data.tokens[token_slot_index as usize].vault, *owner_token_account_ai.key); 
    check_eq!(*mango_prog_ai.key, mango_v3_id::ID);
    check!(fund_data.mango_positions.mango_account != Pubkey::default(), FundError::MangoNotInitialized);
    // check_eq!(mango_group.tokens[mango_token_index].root_bank, )   
    if(mango_token_index as usize != QUOTE_INDEX){
        check!(fund_data.mango_positions.deposit_index == mango_token_index || 
            fund_data.mango_positions.deposit_index == u8::MAX, FundError::InvalidMangoState);
        fund_data.mango_positions.deposit_index = mango_token_index;
        fund_data.tokens[token_slot_index as usize].is_on_mango = 1;
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

    let mut fund_data = FundData::load_mut_checked(fund_state_ai, program_id)?;
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
            side, i64::MAX, quantity, 0, OrderType::Market, false)?,
        &[
            mango_prog_ai.clone(),
            mango_group_ai.clone(),
            mango_account_ai.clone(),
            fund_pda_ai.clone(),
            mango_cache_ai.clone(),
            perp_market_ai.clone(),
            bids_ai.clone(),
            asks_ai.clone(),
            event_queue_ai.clone(),
            default_ai.clone(),
            
        ],
        &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
    )?;

    let mango_group_data = MangoGroup::load_checked(mango_group_ai, mango_prog_ai.key)?;
    check_eq!(mango_group_data.perp_markets[perp_market_id as usize].perp_market, *perp_market_ai.key);
    

    let fund_perp_makret_index = fund_data.get_mango_perp_index(perp_market_id);
    if(fund_perp_makret_index == None){
        let new_fund_perp_makret_index = fund_data.get_mango_perp_index(u8::MAX).unwrap();
        fund_data.mango_positions.perp_markets[new_fund_perp_makret_index] = perp_market_id;
    }
    else{
        let mango_account_data = MangoAccount::load_checked(mango_account_ai, mango_prog_ai.key, mango_group_ai.key)?;
        let base_pos = mango_account_data.perp_accounts[perp_market_id as usize].base_position;
        if((side == Side::Bid && quantity.checked_add(base_pos).unwrap() == 0) || (side == Side::Ask && quantity == base_pos)){
            msg!("Clearing Perp market on Fund");
            fund_data.mango_positions.perp_markets[fund_perp_makret_index.unwrap() as usize] = u8::MAX;
        }
    }
    //Settle PnL to be executed right after place_perp_order...

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
    check!(fund_data.mango_positions.mango_account != Pubkey::default(), FundError::MangoNotInitialized);
    //Check for Mango v3 ID 
    check_eq!(*mango_prog_ai.key, mango_v3_id::ID);

    check_eq!(fund_data.tokens[token_slot_index as usize].vault, *fund_token_ai.key);

    
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
    fund_data.tokens[token_slot_index as usize].balance = dest_info.amount;
    // let mango_group = MangoGroup::load_checked(mango_group_ai, mango_prog_ai.key)?;

    let mango_account = MangoAccount::load_checked(mango_account_ai, mango_prog_ai.key, mango_group_ai.key)?;
    
    let deposits_after = mango_account.deposits[mango_token_index as usize];
    if mango_token_index as usize != QUOTE_INDEX {
        check!(deposits_after >= fund_data.mango_positions.investor_debts[1] , FundError::InvalidAmount);
        if deposits_after == 0 {
            fund_data.mango_positions.deposit_index = u8::MAX;
            fund_data.tokens[token_slot_index as usize].is_on_mango = 0;
        }
    } else {
        check!(deposits_after >= fund_data.mango_positions.investor_debts[0] , FundError::InvalidAmount);
    }
    
    // fund_data.tokens[0].balance = parse_token_account(fund_token_ai)?.amount;

    Ok(())
}

pub fn mango_withdraw_investor(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> Result<(), ProgramError>
{
    const NUM_FIXED: usize = 19;
    let accounts = array_ref![accounts, 0, NUM_FIXED];
    let [
        fund_state_ai,      //write
        investor_state_ai,  //write
        investor_ai,        //signer
        mango_prog_ai,      //
        mango_group_ai,     // read
        mango_account_ai,   // write
        fund_pda_ai,           // read
        mango_cache_ai,     // read
        usdc_root_bank_ai,       // read
        usdc_node_bank_ai,       // write
        usdc_vault_ai,           // write
        usdc_investor_token_ai,   // write
        token_root_bank_ai,       // read
        token_node_bank_ai,       // write
        token_vault_ai,           // write
        token_investor_token_ai,   // write
        signer_ai,          // read
        token_prog_ai,      // read
        default_ai
    ] = accounts;

    let mut fund_data = FundData::load_mut_checked(fund_state_ai, program_id)?;
    let mut investor_data = InvestorData::load_mut_checked(investor_state_ai, program_id)?;

    check!(investor_data.owner == *investor_ai.key, ProgramError::MissingRequiredSignature);
    check!(investor_ai.is_signer, ProgramError::MissingRequiredSignature);
    check_eq!(investor_data.manager, fund_data.manager_account);
    check!(investor_data.has_withdrawn == true && investor_data.withdrawn_from_margin == false, FundError::InvalidStateAccount);
    check_eq!(*mango_prog_ai.key, mango_v3_id::ID);

    let open_orders_accs = [Pubkey::default(); MAX_PAIRS];
    let usdc_quantity:u64 =  U64F64::to_num(investor_data.margin_debt[0]);
    invoke_signed(
        &withdraw(mango_prog_ai.key, mango_group_ai.key, mango_account_ai.key, fund_pda_ai.key,
            mango_cache_ai.key, usdc_root_bank_ai.key, usdc_node_bank_ai.key, usdc_vault_ai.key, usdc_investor_token_ai.key,
            signer_ai.key, &open_orders_accs, usdc_quantity, false)?,
        &[
            mango_prog_ai.clone(),
            mango_group_ai.clone(),
            mango_account_ai.clone(),
            fund_pda_ai.clone(),
            mango_cache_ai.clone(),
            usdc_root_bank_ai.clone(),
            usdc_node_bank_ai.clone(),
            usdc_vault_ai.clone(),
            usdc_investor_token_ai.clone(),
            signer_ai.clone(),
            default_ai.clone(),
            token_prog_ai.clone()
        ],
        &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
    )?;

    let token_quantity:u64 =  U64F64::to_num(investor_data.margin_debt[1]);
    if token_quantity > 0 {
        invoke_signed(
            &withdraw(mango_prog_ai.key, mango_group_ai.key, mango_account_ai.key, fund_pda_ai.key,
                mango_cache_ai.key, token_root_bank_ai.key, token_node_bank_ai.key, token_vault_ai.key, token_investor_token_ai.key,
                signer_ai.key, &open_orders_accs, usdc_quantity, false)?,
            &[
                mango_prog_ai.clone(),
                mango_group_ai.clone(),
                mango_account_ai.clone(),
                fund_pda_ai.clone(),
                mango_cache_ai.clone(),
                token_root_bank_ai.clone(),
                token_node_bank_ai.clone(),
                token_vault_ai.clone(),
                token_investor_token_ai.clone(),
                signer_ai.clone(),
                default_ai.clone(),
                token_prog_ai.clone()
            ],
            &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
        )?;
    }
    
    msg!("invoke done");
    fund_data.mango_positions.investor_debts[0] = fund_data.mango_positions.investor_debts[0].checked_sub(U64F64::to_num(investor_data.margin_debt[0])).unwrap();
    fund_data.mango_positions.investor_debts[1] = fund_data.mango_positions.investor_debts[1].checked_sub(U64F64::to_num(investor_data.margin_debt[1])).unwrap();
    investor_data.margin_debt = [ZERO_U64F64; 2];
    investor_data.withdrawn_from_margin = true;
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




