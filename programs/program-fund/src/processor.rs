use bytemuck::bytes_of;
use fixed::types::U64F64;
use fixed_macro::types::U64F64;

use solana_program::{
    account_info::{AccountInfo, next_account_info},
    msg,
    instruction:: {AccountMeta, Instruction},
    program_error::ProgramError,
    program_pack::{Pack, IsInitialized},
    pubkey::Pubkey,
    program::{invoke, invoke_signed},
    clock::Clock,
    sysvar::Sysvar
};

use arrayref::{array_ref, array_refs};

use spl_token::state::{Account, Mint};

use crate::error::FundError;
use crate::instruction::{FundInstruction, Data};
use crate::state::{NUM_TOKENS, MAX_INVESTORS, NUM_MARGIN, FundData, InvestorData, PlatformData};
use crate::mango_utils::*;
use crate::tokens::*;
use mango::state::{MarginAccount, MangoGroup, NUM_MARKETS, load_open_orders, Loadable as MangoLoadable};

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

pub mod investin_admin {
    use solana_program::declare_id;
    // set investin admin
    // declare_id!("owZmWQkqtY3Kqnxfua1KTHtR2S6DgBTP75JKbh15VWG");
    declare_id!("B1J3ttZ2PLCG4KUQHLtkgYSZUmD6f8KNLjGSWzz8eE1Y"); // devnet admin
    
}

pub mod usdc_mint {
    use solana_program::declare_id;
    // set investin admin
    declare_id!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
}

pub mod raydium_id {
    use solana_program::declare_id;
    // set investin admin
    declare_id!("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8");
}

pub mod orca_id {
    use solana_program::declare_id;
    // set investin admin
    declare_id!("9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP");
}

pub struct Fund {}

impl Fund {
    // Fund Initialize
    pub fn initialize (
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        min_amount: u64,
        min_return: u64,
        performance_fee_percentage: u64,
        no_of_tokens: u8
    ) -> Result<(), ProgramError> {

        let accounts_iter = &mut accounts.iter();

        let platform_acc = next_account_info(accounts_iter)?;
        let fund_state_acc = next_account_info(accounts_iter)?;
        let manager_acc = next_account_info(accounts_iter)?;

        let mut platform_data = PlatformData::load_mut_checked(platform_acc, program_id)?;
        let mut fund_data = FundData::load_mut_checked(fund_state_acc, program_id)?;

        //  check if already init
        check!(!fund_data.is_initialized(), FundError::FundAccountAlreadyInit);
        //check_eq!(fund_data.version, 0);
        check!(platform_data.is_initialized(), ProgramError::InvalidAccountData);
        check!(min_return >= 500, ProgramError::InvalidArgument);
        check!(min_amount >= 10000000, ProgramError::InvalidArgument);
        check!(no_of_tokens as usize <= NUM_TOKENS, ProgramError::InvalidArgument); // max 8 tokens
        check!(performance_fee_percentage >= 100 && performance_fee_percentage <= 4000, ProgramError::InvalidArgument);

        // save manager's wallet address
        fund_data.manager_account = *manager_acc.key;
        check!(manager_acc.is_signer, ProgramError::MissingRequiredSignature);

        // get nonce for signing later
        let (pda, nonce) = Pubkey::find_program_address(&[&*manager_acc.key.as_ref()], program_id);
        fund_data.fund_pda = pda;
        fund_data.signer_nonce = nonce;

        let usdc_mint_acc = next_account_info(accounts_iter)?;
        let fund_btoken_acc = next_account_info(accounts_iter)?;

        check_eq!(platform_data.token_list[0].mint, *usdc_mint_acc.key);

        let usdc_vault = parse_token_account(fund_btoken_acc)?;
        check_eq!(usdc_vault.owner, fund_data.fund_pda);
        check_eq!(usdc_vault.mint, *usdc_mint_acc.key); // check for USDC mint

        fund_data.tokens[0].index[0] = 0;
        fund_data.tokens[0].index[1] = 0;
        fund_data.tokens[0].mux = 0;
        fund_data.tokens[0].balance = 0;
        fund_data.tokens[0].debt = 0;
        fund_data.tokens[0].is_active = true;
        fund_data.tokens[0].vault = *fund_btoken_acc.key;
        fund_data.no_of_assets = 1;

        // whitelisted tokens
        for index in 1..no_of_tokens {
            let mint_acc = next_account_info(accounts_iter)?;
            let vault_acc = next_account_info(accounts_iter)?;

            let asset_vault = parse_token_account(vault_acc)?;
            check_eq!(asset_vault.owner, fund_data.fund_pda);

            let token_index_1 = platform_data.get_token_index(mint_acc.key, 0);
            let token_index_2 = platform_data.get_token_index(mint_acc.key, 1);
            
            // both indexes cant be None
            check!(((token_index_1 != None) || (token_index_2 != None)), ProgramError::InvalidAccountData);
        
            if token_index_1 != None {
                fund_data.tokens[index as usize].mux = 0;
                fund_data.tokens[index as usize].index[0] = token_index_1.unwrap() as u8;
            }
            else {
                fund_data.tokens[index as usize].index[0] = 255; // Max u8
            }
        
            if token_index_2 != None {
                fund_data.tokens[index as usize].mux = 1;
                fund_data.tokens[index as usize].index[1] = token_index_2.unwrap() as u8;
            }
            else {
                fund_data.tokens[index as usize].index[1] = 255;
            }
        
            fund_data.tokens[index as usize].is_active = true;    
            fund_data.tokens[index as usize].balance = 0;
            fund_data.tokens[index as usize].debt = 0;
            fund_data.tokens[index as usize].vault = *vault_acc.key;
            fund_data.no_of_assets += 1;
        }

        fund_data.min_amount = min_amount;
        fund_data.min_return = U64F64::from_num(min_return / 100);
        fund_data.performance_fee_percentage = U64F64::from_num(performance_fee_percentage / 100);

        fund_data.total_amount = U64F64!(0); 
        fund_data.prev_performance = U64F64!(1.00);
        fund_data.number_of_active_investments = 0;
        fund_data.no_of_investments = 0;
        fund_data.mango_positions[0].margin_account = Pubkey::default();
        fund_data.mango_positions[1].margin_account = Pubkey::default();

        fund_data.is_initialized = true;
        fund_data.version = 1; // v1 funds

        // update platform_data
        platform_data.no_of_active_funds += 1;

        Ok(())
    }

