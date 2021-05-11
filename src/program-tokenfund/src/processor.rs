use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    log::{sol_log_compute_units, sol_log_params, sol_log_slice},
    pubkey::Pubkey,
    program::invoke,
};

use crate::instruction::FundInstruction;
use spl_token::state::Account as TokenAccount;

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    msg!("Fund Program Entrypoint");
    
    let tag:u8 = _instruction_data[0].into();
    let amt:u64 = _instruction_data[1].into();

    let mut instruction = FundInstruction::DepositLamports{amount: 0};
    if tag == 0 {
        instruction = FundInstruction::DepositLamports{amount: amt};
    }
    else {
        instruction = FundInstruction::WithdrawLamports{amount: amt};
    }

    let account_info_iter = &mut accounts.iter();

    let fund_account = next_account_info(account_info_iter)?;
    let investor_account = next_account_info(account_info_iter)?;
    let fund_token_account = next_account_info(account_info_iter)?;
    let investor_token_account = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;

    match instruction {
        FundInstruction::DepositLamports { amount: u64 } => {
            msg!("Depositing Lamports");
            // let mut account_data = FundData::try_from_slice(*fund_info.data.borrow())?;
            // account_data.owner = *investor_account.key;

            let owner_change_ix = spl_token::instruction::set_authority(
                token_program.key,
                investor_token_account.key, 
                Some(fund_account.key),
                spl_token::instruction::AuthorityType::AccountOwner,
                investor_account.key,
                &[&investor_account.key],
            )?;

            msg!("Calling the token program to transfer token account ownership...");
            invoke(
                &owner_change_ix,
                &[
                    fund_token_account.clone(),
                    fund_account.clone(),
                    token_program.clone(),
                ],
            )?;
            
            let transfer_to_fund_ix = spl_token::instruction::transfer(
                token_program.key,
                investor_token_account.key,
                fund_token_account.key,
                fund_account.key,
                &[&investor_account.key],
                amt,
            )?;
            msg!("Calling the token program to transfer tokens from investor_token_account to fund_token_account...");
            invoke(
                &transfer_to_fund_ix,
                &[
                    investor_token_account.clone(),
                    fund_token_account.clone(),
                    investor_account.clone(),
                    token_program.clone(),
                ],
            )?;

            // account_data.amount += amount;
        }
        
        FundInstruction::WithdrawLamports { amount: u64 } => {
            msg!("Withdraw Lamports");
            // let mut account_data = FundData::try_from_slice(*fund_info.data.borrow())?;

            // if account_data.amount <= amount {
            //     /// return Err(ProgramError::InvalidAmount)
            // }
            
            let fund_account = next_account_info(account_info_iter)?;
            let investor_account = next_account_info(account_info_iter)?;
            // if account_data.owner != *investor_account.key {
            //     /// return Err(ProgramError::InvalidAccountData); //define Error
            // }
            let fund_token_account = next_account_info(account_info_iter)?;
            let investor_token_account = next_account_info(account_info_iter)?;
            let token_program = next_account_info(account_info_iter)?;

            let transfer_to_fund_ix = spl_token::instruction::transfer(
                token_program.key,
                fund_token_account.key,
                investor_token_account.key,
                fund_account.key,
                &[&investor_account.key],
                amt,
            )?;
            msg!("Calling the token program to transfer tokens from fund_token_account to investor_token_account...");
            invoke(
                &transfer_to_fund_ix,
                &[
                    investor_token_account.clone(),
                    fund_token_account.clone(),
                    investor_account.clone(),
                    token_program.clone(),
                ],
            )?;
            // account_data.amount -= amount;
        }
    }
    Ok(())
}