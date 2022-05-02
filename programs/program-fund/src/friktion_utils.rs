use bytemuck::{bytes_of};
use std::{mem::size_of};
use fixed::{types::U64F64};
use fixed_macro::types::U64F64;
use fixed::types::I80F48;
use fixed_macro::types::I80F48;
use fixed::traits::FromFixed;
use num_enum::TryFromPrimitive;
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    msg,
    log::sol_log_compute_units,    
    system_instruction::create_account,
    instruction:: {AccountMeta, Instruction},
    program_error::ProgramError,
    program_pack::{Pack, IsInitialized},
    pubkey::Pubkey,
    program::{invoke, invoke_signed},
    sysvar::{clock::Clock, rent::Rent, Sysvar}
};
use bincode::serialize;

// use anchor_lang::prelude::*;
use anchor_lang::{prelude::CpiContext, AnchorDeserialize, accounts::{sysvar, account}};
use arrayref::{array_ref, array_refs};
use spl_token::state::{Account, Mint};
use mango::state::{MangoAccount, MangoGroup, MangoCache, PerpMarket, MAX_TOKENS, MAX_PAIRS, QUOTE_INDEX};
use mango::instruction::{ cancel_all_perp_orders, withdraw, place_perp_order, consume_events };
use mango::matching::{Side, OrderType, Book};
use volt_abi::*;

use crate::error::FundError;
use crate::instruction::{FundInstruction, Data};
use crate::state::{FundAccount, InvestorData, PlatformData};
use crate::processor::{parse_token_account};

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

pub fn read_friktion_data(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> Result<(), ProgramError> {
    let accounts_iter = &mut accounts.iter();
    let friktion_epoch_info_ai = next_account_info(accounts_iter)?;
    msg!("Getting Data");
    let raw_data: &[u8] = &(friktion_epoch_info_ai.data.borrow())[8..];
    // raw_data = &raw_data[8..];
    let epoch_info = volt_abi::FriktionEpochInfo::try_from_slice(raw_data)?;
    msg!("Price: {}, pct_pnl: {}, number_info: {:?}, pnl: {}", epoch_info.vault_token_price, epoch_info.pct_pnl, epoch_info.number, epoch_info.pnl);
    let pending_deposit_info_ai = next_account_info(accounts_iter)?;
    if pending_deposit_info_ai.data_len() > 0 {
        let pending_info_data: &[u8] = &(pending_deposit_info_ai.data.borrow())[8..];
        let pending_deposit_info = volt_abi::PendingDeposit::try_from_slice(pending_info_data)?;
        msg!("Underlying Deposited: {}, Round_Num {}", pending_deposit_info.num_underlying_deposited, pending_deposit_info.round_number);
    }
    let pending_withdrawal_info_ai = next_account_info(accounts_iter)?;
    if pending_withdrawal_info_ai.data_len() > 0 {
        let pending_withdrawal_data: &[u8] = &(pending_withdrawal_info_ai.data.borrow())[8..];
        let pending_withdrawal_info = volt_abi::PendingWithdrawal::try_from_slice(pending_withdrawal_data)?;
        msg!("Underlying withdrawaled: {}, Round_Num {}", pending_withdrawal_info.num_volt_redeemed, pending_withdrawal_info.round_number);
    }
    Ok(())
}

pub fn friktion_deposit_ins(
    program_id: &Pubkey,
    authority_pk: &Pubkey,
    dao_authority_pk: &Pubkey,
    authority_check_pk: &Pubkey,
    vault_mint_pk: &Pubkey,
    volt_vault_pk: &Pubkey,
    vault_authority_pk: &Pubkey,
    extra_volt_data_pk: &Pubkey,
    whitelist_pk: &Pubkey,
    deposit_pool_pk: &Pubkey,
    writer_token_pool_pk: &Pubkey,
    vault_token_destination_pk: &Pubkey,
    underlying_token_source_pk: &Pubkey,
    round_info_pk: &Pubkey,
    round_volt_tokens_pk: &Pubkey,
    round_underlying_tokens_pk: &Pubkey,
    pending_deposit_info_pk: &Pubkey,
    epoch_info_pk: &Pubkey,
    entropy_program_pk: &Pubkey,
    entropy_group_pk: &Pubkey,
    entropy_account_pk: &Pubkey,
    entropy_cache_pk: &Pubkey,
    system_program_pk: &Pubkey,
    token_program_pk: &Pubkey,
    discrim: u64,
    amount: u64
) -> Result<Instruction, ProgramError> {
    let mut accounts = vec![
        AccountMeta::new(*authority_pk, true),
        AccountMeta::new(*dao_authority_pk, true),
        AccountMeta::new(*authority_check_pk, true),
        AccountMeta::new(*vault_mint_pk, false),
        AccountMeta::new(*volt_vault_pk, false),
        AccountMeta::new_readonly(*vault_authority_pk, false),
        AccountMeta::new_readonly(*extra_volt_data_pk, false),
        AccountMeta::new_readonly(*whitelist_pk, false),
        AccountMeta::new(*deposit_pool_pk, false),
        AccountMeta::new_readonly(*writer_token_pool_pk, false),
        AccountMeta::new(*vault_token_destination_pk, false),
        AccountMeta::new(*underlying_token_source_pk, false),
        AccountMeta::new(*round_info_pk, false),
        AccountMeta::new(*round_volt_tokens_pk, false),
        AccountMeta::new(*round_underlying_tokens_pk, false),
        AccountMeta::new(*pending_deposit_info_pk, false),
        AccountMeta::new(*epoch_info_pk, false),
        AccountMeta::new_readonly(*entropy_program_pk, false),
        AccountMeta::new_readonly(*entropy_group_pk, false),
        AccountMeta::new_readonly(*entropy_account_pk, false),
        AccountMeta::new_readonly(*entropy_cache_pk, false),
        AccountMeta::new_readonly(*system_program_pk, false),
        AccountMeta::new_readonly(*token_program_pk, false),
    ];

    // let instr = FundInstruction::FriktionDepositInstr { discrim, amount };
    let mut cpi_data = Vec::<u8>::new();
    cpi_data.extend_from_slice(&discrim.to_le_bytes());
    cpi_data.extend_from_slice(&amount.to_le_bytes());
    msg!("data for cpi: {:?}", cpi_data);
    Ok(Instruction { program_id: *program_id, accounts, data: cpi_data })
}

pub fn friktion_cancel_pending_deposit_ins(
    program_id: &Pubkey,
    authority_pk: &Pubkey,
    vault_mint_pk: &Pubkey,
    volt_vault_pk: &Pubkey,
    extra_volt_data_pk: &Pubkey,
    vault_authority_pk: &Pubkey,
    underlying_token_destination_pk: &Pubkey,
    round_info_pk: &Pubkey,
    round_underlying_tokens_pk: &Pubkey,
    pending_deposit_info_pk: &Pubkey,
    epoch_info_pk: &Pubkey,
    system_program_pk: &Pubkey,
    token_program_pk: &Pubkey,
    discrim: u64
) -> Result<Instruction, ProgramError> {
    let mut accounts = vec![
        AccountMeta::new(*authority_pk, true),
        AccountMeta::new(*vault_mint_pk, false),
        AccountMeta::new(*volt_vault_pk, false),
        AccountMeta::new_readonly(*extra_volt_data_pk, false),
        AccountMeta::new_readonly(*vault_authority_pk, false),
        AccountMeta::new(*underlying_token_destination_pk, false),
        AccountMeta::new(*round_info_pk, false),
        AccountMeta::new(*round_underlying_tokens_pk, false),
        AccountMeta::new(*pending_deposit_info_pk, false),
        AccountMeta::new(*epoch_info_pk, false),
        AccountMeta::new_readonly(*system_program_pk, false),
        AccountMeta::new_readonly(*token_program_pk, false),
        AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false)
    ];

    // let instr = FundInstruction::FriktionDepositInstr { discrim, amount };
    let mut cpi_data = Vec::<u8>::new();
    cpi_data.extend_from_slice(&discrim.to_le_bytes());
    msg!("data for cpi: {:?}", cpi_data);
    Ok(Instruction { program_id: *program_id, accounts, data: cpi_data })
}