    // investor deposit
    pub fn deposit (
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
        index: u8
    ) -> Result<(), ProgramError> {
        const NUM_FIXED:usize = 6;
        let accounts = array_ref![accounts, 0, NUM_FIXED];

        let [
            fund_state_acc,
            investor_state_acc,
            investor_acc,
            investor_btoken_acc,
            router_btoken_acc,
            token_prog_acc
        ] = accounts;

        let mut fund_data = FundData::load_mut_checked(fund_state_acc, program_id)?;
        let mut investor_data = InvestorData::load_mut_checked(investor_state_acc, program_id)?;

        // check if fund state acc passed is initialised
        check!(fund_data.is_initialized(), FundError::InvalidStateAccount);

        // let depositors: u64 = U64F64::to_num(U64F64::from_num(fund_data.no_of_investments).checked_sub(U64F64::from_num(fund_data.number_of_active_investments)).unwrap());

        // check!(depositors < 10, FundError::DepositLimitReached);
        // check if amount deposited is more than the minimum amount for the fund
        check!(amount >= fund_data.min_amount, FundError::InvalidAmount);
        // check if investor has signed the transaction
        check!(investor_acc.is_signer, FundError::IncorrectSignature);

        // check if investor_state_account is already initialised
        check!(!investor_data.is_initialized(), FundError::InvestorAccountAlreadyInit);
        
        investor_data.is_initialized = true;
        investor_data.owner = *investor_acc.key;
        // Store manager's address
        investor_data.manager = fund_data.manager_account;

        // update queue
        // let index = fund_data.no_of_investments - fund_data.number_of_active_investments;
        // queue slot should be empty
        check_eq!(fund_data.investors[index as usize], Pubkey::default());
        fund_data.investors[index as usize] = *investor_state_acc.key;
        fund_data.no_of_investments += 1;

        // check router vault account is owned by router
        let (router_pda, _nonce) = Pubkey::find_program_address(&["router".as_ref()], program_id);
        let router_owner = parse_token_account(router_btoken_acc)?.owner;
        check_eq!(router_owner, router_pda);

        check!(*token_prog_acc.key == spl_token::id(), FundError::IncorrectProgramId);

        msg!("Depositing tokens..");
        let deposit_instruction = spl_token::instruction::transfer(
            token_prog_acc.key,
            investor_btoken_acc.key,
            router_btoken_acc.key,
            investor_acc.key,
            &[&investor_acc.key],
            amount
        )?;
        let deposit_accs = [
            investor_btoken_acc.clone(),
            router_btoken_acc.clone(),
            investor_acc.clone(),
            token_prog_acc.clone()
        ];
        invoke(&deposit_instruction, &deposit_accs)?;

        msg!("Deposit done..");
        
        investor_data.amount_in_router += amount;
        fund_data.amount_in_router += amount;
    
        Ok(())
    }

