use borsh::{BorshDeserialize, BorshSerialize, BorshSchema};
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
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct FundData {
    /// The account allowed to update the data
    pub owner: Pubkey,

    /// The data contained by the account, could be anything serializable
    pub amount: u64,

    pub manager: Pubkey,
}



#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum FundInstruction {
    /// Accounts expected
    
    /// 0. [WRITE]  Main Account
    /// 1. [SIGNER] Investor Account
    /// 2. []       Router Token Account
    /// 3. []       Investor Token Account
    /// 4. []       Manager Token Account
    /// 5. []       PDA of Router
    /// 6. []       PDA of Manager
    /// 7. []       Token Program
    InvestorDeposit {
        amount: u64
    },

    /// 0. [WRITE]  Main Account
    /// 1. []       Investor Account
    /// 2. []       Router Token Account
    /// 3. []       Investor Token Account
    /// 4. []       Manager Token Account
    /// 5. []       PDA of Router
    /// 6. []       PDA of Manager
    /// 7. []       Token Program
    InvestorWithdraw {
        amount: u64
    },

    /// 0. [WRITE]  Main Account
    /// 1. []       Investor Account
    /// 2. []       Router Token Account
    /// 3. []       Investor Token Account
    /// 4. []       Manager Token Account
    /// 5. []       PDA of Router
    /// 6. []       PDA of Manager
    /// 7. []       Token Program
    ManagerTransfer {
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

    let mut instruction = FundInstruction::InvestorDeposit{amount: 0};
    if tag == 0 {
        instruction = FundInstruction::InvestorDeposit{amount: amt};
    }
    else {
        if tag == 1 {
            instruction = FundInstruction::InvestorWithdraw{amount: amt};
        }
        else{
            instruction = FundInstruction::ManagerTransfer {amount: amt};
        }
    }
    let account_info_iter = &mut accounts.iter();

    let fund_account = next_account_info(account_info_iter)?;
    let investor_account = next_account_info(account_info_iter)?;
    let temp_token_account = next_account_info(account_info_iter)?;    
    let investor_token_account = next_account_info(account_info_iter)?;
    let manager_token_account = next_account_info(account_info_iter)?;
    let pda_account = next_account_info(account_info_iter)?;
    let mpda_account = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;

  
    // let temp_token_account = next_account_info(account_info_iter)?;
    let (pda, _nonce) = Pubkey::find_program_address(&[b"fund"], program_id);

    let (man_pda, bump_seed) = Pubkey::find_program_address(&[b"manager"], program_id);
    msg!("Manager PDA: {:?}", man_pda);

    msg!("PDA: {:?}", pda);

    match instruction {
        FundInstruction::InvestorDeposit { amount: u64 } => {
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
            msg!("Transfer done");
            msg!("Writing to state account: {:?}", *fund_account.key);
            
            let mut account_data = FundData::try_from_slice(*fund_account.data.borrow())?;

            account_data.owner = *investor_account.key;
            account_data.manager = *manager_token_account.key;
            account_data.amount += amt;
            account_data.serialize(&mut *fund_account.data.borrow_mut());
        }
        
        FundInstruction::InvestorWithdraw { amount: u64 } => {
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
            let mut account_data = FundData::try_from_slice(*fund_account.data.borrow())?;
            
            if account_data.owner != *investor_account.key {
                msg!("Owner mismatch");
            }

            msg!("authority changed");
            let transfer_to_fund_ix = spl_token::instruction::transfer(
                token_program.key,
                manager_token_account.key,
                investor_token_account.key,
                &man_pda,
                &[&man_pda],
                account_data.amount,
            )?;
            msg!("Calling the token program to transfer tokens from fund_token_account to investor_token_account...");
            invoke_signed(
                &transfer_to_fund_ix,
                &[
                    manager_token_account.clone(),
                    investor_token_account.clone(),
                    mpda_account.clone(),
                    token_program.clone(),
                ],
                &[&[&b"manager"[..], &[bump_seed]]],
            )?;
            account_data.amount = 0;
            account_data.serialize(&mut *fund_account.data.borrow_mut());
        }

        FundInstruction::ManagerTransfer {amount: u64} => {
            
            msg!("Manager transfering funds");
            let mut account_data = FundData::try_from_slice(*fund_account.data.borrow())?;
            
            if account_data.manager != *manager_token_account.key {
                msg!("Manager mismatch");
            }
            let transfer_to_fund_ix = spl_token::instruction::transfer(
                token_program.key,
                temp_token_account.key,
                manager_token_account.key,
                &pda,
                &[&pda],
                account_data.amount
            )?;
            msg!("Calling the token program to transfer tokens from temp_token_account to manager_token_account...");
            invoke_signed(
                &transfer_to_fund_ix,
                &[
                    temp_token_account.clone(),
                    manager_token_account.clone(),
                    pda_account.clone(),
                    token_program.clone(),
                ],
                &[&[&b"fund"[..], &[_nonce]]],
            )?;
        }
    }
    Ok(())
}
