use bytemuck::bytes_of;
use arrayref::{array_ref, array_refs};

use solana_program::{
    account_info::{AccountInfo, next_account_info},
    msg,
    system_instruction::*,
    instruction:: {AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    program::invoke_signed,
    sysvar::Sysvar,
};
use crate::state::FundData;
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


pub fn route (
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8]
) -> Result<(), ProgramError> {
    let accounts_iter = &mut accounts.iter();
    // let sysvar_ix_ai = next_account_info(accounts_iter)?;
    // check_eq!(*sysvar_ix_ai.key, solana_program::sysvar::instructions::id());
    let manager_ai = next_account_info(accounts_iter)?;
    check_eq!(manager_ai.is_signer, true);
    let fund_state_ai = next_account_info(accounts_iter)?;
    let mut fund_data = FundData::load_mut_checked(fund_state_ai, program_id)?;
    check_eq!(fund_data.manager_account, *manager_ai.key);
    check_eq!(fund_data.guard.is_active, true);
    let whitelisted_prog_ai = next_account_info(accounts_iter)?;
    // let ix_id = solana_program::sysvar::instructions::load_current_index_checked(sysvar_ix_ai);
    // let ix_pos = solana_program::sysvar::instructions::get_instruction_relative(0, sysvar_ix_ai);
    // msg!("Ix ID: {:?}, Ix Data: {:?} ", ix_id, ix_pos);
    let mut first_two_bytes = array_ref![data, 0, 2];
    let fun_sec = u16::from_le_bytes(*first_two_bytes);
    msg!("selector: {:?} ", fun_sec);
    msg!("data: {:?}", data.to_vec());

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

    // msg!("relay instruction:: {:?}", relay_instruction);
    // check margin account

    invoke_signed(
        &relay_instruction,
        accounts.clone(),
        &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
    )?;

    if fun_sec == 21988 { //SetTokenLedger
        return Ok(());
    }
    

    // let (source_index, dest_index) = match fun_sec {

    //     46936 => {
    //         //Check for Serum
    //         if *accounts[13].key == *accounts[14].key {
    //             (14, 15)
    //         } else {
    //             (15, 14)
    //         }
    //     },

    //     58181 => {
    //         (18, 19) //Raydium
    //     },

    //     49339 => {
    //         (8, 11) //Orca
    //     },

    //     25655 => { //Step
    //         (8, 11)
    //     },

    //     59643 => { //Aldrin
    //         let check = array_ref![data, 24, 1];
    //         let x = u8::from_le_bytes(*check);
    //         if x == 1{
    //             (11, 12)
    //         } else {
    //             (12, 11)
    //         }
    //     },

    //     9895 => { //Cropper
    //         (9, 12)
    //     },

    //     63519 => { //Mercurial
    //         (8, 9)
    //     },

    //     40593 => { //Saber
    //         (9, 11)
    //     },

    //     _ => { return Err(FundError::InvalidTokenAccount) }

    // };

    // msg!("SI: {:?}, DI: {:?}", source_index, dest_index);


    //  01
    //SetTokenLedger: e455b9704e4f4d02 = 21988 
    //Serum: 58b746f9d67652d20101c00e160200000000000000000000000000 = 46936 , 11, 12, 13
    //Rayfium: 45e3625dedcadf8c = 58181 16, 17
    //Orca: bbc076d43e6d1cd50100e1f50500000000000000000000000000 = 49339 6,9
    //Step: 376411f3f2b52ba501f37eec3304000000000000000000000000 = 25655 6, 9
    //Aldrin: fbe877a6e1b9a9a10095051e02000000000100 = 59643 9, 10
    //Aldrin V2: 
    //Cropper: a7263b25843c5f44002e0476330000000000 = 9895 7, 10
    //Mercurial: 1ff83ce2d7a837c70100e40b5402000000000000000000000000 = 63519 6, 7
    //Saber: 919eb8d4034a9c76003a98f83f0000000000 = 40593 7, 9

    let token_in_fund_slot = fund_data.guard.token_in_slot as usize;
    let token_out_fund_slot = fund_data.guard.token_out_slot as usize;

    // if fund_data.guard.hop == 0 {
    //     check_eq!(fund_data.guard.token_in, *accounts[source_index].key);
    //     check_eq!(fund_data.guard.token_out, *accounts[dest_index].key);
    //     let source_token_data = parse_token_account(&accounts[source_index])?;
    //     let dest_token_data = parse_token_account(&accounts[dest_index])?;
    //     fund_data.tokens[token_in_fund_slot].balance = source_token_data.amount;
    //     fund_data.tokens[token_out_fund_slot].balance = dest_token_data.amount;
    //     check!(fund_data.tokens[token_in_fund_slot].balance >= fund_data.tokens[token_in_fund_slot].debt, ProgramError::InsufficientFunds);
    //     check!(fund_data.tokens[fund_data.guard.token_out_slot as usize].balance >= fund_data.tokens[fund_data.guard.token_out_slot as usize].debt, ProgramError::InsufficientFunds);
    // } else {
    //     if fund_data.guard.count == 0 {
    //         fund_data.guard.count = 1;
    //         check_eq!(fund_data.guard.token_in, *accounts[source_index].key);
    //         check_eq!(fund_data.guard.token_hop, *accounts[dest_index].key);
    //         let source_token_data = parse_token_account(&accounts[source_index])?;
    //         fund_data.tokens[token_in_fund_slot].balance = source_token_data.amount;
    //         check!(fund_data.tokens[token_in_fund_slot].balance >= fund_data.tokens[token_in_fund_slot].debt, ProgramError::InsufficientFunds);
    //     } else {
    //         check_eq!(fund_data.guard.token_hop, *accounts[source_index].key);
    //         check_eq!(fund_data.guard.token_out, *accounts[dest_index].key);
    //         let dest_token_data = parse_token_account(&accounts[dest_index])?;
    //         fund_data.tokens[token_out_fund_slot].balance = dest_token_data.amount;
    //         check!(fund_data.tokens[token_out_fund_slot].balance >= fund_data.tokens[token_out_fund_slot].debt, ProgramError::InsufficientFunds);
    //     }
    // }

    // if fund_data.guard.hop == fund_data.guard.count {
    //     fund_data.guard.is_active = false;
    //     fund_data.guard.token_in = Pubkey::default();
    //     fund_data.guard.token_out = Pubkey::default();
    //     fund_data.guard.token_hop = Pubkey::default();
    //     fund_data.guard.token_in_slot = u8::MAX;
    //     fund_data.guard.token_out_slot = u8::MAX;
    // }

    
    

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
    let fund_state_ai = next_account_info(accounts_iter)?;
    let manager_ai = next_account_info(accounts_iter)?;
    let fund_pda = next_account_info(accounts_iter)?;
    let source_token_ai = next_account_info(accounts_iter)?;
    let dest_token_ai = next_account_info(accounts_iter)?;

    let oracle_ai = next_account_info(accounts_iter)?;
    let oracle_ai_opt = next_account_info(accounts_iter)?;

    let hop_token_ai = next_account_info(accounts_iter)?;
    let mut fund_data = FundData::load_mut_checked(fund_state_ai, program_id)?;
    check_eq!(fund_data.is_initialized(), true);
    check_eq!(fund_data.manager_account, *manager_ai.key);
    check_eq!(manager_ai.is_signer, true);
    let platform_data = PlatformData::load_checked(platform_ai, program_id)?;
    
    let source_token_data = parse_token_account(source_token_ai)?;
    check_eq!(source_token_data.owner, *fund_pda.key);
    check_eq!(source_token_data.mint, platform_data.token_list[fund_data.tokens[token_in_fund_slot as usize].index[0] as usize].mint);
    
    let dest_token_data = parse_token_account(dest_token_ai)?;
    check_eq!(dest_token_data.owner, *fund_pda.key);
    check_eq!(source_token_data.mint, platform_data.token_list[fund_data.tokens[token_out_fund_slot as usize].index[0] as usize].mint);

    if *hop_token_ai.key != Pubkey::default() {
        fund_data.guard.hop = 1;
        fund_data.guard.count = 0;
        let hop_token_data = parse_token_account(hop_token_ai)?;
        check_eq!(hop_token_data.owner, *fund_pda.key);
    }

    // TODO: add oracle decoding to get minAmountOut
    // let feed_result_1 = AggregatorAccountData::new(oracle_ai)?.get_result()?;

    // TODO
    fund_data.guard.amount_in = amount_in;
    fund_data.guard.min_amount_out = 0;

    fund_data.guard.is_active = true;
    fund_data.guard.token_in = *source_token_ai.key; 
    fund_data.guard.token_out = *dest_token_ai.key;
    fund_data.guard.token_hop = *hop_token_ai.key;
    fund_data.guard.token_in_slot = token_in_fund_slot;
    fund_data.guard.token_out_slot = token_out_fund_slot;
    
    Ok(())
}