    // manager transfer
    pub fn transfer (
        program_id: &Pubkey,
        accounts: &[AccountInfo]
    ) -> Result<(), ProgramError> {

        const NUM_FIXED:usize = 11;
        let (
            fixed_accs,
            margin_accs,
            open_orders_accs,
            oracle_accs,
            investor_state_accs
        ) = array_refs![accounts, NUM_FIXED, NUM_MARGIN, NUM_MARGIN, NUM_MARGIN; ..;];

        let [
            platform_acc,
            fund_state_acc,
            mango_group_acc,
            clock_sysvar_acc,
            manager_acc,
            router_btoken_acc,
            fund_btoken_acc,
            manager_btoken_acc,
            investin_btoken_acc,
            pda_router_acc,
            token_prog_acc
        ] = fixed_accs;

        let platform_data = PlatformData::load_checked(platform_acc, program_id)?;
        let mut fund_data = FundData::load_mut_checked(fund_state_acc, program_id)?;

        let mut margin_equity = U64F64!(0);
        for i in 0..NUM_MARGIN {
            if fund_data.mango_positions[i as usize].state != 0 {
                let mango_group_data = MangoGroup::load(mango_group_acc)?;
                let margin_data = MarginAccount::load(&margin_accs[i])?;
                let token_index = fund_data.mango_positions[i].margin_index as usize;
                let equity = get_margin_valuation(token_index, &mango_group_data, &margin_data, &oracle_accs[i], &open_orders_accs[i])?;
                msg!("equity now:: {:?}", equity);
                margin_equity += equity.checked_mul(fund_data.mango_positions[i].fund_share / fund_data.mango_positions[i].share_ratio).unwrap();
            }
        }

        // check if manager signed the tx
        check!(manager_acc.is_signer, FundError::IncorrectProgramId);
        check_eq!(fund_data.manager_account, *manager_acc.key);
        check!(fund_data.is_initialized(), ProgramError::AccountAlreadyInitialized);

        // check if router PDA matches
        check!(*pda_router_acc.key == platform_data.router, FundError::IncorrectPDA);

        // update start performance for investors
        update_amount_and_performance(&platform_data, &mut fund_data, &clock_sysvar_acc, margin_equity, true)?;

        let mut transferable_amount: u64 = 0;
        let mut fee: u64 = 0;

        for investor_state_acc in investor_state_accs.iter() {
            let index = fund_data.get_investor_index(investor_state_acc.key).unwrap();
            let mut investor_data = InvestorData::load_mut_checked(investor_state_acc, program_id)?;

            // validation checks
            check_eq!(fund_data.investors[index], *investor_state_acc.key);            
            check!(investor_data.amount_in_router > 0, ProgramError::InvalidAccountData);
            check_eq!(investor_data.manager, *manager_acc.key);

            investor_data.amount = U64F64::to_num(U64F64::from_num(investor_data.amount_in_router).checked_mul(U64F64!(0.98)).unwrap());

            // update transfer variables
            transferable_amount = transferable_amount.checked_add(investor_data.amount).unwrap();
            fee = fee.checked_add(U64F64::to_num(
                U64F64::from_num(investor_data.amount).checked_div(U64F64::from_num(100)).unwrap()
            )).unwrap();

            // update fund amount in router
            fund_data.amount_in_router = fund_data.amount_in_router.checked_sub(investor_data.amount_in_router).unwrap();
            
            // update investor variables
            investor_data.amount_in_router = 0;
            investor_data.start_performance = fund_data.prev_performance;

            // zero out slot
            fund_data.investors[index] = Pubkey::default();
            fund_data.number_of_active_investments += 1;
        }

        msg!("transferable amount:: {:?}", transferable_amount);

        msg!("Calling transfer instructions");
        invoke_signed(
            &(spl_token::instruction::transfer(
                token_prog_acc.key,
                router_btoken_acc.key,
                fund_btoken_acc.key,
                pda_router_acc.key,
                &[pda_router_acc.key],
                transferable_amount
            ))?,
            &[
                router_btoken_acc.clone(),
                fund_btoken_acc.clone(),
                pda_router_acc.clone(),
                token_prog_acc.clone()
            ],
            &[&["router".as_ref(), bytes_of(&platform_data.router_nonce)]]
        )?;

        msg!("Management Fee Transfer {:?}", fee);
        invoke_signed(
            &(spl_token::instruction::transfer(
                token_prog_acc.key,
                router_btoken_acc.key,
                manager_btoken_acc.key,
                pda_router_acc.key,
                &[pda_router_acc.key],
                fee
            ))?,
            &[
                router_btoken_acc.clone(),
                manager_btoken_acc.clone(),
                pda_router_acc.clone(),
                token_prog_acc.clone()
            ],
            &[&["router".as_ref(), bytes_of(&platform_data.router_nonce)]]
        )?;

        msg!("Protocol Fee Transfer");
        check_eq!(platform_data.investin_vault, *investin_btoken_acc.key);
        invoke_signed(
            &(spl_token::instruction::transfer(
                token_prog_acc.key,
                router_btoken_acc.key,
                investin_btoken_acc.key,
                pda_router_acc.key,
                &[pda_router_acc.key],
                fee
            ))?,
            &[
                router_btoken_acc.clone(),
                investin_btoken_acc.clone(),
                pda_router_acc.clone(),
                token_prog_acc.clone()
            ],
            &[&["router".as_ref(), bytes_of(&platform_data.router_nonce)]]
        )?;
        
        msg!("Transfers completed");

        fund_data.tokens[0].balance = parse_token_account(&fund_btoken_acc)?.amount;
        // dont update performance now
        update_amount_and_performance(&platform_data, &mut fund_data, &clock_sysvar_acc, margin_equity, false)?;

        Ok(())
    }
    // investor withdraw
    pub fn withdraw_from_fund(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> Result<(), ProgramError> {

        const NUM_FIXED:usize = 8;
        let accounts = array_ref![accounts, 0, NUM_FIXED + 2*NUM_TOKENS];

        let (
            fixed_accs,
            inv_token_accs,
            fund_token_accs,
        ) = array_refs![accounts, NUM_FIXED, NUM_TOKENS, NUM_TOKENS];

        let [
            platform_acc,
            fund_state_acc,
            investor_state_acc,
            investor_acc,
            router_btoken_acc,
            pda_man_acc,
            pda_router_acc,
            token_prog_acc
        ] = fixed_accs;

        let platform_data = PlatformData::load_checked(platform_acc, program_id)?;
        let mut fund_data = FundData::load_mut_checked(fund_state_acc, program_id)?;
        let mut investor_data = InvestorData::load_mut_checked(investor_state_acc, program_id)?;

        check!(investor_acc.is_signer, FundError::IncorrectSignature);
        check_eq!(investor_data.owner, *investor_acc.key);
        check_eq!(investor_data.manager, fund_data.manager_account);

        // Manager has not transferred to vault
        if investor_data.amount_in_router != 0  {
            invoke_signed(
                &(spl_token::instruction::transfer(
                    token_prog_acc.key,
                    router_btoken_acc.key,
                    inv_token_accs[0].key,
                    pda_router_acc.key,
                    &[pda_router_acc.key],
                    investor_data.amount_in_router
                ))?,
                &[
                    router_btoken_acc.clone(),
                    inv_token_accs[0].clone(),
                    pda_router_acc.clone(),
                    token_prog_acc.clone()
                ],
                &[&["router".as_ref(), bytes_of(&platform_data.router_nonce)]]
            )?;
            fund_data.amount_in_router -= investor_data.amount_in_router;
            fund_data.no_of_investments -= 1;
            investor_data.amount_in_router = 0;
            investor_data.is_initialized = false;
            let index = fund_data.get_investor_index(investor_state_acc.key).unwrap();
            fund_data.investors[index] = Pubkey::default();
            // close investor account
            close_investor_account(investor_acc, investor_state_acc)?;
        } 

        if investor_data.has_withdrawn == true {//&& investor_data.withdrawn_from_margin == false {
            for i in 0..NUM_TOKENS {
                // TODO:: check if fund_debt on inv_acc <= fund_debt on fund
                if investor_data.token_debts[i] < 10 {
                    continue;
                }
                let mint_1 = platform_data.token_list[investor_data.token_indexes[i] as usize].mint;
                let mint_2 = platform_data.token_list[fund_data.tokens[i].index[fund_data.tokens[i].mux as usize] as usize].mint;
                check_eq!(mint_1, mint_2);
                msg!("withdrawing:: {:?}", investor_data.token_debts[i]);
                msg!("balance:: {:?}", parse_token_account(&fund_token_accs[i])?.amount);
                invoke_signed(
                    &(spl_token::instruction::transfer(
                        token_prog_acc.key,
                        fund_token_accs[i].key,
                        inv_token_accs[i].key,
                        pda_man_acc.key,
                        &[pda_man_acc.key],
                        investor_data.token_debts[i]
                    ))?,
                    &[
                        fund_token_accs[i].clone(),
                        inv_token_accs[i].clone(),
                        pda_man_acc.clone(),
                        token_prog_acc.clone()
                    ],
                    &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
                )?;
                fund_data.tokens[i].balance = parse_token_account(&fund_token_accs[i])?.amount;
                fund_data.tokens[i].debt -= investor_data.token_debts[i];
                investor_data.token_debts[i] = 0;
                investor_data.token_indexes[i] = 0;
            }
            investor_data.amount = 0;
            investor_data.start_performance = U64F64!(0);
            investor_data.amount_in_router = 0;
            investor_data.has_withdrawn = false;
            investor_data.is_initialized = false;
            // check if there are no margin debts
            check_eq!(investor_data.margin_debt[0], 0);
            check_eq!(investor_data.margin_debt[1], 0);
            // close investor account
            close_investor_account(investor_acc, investor_state_acc)?;
        }
        Ok(())
    }

    pub fn withdraw_settle(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> Result<(), ProgramError> {

        const NUM_FIXED:usize = 6;
        let accounts = array_ref![accounts, 0, NUM_FIXED + 3*NUM_MARGIN];

        let (
            fixed_accs,
            margin_accs,
            open_orders_accs,
            oracle_accs
        ) = array_refs![accounts, NUM_FIXED, NUM_MARGIN, NUM_MARGIN, NUM_MARGIN];

        let [
            platform_acc,
            fund_state_acc,
            investor_state_acc,
            investor_acc,
            mango_group_acc,
            clock_sysvar_acc
        ] = fixed_accs;

        let platform_data = PlatformData::load_mut_checked(platform_acc, program_id)?;
        let mut fund_data = FundData::load_mut_checked(fund_state_acc, program_id)?;
        let mut investor_data = InvestorData::load_mut_checked(investor_state_acc, program_id)?;

        check!(investor_data.owner == *investor_acc.key, ProgramError::MissingRequiredSignature);
        check!(investor_acc.is_signer, ProgramError::MissingRequiredSignature);
        check_eq!(investor_data.manager, fund_data.manager_account);
        check_eq!(investor_data.has_withdrawn, false);

        // calculate current margin equity for fund
        let mut margin_equity = [U64F64!(0); 2];
        for i in 0..NUM_MARGIN {
            if fund_data.mango_positions[i as usize].state != 0 {
                let mango_group_data = MangoGroup::load(mango_group_acc)?;
                let margin_data = MarginAccount::load(&margin_accs[i])?;
                let token_index = fund_data.mango_positions[i].margin_index as usize;
                let equity = get_margin_valuation(token_index, &mango_group_data, &margin_data, &oracle_accs[i], &open_orders_accs[i])?;
                margin_equity[i] += equity.checked_mul(fund_data.mango_positions[i].fund_share / fund_data.mango_positions[i].share_ratio).unwrap();
            }
        }

        if investor_data.amount != 0 && investor_data.start_performance != 0 {
            update_amount_and_performance(&platform_data, &mut fund_data, &clock_sysvar_acc, margin_equity[0]+margin_equity[1], true)?;
            let share = get_share(&mut fund_data, &mut investor_data)?;
            for i in 0..NUM_TOKENS {
                let mut withdraw_amount: u64 = U64F64::to_num(
                    U64F64::from_num(fund_data.tokens[i].balance-fund_data.tokens[i].debt)
                .checked_mul(share).unwrap());
                investor_data.token_indexes[i] = fund_data.tokens[i].index[fund_data.tokens[i].mux as usize];
                if fund_data.number_of_active_investments == 1 { // ceil for last investor
                    withdraw_amount += 1; // ceil
                    if withdraw_amount + fund_data.tokens[i].debt > fund_data.tokens[i].balance {
                        withdraw_amount -= 1;
                    }
                }
                investor_data.token_debts[i] = withdraw_amount;
                fund_data.tokens[i].debt += withdraw_amount;
                check!(fund_data.tokens[i].balance >= fund_data.tokens[i].debt, ProgramError::InvalidAccountData);
            }
            // active margin trade
            for i in 0..NUM_MARGIN {
                if fund_data.mango_positions[i as usize].state != 0 {
                    if margin_equity[i] < 0.1 {
                        continue; // ignore
                    }
                    investor_data.margin_position_id[i] = fund_data.mango_positions[i].position_id as u64;
                    investor_data.margin_debt[i] = share.checked_mul(fund_data.mango_positions[i].fund_share).unwrap();
                    fund_data.mango_positions[i].fund_share = fund_data.mango_positions[i].fund_share.checked_sub(
                        investor_data.margin_debt[i]).unwrap();
                    // update margin equity for current withdrawal
                    investor_data.withdrawn_from_margin = true;
                    margin_equity[i] = margin_equity[i].checked_sub(margin_equity[i].checked_mul(share).unwrap()).unwrap();
                }
            }
            fund_data.number_of_active_investments -= 1;
            fund_data.no_of_investments -= 1;
            investor_data.has_withdrawn = true;
            update_amount_and_performance(&platform_data, &mut fund_data, &clock_sysvar_acc, margin_equity[0]+margin_equity[1], false)?;
        }
        Ok(())
    }

    pub fn swap(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        swap_index: u8,
        data: Data
    ) -> Result<(), ProgramError> {
                
        let accounts_iter = &mut accounts.iter();
        let platform_state_acc = next_account_info(accounts_iter)?;

        let fund_state_acc = next_account_info(accounts_iter)?;
        let platform_data = PlatformData::load_checked(platform_state_acc, program_id)?;
        let mut fund_data = FundData::load_mut_checked(fund_state_acc, program_id)?;

        check!(platform_data.is_initialized(), ProgramError::InvalidAccountData);
        check!(fund_data.is_initialized(), ProgramError::InvalidAccountData);
        check!(swap_index < 2, ProgramError::InvalidArgument);

        let (source_info, dest_info) = match swap_index {
            0 => swap_instruction_raydium(&data, &fund_data, accounts)?,
            1 => swap_instruction_orca(&data, &fund_data, accounts)?,
            _ => return Err(ProgramError::InvalidArgument)
        };

        let source_index = platform_data.get_token_index(&source_info.mint, swap_index);
        let dest_index = platform_data.get_token_index(&dest_info.mint, swap_index);
        msg!("source_index:: {:?} ", source_index);
        msg!("dest index:: {:?}", dest_index);

        msg!("source mint:: {:?}", source_info.mint);
        msg!("dest mint:: {:?}", dest_info.mint);

        if source_info.mint == usdc_mint::ID {
            let di = fund_data.get_token_slot(dest_index.unwrap(), swap_index as usize).unwrap();
            fund_data.tokens[0].balance = source_info.amount;
            fund_data.tokens[di].balance = dest_info.amount;
            fund_data.tokens[di].mux = swap_index;
            // check the balance validity
            check!(fund_data.tokens[di].balance >= fund_data.tokens[di].debt, ProgramError::InsufficientFunds);
        }
        else if dest_info.mint == usdc_mint::ID {
            let si = fund_data.get_token_slot(source_index.unwrap(), swap_index as usize).unwrap();
            fund_data.tokens[0].balance = dest_info.amount;
            fund_data.tokens[si].balance = source_info.amount;
            fund_data.tokens[si].mux = swap_index;
            // check balance validity
            check!(fund_data.tokens[si].balance >= fund_data.tokens[si].debt, ProgramError::InsufficientFunds);
        }
        else {
            let si = fund_data.get_token_slot(source_index.unwrap(), swap_index as usize).unwrap();
            let di = fund_data.get_token_slot(dest_index.unwrap(), swap_index as usize).unwrap();

            fund_data.tokens[si].balance = source_info.amount;
            fund_data.tokens[di].balance = dest_info.amount;

            fund_data.tokens[si].mux = swap_index;
            fund_data.tokens[di].mux = swap_index;

            check!(fund_data.tokens[si].balance >= fund_data.tokens[si].debt, ProgramError::InsufficientFunds);
            check!(fund_data.tokens[di].balance >= fund_data.tokens[di].debt, ProgramError::InsufficientFunds);
        }

        // check USDC balance validity
        check!(fund_data.tokens[0].balance >= fund_data.tokens[0].debt, ProgramError::InsufficientFunds);

        Ok(())
    }

    // manager Performance Fee Claim
    pub fn claim (
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> Result<(), ProgramError> {
        const NUM_FIXED:usize = 10;
        let accounts = array_ref![accounts, 0, NUM_FIXED + 3*NUM_MARGIN];

        let (
            fixed_accs,
            margin_accs,
            open_orders_accs,
            oracle_accs
        ) = array_refs![accounts, NUM_FIXED, NUM_MARGIN, NUM_MARGIN, NUM_MARGIN];

        let [
            platform_acc,
            fund_state_acc,
            mango_group_acc,
            clock_sysvar_acc,
            manager_acc,
            fund_btoken_acc,
            manager_btoken_acc,
            investin_btoken_acc,
            pda_man_acc,
            token_prog_acc
        ] = fixed_accs;

        let mut fund_data = FundData::load_mut_checked(fund_state_acc, program_id)?;
        let platform_data = PlatformData::load_checked(platform_acc, program_id)?;


        let mut margin_equity = U64F64!(0);
        for i in 0..NUM_MARGIN {
            if fund_data.mango_positions[i as usize].state != 0 {
                let mango_group_data = MangoGroup::load(mango_group_acc)?;
                let margin_data = MarginAccount::load(&margin_accs[i])?;
                let token_index = fund_data.mango_positions[i].margin_index as usize;
                let equity = get_margin_valuation(token_index, &mango_group_data, &margin_data, &oracle_accs[i], &open_orders_accs[i])?;
                margin_equity += equity.checked_mul(fund_data.mango_positions[i].fund_share / fund_data.mango_positions[i].share_ratio).unwrap();
            }
        }
        // check if manager signed the tx
        check!(manager_acc.is_signer, FundError::IncorrectSignature);
        check_eq!(fund_data.manager_account, *manager_acc.key);

        update_amount_and_performance(&platform_data, &mut fund_data, &clock_sysvar_acc, margin_equity, true)?;

        msg!("Invoking transfer instructions");
        let performance_fee_manager: u64 = U64F64::to_num(U64F64::from_num(fund_data.performance_fee)
        .checked_mul(U64F64::from_num(90)).unwrap()
        .checked_div(U64F64::from_num(100)).unwrap());

        let transfer_instruction = spl_token::instruction::transfer(
            token_prog_acc.key,
            fund_btoken_acc.key,
            manager_btoken_acc.key,
            pda_man_acc.key,
            &[pda_man_acc.key],
            performance_fee_manager
        )?;
        let transfer_accs = [
            fund_btoken_acc.clone(),
            manager_btoken_acc.clone(),
            pda_man_acc.clone(),
            token_prog_acc.clone()
        ];
        let signer_seeds = [fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)];
        invoke_signed(&transfer_instruction, &transfer_accs, &[&signer_seeds])?;

        let performance_fee_investin: u64 = U64F64::to_num(U64F64::from_num(fund_data.performance_fee)
        .checked_div(U64F64::from_num(10)).unwrap());
        check_eq!(platform_data.investin_vault, *investin_btoken_acc.key);
        let transfer_instruction = spl_token::instruction::transfer(
            token_prog_acc.key,
            fund_btoken_acc.key,
            investin_btoken_acc.key,
            pda_man_acc.key,
            &[pda_man_acc.key],
            performance_fee_investin
        )?;
        let transfer_accs = [
            fund_btoken_acc.clone(),
            investin_btoken_acc.clone(),
            pda_man_acc.clone(),
            token_prog_acc.clone()
        ];
        let signer_seeds = [fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)];
        invoke_signed(&transfer_instruction, &transfer_accs, &[&signer_seeds])?;
        msg!("Transfer Complete");

        fund_data.tokens[0].balance = parse_token_account(&fund_btoken_acc)?.amount;
        fund_data.performance_fee = U64F64!(0);

        update_amount_and_performance(&platform_data, &mut fund_data, &clock_sysvar_acc, margin_equity, false)?;
        
        Ok(())

    }

    pub fn flush_debts (
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        index: u8,
        count: u8
    ) -> Result<(), ProgramError> {

        let accounts_iter = &mut accounts.iter();
        let platform_state_acc = next_account_info(accounts_iter)?;
        let fund_state_acc = next_account_info(accounts_iter)?;
        let manager_acc = next_account_info(accounts_iter)?;
        let vault_acc = next_account_info(accounts_iter)?;
        let pda_man_acc = next_account_info(accounts_iter)?;
        let token_prog_acc = next_account_info(accounts_iter)?;

        let platform_data = PlatformData::load_checked(platform_state_acc, program_id)?;
        let mut fund_data = FundData::load_mut_checked(fund_state_acc, program_id)?;

        check_eq!(manager_acc.is_signer, true);
        check_eq!(fund_data.manager_account, *manager_acc.key);
        check_eq!(fund_data.fund_pda, *pda_man_acc.key);

        check_eq!(fund_data.tokens[index as usize].is_active, true);
        check_eq!(fund_data.tokens[index as usize].vault, *vault_acc.key);

        let token_mint = platform_data.token_list[fund_data.tokens[index as usize].index[fund_data.tokens[index as usize].mux as usize] as usize].mint;

        for i in 0..count {
            let investor_state_acc = next_account_info(accounts_iter)?;
            let investor_token_acc = next_account_info(accounts_iter)?;

            let mut investor_data = InvestorData::load_mut_checked(investor_state_acc, program_id)?;
            let mint_1 = platform_data.token_list[investor_data.token_indexes[index as usize] as usize].mint;

            // validation checks
            check_eq!(investor_data.manager, *manager_acc.key);
            check_eq!(parse_token_account(investor_token_acc)?.owner, investor_data.owner);
            check_eq!(token_mint, mint_1);

            invoke_signed(
                &(spl_token::instruction::transfer(
                    token_prog_acc.key,
                    vault_acc.key,
                    investor_token_acc.key,
                    pda_man_acc.key,
                    &[pda_man_acc.key],
                    investor_data.token_debts[index as usize]
                ))?,
                &[
                    vault_acc.clone(),
                    investor_token_acc.clone(),
                    pda_man_acc.clone(),
                    token_prog_acc.clone()
                ],
                &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
            )?;
            fund_data.tokens[index as usize].debt -= investor_data.token_debts[index as usize];
            investor_data.token_debts[index as usize] = 0;
            investor_data.token_indexes[index as usize] = 0;
            
        }
        fund_data.tokens[index as usize].balance = parse_token_account(vault_acc)?.amount;

        Ok(())
    }

    pub fn admin_control (
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        intialize_platform: u8,
        freeze_platform: u8,
        unfreeze_platform: u8,
        change_vault: u8,
        freeze_fund: u8,
        unfreeze_fund: u8,
        change_min_amount: u64,
        change_min_return: u64,
        change_perf_fee: u64
    ) -> Result<(), ProgramError> {
        let accounts_iter = &mut accounts.iter();

        let platform_state_acc = next_account_info(accounts_iter)?;
        let investin_admin_acc = next_account_info(accounts_iter)?;
        let investin_vault_acc = next_account_info(accounts_iter)?;
        let mint_acc = next_account_info(accounts_iter)?;

        let mut platform_data = PlatformData::load_mut_checked(platform_state_acc, program_id)?;
        check!(investin_admin_acc.is_signer, FundError::IncorrectSignature);

        //check_eq!(investin_admin::ID, *investin_admin_acc.key);

        if intialize_platform == 1 {
            platform_data.is_initialized = true;
            platform_data.version = 1;
            platform_data.no_of_active_funds = 0;

            // add router pda
            let (router_pda, nonce) = 
                Pubkey::find_program_address(&["router".as_ref()], program_id
            );
            platform_data.router = router_pda;
            platform_data.router_nonce = nonce;

            // add investin accs
            platform_data.investin_admin = *investin_admin_acc.key;
            platform_data.investin_vault = *investin_vault_acc.key;

            // add USDC as base token
            let mint_info = Mint::unpack(&mint_acc.data.borrow())?;
            platform_data.token_list[0].mint = *mint_acc.key;
            platform_data.token_list[0].decimals = mint_info.decimals as u64;
            platform_data.token_list[0].pool_coin_account = Pubkey::default();
            platform_data.token_list[0].pool_pc_account = Pubkey::default();
            platform_data.token_list[0].pool_price = U64F64!(0);
            platform_data.token_count = 1;
        }
        msg!("done");
        // freeze the platform
        if freeze_platform == 1 {
            platform_data.is_initialized = false;
        }
        if unfreeze_platform == 1 {
            platform_data.is_initialized = true;
        }
        if change_vault == 1 {
            check!(*investin_vault_acc.key != Pubkey::default(), ProgramError::InvalidArgument);
            platform_data.investin_vault = *investin_vault_acc.key;
        }
        if freeze_fund == 1 {
            let fund_state_acc = next_account_info(accounts_iter)?;
            let mut fund_data = FundData::load_mut_checked(fund_state_acc, program_id)?;
            fund_data.is_initialized = false;
        }
        if unfreeze_fund == 1 {
            let fund_state_acc = next_account_info(accounts_iter)?;
            let mut fund_data = FundData::load_mut_checked(fund_state_acc, program_id)?;
            fund_data.is_initialized = true;
        }
        if change_min_amount > 0 {
            let fund_state_acc = next_account_info(accounts_iter)?;
            let mut fund_data = FundData::load_mut_checked(fund_state_acc, program_id)?;
            fund_data.min_amount = change_min_amount;
        }
        if change_min_return > 0 {
            let fund_state_acc = next_account_info(accounts_iter)?;
            let mut fund_data = FundData::load_mut_checked(fund_state_acc, program_id)?;
            fund_data.min_return = U64F64::from_num(change_min_return / 100);
        }
        if change_perf_fee > 0 {
            let fund_state_acc = next_account_info(accounts_iter)?;
            let mut fund_data = FundData::load_mut_checked(fund_state_acc, program_id)?;
            fund_data.performance_fee_percentage = U64F64::from_num(change_perf_fee / 100);
        }

        Ok(())
    }

    // instruction processor
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        data: &[u8]
    ) -> Result<(), ProgramError> {
        let instruction = FundInstruction::unpack(data).ok_or(ProgramError::InvalidInstructionData)?;
        match instruction {
            FundInstruction::Initialize { min_amount, min_return, performance_fee_percentage, no_of_tokens } => {
                msg!("FundInstruction::Initialize");
                return Self::initialize(program_id, accounts, min_amount, min_return, performance_fee_percentage, no_of_tokens);
            }
            FundInstruction::InvestorDeposit { amount, index } => {
                msg!("FundInstruction::InvestorDeposit");
                return Self::deposit(program_id, accounts, amount, index);
            }
            FundInstruction::ManagerTransfer => {
                msg!("FundInstruction::ManagerTransfer");
                return Self::transfer(program_id, accounts);
            }
            FundInstruction::InvestorWithdrawFromFund => {
                msg!("FundInstruction::InvestorWithdraw");
                return Self::withdraw_from_fund(program_id, accounts);
            }
            FundInstruction::InvestorWithdrawSettleFunds => {
                msg!("FundInstruction::InvestorWithdraw");
                return Self::withdraw_settle(program_id, accounts);
            }
            FundInstruction::Swap { swap_index, data } => {
                msg!("FundInstruction::Swap");
                return Self::swap(program_id, accounts, swap_index, data);
            }
            FundInstruction::ClaimPerformanceFee {} => {
                msg!("FundInstruction::ClaimPerformanceFee");
                return Self::claim(program_id, accounts);
            }
            FundInstruction::MangoInitialize  => {
                msg!("FundInstruction::MangoInitialize");
                return mango_init_margin_account(program_id, accounts);
            }
            FundInstruction::AdminControl { intialize_platform,
                freeze_platform, unfreeze_platform, change_vault, freeze_fund, unfreeze_fund,
                change_min_amount, change_min_return, change_perf_fee } => {
                msg!("FundInstruction::AdminControl");
                return Self::admin_control(
                    program_id,
                    accounts,
                    intialize_platform,
                    freeze_platform,
                    unfreeze_platform,
                    change_vault,
                    freeze_fund,
                    unfreeze_fund,
                    change_min_amount,
                    change_min_return,
                    change_perf_fee
                );
            }
            FundInstruction::MangoDeposit { quantity } => {
                msg!("FundInstruction::MangoDeposit");
                return mango_deposit(program_id, accounts, quantity);
            }
            FundInstruction::MangoOpenPosition { side, price, trade_size } => {
                msg!("FundInstruction::MangoPlaceOrder");
                return mango_open_position(program_id, accounts, side, price, trade_size);
            }
            FundInstruction::MangoSettlePosition => {
                msg!("FundInstruction::MangoSettleFunds");
                return mango_settle_position(program_id, accounts);
            }
            FundInstruction::MangoClosePosition { price } => {
                msg!("FundInstruction::MangoClosePosition");
                return mango_close_position(program_id, accounts, price);
            }
            FundInstruction::MangoWithdrawToFund => {
                msg!("FundInstruction::MangoWithdrawToFund");
                return mango_withdraw_fund(program_id, accounts);
            }
            FundInstruction::MangoWithdrawInvestor => {
                msg!("FundInstruction::MangoWithdrawInvestor");
                return mango_withdraw_investor(program_id, accounts);
            }
            FundInstruction::MangoWithdrawInvestorPlaceOrder { price } => {
                msg!("FundInstruction::MangoWithdrawInvestorPlaceOrder");
                return mango_withdraw_investor_place_order(program_id, accounts, price);
            }
            FundInstruction::MangoWithdrawInvestorSettle => {
                msg!("FundInstruction::MangoWithdrawInvestorSettle");
                return mango_withdraw_investor_settle(program_id, accounts);
            }
            FundInstruction::AddTokenToWhitelist { token_id, pc_index} => {
                msg!("FundInstruction::AddTokenToWhitelist");
                return add_token_to_whitelist(program_id, accounts, token_id, pc_index);
            }
            FundInstruction::UpdateTokenPrices { count } => {
                msg!("FundInstruction::UpdateTokenPrices");
                return update_token_prices(program_id, accounts, count);
            }
            FundInstruction::AddTokenToFund { index } => {
                msg!("FundInstruction::AddTokenToFund");
                return add_token_to_fund(program_id, accounts, index);
            }
            FundInstruction::RemoveTokenFromFund {index} => {
                msg!("FundInstruction::RemoveTokenFromFund");
                return remove_token_from_fund(program_id, accounts, index);
            }
            FundInstruction::FlushDebts {index, count} => {
                msg!("FundInstruction::FlushDebts");
                return Self::flush_debts(program_id, accounts, index, count);
            }
        }
    }
}

