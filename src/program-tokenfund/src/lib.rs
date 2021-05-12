use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    log::{sol_log_compute_units, sol_log_params, sol_log_slice},
    pubkey::Pubkey,
    program::{invoke, invoke_signed},
};
use spl_token::state::Account as TokenAccount;

/// Struct wrapping data and providing metadata
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct FundData {

    /// The account allowed to update the data
    //pub owner: Pubkey,

    /// The data contained by the account, could be anything serializable
    pub amount: u64,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum FundInstruction {
    /// Accounts expected
    /// 0. [WRITE] Fund Account
    /// 1. [SIGNER] Investor Account

    DepositLamports {
        amount: u64
    },

    WithdrawLamports {
        amount: u64
    }
}

// Declare and export the program's entrypoint
entrypoint!(process_instruction);

// Program entrypoint's implementation
pub fn process_instruction(
    program_id: &Pubkey, // Public key of the account the hello world program was loaded into
    accounts: &[AccountInfo], // The account to say hello to
    _instruction_data: &[u8], // Ignored, all helloworld instructions are hellos
) -> ProgramResult {
    msg!("Token Program Entrypoint");
    
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

    let temp_token_account = next_account_info(account_info_iter)?;
    // let temp_token_account = next_account_info(account_info_iter)?;
    let (pda, _nonce) = Pubkey::find_program_address(&[b"fund"], program_id);
    msg!("PDA: {:?}", pda);
    let pda_account = next_account_info(account_info_iter)?;

    match instruction {
        FundInstruction::DepositLamports { amount: u64 } => {
            msg!("Depositing Lamports");
            // let mut account_data = FundData::try_from_slice(*fund_info.data.borrow())?;
            // account_data.owner = *investor_account.key;

            // let owner_change_ix = spl_token::instruction::set_authority(
            //     token_program.key,
            //     investor_token_account.key, 
            //     Some(fund_account.key),
            //     spl_token::instruction::AuthorityType::AccountOwner,
            //     investor_account.key,
            //     &[&investor_account.key],
            // )?;

            // msg!("Calling the token program to transfer token account ownership...");
            // invoke(
            //     &owner_change_ix,
            //     &[
            //         fund_token_account.clone(),
            //         fund_account.clone(),
            //         token_program.clone(),
            //     ],
            // )?;
            
            let transfer_to_fund_ix = spl_token::instruction::transfer(
                token_program.key,
                investor_token_account.key,
                temp_token_account.key,
                investor_account.key,
                &[&investor_account.key],
                amt,
            )?;
            msg!("Calling the token program to transfer tokens from investor_token_account to fund_token_account...");
            invoke(
                &transfer_to_fund_ix,
                &[
                    investor_token_account.clone(),
                    temp_token_account.clone(),
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
            //msg!("approving withdraw");


            // let owner_change_ix = spl_token::instruction::set_authority(
            //     token_program.key,
            //     temp_token_account.key,
            //     Some(&pda),
            //     spl_token::instruction::AuthorityType::AccountOwner,
            //     investor_account.key,
            //     &[&investor_account.key],
            // )?;
            // invoke(
            //     &owner_change_ix,
            //     &[
            //         temp_token_account.clone(),
            //         investor_account.clone(),
            //         token_program.clone(),
            //     ],
            // )?;

            msg!("authority changed");
            let transfer_to_fund_ix = spl_token::instruction::transfer(
                token_program.key,
                temp_token_account.key,
                investor_token_account.key,
                &pda,
                &[&pda],
                amt,
            )?;
            msg!("Calling the token program to transfer tokens from fund_token_account to investor_token_account...");
            invoke_signed(
                &transfer_to_fund_ix,
                &[
                    temp_token_account.clone(),
                    investor_token_account.clone(),
                    pda_account.clone(),
                    token_program.clone(),
                ],
                &[&[&b"fund"[..], &[_nonce]]],
            )?;
            // account_data.amount -= amount;
        }
    }
    Ok(())
}
