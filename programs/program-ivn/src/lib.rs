use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::{invoke_signed, invoke},
    program_pack::Pack,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use bytemuck::bytes_of;
use arrayref::array_ref;
use spl_token::state::Account;

macro_rules! check_eq {
    ($x:expr, $y:expr) => {
        if ($x != $y) {
            return Err(ProgramError::InvalidAccountData)
        }
    }
}

// declare mints for account check
pub mod ab_ivn_mint {
    use solana_program::declare_id;
    declare_id!("2ZfTbMeJqfTRR2wmMYCbT2W9tkEwEZwf4u5x5LMiCsj1");
}

pub mod ivn_mint {
    use solana_program::declare_id;
    declare_id!("iVNcrNE9BRZBC9Aqf753iZiZfbszeAVUoikgT9yvr2a");
}

// program entrypoint
entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let pda_acc = next_account_info(accounts_iter)?;
    let ab_ivn_from_acc = next_account_info(accounts_iter)?;
    let ab_ivn_to_acc = next_account_info(accounts_iter)?;
    let ivn_from_acc = next_account_info(accounts_iter)?;
    let ivn_to_acc = next_account_info(accounts_iter)?;
    let owner_acc = next_account_info(accounts_iter)?;
    let token_prog_acc = next_account_info(accounts_iter)?;

    let (pda, nonce) = Pubkey::find_program_address(&["swap".as_ref()], program_id);

    let &ix_data = array_ref![data, 0, 8];
    let amount = u64::from_le_bytes(ix_data);

    // account mint checks
    let ab_ivn_from_data = Account::unpack(&ab_ivn_from_acc.try_borrow_data()?)?;
    let ab_ivn_to_data = Account::unpack(&ab_ivn_to_acc.try_borrow_data()?)?;
    check_eq!(ab_ivn_from_data.mint, ab_ivn_mint::ID);
    check_eq!(ab_ivn_to_data.mint, ab_ivn_mint::ID);

    let ivn_from_data = Account::unpack(&ivn_from_acc.try_borrow_data()?)?;
    let ivn_to_data = Account::unpack(&ivn_to_acc.try_borrow_data()?)?;
    check_eq!(ivn_from_data.mint, ivn_mint::ID);
    check_eq!(ivn_to_data.mint, ivn_mint::ID);

    // vault owner checks
    check_eq!(ab_ivn_to_data.owner, pda);
    check_eq!(ivn_from_data.owner, pda);

    invoke(
        &(spl_token::instruction::transfer(
            token_prog_acc.key,
            ab_ivn_from_acc.key,
            ab_ivn_to_acc.key,
            owner_acc.key,
            &[&owner_acc.key],
            amount
        ))?,
        &[
            token_prog_acc.clone(),
            ab_ivn_from_acc.clone(),
            ab_ivn_to_acc.clone(),
            owner_acc.clone()
        ]
    )?;
    
    msg!("abIVN locked:: {:?}", amount);

    let transfer_amount = amount / 1000; // account for decimals

    invoke_signed(
        &(spl_token::instruction::transfer(
            token_prog_acc.key,
            ivn_from_acc.key,
            ivn_to_acc.key,
            pda_acc.key,
            &[pda_acc.key],
            transfer_amount
        ))?,
        &[
            ivn_from_acc.clone(),
            ivn_to_acc.clone(),
            pda_acc.clone(),
            token_prog_acc.clone()
        ],
        &[&["swap".as_ref(), bytes_of(&nonce)]]
    )?;
    
    Ok(())
}