// calculate prices, get fund valuation and performance
pub fn update_amount_and_performance(
    platform_data: &PlatformData,
    fund_data: &mut FundData,
    clock_sysvar_acc: &AccountInfo,
    margin_equity: U64F64,
    update_perf: bool
) -> Result<(), ProgramError> {
    
    msg!("called update_amount_and_performance");

    // get price account info
    let clock = &Clock::from_account_info(clock_sysvar_acc)?;

    // add USDT balance (not decimal adjusted)
    let mut fund_val = U64F64::from_num(fund_data.tokens[0].balance - fund_data.tokens[0].debt).
        checked_add(margin_equity).unwrap();
    msg!("margin_equity:: {:?}", margin_equity);

    // Calculate prices for all tokens with balances
    for i in 1..NUM_TOKENS {

        // dont update if token balance == 0
        if (fund_data.tokens[i].balance - fund_data.tokens[i].debt) == 0 { continue; }

        // get last mux
        let mux = fund_data.tokens[i].mux as usize;
        // get index of token
        let token_info = platform_data.token_list[fund_data.tokens[i].index[mux] as usize];

        if clock.unix_timestamp - token_info.last_updated > 100 {
            msg!("price not up-to-date.. aborting");
            return Err(FundError::PriceStaleInAccount.into())
        }
        // calculate price in terms of base token
        let mut val: U64F64 = U64F64::from_num(fund_data.tokens[i].balance - fund_data.tokens[i].debt)
        .checked_mul(token_info.pool_price).unwrap();

         if token_info.pc_index != 0 {
             val = val.checked_mul(platform_data.token_list[token_info.pc_index as usize].pool_price).unwrap();
         }

        fund_val = fund_val.checked_add(val).unwrap();
    }
    if update_perf {
        let mut perf = U64F64::from_num(fund_data.prev_performance);
        // only case where performance is not updated:
        // when no investments and no performance fee for manager
        if fund_data.number_of_active_investments != 0 || fund_data.performance_fee != 0 {
            perf = fund_val.checked_div(fund_data.total_amount).unwrap()
            .checked_mul(U64F64::from_num(fund_data.prev_performance)).unwrap();
        }
        // adjust for manager performance fee
        fund_data.performance_fee = U64F64::to_num(U64F64::from_num(perf)
            .checked_div(U64F64::from_num(fund_data.prev_performance)).unwrap()
            .checked_mul(U64F64::from_num(fund_data.performance_fee)).unwrap());
        fund_data.prev_performance = U64F64::to_num(perf);
    }
    
    fund_data.total_amount = fund_val;
    
    msg!("updated amount: {:?}", fund_data.total_amount);
    
    Ok(())
}
pub fn get_price(
    mango_group: &MangoGroup,
    oracle_acc: &AccountInfo,
    token_index: usize
) -> Result<U64F64, ProgramError> {
    let mut price = U64F64!(0);
    let quote_decimals: u8 = mango_group.mint_decimals[NUM_MARKETS];

    check_eq!(mango_group.oracles[token_index], *oracle_acc.key);

    // TODO store this info in MangoGroup, first make sure it cannot be changed by solink
    let quote_adj = U64F64::from_num(
        10u64.pow(quote_decimals.checked_sub(mango_group.oracle_decimals[token_index]).unwrap() as u32)
    );
    let answer = flux_aggregator::read_median(oracle_acc)?; // this is in USD cents

    let value = U64F64::from_num(answer.median);

    let base_adj = U64F64::from_num(10u64.pow(mango_group.mint_decimals[token_index] as u32));
    price = quote_adj.checked_div(base_adj).unwrap().checked_mul(value).unwrap();

    Ok(price)
}