pub fn friktion_withdraw_ins(
    program_id: &Pubkey,
    authority_pk: &Pubkey,
    dao_authority_pk: &Pubkey,
    authority_check_pk: &Pubkey,
    vault_mint_pk: &Pubkey,
    volt_vault_pk: &Pubkey,
    vault_authority_pk: &Pubkey,
    extra_volt_data_pk: &Pubkey,
    whitelist_pk: &Pubkey,
    deposit_pool_pk: &Pubkey,
    vault_token_source_pk: &Pubkey,
    underlying_token_destination_pk: &Pubkey,
    round_info_pk: &Pubkey,
    round_underlying_tokens_pk: &Pubkey,
    pending_withdrawal_info_pk: &Pubkey,
    epoch_info_pk: &Pubkey,
    fee_acct_pk: &Pubkey,
    system_program_pk: &Pubkey,
    token_program_pk: &Pubkey,
    discrim: u64,
    withdraw_amount: u64
) -> Result<Instruction, ProgramError> {
    let mut accounts = vec![
        AccountMeta::new(*authority_pk, true),
        AccountMeta::new(*dao_authority_pk, true),
        AccountMeta::new(*authority_check_pk, true),
        AccountMeta::new(*vault_mint_pk, false),
        AccountMeta::new(*volt_vault_pk, false),
        AccountMeta::new_readonly(*vault_authority_pk, false),
        AccountMeta::new_readonly(*extra_volt_data_pk, false),
        AccountMeta::new_readonly(*whitelist_pk, false),
        AccountMeta::new(*deposit_pool_pk, false),
        AccountMeta::new(*vault_token_source_pk, false),
        AccountMeta::new(*underlying_token_destination_pk, false),
        AccountMeta::new(*round_info_pk, false),
        AccountMeta::new(*round_underlying_tokens_pk, false),
        AccountMeta::new(*pending_withdrawal_info_pk, false),
        AccountMeta::new(*epoch_info_pk, false),
        AccountMeta::new(*fee_acct_pk, false),
        AccountMeta::new_readonly(*system_program_pk, false),
        AccountMeta::new_readonly(*token_program_pk, false),
        AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false)
    ];

    // let instr = FundInstruction::FriktionDepositInstr { discrim, amount };
    let mut cpi_data = Vec::<u8>::new();
    cpi_data.extend_from_slice(&discrim.to_le_bytes());
    cpi_data.extend_from_slice(&withdraw_amount.to_le_bytes());
    msg!("data for cpi: {:?}", cpi_data);
    Ok(Instruction { program_id: *program_id, accounts, data: cpi_data })
}

pub fn friktion_cancel_pending_withdrawal_ins(
    program_id: &Pubkey,
    authority_pk: &Pubkey,
    vault_mint_pk: &Pubkey,
    volt_vault_pk: &Pubkey,
    extra_volt_data_pk: &Pubkey,
    vault_authority_pk: &Pubkey,
    vault_token_destination_pk: &Pubkey,
    round_info_pk: &Pubkey,
    pending_withdrawal_info_pk: &Pubkey,
    epoch_info_pk: &Pubkey,
    system_program_pk: &Pubkey,
    token_program_pk: &Pubkey,
    discrim: u64
) -> Result<Instruction, ProgramError> {
    let mut accounts = vec![
        AccountMeta::new(*authority_pk, true),
        AccountMeta::new(*vault_mint_pk, false),
        AccountMeta::new(*volt_vault_pk, false),
        AccountMeta::new_readonly(*extra_volt_data_pk, false),
        AccountMeta::new_readonly(*vault_authority_pk, false),
        AccountMeta::new(*vault_token_destination_pk, false),
        AccountMeta::new(*round_info_pk, false),
        AccountMeta::new(*pending_withdrawal_info_pk, false),
        AccountMeta::new(*epoch_info_pk, false),
        AccountMeta::new_readonly(*system_program_pk, false),
        AccountMeta::new_readonly(*token_program_pk, false),
        AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false)
    ];

    // let instr = FundInstruction::FriktionDepositInstr { discrim, amount };
    let mut cpi_data = Vec::<u8>::new();
    cpi_data.extend_from_slice(&discrim.to_le_bytes());
    msg!("data for cpi: {:?}", cpi_data);
    Ok(Instruction { program_id: *program_id, accounts, data: cpi_data })
}

