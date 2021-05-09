use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    log::{sol_log_compute_units, sol_log_params, sol_log_slice},
    pubkey::Pubkey,
};

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
    msg!("Fund program entrypoint");


    msg!("printing instruction data, : {:?}", _instruction_data[0]);
    msg!("printing  second instruction data, : {:?}", _instruction_data[1]);
    
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

    let fund_info = next_account_info(account_info_iter)?;
    let client_info = next_account_info(account_info_iter)?;

    match instruction {
        FundInstruction::DepositLamports {amount: u64} => {
            msg!("Depositing Lamports");
            let mut account_data = FundData::try_from_slice(*fund_info.data.borrow())?;
            //account_data.owner = *client_info.key;
            **client_info.try_borrow_mut_lamports()? -= amt;
            // Deposit five lamports into the destination
            **fund_info.try_borrow_mut_lamports()? += amt;
            account_data.amount = amt;
        }
        FundInstruction::WithdrawLamports {amount: u64} => {
            msg!("Withdraw Lamports");
            //let mut account_data = FundData::try_from_slice(*fund_info.data.borrow())?;
            //account_data.owner = *client_info.key;
            msg!("Withdrawing.....");
            **fund_info.try_borrow_mut_lamports()? -= amt;
            // Deposit five lamports into the destination
            **client_info.try_borrow_mut_lamports()? += amt;
            //account_data.amount = 0;
        }
    }
    Ok(())
}

