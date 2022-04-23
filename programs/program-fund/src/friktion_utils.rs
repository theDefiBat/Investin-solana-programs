use bytemuck::bytes_of;
use std::{mem::size_of};
use fixed::types::U64F64;
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
use anchor_lang::{prelude::CpiContext, AnchorDeserialize, accounts::sysvar};
use arrayref::{array_ref, array_refs};
use spl_token::state::{Account, Mint};
use mango::state::{MangoAccount, MangoGroup, MangoCache, PerpMarket, MAX_TOKENS, MAX_PAIRS, QUOTE_INDEX};
use mango::instruction::{ cancel_all_perp_orders, withdraw, place_perp_order, consume_events };
use mango::matching::{Side, OrderType, Book};
use volt_abi::*;

use crate::error::FundError;
use crate::instruction::{FundInstruction, Data};
use crate::state::{FundAccount, InvestorData, PlatformData};

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
    let pending_info_data: &[u8] = &(pending_deposit_info_ai.data.borrow())[8..];
    let pending_deposit_info = volt_abi::PendingDeposit::try_from_slice(pending_info_data)?;
    msg!("Underlying Deposited: {}, Round_Num {}", pending_deposit_info.num_underlying_deposited, pending_deposit_info.round_number);
    let pending_withdrawal_info_ai = next_account_info(accounts_iter)?;
    let pending_withdrawal_data: &[u8] = &(pending_withdrawal_info_ai.data.borrow())[8..];
    let pending_withdrawal_info = volt_abi::PendingWithdrawal::try_from_slice(pending_withdrawal_data)?;
    msg!("Underlying withdrawaled: {}, Round_Num {}", pending_withdrawal_info.num_volt_redeemed, pending_withdrawal_info.round_number);
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


// pub fn friktion_
pub fn friktion_deposit0(
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

        check!(manager_ai.is_signer, ProgramError::MissingRequiredSignature);
        let mut fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;
        check!(fund_data.manager_account == *manager_ai.key, FundError::ManagerMismatch);
        let pda_signer_nonce = fund_data.signer_nonce;
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
    
            check!(manager_ai.is_signer, ProgramError::MissingRequiredSignature);
            let mut fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;
            check!(fund_data.manager_account == *manager_ai.key, FundError::ManagerMismatch);
            let pda_signer_nonce = fund_data.signer_nonce;
            drop(fund_data);
            msg!("Trying CPI");
            let mut authority_check_ai_new = fund_account_ai.clone();
            authority_check_ai_new.is_signer = true;
    
            let dummy_acc_info = &authority_check_ai_new.clone();
            msg!("{:?} --- {:?}", authority_check_ai_new.is_signer, dummy_acc_info.is_signer);
            // authority_check_ai.is_signer = true;
            // let authority_check_ai_new = AccountInfo::new(authority_check_ai.key, true, authority_check_ai.is_writable, *authority_check_ai.laudachipppa(), *authority_check_ai.data.clone(), authority_check_ai.owner, authority_check_ai.executable, authority_check_ai.rent_epoch);
            volt_abi::cpi::deposit(
                CpiContext::new_with_signer(volt_program_ai.clone(), 
                volt_abi::cpi::accounts::Deposit {
                    authority: (*authority_ai).clone(),
                    dao_authority: (*dummy_acc_info).clone(),
                    // dao_authority: (*dao_authority_ai).clone(),
                    authority_check: (*dummy_acc_info).clone(),
                    // authority_check: authority_check_ai_new,
                    vault_mint: (*vault_mint_ai).clone(),
                    volt_vault: (*volt_vault_ai).clone(),
                    vault_authority: (*vault_authority_ai).clone(),
                    extra_volt_data: (*extra_volt_data_ai).clone(),
                    whitelist: (*whitelist_ai).clone(),
                    deposit_pool: (*deposit_pool_ai).clone(),
                    writer_token_pool: (*writer_token_pool_ai).clone(),
                    vault_token_destination: (*vault_token_destination_ai).clone(),
                    underlying_token_source: (*underlying_token_source_ai).clone(),
                    round_info: (*round_info_ai).clone(),
                    round_volt_tokens: (*round_volt_tokens_ai).clone(),
                    round_underlying_tokens: (*round_underlying_tokens_ai).clone(),
                    pending_deposit_info: (*pending_deposit_info_ai).clone(),
                    epoch_info: (*epoch_info_ai).clone(),
                    entropy_program: (*entropy_program_ai).clone(),
                    entropy_group: (*entropy_group_ai).clone(),
                    entropy_account: (*entropy_account_ai).clone(),
                    entropy_cache: (*entropy_cache_ai).clone(), 
                    system_program: (*system_program_ai).clone(),
                    token_program: (*token_program_ai).clone(),
                }, 
                &[&[&*manager_ai.key.as_ref(), bytes_of(&pda_signer_nonce)]]),
                deposit_amount
            ).unwrap();
    
            Ok(())
    
        }