use bytemuck::bytes_of;

use solana_program::{
    account_info::{AccountInfo, next_account_info},
    msg,
    instruction:: {AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    program::invoke_signed,
};
use crate::state::FundData;
macro_rules! check_eq {
    ($x:expr, $y:expr) => {
        if ($x != $y) {
            return Err(ProgramError::InvalidAccountData)
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

    // msg!("relay instruction:: {:?}", relay_instruction);
    // check margin account

    invoke_signed(
        &relay_instruction,
        accounts.clone(),
        &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
    )?;


    // let pda_index = accounts.iter().position(|x| *x.key == margin_pda);
    // if pda_index == None {
    //     return Ok(())
    // }
    // let margin_account = &accounts[pda_index.unwrap()];
    // msg!("margin_account: {:?}", margin_account);
    // check_eq!(margin_pda, *margin_account.key);
    // // update assets valuation and check collateral ratio
    // let mut margin_data = MarginAccount::load_mut_checked(margin_account, program_id)?;
    
    // // TODO:: need to figure out indexing of accounts
    // // where to store the mapping of indexes?
    // //  how standard?
    
    // let vault_index = accounts.iter().position(|x| *x.key == margin_data.vault);
    // if vault_index != None {
    //     margin_data.update_assets(&accounts[vault_index.unwrap()])?; 
    // }
    Ok(())
}