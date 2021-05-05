use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

pub fn Deposit (
    program_id: &Pubkey,
	accounts: &[AccountInfo],
	instruction_data: &[u8],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let initializer = next_account_info(account_info_iter)?;

    //transfer lamports
    **initializer.try_borrow_mut_lamport()? -= 5;

    **program_id.try_borrow_mut_lamport()? += 5;

    ok(())
}