pub fn get_assets_liabs(
    token_index: usize,
    mango_group: &MangoGroup,
    margin_account: &MarginAccount,
    oracle_accs: &AccountInfo,
    open_orders_acc: &AccountInfo
) -> Result<(U64F64, U64F64), ProgramError> {

    // asset valuation
    let price = get_price(mango_group, oracle_accs, token_index)?;
    let mut assets = mango_group.indexes[token_index].deposit
    .checked_mul(margin_account.deposits[token_index]).unwrap()
    .checked_mul(price).unwrap();

    // USDC assets
    assets =  mango_group.indexes[NUM_MARKETS].deposit
    .checked_mul(margin_account.deposits[NUM_MARKETS]).unwrap()
    .checked_add(assets).unwrap();

    // open orders assets
    if *open_orders_acc.key != Pubkey::default() {
        let open_orders = load_open_orders(open_orders_acc)?;
        assets = U64F64::from_num(open_orders.native_coin_total)
        .checked_mul(price).unwrap()
        .checked_add(U64F64::from_num(open_orders.native_pc_total + open_orders.referrer_rebates_accrued)).unwrap()
        .checked_add(assets).unwrap();
    }
    
    // token liabs
    let mut liabs = mango_group.indexes[token_index].borrow
    .checked_mul(margin_account.borrows[token_index]).unwrap()
    .checked_mul(price).unwrap();

    // USDC liabs
    liabs = mango_group.indexes[NUM_MARKETS].borrow
    .checked_mul(margin_account.borrows[NUM_MARKETS]).unwrap()
    .checked_add(liabs).unwrap();
    Ok((assets, liabs))
}