pub fn friktion_claim_pending_withdrawal_ins(
    program_id: &Pubkey,
    authority_pk: &Pubkey,
    volt_vault_pk: &Pubkey,
    extra_volt_data_pk: &Pubkey,
    vault_authority_pk: &Pubkey,
    vault_mint_pk: &Pubkey,
    underlying_token_destination_pk: &Pubkey,
    pending_withdrawal_round_info_pk: &Pubkey,
    pending_withdrawal_info_pk: &Pubkey,
    round_underlying_tokens_for_pending_withdrawals_pk: &Pubkey,
    system_program_pk: &Pubkey,
    token_program_pk: &Pubkey,
    discrim: u64
) -> Result<Instruction, ProgramError> {
    let mut accounts = vec![
        AccountMeta::new(*authority_pk, true),
        AccountMeta::new(*volt_vault_pk, false),
        AccountMeta::new_readonly(*extra_volt_data_pk, false),
        AccountMeta::new_readonly(*vault_authority_pk, false),
        AccountMeta::new(*vault_mint_pk, false),
        AccountMeta::new(*underlying_token_destination_pk, false),
        AccountMeta::new(*pending_withdrawal_round_info_pk, false),
        AccountMeta::new(*pending_withdrawal_info_pk, false),
        AccountMeta::new(*round_underlying_tokens_for_pending_withdrawals_pk, false),
        AccountMeta::new_readonly(*system_program_pk, false),
        AccountMeta::new_readonly(*token_program_pk, false),
    ];

    // let instr = FundInstruction::FriktionDepositInstr { discrim, amount };
    let mut cpi_data = Vec::<u8>::new();
    cpi_data.extend_from_slice(&discrim.to_le_bytes());
    msg!("data for cpi: {:?}", cpi_data);
    Ok(Instruction { program_id: *program_id, accounts, data: cpi_data })
}

pub fn friktion_claim_pending_deposit_ins(
    program_id: &Pubkey,
    authority_pk: &Pubkey,
    volt_vault_pk: &Pubkey,
    extra_volt_data_pk: &Pubkey,
    vault_authority_pk: &Pubkey,
    user_vault_tokens_pk: &Pubkey,
    pending_deposit_round_info_pk: &Pubkey,
    pending_deposit_round_volt_tokens_pk: &Pubkey,
    pending_deposit_info_pk: &Pubkey,
    system_program_pk: &Pubkey,
    token_program_pk: &Pubkey,
    discrim: u64
) -> Result<Instruction, ProgramError> {
    let mut accounts = vec![
        AccountMeta::new(*authority_pk, true),
        AccountMeta::new(*volt_vault_pk, false),
        AccountMeta::new_readonly(*extra_volt_data_pk, false),
        AccountMeta::new_readonly(*vault_authority_pk, false),
        AccountMeta::new(*user_vault_tokens_pk, false),
        AccountMeta::new(*pending_deposit_round_info_pk, false),
        AccountMeta::new(*pending_deposit_round_volt_tokens_pk, false),
        AccountMeta::new(*pending_deposit_info_pk, false),
        AccountMeta::new_readonly(*system_program_pk, false),
        AccountMeta::new_readonly(*token_program_pk, false),
    ];

    // let instr = FundInstruction::FriktionDepositInstr { discrim, amount };
    let mut cpi_data = Vec::<u8>::new();
    cpi_data.extend_from_slice(&discrim.to_le_bytes());
    msg!("data for cpi: {:?}", cpi_data);
    Ok(Instruction { program_id: *program_id, accounts, data: cpi_data })
}

pub fn friktion_add_to_fund(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    ul_token_slot: u8
) -> Result<(), ProgramError> {
    //Check is  active is false!
    let accounts_iter = &mut accounts.iter();
    let platform_acc = next_account_info(accounts_iter)?;
    let manager_ai = next_account_info(accounts_iter)?;
    let fund_account_acc = next_account_info(accounts_iter)?;
    let volt_vault_ai = next_account_info(accounts_iter)?;
    let vault_acc = next_account_info(accounts_iter)?;
    let volt_vault_data = &(vault_acc.try_borrow_data()?)[8..];
    let volt_vault_info = volt_abi::VoltVault::try_from_slice(volt_vault_data)?;
    check!(manager_ai.is_signer == true, FundError::IncorrectSignature);
    //Verify ul_mint
    let platform_data = PlatformData::load_checked(platform_acc, program_id)?;
    let mut fund_data = FundAccount::load_mut_checked(fund_account_acc, program_id)?;
    check!(fund_data.friktion_vault.is_active == false, FundError::InvalidStateAccount);
    // verify that it is whitelisted on PLATFORM
    
    // let token_index = platform_data.get_token_index(mint_acc.key, fund_data.tokens[ul_token_slot as usize].mux);
    // // let token_index_2 = platform_data.get_token_index(mint_acc.key, 1);
    // // both indexes cant be None
    // check!(ul_token_slot!=255, FundError::Default);
    // check!(((token_index_1 == fund_data.tokens[ul_token_slot as usize].index[0])), ProgramError::InvalidAccountData);
    // check!(volt_vault_info.underlying_asset_mint == platform_data.)

    //also verify that it is whitelisted on FUND
    fund_data.friktion_vault.volt_vault_id = *volt_vault_ai.key;
    fund_data.friktion_vault.ul_token_slot = ul_token_slot;
    fund_data.friktion_vault.is_active = true;
    fund_data.friktion_vault.fc_token_balance = 0;
    fund_data.friktion_vault.fc_token_debt = 0;
    fund_data.friktion_vault.ul_debt = 0;
    fund_data.friktion_vault.ul_token_balance = 0;
    fund_data.friktion_vault.total_value_in_ul = 0;

    Ok(())
}

