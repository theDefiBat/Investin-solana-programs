use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};

use crate::deposit::Deposit;
use crate::withdraw::Withdraw;

entrypoint!(process_instruction);
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    Deposit::process(program_id, accounts, instruction_data)
    Withdraw::process(program_id, accounts, instruction_data)
}