pub fn get_margin_valuation(
    token_index: usize,
    mango_group: &MangoGroup,
    margin_account: &MarginAccount,
    oracle_accs: &AccountInfo,
    open_orders_acc: &AccountInfo
) -> Result<U64F64, ProgramError> {
    let (assets, liabs) = get_assets_liabs(token_index, mango_group, margin_account, oracle_accs, open_orders_acc)?;
    if liabs > assets {
        Ok(U64F64!(0))
    }
    else {
        Ok(assets - liabs)
    }
}

pub fn get_equity_and_coll_ratio(
    token_index: usize,
    mango_group: &MangoGroup,
    margin_account: &MarginAccount,
    oracle_accs: &AccountInfo,
    open_orders_acc: &AccountInfo
) -> Result<(U64F64, U64F64), ProgramError> {
    let (assets, liabs) = get_assets_liabs(token_index, mango_group, margin_account, oracle_accs, open_orders_acc)?;
    let mut coll_ratio = U64F64::MAX;
    if liabs != 0 {
        coll_ratio = assets.checked_div(liabs).unwrap();
    }
    if liabs > assets {
        Ok((U64F64!(0), coll_ratio))
    }
    else {
        Ok((assets - liabs, coll_ratio))
    }
}

pub fn parse_token_account (account_info: &AccountInfo) -> Result<Account, ProgramError> {
    if account_info.owner != &spl_token::ID {
        msg!("Account not owned by spl-token program");
        return Err(ProgramError::IncorrectProgramId);
    }
    let parsed = Account::unpack(&account_info.try_borrow_data()?)?;
    if !parsed.is_initialized() {
        msg!("Token account not initialized");
        return Err(ProgramError::UninitializedAccount);
    }
    Ok(parsed)
}

