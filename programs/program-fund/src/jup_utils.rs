use bytemuck::bytes_of;
use arrayref::{array_ref, array_refs};
use fixed::types::U64F64;
use fixed_macro::types::U64F64;
use std::convert::TryInto;
use std::convert::TryFrom;


use solana_program::{
    account_info::{AccountInfo, next_account_info},
    msg,
    system_instruction::*,
    instruction:: {AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    program::invoke_signed,
    sysvar::{Sysvar, clock::Clock},
};
use crate::state::{FundData, FundAccount, PlatformData};
use crate::error::FundError;
use crate::processor::{raydium_id, orca_id, parse_token_account};
pub use switchboard_aggregator::AggregatorAccountData;



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



pub mod jupiter_pid {
    use solana_program::declare_id;
    declare_id!("JUP2jxvXaqu7NQY1GmNF4m1vodw12LVXYxbFL2uJvfo");
}

pub mod serum_dex {
    use solana_program::declare_id;
    declare_id!("9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin");
}


pub fn jup_swap(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8]
) -> Result<(), ProgramError> {
    let accounts_iter = &mut accounts.iter();
    let sysvar_ix_ai = next_account_info(accounts_iter)?;
    check_eq!(*sysvar_ix_ai.key, solana_program::sysvar::instructions::id());
    let manager_ai = next_account_info(accounts_iter)?;
    check!(manager_ai.is_signer, ProgramError::MissingRequiredSignature);
    let fund_pda_ai = next_account_info(accounts_iter)?;
    let mut fund_data = FundAccount::load_mut_checked(fund_pda_ai, program_id)?;
    check_eq!(fund_data.guard.is_active, true);
    check!(fund_data.manager_account == *manager_ai.key, FundError::ManagerMismatch);
    let pda_signer_nonce = fund_data.signer_nonce;
    let whitelisted_prog_ai = next_account_info(accounts_iter)?;
    check!(*whitelisted_prog_ai.key == jupiter_pid::ID, FundError::IncorrectProgramId);
    let (mut check_for_guard, mut index) = (false, 1);
    while !check_for_guard {
        let ix = solana_program::sysvar::instructions::get_instruction_relative(index, sysvar_ix_ai)?;
        check_for_guard = if ix.program_id == *program_id && ix.data[0] == 26 {
            true
        } else {
            index += 1; 
            false 
        }
    }
    
    let mut meta_accounts = vec![];
    
    meta_accounts.extend(accounts_iter.map(|a| {
        if *a.key == fund_data.fund_pda { // pda will sign
            AccountMeta::new(*a.key, true)
        } else if a.is_writable {
            AccountMeta::new(*a.key, a.is_signer)
        } else {
            AccountMeta::new_readonly(*a.key, a.is_signer)
        }
    }));
    let relay_instruction = Instruction {
        program_id: *whitelisted_prog_ai.key,
        accounts: meta_accounts,
        data: data.to_vec(),
    };
    drop(fund_data);
    msg!("Firing CPI");
    invoke_signed(
        &relay_instruction,
        accounts.clone(),
        &[&[&*manager_ai.key.as_ref(), bytes_of(&pda_signer_nonce)]]
    )?;

    fund_data = FundAccount::load_mut_checked(fund_pda_ai, program_id)?;
    let token_in_fund_slot = fund_data.guard.token_in_slot as usize;
    let token_out_fund_slot = fund_data.guard.token_out_slot as usize;

    Ok(())
}

pub fn init_open_order_accs(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8]
) -> Result<(), ProgramError> {
    let accounts_iter = &mut accounts.iter();
    let manager_ai = next_account_info(accounts_iter)?;
    check!(manager_ai.is_signer, ProgramError::MissingRequiredSignature);
    let fund_pda_ai = next_account_info(accounts_iter)?;
    let mut fund_data = FundAccount::load_mut_checked(fund_pda_ai, program_id)?;
    check!(fund_data.manager_account == *manager_ai.key, FundError::ManagerMismatch);
    let pda_signer_nonce = fund_data.signer_nonce;
    let whitelisted_prog_ai = next_account_info(accounts_iter)?;
    check!(*whitelisted_prog_ai.key == serum_dex::ID, FundError::IncorrectProgramId);
    let selector = array_ref![data, 1, 4];
    let discrim = u32::from_le_bytes(*selector);
    msg!("Serum Discrim:: {:?}", discrim);
    check!(discrim == 15 || discrim == 14, FundError::InvalidInstruction); //Allow InitOpenOrders and CloseOpenOrders on Serum
    let mut meta_accounts = vec![];
    
    meta_accounts.extend(accounts_iter.map(|a| {
        if *a.key == fund_data.fund_pda { // pda will sign
            AccountMeta::new(*a.key, true)
        } else if a.is_writable {
            AccountMeta::new(*a.key, a.is_signer)
        } else {
            AccountMeta::new_readonly(*a.key, a.is_signer)
        }
    }));
    let relay_instruction = Instruction {
        program_id: *whitelisted_prog_ai.key,
        accounts: meta_accounts,
        data: data.to_vec(),
    };
    drop(fund_data);
    msg!("Firing CPI");
    invoke_signed(
        &relay_instruction,
        accounts.clone(),
        &[&[&*manager_ai.key.as_ref(), bytes_of(&pda_signer_nonce)]]
    )?;
    Ok(())


}