pub fn friktion_remove_from_fund(
    program_id: &Pubkey,
    accounts: &[AccountInfo]
) -> Result<(), ProgramError> {

    let accounts_iter = &mut accounts.iter();
    let manager_ai = next_account_info(accounts_iter)?;
    let fund_account_acc = next_account_info(accounts_iter)?;
    let mut fund_data = FundAccount::load_mut_checked(fund_account_acc, program_id)?;

    //Check total value = 0 set isActive to false and zero out data
    fund_data.friktion_vault.volt_vault_id = Pubkey::default();
    fund_data.friktion_vault.ul_token_slot = 255;
    fund_data.friktion_vault.is_active = false;
    fund_data.friktion_vault.fc_token_balance = 0;
    fund_data.friktion_vault.fc_token_debt = 0;
    fund_data.friktion_vault.ul_debt = 0;
    fund_data.friktion_vault.ul_token_balance = 0;
    fund_data.friktion_vault.total_value_in_ul = 0;
    Ok(())
}

// pub fn friktion_
pub fn friktion_deposit(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    deposit_amount: u64
) -> Result<(), ProgramError> {
    const NUM_FIXED:usize = 26;
    let accounts = array_ref![accounts, 0, NUM_FIXED];
    
        let [
            fund_account_ai,
            manager_ai,
            volt_program_ai,
            authority_ai,
            dao_authority_ai,
            authority_check_ai,
            vault_mint_ai,
            volt_vault_ai,
            vault_authority_ai,
            extra_volt_data_ai,
            whitelist_ai,
            deposit_pool_ai,
            writer_token_pool_ai,
            vault_token_destination_ai,
            underlying_token_source_ai,
            round_info_ai,
            round_volt_tokens_ai,
            round_underlying_tokens_ai,
            pending_deposit_info_ai,
            epoch_info_ai,
            entropy_program_ai,
            entropy_group_ai,
            entropy_account_ai,
            entropy_cache_ai,
            system_program_ai,
            token_program_ai,
        ] = accounts;

        //TODO::
        // UPDATE balance of ul_token_slot on fund 
        // Take token_slot as param and init if needed 

        check!(manager_ai.is_signer, ProgramError::MissingRequiredSignature);
        let mut fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;
        check!(fund_data.manager_account == *manager_ai.key, FundError::ManagerMismatch);
        let pda_signer_nonce = fund_data.signer_nonce;

        //check
        //*underlying_token_source_ai.key == fund_data.tokens[fund_data.friktion_vault.ul_token_slot as usize].vault
        

        
        drop(fund_data);
        msg!("Trying CPI");
        // authority_check_ai.is_signer = true;
        // let authority_check_ai_new = AccountInfo::new(authority_check_ai.key, true, authority_check_ai.is_writable, *authority_check_ai.laudachipppa(), *authority_check_ai.data.clone(), authority_check_ai.owner, authority_check_ai.executable, authority_check_ai.rent_epoch);
        invoke_signed(
            &friktion_deposit_ins(
                volt_program_ai.key, 
                authority_ai.key,
                dao_authority_ai.key,
                authority_check_ai.key, 
                vault_mint_ai.key, 
                volt_vault_ai.key,
                vault_authority_ai.key,
                extra_volt_data_ai.key,
                whitelist_ai.key,
                deposit_pool_ai.key, 
                writer_token_pool_ai.key, 
                vault_token_destination_ai.key, 
                underlying_token_source_ai.key, 
                round_info_ai.key, 
                round_volt_tokens_ai.key, 
                round_underlying_tokens_ai.key, 
                pending_deposit_info_ai.key, 
                epoch_info_ai.key, 
                entropy_program_ai.key, 
                entropy_group_ai.key, 
                entropy_account_ai.key, 
                entropy_cache_ai.key, 
                system_program_ai.key, 
                token_program_ai.key, 
                13182846803881894898, 
                deposit_amount
            )?,
            &[
                volt_program_ai.clone(),
                authority_ai.clone(),
                dao_authority_ai.clone(),
                authority_check_ai.clone(),
                vault_mint_ai.clone(),
                volt_vault_ai.clone(),
                vault_authority_ai.clone(),
                extra_volt_data_ai.clone(),
                whitelist_ai.clone(),
                deposit_pool_ai.clone(),
                writer_token_pool_ai.clone(),
                vault_token_destination_ai.clone(),
                underlying_token_source_ai.clone(),
                round_info_ai.clone(),
                round_volt_tokens_ai.clone(),
                round_underlying_tokens_ai.clone(),
                pending_deposit_info_ai.clone(),
                epoch_info_ai.clone(),
                entropy_program_ai.clone(),
                entropy_group_ai.clone(),
                entropy_account_ai.clone(),
                entropy_cache_ai.clone(),
                system_program_ai.clone(),
                token_program_ai.clone()
            ], 
            &[&[&*manager_ai.key.as_ref(), bytes_of(&pda_signer_nonce)]]
        );

        fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;
        let tsi = fund_data.friktion_vault.ul_token_slot as usize;
        let ul_fund_token_data = parse_token_account(underlying_token_source_ai)?;
        fund_data.tokens[tsi].balance = ul_fund_token_data.amount;
        check!(fund_data.tokens[tsi].balance >= fund_data.tokens[tsi].debt, ProgramError::InsufficientFunds);
        Ok(())

    }