pub fn check_owner(account_info: &AccountInfo, key: &Pubkey) -> Result<(), ProgramError>{
    let owner = parse_token_account(account_info)?.owner;
    check!(owner == *key, FundError::InvalidTokenAccount);
    Ok(())
}

pub fn swap_instruction_raydium(
    data: &Data,
    fund_data: &FundData,
    accounts: &[AccountInfo]
) -> Result<(Account, Account), ProgramError>{
    let accounts = array_ref![accounts, 0, 22];
    let [
        _platform_state_acc,
        _fund_state_acc,
        manager_acc,
        pool_prog_acc,
        token_prog_acc,
        amm_id,
        amm_authority,
        amm_open_orders,
        amm_target_orders,
        pool_coin_token_acc,
        pool_pc_token_acc,
        dex_prog_acc,
        dex_market_acc,
        bids_acc,
        asks_acc,
        event_queue_acc,
        coin_vault_acc,
        pc_vault_acc,
        signer_acc,
        source_token_acc,
        dest_token_acc,
        owner_token_acc
    ] = accounts;

    invoke_signed(
        &(Instruction::new_with_borsh(
            *pool_prog_acc.key,
            &data,
            vec![
                AccountMeta::new_readonly(*token_prog_acc.key, false),
                AccountMeta::new(*amm_id.key, false),
                AccountMeta::new(*amm_authority.key, false),
                AccountMeta::new(*amm_open_orders.key, false),
                AccountMeta::new(*amm_target_orders.key, false),
                AccountMeta::new(*pool_coin_token_acc.key, false),
                AccountMeta::new(*pool_pc_token_acc.key, false),
                AccountMeta::new_readonly(*dex_prog_acc.key, false),
                AccountMeta::new(*dex_market_acc.key, false),
                AccountMeta::new(*bids_acc.key, false),
                AccountMeta::new(*asks_acc.key, false),
                AccountMeta::new(*event_queue_acc.key, false),
                AccountMeta::new(*coin_vault_acc.key, false),
                AccountMeta::new(*pc_vault_acc.key, false),
                AccountMeta::new(*signer_acc.key, false),
                AccountMeta::new(*source_token_acc.key, false),
                AccountMeta::new(*dest_token_acc.key, false),
                AccountMeta::new(*owner_token_acc.key, true)
            ],
        )),
        &[
            token_prog_acc.clone(),
            amm_id.clone(),
            amm_authority.clone(),
            amm_open_orders.clone(),
            amm_target_orders.clone(),
            pool_coin_token_acc.clone(),
            pool_pc_token_acc.clone(),
            dex_prog_acc.clone(),
            dex_market_acc.clone(),
            bids_acc.clone(),
            asks_acc.clone(),
            event_queue_acc.clone(),
            coin_vault_acc.clone(),
            pc_vault_acc.clone(),
            signer_acc.clone(),
            source_token_acc.clone(),
            dest_token_acc.clone(),
            owner_token_acc.clone(),
        ],
        &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
    )?;
    msg!("swap instruction done");

    let source_info = parse_token_account(source_token_acc)?;
    let dest_info = parse_token_account(dest_token_acc)?;

    // owner checks
    check_eq!(source_info.owner, fund_data.fund_pda);
    check_eq!(dest_info.owner, fund_data.fund_pda);

    // check program id
    check_eq!(*pool_prog_acc.key, raydium_id::ID);

    check_eq!(fund_data.manager_account, *manager_acc.key);
    check_eq!(manager_acc.is_signer, true);

    Ok((source_info, dest_info))
}