pub fn check_swap_guard (
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> Result<(), ProgramError> {
    let accounts_iter = &mut accounts.iter();

    // let manager_ai = next_account_info(accounts_iter)?;
    // check_eq!(manager_ai.is_signer, true);
    let fund_state_ai = next_account_info(accounts_iter)?;
    let mut fund_data = FundData::load_mut_checked(fund_state_ai, program_id)?;
    // check_eq!(fund_data.manager_account, *manager_ai.key);

    let source_token_ai = next_account_info(accounts_iter)?;
    let dest_token_ai = next_account_info(accounts_iter)?;

    let source_amount = parse_token_account(source_token_ai)?.amount;
    let dest_amount = parse_token_account(dest_token_ai)?.amount;

    let prev_amount =  fund_data.tokens[fund_data.guard.token_out_slot as usize].balance;
    
    let swap_amount_in = source_amount - fund_data.tokens[fund_data.guard.token_in_slot as usize].balance;
    let swap_amount_out = dest_amount - fund_data.tokens[fund_data.guard.token_out_slot as usize].balance;

    check_eq!(swap_amount_in, fund_data.guard.amount_in);
    check!(swap_amount_out > fund_data.guard.min_amount_out, ProgramError::InvalidAccountData); // minAmountOut guard check

    check!(fund_data.tokens[fund_data.guard.token_out_slot as usize].balance > fund_data.tokens[fund_data.guard.token_out_slot as usize].debt, ProgramError::InvalidAccountData);

    // check in_slot debt is valid
    check!(fund_data.tokens[fund_data.guard.token_in_slot as usize].balance > fund_data.tokens[fund_data.guard.token_in_slot as usize].debt, ProgramError::InvalidAccountData);

    if fund_data.guard.hop == fund_data.guard.count {
            fund_data.guard.is_active = false;
            fund_data.guard.token_in = Pubkey::default();
            fund_data.guard.token_out = Pubkey::default();
            fund_data.guard.token_hop = Pubkey::default();
            fund_data.guard.token_in_slot = u8::MAX;
            fund_data.guard.token_out_slot = u8::MAX;
    }
    else {
        return Err(ProgramError::InvalidAccountData);
    }
}

pub fn route2 (
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8]
) -> Result<(), ProgramError> {

    let accounts_iter = &mut accounts.iter();
    let manager_ai = next_account_info(accounts_iter)?;
    check_eq!(manager_ai.is_signer, true);
    let fund_state_ai = next_account_info(accounts_iter)?;
    let mut fund_data = FundData::load_mut_checked(fund_state_ai, program_id)?;
    check_eq!(fund_data.manager_account, *manager_ai.key);

    let whitelisted_prog_ai = next_account_info(accounts_iter)?;
    msg!("data: {:?}", data.to_vec());
    let mut meta_accounts = vec![];
    
    
    meta_accounts.extend(accounts_iter.map(|a| {
        if *a.key == fund_data.fund_pda { // pda will sign
            AccountMeta::new(*a.key, true)
        }
        else if a.is_writable {
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

    invoke_signed(
        &relay_instruction,
        accounts.clone(),
        &[&[&*manager_ai.key.as_ref(), bytes_of(&fund_data.signer_nonce)]],
    )?;
    
    Ok(())
}