pub fn set_swap_guard(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    token_in_fund_slot: u8,
    token_out_fund_slot: u8,
    amount_in: u64
) -> Result<(), ProgramError> {
    let accounts_iter = &mut accounts.iter();
    let platform_ai = next_account_info(accounts_iter)?;
    let manager_ai = next_account_info(accounts_iter)?;
    let fund_pda_ai = next_account_info(accounts_iter)?;
    

    let mut fund_data = FundAccount::load_mut_checked(fund_pda_ai, program_id)?;
    let platform_data = PlatformData::load_checked(platform_ai, program_id)?;
    check!(fund_data.is_initialized == true, FundError::FundAccountAlreadyInit);
    check!(fund_data.manager_account == *manager_ai.key, FundError::ManagerMismatch);
    check!(manager_ai.is_signer == true, FundError::IncorrectSignature);
    let source_token_info = platform_data.token_list[fund_data.tokens[token_in_fund_slot as usize].index[fund_data.tokens[token_in_fund_slot as usize].mux as usize] as usize];
    let dest_token_info = platform_data.token_list[fund_data.tokens[token_out_fund_slot as usize].index[fund_data.tokens[token_out_fund_slot as usize].mux as usize] as usize];

    let now_ts = Clock::get()?.unix_timestamp;
    
    
    let mut input_value = U64F64::from_num(amount_in);
    if token_in_fund_slot != 0 {
        if now_ts - source_token_info.last_updated > 100 {
            msg!("price not up-to-date.. aborting");
            return Err(FundError::PriceStaleInAccount.into())
        }
        input_value = input_value.checked_mul(source_token_info.pool_price).unwrap();
        if source_token_info.pc_index != 0 {
            let underlying_token_info = platform_data.token_list[source_token_info.pc_index as usize];
            if now_ts - underlying_token_info.last_updated > 100 {
               msg!("price not up-to-date.. aborting");
               return Err(FundError::PriceStaleInAccount.into())
           }
            input_value = input_value.checked_mul(underlying_token_info.pool_price).unwrap();
        }
    }

    fund_data.guard.input_value = input_value;

    msg!("input value: {:?}", input_value);


    
    let mut output_price = if token_out_fund_slot != 0 {
        if now_ts - dest_token_info.last_updated > 100 {
            msg!("price not up-to-date.. aborting");
            return Err(FundError::PriceStaleInAccount.into())
        }
        dest_token_info.pool_price
    } else {
        U64F64!(1)
    };
    
    if dest_token_info.pc_index != 0 {
        let underlying_token_info = platform_data.token_list[dest_token_info.pc_index as usize];
        if now_ts - underlying_token_info.last_updated > 100 {
           msg!("price not up-to-date.. aborting");
           return Err(FundError::PriceStaleInAccount.into())
       }
        output_price = output_price.checked_mul(underlying_token_info.pool_price).unwrap();
    }
    msg!("output price: {:?}", output_price);

    fund_data.guard.min_amount_out = U64F64::to_num(input_value.checked_div(output_price).unwrap().checked_mul(U64F64!(0.95)).unwrap());
    fund_data.guard.amount_in = amount_in;
    fund_data.guard.is_active = true;
    fund_data.guard.triggered_at = now_ts;
    fund_data.guard.token_in_slot = token_in_fund_slot;
    fund_data.guard.token_out_slot = token_out_fund_slot;
    msg!("amount_in {:?}, min_aount_out {:?}", fund_data.guard.amount_in, fund_data.guard.min_amount_out);

    
    Ok(())
}


pub fn check_swap_guard(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> Result<(), ProgramError> {
    let accounts_iter = &mut accounts.iter();

    let manager_ai = next_account_info(accounts_iter)?;
    check_eq!(manager_ai.is_signer, true);
    let fund_pda_ai = next_account_info(accounts_iter)?;
    let mut fund_data = FundAccount::load_mut_checked(fund_pda_ai, program_id)?;
    check_eq!(fund_data.manager_account, *manager_ai.key);
    let si = fund_data.guard.token_in_slot as usize;
    let di = fund_data.guard.token_out_slot as usize;

    check!(Clock::get()?.unix_timestamp - fund_data.guard.triggered_at < 100, FundError::PriceStaleInAccount);

    let source_token_ai = next_account_info(accounts_iter)?;
    check_eq!(fund_data.tokens[si].vault, *source_token_ai.key);
    let dest_token_ai = next_account_info(accounts_iter)?;
    check_eq!(fund_data.tokens[di].vault, *dest_token_ai.key);

    let source_amount = parse_token_account(source_token_ai)?.amount;
    let dest_amount = parse_token_account(dest_token_ai)?.amount;

    
    // let prev_amount =  fund_data.tokens[di].balance;
    
    let swap_amount_in = fund_data.tokens[si].balance - source_amount;
    let swap_amount_out = dest_amount - fund_data.tokens[di].balance;

    msg!("Checking amount_in {:?}, guard_amount_in {:?}", swap_amount_in, fund_data.guard.amount_in);
    check!(fund_data.guard.amount_in >= swap_amount_in, ProgramError::InsufficientFunds); //amountIn check
    check!(swap_amount_out >= fund_data.guard.min_amount_out, FundError::MinAmountFailed); // minAmountOut guard check

    fund_data.tokens[si].balance = source_amount;
    fund_data.tokens[di].balance = dest_amount;
    check!(fund_data.tokens[di].balance >= fund_data.tokens[di].debt, ProgramError::InsufficientFunds);
    check!(fund_data.tokens[si].balance >= fund_data.tokens[si].debt, ProgramError::InsufficientFunds);
    msg!("Vol: {:?}", fund_data.guard.input_value);
    fund_data.guard.is_active = false;
    fund_data.guard.amount_in = 0;
    fund_data.guard.min_amount_out = 0;
    fund_data.guard.triggered_at = 0;
    fund_data.guard.token_in_slot = u8::MAX;
    fund_data.guard.token_out_slot = u8::MAX;
    fund_data.guard.input_value = U64F64!(0);
    Ok(())

}