pub fn swap_instruction_orca(
    data: &Data,
    fund_data: &FundData,
    accounts: &[AccountInfo]
) -> Result<(Account, Account), ProgramError>{

    let accounts = array_ref![accounts, 0, 14];
    let [
        _platform_state_acc,
        _fund_state_acc,
        manager_acc,
        orca_prog_id,
        swap_acc,
        swap_authority,
        fund_pda_acc, // take this as transfer authority
        user_source,
        pool_source,
        pool_dest,
        user_dest,
        pool_mint,
        fee_account,
        token_prog_acc
    ] = accounts;

    invoke_signed(
        &(Instruction::new_with_borsh(
            *orca_prog_id.key,
            &data,
            vec![
                AccountMeta::new_readonly(*swap_acc.key, false),
                AccountMeta::new_readonly(*swap_authority.key, false),
                AccountMeta::new_readonly(*fund_pda_acc.key, true),
                AccountMeta::new(*user_source.key, false),
                AccountMeta::new(*pool_source.key, false),
                AccountMeta::new(*pool_dest.key, false),
                AccountMeta::new(*user_dest.key, false),
                AccountMeta::new(*pool_mint.key, false),
                AccountMeta::new(*fee_account.key, false),
                AccountMeta::new_readonly(*token_prog_acc.key, false)
            ],
        )),
        &[
            swap_acc.clone(),
            swap_authority.clone(),
            fund_pda_acc.clone(),
            user_source.clone(),
            pool_source.clone(),
            pool_dest.clone(),
            user_dest.clone(),
            pool_mint.clone(),
            fee_account.clone(),
            token_prog_acc.clone(),
        ],
        &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
    )?;
    msg!("swap instruction done");

    let source_info = parse_token_account(user_source)?;
    let dest_info = parse_token_account(user_dest)?;

     // owner checks
    check_eq!(source_info.owner, fund_data.fund_pda);
    check_eq!(dest_info.owner, fund_data.fund_pda);

    // check program id
    check_eq!(*orca_prog_id.key, orca_id::ID);

    check_eq!(fund_data.manager_account, *manager_acc.key);
    check_eq!(manager_acc.is_signer, true);

    Ok((source_info, dest_info))
}
pub fn get_share(
    fund_data: &mut FundData,
    investor_data: &mut InvestorData,
) -> Result<U64F64, ProgramError> {
    let perf_share = U64F64::from_num(fund_data.prev_performance)
    .checked_div(U64F64::from_num(investor_data.start_performance)).unwrap();

    msg!("performance: {:?}", perf_share);

    let actual_amount: u64 = investor_data.amount;

    let mut investment_return = U64F64::from_num(actual_amount)
    .checked_mul(perf_share).unwrap();

    // check if withdraw exceed
    // check!(amount <= U64F64::to_num(total_share), ProgramError::InsufficientFunds);

    // in case of profit
    if investment_return > actual_amount {
        let profit = U64F64::from_num(investment_return)
        .checked_sub(U64F64::from_num(actual_amount)).unwrap();
        let performance: u64 = U64F64::to_num(profit.checked_div(U64F64::from_num(actual_amount)).unwrap()
        .checked_mul(U64F64::from_num(100)).unwrap());
        // if performance exceeds min return; update manager performance fees

        // TODO xoheb jo bola woh idher karna hai (trimming)
        if performance >= fund_data.min_return {
            investment_return = U64F64::from_num(profit)
            .checked_mul(
                (U64F64::from_num(100).checked_sub(fund_data.performance_fee_percentage).unwrap())
                .checked_div(U64F64::from_num(100)).unwrap()
                ).unwrap()
            .checked_add(U64F64::from_num(actual_amount)).unwrap();

            fund_data.performance_fee = U64F64::to_num(U64F64::from_num(fund_data.performance_fee)
            .checked_add(U64F64::from_num(profit)
            .checked_mul(
                U64F64::from_num(fund_data.performance_fee_percentage)
                .checked_div(U64F64::from_num(100)).unwrap()
            ).unwrap()).unwrap()
            );
        }
    }

    let share = U64F64::from_num(investment_return)
    .checked_div(fund_data.total_amount).unwrap();

    Ok(share)
}

pub fn close_investor_account (
    investor_acc: &AccountInfo,
    investor_state_acc: &AccountInfo
)-> Result<(), ProgramError> {

    let dest_starting_lamports = investor_acc.lamports();
    **investor_acc.lamports.borrow_mut() = dest_starting_lamports
            .checked_add(investor_state_acc.lamports())
            .ok_or(ProgramError::AccountBorrowFailed)?;
    **investor_state_acc.lamports.borrow_mut() = 0;

    Ok(())
}