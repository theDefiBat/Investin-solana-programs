//! Program entrypoint

#![cfg(not(feature = "no-entrypoint"))]

use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};

// use crate::deposit::Deposit;
// use crate::withdraw::Withdraw;

entrypoint!(process_instruction);
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // let choice = 1; //have to get this from instruction_data

    // match choice {
    //     1 => crate::deposit::Deposit(program_id, accounts, instruction_data),

    //     2 => crate::withdraw::Withdraw(program_id, accounts, instruction_data)
    // }

    crate::deposit::Deposit(program_id, accounts, instruction_data)
    // Withdraw::process(program_id, accounts, instruction_data)
}