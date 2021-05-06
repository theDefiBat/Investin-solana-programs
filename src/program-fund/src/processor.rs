//! Program instruction processor
use {
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        program_error::ProgramError,
        program_pack::IsInitialized,
        pubkey::Pubkey,
    },
};

/// Struct wrapping data and providing metadata
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct FundData {

    /// The account allowed to update the data
    pub owner: Pubkey,

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

/// Instruction processor
pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {

    let instruction = FundInstruction::try_from_slice(_instruction_data)?;
    // Create in iterator to safety reference accounts in the slice
    let account_info_iter = &mut accounts.iter();

    let fund_info = next_account_info(account_info_iter)?;
    let client_info = next_account_info(account_info_iter)?;

    match instruction {
        FundInstruction::DepositLamports { amount: u64 } => {
            msg!("Depositing Lamports");
            let mut account_data = FundData::try_from_slice(*fund_info.data.borrow())?;
            account_data.owner = *client_info.key;
            **client_info.try_borrow_mut_lamports()? -= 5;
            // Deposit five lamports into the destination
            **fund_info.try_borrow_mut_lamports()? += 5;
            account_data.amount = 5;
        }
        FundInstruction::WithdrawLamports { amount: u64 } => {
            msg!("Withdraw Lamports");
            let mut account_data = FundData::try_from_slice(*fund_info.data.borrow())?;
            account_data.owner = *client_info.key;
            **fund_info.try_borrow_mut_lamports()? -= 5;
            // Deposit five lamports into the destination
            **client_info.try_borrow_mut_lamports()? += 5;
            account_data.amount = 0;
        }
    }
    Ok(())
}