pub fn friktion_withdraw(
program_id: &Pubkey,
accounts: &[AccountInfo],
withdraw_amount: u64
) -> Result<(), ProgramError> {
const NUM_FIXED:usize = 22;
let accounts = array_ref![accounts, 0, NUM_FIXED];

    let [
        fund_account_ai,
        manager_ai,
        volt_program_ai,
        authority_ai,
        dao_authority_ai,
        authority_check_ai,
        vault_mint_ai,
        volt_vault_ai,
        vault_authority_ai,
        extra_volt_data_ai,
        whitelist_ai,
        deposit_pool_ai,
        vault_token_source_ai,
        underlying_token_destination_ai,
        round_info_ai,
        round_underlying_tokens_ai,
        pending_withdrawal_info_ai,
        epoch_info_ai,
        fee_acct_ai,
        system_program_ai,
        token_program_ai,
        sysvar_rent_ai
    ] = accounts;

    check!(manager_ai.is_signer, ProgramError::MissingRequiredSignature);
    let mut fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;
    check!(fund_data.manager_account == *manager_ai.key, FundError::ManagerMismatch);
    let pda_signer_nonce = fund_data.signer_nonce;
    drop(fund_data);
    msg!("Trying CPI");
    // authority_check_ai.is_signer = true;
    // let authority_check_ai_new = AccountInfo::new(authority_check_ai.key, true, authority_check_ai.is_writable, *authority_check_ai.laudachipppa(), *authority_check_ai.data.clone(), authority_check_ai.owner, authority_check_ai.executable, authority_check_ai.rent_epoch);
    invoke_signed(
        &friktion_withdraw_ins(
            volt_program_ai.key, 
            authority_ai.key,
            dao_authority_ai.key,
            authority_check_ai.key, 
            vault_mint_ai.key, 
            volt_vault_ai.key,
            vault_authority_ai.key,
            extra_volt_data_ai.key,
            whitelist_ai.key,
            deposit_pool_ai.key, 
            vault_token_source_ai.key, 
            underlying_token_destination_ai.key, 
            round_info_ai.key, 
            round_underlying_tokens_ai.key, 
            pending_withdrawal_info_ai.key, 
            epoch_info_ai.key, 
            fee_acct_ai.key, 
            system_program_ai.key, 
            token_program_ai.key, 
            2495396153584390839, 
            withdraw_amount
        )?,
        &[
            volt_program_ai.clone(),
            authority_ai.clone(),
            dao_authority_ai.clone(),
            authority_check_ai.clone(),
            vault_mint_ai.clone(),
            volt_vault_ai.clone(),
            vault_authority_ai.clone(),
            extra_volt_data_ai.clone(),
            whitelist_ai.clone(),
            deposit_pool_ai.clone(),
            vault_token_source_ai.clone(),
            underlying_token_destination_ai.clone(),
            round_info_ai.clone(),
            round_underlying_tokens_ai.clone(),
            pending_withdrawal_info_ai.clone(),
            epoch_info_ai.clone(),
            fee_acct_ai.clone(),
            system_program_ai.clone(),
            token_program_ai.clone(),
            sysvar_rent_ai.clone()
        ], 
        &[&[&*manager_ai.key.as_ref(), bytes_of(&pda_signer_nonce)]]
    );
    
    Ok(())

}
    
pub fn friktion_investor_withdraw(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    withdraw_amount: u64
    ) -> Result<(), ProgramError> {
    const NUM_FIXED:usize = 23;
    let accounts = array_ref![accounts, 0, NUM_FIXED];
    
        let [
            fund_account_ai,
            investor_state_ai,
            investor_ai,
            volt_program_ai,
            authority_ai,
            dao_authority_ai,
            authority_check_ai,
            vault_mint_ai,
            volt_vault_ai,
            vault_authority_ai,
            extra_volt_data_ai,
            whitelist_ai,
            deposit_pool_ai,
            vault_token_source_ai,
            underlying_token_destination_ai,
            round_info_ai,
            round_underlying_tokens_ai,
            pending_withdrawal_info_ai,
            epoch_info_ai,
            fee_acct_ai,
            system_program_ai,
            token_program_ai,
            sysvar_rent_ai
        ] = accounts;
    
        check!(investor_ai.is_signer, ProgramError::MissingRequiredSignature);
        let mut investor_data = InvestorData::load_mut_checked(investor_state_ai, program_id)?;
        let mut fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;
        check!(fund_data.manager_account == *manager_ai.key, FundError::ManagerMismatch);
        let pda_signer_nonce = fund_data.signer_nonce;

        check!(investor_data.owner == *investor_ai.key, ProgramError::MissingRequiredSignature);
        check!(investor_ai.is_signer, ProgramError::MissingRequiredSignature);
        check_eq!(investor_data.manager, fund_data.manager_account);
        check!(investor_data.has_withdrawn == true && investor_data.withdrawn_from_margin == false, FundError::InvalidStateAccount);

       
        drop(fund_data);
        msg!("Trying CPI");
        // authority_check_ai.is_signer = true;
        // let authority_check_ai_new = AccountInfo::new(authority_check_ai.key, true, authority_check_ai.is_writable, *authority_check_ai.laudachipppa(), *authority_check_ai.data.clone(), authority_check_ai.owner, authority_check_ai.executable, authority_check_ai.rent_epoch);
        invoke_signed(
            &friktion_withdraw_ins(
                volt_program_ai.key, 
                authority_ai.key,
                dao_authority_ai.key,
                authority_check_ai.key, 
                vault_mint_ai.key, 
                volt_vault_ai.key,
                vault_authority_ai.key,
                extra_volt_data_ai.key,
                whitelist_ai.key,
                deposit_pool_ai.key, 
                vault_token_source_ai.key, 
                underlying_token_destination_ai.key, 
                round_info_ai.key, 
                round_underlying_tokens_ai.key, 
                pending_withdrawal_info_ai.key, 
                epoch_info_ai.key, 
                fee_acct_ai.key, 
                system_program_ai.key, 
                token_program_ai.key, 
                2495396153584390839, 
                withdraw_amount
            )?,
            &[
                volt_program_ai.clone(),
                authority_ai.clone(),
                dao_authority_ai.clone(),
                authority_check_ai.clone(),
                vault_mint_ai.clone(),
                volt_vault_ai.clone(),
                vault_authority_ai.clone(),
                extra_volt_data_ai.clone(),
                whitelist_ai.clone(),
                deposit_pool_ai.clone(),
                vault_token_source_ai.clone(),
                underlying_token_destination_ai.clone(),
                round_info_ai.clone(),
                round_underlying_tokens_ai.clone(),
                pending_withdrawal_info_ai.clone(),
                epoch_info_ai.clone(),
                fee_acct_ai.clone(),
                system_program_ai.clone(),
                token_program_ai.clone(),
                sysvar_rent_ai.clone()
            ], 
            &[&[&*investor_data.manager.key.as_ref(), bytes_of(&pda_signer_nonce)]]
        );
        
        Ok(())
    
    }
     

pub fn friktion_cancel_pending_deposit(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> Result<(), ProgramError> {
    const NUM_FIXED:usize = 16;
    let accounts = array_ref![accounts, 0, NUM_FIXED];
    
        let [
            fund_account_ai,
            manager_ai,
            volt_program_ai,
            authority_ai,
            vault_mint_ai,
            volt_vault_ai,
            extra_volt_data_ai,
            vault_authority_ai,
            underlying_token_destination_ai,
            round_info_ai,
            round_underlying_tokens_ai,
            pending_deposit_info_ai,
            epoch_info_ai,
            system_program_ai,
            token_program_ai,
            sysvar_rent_ai
        ] = accounts;

        check!(manager_ai.is_signer, ProgramError::MissingRequiredSignature);
        let mut fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;
        check!(fund_data.manager_account == *manager_ai.key, FundError::ManagerMismatch);
        let pda_signer_nonce = fund_data.signer_nonce;
        drop(fund_data);
        msg!("Trying CPI");
        // authority_check_ai.is_signer = true;
        // let authority_check_ai_new = AccountInfo::new(authority_check_ai.key, true, authority_check_ai.is_writable, *authority_check_ai.laudachipppa(), *authority_check_ai.data.clone(), authority_check_ai.owner, authority_check_ai.executable, authority_check_ai.rent_epoch);
        invoke_signed(
            &friktion_cancel_pending_deposit_ins(
                volt_program_ai.key, 
                authority_ai.key,
                vault_mint_ai.key, 
                volt_vault_ai.key,
                extra_volt_data_ai.key,
                vault_authority_ai.key,
                underlying_token_destination_ai.key, 
                round_info_ai.key, 
                round_underlying_tokens_ai.key, 
                pending_deposit_info_ai.key,
                epoch_info_ai.key, 
                system_program_ai.key, 
                token_program_ai.key, 
                1418778437388742568, 
            )?,
            &[
                volt_program_ai.clone(),
                authority_ai.clone(),
                vault_mint_ai.clone(),
                volt_vault_ai.clone(),
                vault_authority_ai.clone(),
                extra_volt_data_ai.clone(),
                underlying_token_destination_ai.clone(),
                round_info_ai.clone(),
                round_underlying_tokens_ai.clone(),
                pending_deposit_info_ai.clone(),
                epoch_info_ai.clone(),
                system_program_ai.clone(),
                token_program_ai.clone(),
                sysvar_rent_ai.clone()
            ], 
            &[&[&*manager_ai.key.as_ref(), bytes_of(&pda_signer_nonce)]]
        );
        fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;
        let tsi = fund_data.friktion_vault.ul_token_slot as usize;
        let ul_fund_token_data = parse_token_account(underlying_token_destination_ai)?;
        fund_data.tokens[tsi].balance = ul_fund_token_data.amount;
        check!(fund_data.tokens[tsi].balance >= fund_data.tokens[tsi].debt, ProgramError::InsufficientFunds);
        check!(fund_data.tokens[tsi].vault == *underlying_token_destination_ai.key, FundError::InvalidTokenAccount);

        Ok(())

    }

pub fn friktion_cancel_pending_withdrawal(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> Result<(), ProgramError> {
    const NUM_FIXED:usize = 15;
    let accounts = array_ref![accounts, 0, NUM_FIXED];
    
        let [
            fund_account_ai,
            manager_ai,
            volt_program_ai,
            authority_ai,
            vault_mint_ai,
            volt_vault_ai,
            extra_volt_data_ai,
            vault_authority_ai,
            vault_token_destination_ai,
            round_info_ai,
            pending_withdrawal_info_ai,
            epoch_info_ai,
            system_program_ai,
            token_program_ai,
            sysvar_rent_ai
        ] = accounts;

        check!(manager_ai.is_signer, ProgramError::MissingRequiredSignature);
        let mut fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;
        check!(fund_data.manager_account == *manager_ai.key, FundError::ManagerMismatch);
        let pda_signer_nonce = fund_data.signer_nonce;
        drop(fund_data);
        msg!("Trying CPI");
        // authority_check_ai.is_signer = true;
        // let authority_check_ai_new = AccountInfo::new(authority_check_ai.key, true, authority_check_ai.is_writable, *authority_check_ai.laudachipppa(), *authority_check_ai.data.clone(), authority_check_ai.owner, authority_check_ai.executable, authority_check_ai.rent_epoch);
        invoke_signed(
            &friktion_cancel_pending_withdrawal_ins(
                volt_program_ai.key, 
                authority_ai.key,
                vault_mint_ai.key, 
                volt_vault_ai.key,
                extra_volt_data_ai.key,
                vault_authority_ai.key,
                vault_token_destination_ai.key, 
                round_info_ai.key, 
                pending_withdrawal_info_ai.key,
                epoch_info_ai.key, 
                system_program_ai.key, 
                token_program_ai.key, 
                5803633322374918876, 
            )?,
            &[
                volt_program_ai.clone(),
                authority_ai.clone(),
                vault_mint_ai.clone(),
                volt_vault_ai.clone(),
                vault_authority_ai.clone(),
                extra_volt_data_ai.clone(),
                vault_token_destination_ai.clone(),
                round_info_ai.clone(),
                pending_withdrawal_info_ai.clone(),
                epoch_info_ai.clone(),
                system_program_ai.clone(),
                token_program_ai.clone(),
                sysvar_rent_ai.clone()
            ], 
            &[&[&*manager_ai.key.as_ref(), bytes_of(&pda_signer_nonce)]]
        );

        Ok(())

    }

pub fn friktion_claim_pending_withdrawal(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> Result<(), ProgramError> {
    const NUM_FIXED:usize = 14;
    let accounts = array_ref![accounts, 0, NUM_FIXED];
    
        let [
            fund_account_ai,
            manager_ai,
            volt_program_ai,
            authority_ai,
            volt_vault_ai,
            extra_volt_data_ai,
            vault_authority_ai,
            vault_mint_ai,
            underlying_token_destination_ai,
            pending_withdrawal_round_info_ai,
            pending_withdrawal_info_ai,
            round_underlying_tokens_for_pending_withdrawals_ai,
            system_program_ai,
            token_program_ai,
        ] = accounts;

        check!(manager_ai.is_signer, ProgramError::MissingRequiredSignature);
        let mut fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;
        check!(fund_data.manager_account == *manager_ai.key, FundError::ManagerMismatch);
        let pda_signer_nonce = fund_data.signer_nonce;
        drop(fund_data);
        msg!("Trying CPI");
        // authority_check_ai.is_signer = true;
        // let authority_check_ai_new = AccountInfo::new(authority_check_ai.key, true, authority_check_ai.is_writable, *authority_check_ai.laudachipppa(), *authority_check_ai.data.clone(), authority_check_ai.owner, authority_check_ai.executable, authority_check_ai.rent_epoch);
        invoke_signed(
            &friktion_claim_pending_withdrawal_ins(
                volt_program_ai.key, 
                authority_ai.key,
                volt_vault_ai.key,
                extra_volt_data_ai.key,
                vault_authority_ai.key,
                vault_mint_ai.key, 
                underlying_token_destination_ai.key, 
                pending_withdrawal_round_info_ai.key, 
                pending_withdrawal_info_ai.key,
                round_underlying_tokens_for_pending_withdrawals_ai.key, 
                system_program_ai.key, 
                token_program_ai.key, 
                1912432049757161649, 
            )?,
            &[
                volt_program_ai.clone(),
                authority_ai.clone(),
                volt_vault_ai.clone(),
                vault_authority_ai.clone(),
                extra_volt_data_ai.clone(),
                vault_mint_ai.clone(),
                underlying_token_destination_ai.clone(),
                pending_withdrawal_round_info_ai.clone(),
                pending_withdrawal_info_ai.clone(),
                round_underlying_tokens_for_pending_withdrawals_ai.clone(),
                system_program_ai.clone(),
                token_program_ai.clone(),
            ], 
            &[&[&*manager_ai.key.as_ref(), bytes_of(&pda_signer_nonce)]]
        );
        fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;
        let tsi = fund_data.friktion_vault.ul_token_slot as usize;
        let ul_fund_token_data = parse_token_account(underlying_token_destination_ai)?;
        fund_data.tokens[tsi].balance = ul_fund_token_data.amount;
        check!(fund_data.tokens[tsi].balance >= fund_data.tokens[tsi].debt, ProgramError::InsufficientFunds);
        check!(fund_data.tokens[tsi].vault == *underlying_token_destination_ai.key, FundError::InvalidTokenAccount);

        Ok(())

    }
    
pub fn friktion_claim_pending_deposit(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> Result<(), ProgramError> {
    const NUM_FIXED:usize = 13;
    let accounts = array_ref![accounts, 0, NUM_FIXED];
    
        let [
            fund_account_ai,
            manager_ai,
            volt_program_ai,
            authority_ai,
            volt_vault_ai,
            extra_volt_data_ai,
            vault_authority_ai,
            user_vault_tokens_ai,
            pending_deposit_round_info_ai,
            pending_deposit_round_volt_tokens_ai,
            pending_deposit_info_ai,
            system_program_ai,
            token_program_ai,
        ] = accounts;

        check!(manager_ai.is_signer, ProgramError::MissingRequiredSignature);
        let mut fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;
        check!(fund_data.manager_account == *manager_ai.key, FundError::ManagerMismatch);
        let pda_signer_nonce = fund_data.signer_nonce;
        drop(fund_data);
        msg!("Trying CPI");
        // authority_check_ai.is_signer = true;
        // let authority_check_ai_new = AccountInfo::new(authority_check_ai.key, true, authority_check_ai.is_writable, *authority_check_ai.laudachipppa(), *authority_check_ai.data.clone(), authority_check_ai.owner, authority_check_ai.executable, authority_check_ai.rent_epoch);
        invoke_signed(
            &friktion_claim_pending_deposit_ins(
                volt_program_ai.key, 
                authority_ai.key,
                volt_vault_ai.key,
                extra_volt_data_ai.key,
                vault_authority_ai.key,
                user_vault_tokens_ai.key,
                pending_deposit_round_info_ai.key,
                pending_deposit_round_volt_tokens_ai.key,
                pending_deposit_info_ai.key, 
                system_program_ai.key, 
                token_program_ai.key, 
                13863699443424371388, 
            )?,
            &[
                volt_program_ai.clone(),
                authority_ai.clone(),
                volt_vault_ai.clone(),
                vault_authority_ai.clone(),
                extra_volt_data_ai.clone(),
                user_vault_tokens_ai.clone(),
                pending_deposit_round_info_ai.clone(),
                pending_deposit_round_volt_tokens_ai.clone(),
                pending_deposit_info_ai.clone(),
                system_program_ai.clone(),
                token_program_ai.clone(),
            ], 
            &[&[&*manager_ai.key.as_ref(), bytes_of(&pda_signer_nonce)]]
        );
        

        Ok(())

    }
        

// // pub fn friktion_deposit(
// //     program_id: &Pubkey,
// //     accounts: &[AccountInfo],
// //     deposit_amount: u64
// // ) -> Result<(), ProgramError> {
//         const NUM_FIXED:usize = 26;
//         let accounts = array_ref![accounts, 0, NUM_FIXED];
        
//             let [
//                 fund_account_ai,
//                 manager_ai,
//                 volt_program_ai,
//                 authority_ai,
//                 dao_authority_ai,
//                 authority_check_ai,
//                 vault_mint_ai,
//                 volt_vault_ai,
//                 vault_authority_ai,
//                 extra_volt_data_ai,
//                 whitelist_ai,
//                 deposit_pool_ai,
//                 writer_token_pool_ai,
//                 vault_token_destination_ai,
//                 underlying_token_source_ai,
//                 round_info_ai,
//                 round_volt_tokens_ai,
//                 round_underlying_tokens_ai,
//                 pending_deposit_info_ai,
//                 epoch_info_ai,
//                 entropy_program_ai,
//                 entropy_group_ai,
//                 entropy_account_ai,
//                 entropy_cache_ai,
//                 system_program_ai,
//                 token_program_ai,
//             ] = accounts;
    
//             check!(manager_ai.is_signer, ProgramError::MissingRequiredSignature);
//             let mut fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;
//             check!(fund_data.manager_account == *manager_ai.key, FundError::ManagerMismatch);
//             let pda_signer_nonce = fund_data.signer_nonce;
//             drop(fund_data);
//             msg!("Trying CPI");
//             let mut authority_check_ai_new = fund_account_ai.clone();
//             authority_check_ai_new.is_signer = true;
    
//             let dummy_acc_info = &authority_check_ai_new.clone();
//             msg!("{:?} --- {:?}", authority_check_ai_new.is_signer, dummy_acc_info.is_signer);
//             // authority_check_ai.is_signer = true;
//             // let authority_check_ai_new = AccountInfo::new(authority_check_ai.key, true, authority_check_ai.is_writable, *authority_check_ai.laudachipppa(), *authority_check_ai.data.clone(), authority_check_ai.owner, authority_check_ai.executable, authority_check_ai.rent_epoch);
//             volt_abi::cpi::deposit(
//                 CpiContext::new_with_signer(volt_program_ai.clone(), 
//                 volt_abi::cpi::accounts::Deposit {
//                     authority: (*authority_ai).clone(),
//                     dao_authority: (*dummy_acc_info).clone(),
//                     // dao_authority: (*dao_authority_ai).clone(),
//                     authority_check: (*dummy_acc_info).clone(),
//                     // authority_check: authority_check_ai_new,
//                     vault_mint: (*vault_mint_ai).clone(),
//                     volt_vault: (*volt_vault_ai).clone(),
//                     vault_authority: (*vault_authority_ai).clone(),
//                     extra_volt_data: (*extra_volt_data_ai).clone(),
//                     whitelist: (*whitelist_ai).clone(),
//                     deposit_pool: (*deposit_pool_ai).clone(),
//                     writer_token_pool: (*writer_token_pool_ai).clone(),
//                     vault_token_destination: (*vault_token_destination_ai).clone(),
//                     underlying_token_source: (*underlying_token_source_ai).clone(),
//                     round_info: (*round_info_ai).clone(),
//                     round_volt_tokens: (*round_volt_tokens_ai).clone(),
//                     round_underlying_tokens: (*round_underlying_tokens_ai).clone(),
//                     pending_deposit_info: (*pending_deposit_info_ai).clone(),
//                     epoch_info: (*epoch_info_ai).clone(),
//                     entropy_program: (*entropy_program_ai).clone(),
//                     entropy_group: (*entropy_group_ai).clone(),
//                     entropy_account: (*entropy_account_ai).clone(),
//                     entropy_cache: (*entropy_cache_ai).clone(), 
//                     system_program: (*system_program_ai).clone(),
//                     token_program: (*token_program_ai).clone(),
//                 }, 
//                 &[&[&*manager_ai.key.as_ref(), bytes_of(&pda_signer_nonce)]]),
//                 deposit_amount
//             ).unwrap();
    
//             Ok(())
    
//         }

pub fn update_friktion_value(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> Result<(), ProgramError> {
    let mut val = 0u64;
    // //for pending dep and withdraws always check round_number matches that of voltVault if not then ask to claim first!!!
    let accounts_iter = &mut accounts.iter();

    let fund_account_ai = next_account_info(accounts_iter)?;
    let mut fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;
    let volt_program_id = next_account_info(accounts_iter)?.key;
    check_eq!(*volt_program_id, volt_abi::id());
    let volt_vault_ai = next_account_info(accounts_iter)?;
    // check!(*volt_vault_ai.key == fund_data.friktion_vault.volt_vault_id, FundError::InvalidStateAccount);
    let volt_vault_info = &(volt_vault_ai.data.borrow())[8..];
    let volt_vault_data = volt_abi::VoltVault::try_from_slice(volt_vault_info)?;
    let current_round = volt_vault_data.round_number;
    let pending_deposit_info_ai = next_account_info(accounts_iter)?;
    let (pending_deposit_pda, bump) = Pubkey::find_program_address(&[volt_vault_ai.key.as_ref(), fund_account_ai.key.as_ref(), b"pendingDeposit"], volt_program_id);
    msg!("pending dep pda {:?}", pending_deposit_pda);
    check!(*pending_deposit_info_ai.key == pending_deposit_pda, FundError::IncorrectPDA);
    if pending_deposit_info_ai.data_len() > 0 {
        let pending_info: &[u8] = &(pending_deposit_info_ai.data.borrow())[8..];
        let pending_deposit_data = volt_abi::PendingDeposit::try_from_slice(pending_info)?;
        check!(pending_deposit_data.round_number == current_round || pending_deposit_data.round_number == 0, FundError::UnclaimedPendingDeposit);
        val = pending_deposit_data.num_underlying_deposited;
        msg!("Underlying Deposited: {}, Round_Num {}", pending_deposit_data.num_underlying_deposited, pending_deposit_data.round_number);
    }

    let fc_tokens_ta_ai = next_account_info(accounts_iter)?;
    let fc_tokens_data = parse_token_account(fc_tokens_ta_ai)?;
    let mut fc_tokens = fc_tokens_data.amount;
    let pending_withdrawal_info_ai = next_account_info(accounts_iter)?;
    let (pending_withdrawal_pda, bump) = Pubkey::find_program_address(&[volt_vault_ai.key.as_ref(), fund_account_ai.key.as_ref(), b"pendingWithdrawal"], volt_program_id);
    msg!("pending wdw pda {:?}", pending_withdrawal_pda);
    check!(*pending_withdrawal_info_ai.key == pending_withdrawal_pda, FundError::IncorrectPDA);
    if pending_withdrawal_info_ai.data_len() > 0 {
        let pending_info: &[u8] = &(pending_withdrawal_info_ai.data.borrow())[8..];
        let pending_withdrawal_data = volt_abi::PendingWithdrawal::try_from_slice(pending_info)?;
        check!(pending_withdrawal_data.round_number == current_round || pending_withdrawal_data.round_number == 0, FundError::UnclaimedPendingwithdrawal);
        fc_tokens = fc_tokens.checked_add(pending_withdrawal_data.num_volt_redeemed).unwrap();
        msg!("Volt Tokens Withdrawan: {}, Round_Num {}", pending_withdrawal_data.num_volt_redeemed, pending_withdrawal_data.round_number);
    }
    let epoch_info_ai = next_account_info(accounts_iter)?;
    let epoch_info = &(epoch_info_ai.data.borrow())[8..];
    let epoch_info_data = volt_abi::FriktionEpochInfo::try_from_slice(epoch_info)?; 
    let fc_tokens_val = epoch_info_data.vault_token_price*(fc_tokens as f64);
    val = val.checked_add(fc_tokens_val as u64).unwrap();
    msg!("Friktion val in ul: {:?}", val);
    fund_data.friktion_vault.volt_vault_id = *volt_vault_ai.key;
    fund_data.friktion_vault.last_updated = Clock::get()?.unix_timestamp;
    fund_data.friktion_vault.total_value_in_ul = val;
    Ok(())
}

