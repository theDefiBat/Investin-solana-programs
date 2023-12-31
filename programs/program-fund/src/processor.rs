use bytemuck::bytes_of;
use std::{mem::size_of, sync::mpsc::RecvTimeoutError, str::FromStr};
use fixed::types::U64F64;
use fixed_macro::types::U64F64;
use fixed::types::I80F48;
use fixed_macro::types::I80F48;
use fixed::traits::FromFixed;
use num_enum::TryFromPrimitive;
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    msg,
    log::sol_log_compute_units,    
    system_instruction::create_account,
    instruction:: {AccountMeta, Instruction},
    program_error::ProgramError,
    program_pack::{Pack, IsInitialized},
    pubkey::Pubkey,
    program::{invoke, invoke_signed},
    sysvar::{clock::Clock, rent::Rent, Sysvar}
};
use bincode::serialize;

// use anchor_lang::prelude::*;
use anchor_lang::{prelude::CpiContext, AnchorDeserialize};
use arrayref::{array_ref, array_refs};
use spl_token::state::{Account, Mint};

use mango::{state::{MangoAccount, MangoGroup, MangoCache, PerpMarket, MAX_TOKENS, MAX_PAIRS, QUOTE_INDEX}, instruction::{cancel_perp_order_by_client_id, place_perp_order2}};
use mango::instruction::{ cancel_all_perp_orders,cancel_perp_order, withdraw, place_perp_order, consume_events };
use mango::matching::{Side, OrderType, Book};


use volt_abi::*;

use crate::{error::FundError, jup_utils::jupiter_pid::check_id};
use crate::instruction::{FundInstruction, Data};
use crate::state::{NUM_TOKENS, MAX_INVESTORS,MAX_LIMIT_ORDERS, NUM_PERP, FundAccount, InvestorData, PlatformData};
// use crate::mango_utils::*;
use crate::jup_utils::*;
use crate::tokens::*;
use crate::friktion_utils::*;

pub const CREATE_REIMBURSEMENT_ACCOUNT_OPCODE: u64 = 0x6f91dd5910a34ca5;
pub const REIMBURSE_OPCODE: u64 = 0xa05c7dbb20b37258;


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

pub const ZERO_I80F48: I80F48 = I80F48!(0);
pub const ZERO_U64F64: U64F64 = U64F64!(0);
pub const ONE_I80F48: I80F48 = I80F48!(1);
pub mod investin_admin {
    use solana_program::declare_id;
    // set investin admin
    // #[cfg(feature = "devnet")]
    // declare_id!("E3Zhv46FWGLDKFM24Ft2tgoqX5NCU49CT8NwH3rDHbsA");
    // #[cfg(not(feature = "devnet"))]
    declare_id!("owZmWQkqtY3Kqnxfua1KTHtR2S6DgBTP75JKbh15VWG");
    // declare_id!("HcikBBJaAUTZXyqHQYHv46NkvwXVigkk2CuQgGuNQEnX");
}

pub mod usdc_mint {
    use solana_program::declare_id;
    // #[cfg(feature = "devnet")]
    // declare_id!("8FRFC6MoGGkMFQwngccyu69VnYbzykGeez7ignHVAFSN");
    // #[cfg(not(feature = "devnet"))]
    declare_id!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
}

pub mod raydium_id {
    use solana_program::declare_id;
    // #[cfg(feature = "devnet")]
    // declare_id!("9rpQHSyFVM1dkkHFQ2TtTzPEW7DVmEyPmN8wVniqJtuC");
    // #[cfg(not(feature = "devnet"))]
    declare_id!("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8");
}

pub mod orca_id {
    use solana_program::declare_id;
    // #[cfg(feature = "devnet")]
    // declare_id!("9rpQHSyFVM1dkkHFQ2TtTzPEW7DVmEyPmN8wVniqJtuC"); //Same as Devnet Raydium for now
    // #[cfg(not(feature = "devnet"))]
    declare_id!("9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP");
}

pub struct Fund {}

impl Fund {
    // Fund Initialize
    pub fn initialize(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        min_amount: u64,
        performance_fee_percentage: u64,
        no_of_tokens: u8,
        is_private: bool
    ) -> Result<(), ProgramError> {

        let accounts_iter = &mut accounts.iter();

        let platform_ai = next_account_info(accounts_iter)?;
        let fund_account_ai = next_account_info(accounts_iter)?;
        let manager_ai = next_account_info(accounts_iter)?;
        let system_program_ai = next_account_info(accounts_iter)?;
        
        let rent = Rent::get()?;        
        let fund_pda_size = size_of::<FundAccount>();
        let (pda, nonce) = Pubkey::find_program_address(&[manager_ai.key.as_ref()], program_id);
        check!(*fund_account_ai.key == pda, FundError::IncorrectPDA);
        invoke_signed(
            &create_account(
                &manager_ai.key,
                &fund_account_ai.key,
                rent.minimum_balance(fund_pda_size).max(1),
                fund_pda_size as u64,
                &program_id,
            ),
            &[manager_ai.clone(), fund_account_ai.clone(), system_program_ai.clone()],
            &[&[&*manager_ai.key.as_ref(), bytes_of(&nonce)]]
        )?;

        let mut platform_data = PlatformData::load_mut_checked(platform_ai, program_id)?;

        let mut fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;



        //  check if already init
        check!(!fund_data.is_initialized(), FundError::FundAccountAlreadyInit);
        //check_eq!(fund_data.version, 0);
        check!(platform_data.is_initialized(), ProgramError::InvalidAccountData);

        check!(min_amount >= 10000000, ProgramError::InvalidArgument);

        check!(no_of_tokens as usize <= NUM_TOKENS, ProgramError::InvalidArgument); // max 8 tokens

        check!(performance_fee_percentage >= 100 && performance_fee_percentage <= 4000, ProgramError::InvalidArgument);
        
        // update platform_data
        // platform_data.no_of_active_funds += 1;
        platform_data.total_v3_funds = platform_data.total_v3_funds.checked_add(1).unwrap();
        // save manager's wallet address
        fund_data.manager_account = *manager_ai.key;
        check!(manager_ai.is_signer, ProgramError::MissingRequiredSignature);
        
        fund_data.fund_pda = pda;
        fund_data.signer_nonce = nonce;

        let usdc_mint_ai = next_account_info(accounts_iter)?;
        let fund_btoken_ai = next_account_info(accounts_iter)?;

        check_eq!(platform_data.token_list[0].mint, *usdc_mint_ai.key);

        let usdc_vault = parse_token_account(fund_btoken_ai)?;
        check_eq!(usdc_vault.owner, fund_data.fund_pda);
        check_eq!(usdc_vault.mint, *usdc_mint_ai.key); // check for USDC mint

        fund_data.tokens[0].index[0] = 0;
        fund_data.tokens[0].index[1] = 0;
        fund_data.tokens[0].mux = 0;
        fund_data.tokens[0].balance = 0;
        fund_data.tokens[0].debt = 0;
        fund_data.tokens[0].is_active = true;
        fund_data.tokens[0].vault = *fund_btoken_ai.key;
        fund_data.no_of_assets = 1;

        // whitelisted tokens
        for index in 1..no_of_tokens {
            let mint_ai = next_account_info(accounts_iter)?;
            let vault_ai = next_account_info(accounts_iter)?;

            let asset_vault = parse_token_account(vault_ai)?;
            check_eq!(asset_vault.owner, fund_data.fund_pda);
            check_eq!(asset_vault.mint, *mint_ai.key); // check for  mint

            let token_index_1 = platform_data.get_token_index(mint_ai.key, 0);
            let token_index_2 = platform_data.get_token_index(mint_ai.key, 1);

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
            fund_data.tokens[index as usize].vault = *vault_ai.key;
            fund_data.no_of_assets += 1;
        }

        fund_data.min_amount = min_amount;
        fund_data.performance_fee_percentage = U64F64::from_num(performance_fee_percentage / 100);

        fund_data.total_amount = U64F64!(0);
        fund_data.prev_performance = U64F64!(1.00);
        fund_data.number_of_active_investments = 0;
        fund_data.no_of_investments = 0;
        fund_data.mango_positions.mango_account = Pubkey::default();
        fund_data.mango_positions.perp_markets = [u8::MAX; 3];
        fund_data.mango_positions.deposit_index = u8::MAX;
        fund_data.mango_positions.markets_active = 0;
        fund_data.mango_positions.deposits_active = 0;
        fund_data.mango_positions.investor_debts = [0; 2];
        fund_data.is_initialized = true;
        fund_data.version = 1; // v1 funds
        fund_data.is_private = is_private; //set from params
        fund_data.fund_v3_index = platform_data.total_v3_funds;


        Ok(())
    }

    // investor deposit
    pub fn deposit(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
        index: u8
    ) -> Result<(), ProgramError> {
        const NUM_FIXED:usize = 6;
        let accounts = array_ref![accounts, 0, NUM_FIXED];

        let [
            fund_account_ai,
            investor_state_ai,
            investor_ai,
            investor_btoken_ai,
            router_btoken_ai,
            token_prog_ai
        ] = accounts;

        let mut fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;
        let mut investor_data = InvestorData::load_mut_checked(investor_state_ai, program_id)?;
        msg!("Is Fund Private?: {:?}", fund_data.is_private);
        // check if fund state acc passed is initialised
        check!(fund_data.is_initialized(), FundError::InvalidStateAccount);
        check!(!(fund_data.is_private) || fund_data.manager_account == *investor_ai.key, FundError::PrivateFund);

        // let depositors: u64 = U64F64::to_num(U64F64::from_num(fund_data.no_of_investments).checked_sub(U64F64::from_num(fund_data.number_of_active_investments)).unwrap());

        // check!(depositors < 10, FundError::DepositLimitReached);
        // check if amount deposited is more than the minimum amount for the fund
        check!(amount >= fund_data.min_amount, FundError::InvalidAmount);
        // check if investor has signed the transaction
        check!(investor_ai.is_signer, FundError::IncorrectSignature);

        // check if investor_state_account is already initialised
        check!(!investor_data.is_initialized(), FundError::InvestorAccountAlreadyInit);

        investor_data.is_initialized = true;
        investor_data.owner = *investor_ai.key;
        // Store manager's address
        investor_data.manager = fund_data.manager_account;

        // update queue
        // let index = fund_data.no_of_investments - fund_data.number_of_active_investments;
        // queue slot should be empty
        check!(fund_data.investors[index as usize] == Pubkey::default(), FundError::InvestorIndexError);
        fund_data.investors[index as usize] = *investor_state_ai.key;
        fund_data.no_of_investments += 1;

        // check router vault account is owned by router
        // this way of getting router_pds is better or passing platformState Acc and reading from it ??
        let (router_pda, _nonce) = Pubkey::find_program_address(&["router".as_ref()], program_id);
        let router_owner = parse_token_account(router_btoken_ai)?.owner;
        check_eq!(router_owner, router_pda);

        check!(*token_prog_ai.key == spl_token::id(), FundError::IncorrectProgramId);

        let deposit_instruction = spl_token::instruction::transfer(
            token_prog_ai.key,
            investor_btoken_ai.key,
            router_btoken_ai.key,
            investor_ai.key,
            &[&investor_ai.key],
            amount
        )?;
        let deposit_accs = [
            investor_btoken_ai.clone(),
            router_btoken_ai.clone(),
            investor_ai.clone(),
            token_prog_ai.clone()
        ];
        invoke(&deposit_instruction, &deposit_accs)?;


        investor_data.amount_in_router += amount;
        fund_data.amount_in_router += amount;

        Ok(())
    }

    // manager transfer
    pub fn transfer(
        program_id: &Pubkey,
        accounts: &[AccountInfo]
    ) -> Result<(), ProgramError> {

        const NUM_FIXED:usize = 13;

        let(fixed_accs, investor_state_accs) = array_refs![accounts, NUM_FIXED; ..;];

        let [
            platform_ai,
            fund_account_ai,
            mango_account_ai,
            mango_group_ai,
            mango_cache_ai,
            mango_prog_ai,
            manager_ai,
            router_btoken_ai,
            fund_btoken_ai,
            manager_btoken_ai,
            investin_btoken_ai,
            pda_router_ai,
            token_prog_ai
        ] = fixed_accs;

        let platform_data = PlatformData::load_checked(platform_ai, program_id)?;
        let mut fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;

        // check if manager signed the tx
        check!(manager_ai.is_signer, FundError::IncorrectProgramId);
        check_eq!(fund_data.manager_account, *manager_ai.key);
        check!(fund_data.is_initialized(), ProgramError::AccountAlreadyInitialized);

        // check if router PDA matches
        check!(*pda_router_ai.key == platform_data.router, FundError::IncorrectPDA);

        // update start performance for investors
        // let (perp_pnl, usdc_deposits) = get_mango_valuation(
        //     &fund_data,
        //     &mango_account_ai,
        //     &mango_group_ai,
        //     &mango_cache_ai,
        //     &mango_prog_ai
        // )?;
        // let mango_val = U64F64::from_fixed(usdc_deposits.checked_add(token_deposits_val).unwrap().checked_add(perp_pnl).unwrap());
        // let mango_val = U64F64::from_fixed(usdc_deposits.checked_add(perp_pnl).unwrap());
        // msg!("perp pnl:: {}, usdc_deposits:: {}, mango_val:: {}", perp_pnl, usdc_deposits, mango_val);
        
        update_amount_and_performance(
            &platform_data,
            &mut fund_data,
            true
        )?;

        let mut transferable_amount: u64 = 0;
        // let mut fee: u64 = 0;

        for investor_state_ai in investor_state_accs.iter() {
            let index = fund_data.get_investor_index(investor_state_ai.key).unwrap();
            let mut investor_data = InvestorData::load_mut_checked(investor_state_ai, program_id)?;

            // validation checks
            check_eq!(fund_data.investors[index], *investor_state_ai.key);
            check!(investor_data.amount_in_router > 0, ProgramError::InvalidAccountData);
            check_eq!(investor_data.manager, *manager_ai.key);

            // investor_data.amount = U64F64::to_num(U64F64::from_num(investor_data.amount_in_router).checked_mul(U64F64!(0.98)).unwrap());
            investor_data.amount = investor_data.amount_in_router;

            // update transfer variables
            transferable_amount = transferable_amount.checked_add(investor_data.amount).unwrap();
            // fee = fee.checked_add(U64F64::to_num(
            //     U64F64::from_num(investor_data.amount_in_router).checked_div(U64F64::from_num(100)).unwrap()
            // )).unwrap();

            // update fund amount in router
            fund_data.amount_in_router = fund_data.amount_in_router.checked_sub(investor_data.amount_in_router).unwrap();

            // update investor variables
            investor_data.amount_in_router = 0;
            investor_data.start_performance = fund_data.prev_performance;

            // zero out slot
            fund_data.investors[index] = Pubkey::default();
            fund_data.number_of_active_investments += 1;
        }


        invoke_signed(
            &(spl_token::instruction::transfer(
                token_prog_ai.key,
                router_btoken_ai.key,
                fund_btoken_ai.key,
                pda_router_ai.key,
                &[pda_router_ai.key],
                transferable_amount
            ))?,
            &[
                router_btoken_ai.clone(),
                fund_btoken_ai.clone(),
                pda_router_ai.clone(),
                token_prog_ai.clone()
            ],
            &[&["router".as_ref(), bytes_of(&platform_data.router_nonce)]]
        )?;

        // Depricated Managment and Platform fees on Investments
        //
        // invoke_signed(
        //     &(spl_token::instruction::transfer(
        //         token_prog_ai.key,
        //         router_btoken_ai.key,
        //         manager_btoken_ai.key,
        //         pda_router_ai.key,
        //         &[pda_router_ai.key],
        //         fee
        //     ))?,
        //     &[
        //         router_btoken_ai.clone(),
        //         manager_btoken_ai.clone(),
        //         pda_router_ai.clone(),
        //         token_prog_ai.clone()
        //     ],
        //     &[&["router".as_ref(), bytes_of(&platform_data.router_nonce)]]
        // )?;

        // check_eq!(platform_data.investin_vault, *investin_btoken_ai.key);
        // invoke_signed(
        //     &(spl_token::instruction::transfer(
        //         token_prog_ai.key,
        //         router_btoken_ai.key,
        //         investin_btoken_ai.key,
        //         pda_router_ai.key,
        //         &[pda_router_ai.key],
        //         fee
        //     ))?,
        //     &[
        //         router_btoken_ai.clone(),
        //         investin_btoken_ai.clone(),
        //         pda_router_ai.clone(),
        //         token_prog_ai.clone()
        //     ],
        //     &[&["router".as_ref(), bytes_of(&platform_data.router_nonce)]]
        // )?;


        fund_data.tokens[0].balance = parse_token_account(&fund_btoken_ai)?.amount;
        // dont update performance now
        update_amount_and_performance(
            &platform_data,
            &mut fund_data,
            false
        )?;
        
        Ok(())
    }

    // investor withdraw
    pub fn withdraw_from_fund(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> Result<(), ProgramError> {

        const NUM_FIXED:usize = 7;
        let accounts = array_ref![accounts, 0, NUM_FIXED + 2*NUM_TOKENS];

        let (
            fixed_accs,
            inv_token_accs,
            fund_token_accs,
        ) = array_refs![accounts, NUM_FIXED, NUM_TOKENS, NUM_TOKENS];

        let [
            platform_ai,
            fund_account_ai,
            investor_state_ai,
            investor_ai,
            router_btoken_ai,
            pda_router_ai,
            token_prog_ai
        ] = fixed_accs;

        let platform_data = PlatformData::load_checked(platform_ai, program_id)?;
        let mut fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;
        let mut investor_data = InvestorData::load_mut_checked(investor_state_ai, program_id)?;

        check!(investor_ai.is_signer, FundError::IncorrectSignature);
        check_eq!(investor_data.owner, *investor_ai.key);
        check_eq!(investor_data.manager, fund_data.manager_account);

        // Manager has not transferred to vault
        if investor_data.amount_in_router != 0  {
            invoke_signed(
                &(spl_token::instruction::transfer(
                    token_prog_ai.key,
                    router_btoken_ai.key,
                    inv_token_accs[0].key,
                    pda_router_ai.key,
                    &[pda_router_ai.key],
                    investor_data.amount_in_router
                ))?,
                &[
                    router_btoken_ai.clone(),
                    inv_token_accs[0].clone(),
                    pda_router_ai.clone(),
                    token_prog_ai.clone()
                ],
                &[&["router".as_ref(), bytes_of(&platform_data.router_nonce)]]
            )?;
            fund_data.amount_in_router = fund_data.amount_in_router.checked_sub(investor_data.amount_in_router).unwrap();
            fund_data.no_of_investments = fund_data.no_of_investments.checked_sub(1).unwrap();
            investor_data.amount_in_router = 0;
            investor_data.is_initialized = false;
            let index = fund_data.get_investor_index(investor_state_ai.key).unwrap();
            fund_data.investors[index] = Pubkey::default();
            // close investor account
            close_investor_account(investor_ai, investor_state_ai)?;
        } else {
            check!(investor_data.has_withdrawn == true && 
                (investor_data.withdrawn_from_margin == true || investor_data.margin_debt[0] == 0) ,
                // &&
                // (investor_data.withdrawn_ul_from_friktion == true || investor_data.withdrawn_ftokens_from_friktion == true),
                 FundError::InvalidInstruction);
            for i in 0..NUM_TOKENS {
                // TODO:: check if fund_debt on inv_acc <= fund_debt on fund
                if investor_data.token_debts[i] < 10 {
                    continue;
                }
                let mint_1 = platform_data.token_list[investor_data.token_indexes[i] as usize].mint;
                let mint_2 = platform_data.token_list[fund_data.tokens[i].index[fund_data.tokens[i].mux as usize] as usize].mint;
                check_eq!(mint_1, mint_2);
                let manager_account = fund_data.manager_account;
                let nonce = fund_data.signer_nonce;

                drop(fund_data);
                invoke_signed(
                    &(spl_token::instruction::transfer(
                        token_prog_ai.key,
                        fund_token_accs[i].key,
                        inv_token_accs[i].key,
                        fund_account_ai.key,
                        &[fund_account_ai.key],
                        investor_data.token_debts[i]
                    ))?,
                    &[
                        fund_token_accs[i].clone(),
                        inv_token_accs[i].clone(),
                        fund_account_ai.clone(),
                        token_prog_ai.clone()
                    ],
                    &[&[bytes_of(&manager_account), bytes_of(&nonce)]]
                )?;
                fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;
                fund_data.tokens[i].balance = parse_token_account(&fund_token_accs[i])?.amount;
                fund_data.tokens[i].debt = fund_data.tokens[i].debt.checked_sub(investor_data.token_debts[i]).unwrap();
                investor_data.token_debts[i] = 0;
                investor_data.token_indexes[i] = 0;
            }
            // check if there are no margin debts
            // check_eq!(investor_data.margin_debt[0], 0);
            // check_eq!(investor_data.margin_debt[1], 0);
            // close investor account
            //&& investor_data.margin_debt[1] == 0
            if investor_data.margin_debt[0] != 0  {
                fund_data.mango_positions.investor_debts[0] = fund_data.mango_positions.investor_debts[0].checked_sub(U64F64::to_num(investor_data.margin_debt[0])).unwrap();
            }
            investor_data.amount = 0;
            investor_data.start_performance = U64F64!(0);
            investor_data.amount_in_router = 0;
            investor_data.has_withdrawn = false;
            investor_data.is_initialized = false;
            close_investor_account(investor_ai, investor_state_ai)?;

        }
        Ok(())
    }
    

    pub fn withdraw_settle(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> Result<(), ProgramError> {
        const NUM_FIXED:usize = 11;
        let accounts = array_ref![accounts, 0, NUM_FIXED + 4*NUM_PERP];
        let (
            fixed_accs,
            perp_accs,
        ) = array_refs![accounts, NUM_FIXED, 4*NUM_PERP];

        let [
            platform_ai,
            fund_account_ai,
            investor_state_ai,
            investor_ai,
            mango_account_ai,
            mango_group_ai,
            mango_cache_ai,
            mango_prog_ai,
            referrer_mango_account_ai,
            ix_sysvar_ai,
            default_ai,
        ] = fixed_accs;

        let platform_data = PlatformData::load_mut_checked(platform_ai, program_id)?;
        let mut fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;
        let mut investor_data = InvestorData::load_mut_checked(investor_state_ai, program_id)?;
        // check_eq!(*mango_prog_ai.key, mango_v3_id::ID);
        check!(investor_data.owner == *investor_ai.key, ProgramError::MissingRequiredSignature);
        check!(investor_ai.is_signer, ProgramError::MissingRequiredSignature);
        check_eq!(investor_data.manager, fund_data.manager_account);
        check_eq!(investor_data.has_withdrawn, false);
        let ix = solana_program::sysvar::instructions::load_instruction_at_checked(1, ix_sysvar_ai)?;
        check!(ix.program_id == *program_id && ix.data[0] == 30, FundError::InvalidInstruction); 
        

        if investor_data.amount != 0 && investor_data.start_performance != ZERO_U64F64 {
            // let (perp_pnl_before, usdc_deposits_before) = get_mango_valuation(
            //     &fund_data,
            //     &mango_account_ai,
            //     &mango_group_ai,
            //     &mango_cache_ai,
            //     &mango_prog_ai
            // )?;
            // let mango_val_before = U64F64::from_fixed(usdc_deposits_before.checked_add(perp_pnl_before).unwrap());
            // msg!("pnl: {:?}, val: {:?}", perp_pnl_before, mango_val_before);
            update_amount_and_performance(
                &platform_data,
                &mut fund_data,
                true
            )?;
            let share = get_share(&mut fund_data, &mut investor_data)?;
            // msg!("share {:?}", share);
            investor_data.share = share;

            for i in 0..NUM_TOKENS {
                let mut withdraw_amount: u64 = U64F64::to_num(
                    U64F64::from_num(fund_data.tokens[i].balance.checked_sub(fund_data.tokens[i].debt).unwrap())
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

            if fund_data.friktion_vault.is_active {
                let ul_withdraw_amount: u64 = U64F64::to_num(
                    U64F64::from_num(fund_data.friktion_vault.ul_token_balance.checked_sub(fund_data.friktion_vault.ul_token_debt).unwrap())
                .checked_mul(share).unwrap());
                investor_data.friktion_ul_debt = ul_withdraw_amount;
                fund_data.friktion_vault.ul_token_debt = fund_data.friktion_vault.ul_token_debt.checked_add(ul_withdraw_amount).unwrap();
                check!(fund_data.friktion_vault.ul_token_balance >= fund_data.friktion_vault.ul_token_debt, ProgramError::InsufficientFunds);
                
                let fc_withdraw_amount: u64 = U64F64::to_num(
                    U64F64::from_num(fund_data.friktion_vault.fc_token_balance.checked_sub(fund_data.friktion_vault.fc_token_debt).unwrap())
                .checked_mul(share).unwrap());
                investor_data.friktion_fc_debt = fc_withdraw_amount;
                fund_data.friktion_vault.fc_token_debt = fund_data.friktion_vault.fc_token_debt.checked_add(fc_withdraw_amount).unwrap();
                check!(fund_data.friktion_vault.fc_token_balance >= fund_data.friktion_vault.fc_token_debt, ProgramError::InsufficientFunds);
            
            }

            

            // if mango_val_before > 0 {
            //     let mut perp_vals: [i64; 4] = get_perp_vals(&fund_data, &mango_account_ai, &mango_prog_ai, &mango_group_ai).unwrap();
            //     // msg!("closing perps: {:?}", perp_vals);
            //     let manager_account = fund_data.manager_account;
            //     let nonce = fund_data.signer_nonce;
                
            //     drop(fund_data);
                
            //     for i in 0..3 {
            //         if perp_vals[i] != 0 {
            //             let mut side:Side = Side::Ask;
            //             if perp_vals[i] < 0 {
            //                 side = Side::Bid;
            //                 perp_vals[i] = perp_vals[i].checked_mul(-1).unwrap();
            //             }
            //             let mut perp_close_amount: i64 = U64F64::to_num(U64F64::from_num(perp_vals[i]).checked_mul(share).unwrap());
            //             // msg!("side: {:?}, qty: {:?}", side, perp_close_amount);
            //             if perp_vals[i] > perp_close_amount {
            //                 perp_close_amount = perp_close_amount.checked_add(1).unwrap();
            //             }
            //             let open_orders_accs = [Pubkey::default(); MAX_PAIRS];
            //             invoke_signed(
            //                 &place_perp_order(mango_prog_ai.key,
            //                     mango_group_ai.key, mango_account_ai.key, fund_account_ai.key,
            //                     mango_cache_ai.key, perp_accs[i*4].key, perp_accs[(i*4) + 1].key, perp_accs[(i*4) + 2].key, perp_accs[(i*4) + 3].key, Some(referrer_mango_account_ai.key), &open_orders_accs,
            //                     side, i64::MAX, perp_close_amount, 0, OrderType::Market, true)?,
            //                 &[
            //                     mango_prog_ai.clone(),
            //                     mango_group_ai.clone(),
            //                     mango_account_ai.clone(),
            //                     fund_account_ai.clone(),
            //                     mango_cache_ai.clone(),
            //                     perp_accs[i*4].clone(),
            //                     perp_accs[i*4 + 1].clone(),
            //                     perp_accs[i*4 + 2].clone(),
            //                     perp_accs[i*4 + 3].clone(),
            //                     referrer_mango_account_ai.clone(),
            //                     default_ai.clone(),
            //                 ],
            //                 &[&[bytes_of(&manager_account), bytes_of(&nonce)]]
            //             )?;
    
            //             // mango_place_perp_order_investor(&fund_data, instruction_accounts, mango_perp_index, side, perp_close_amount);
            //             // pub fn mango_place_perp_order_investor(fund_data: &FundAccount,accounts: &[AccountInfo],perp_market_id: u8,side: Side,quantity: i64
            //         }
            //     }
            //     fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;
            // } 

            // msg!("usdc {:?}, tok {:?}", usdc_deposits_before, token_deposits_before);
            // let (perp_pnl_after, usdc_deposits_after) = get_mango_valuation(
            //     &fund_data,
            //     &mango_account_ai,
            //     &mango_group_ai,
            //     &mango_cache_ai,
            //     &mango_prog_ai
            // )?;
            // let mut mango_val_after = U64F64::from_fixed(usdc_deposits_after.checked_add(token_deposits_val_after).unwrap().checked_add(perp_pnl_after).unwrap());
            // let mut mango_val_after = U64F64::from_fixed(usdc_deposits_after.checked_add(perp_pnl_after).unwrap());
            // let mut investor_mango_value = mango_val_after.checked_mul(share).unwrap();
            // mango_val_after = mango_val_after.checked_sub(investor_mango_value).unwrap();
            // msg!("usdc* {:?}, tok* {:?}", usdc_deposits_after, token_deposits_after);
            
            // if perp_pnl_after < 0 {
            //     // let pnl_ratio:U64F64 = mango_val_after.checked_div(U64F64::from_num(
            //     //     usdc_deposits_after
            //     //     //.checked_add(token_deposits_val_after).unwrap()
            //     // )).unwrap();
            //     // msg!("pnl -ve {:?}", pnl_ratio);
            //     usdc_deposits_after = usdc_deposits_after.checked_sub(perp_pnl_after).unwrap();
            //     // usdc_deposits_after = usdc_deposits_after.checked_mul(I80F48::from_fixed(pnl_ratio)).unwrap();
            //     // token_deposits_after = token_deposits_after.checked_mul(I80F48::from_fixed(pnl_ratio)).unwrap();
            // } else {
            //     usdc_deposits_after = usdc_deposits_after.checked_add(perp_pnl_after).unwrap();
            // }
            // msg!("*pnl: {:?}, mango_val {:?}", perp_pnl_after, mango_val_after);
            // let pnl_diff = perp_pnl_before.checked_sub(perp_pnl_after).unwrap();
            // msg!("pnl_diff {:?}", pnl_diff);
            // let mut pnl_diff_ratio = U64F64!(1);
            // if pnl_diff > 0 {
            //     // pnl_diff_ratio = pnl_diff_ratio.checked_sub(U64F64::from_fixed(pnl_diff).checked_div(investor_mango_value).unwrap()).unwrap();
            //     let comp = U64F64::from_num(pnl_diff);
            //     mango_val_after = mango_val_after.checked_add(comp).unwrap();
            //     investor_mango_value = investor_mango_value.checked_sub(comp).unwrap();
            // }
            
            investor_data.margin_position_id[0] = QUOTE_INDEX as u64;
            // investor_data.margin_position_id[1] = fund_data.mango_positions.deposit_index as u64;
            // investor_data.margin_debt[0] = U64F64::from_fixed(investor_mango_value.checked_mul(I80F48::from_fixed(share)).unwrap()).checked_mul(pnl_diff_ratio).unwrap();
            // investor_data.margin_debt[0] = investor_mango_value;
            
            // investor_data.margin_debt[1] = U64F64::from_fixed(token_deposits_after.checked_mul(I80F48::from_fixed(share)).unwrap()).checked_mul(pnl_diff_ratio).unwrap();
            // msg!("investor debts: {:?}", investor_data.margin_debt);
            // fund_data.mango_positions.investor_debts[0] = fund_data.mango_positions.investor_debts[0].checked_add(U64F64::to_num(investor_mango_value)).unwrap();
            // fund_data.mango_positions.investor_debts[1] = fund_data.mango_positions.investor_debts[1].checked_add(U64F64::to_num(investor_data.margin_debt[1])).unwrap();
            fund_data.number_of_active_investments -= 1;
            fund_data.no_of_investments -= 1;
            investor_data.has_withdrawn = true;
            let nonce = fund_data.signer_nonce;
            let manager_account = fund_data.manager_account;
            // let mut mango_val_after = U64F64::from_fixed(usdc_deposits_after.checked_add(token_deposits_val_after).unwrap().checked_add(perp_pnl_after).unwrap());

            update_amount_and_performance(
                &platform_data,
                &mut fund_data,
                false
            )?;

        }
        Ok(())
    }

    pub fn withdraw_process_limit_orders(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> Result<(), ProgramError> {
        const NUM_FIXED: usize = 9;
                            
        let accounts = array_ref![accounts, 0, NUM_FIXED + 4*MAX_LIMIT_ORDERS];
        let (
            fixed_accs,
            perp_accs,
        ) = array_refs![accounts, NUM_FIXED, 4*MAX_LIMIT_ORDERS];

        let [
            fund_account_ai,
            investor_state_ai,
            investor_ai,
            mango_prog_ai,
            mango_group_ai,     // read
            mango_cache_ai,     // read
            mango_account_ai,   // write
            referrer_mango_account_ai,
            default_ai
        ] = fixed_accs;


        let mut fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;
        // check_eq!(*mango_prog_ai.key, mango_v3_id::ID);
        let nonce = fund_data.signer_nonce;
        let manager_pubkey = fund_data.manager_account;

        let mut investor_data = InvestorData::load_mut_checked(investor_state_ai, program_id)?;
        check!(investor_data.owner == *investor_ai.key, ProgramError::MissingRequiredSignature);
        check!(investor_ai.is_signer, ProgramError::MissingRequiredSignature);
        check_eq!(investor_data.manager, fund_data.manager_account);
        check!(investor_data.share != U64F64!(0), FundError::InvalidStateAccount);

        let open_orders_accs = [Pubkey::default(); MAX_PAIRS];

        let skip_replace = if fund_data.number_of_active_investments == 0 { true } else { false };

        for i in 0..MAX_LIMIT_ORDERS {

            if  fund_data.limit_orders[i].client_order_id == 0  {
                msg!("skip {:?}",i);
                continue;
            }

            // 1) check Limit Order is Valid 
            let mango_account = MangoAccount::load_checked(mango_account_ai, mango_prog_ai.key, mango_group_ai.key)?;
            let valid =  mango_account.find_order_with_client_id((fund_data.limit_orders[i].perp_market_id as usize),fund_data.limit_orders[i].client_order_id);
            drop(mango_account);
            match valid {
                None => {
                    msg!("Alexe - {:?}",i);
                        //order already executed
                        fund_data.limit_orders[i].client_order_id = 0; 
                        continue;
                }
                Some((order_id,_side)) => { //MEANS ORDER IS VALID
                     // update Structs and FLAG
                    let new_client_order_id = (Clock::get()?.unix_timestamp as u64).checked_mul(1000 as u64).unwrap() + (i as u64);
                    let old_client_order_id = fund_data.limit_orders[i].client_order_id;
                    let side = fund_data.limit_orders[i].side;
                    let price = fund_data.limit_orders[i].price;
                    
                    let reduce_only = fund_data.limit_orders[i].reduce_only;
                    let expiry_timestamp = fund_data.limit_orders[i].expiry_timestamp;
                    let limit = fund_data.limit_orders[i].limit;

                    let mul_factor =  U64F64!(1).checked_sub(investor_data.share).unwrap();
                    let new_max_base_quantity = U64F64::to_num(
                            U64F64::from_num(
                                fund_data.limit_orders[i].max_base_quantity
                                ).checked_mul(mul_factor).unwrap());
                    let new_max_quote_quantity = U64F64::to_num(
                            U64F64::from_num(
                                fund_data.limit_orders[i].max_quote_quantity
                                ).checked_mul(mul_factor).unwrap());
                    fund_data.limit_orders[i].max_base_quantity = new_max_base_quantity;
                    fund_data.limit_orders[i].max_quote_quantity = new_max_quote_quantity;
                    fund_data.limit_orders[i].client_order_id = new_client_order_id;

                    drop(fund_data);
                    invoke_signed(
                        &cancel_perp_order_by_client_id(mango_prog_ai.key,
                            mango_group_ai.key, mango_account_ai.key, fund_account_ai.key,
                                perp_accs[i*4].key, perp_accs[(i*4) + 1].key, perp_accs[(i*4) + 2].key, old_client_order_id, true)?,
                        &[
                            mango_prog_ai.clone(),
                            mango_group_ai.clone(),
                            mango_account_ai.clone(),
                            fund_account_ai.clone(),
                            perp_accs[i*4].clone(),
                            perp_accs[i*4 + 1].clone(),
                            perp_accs[i*4 + 2].clone(),
                        ],
                        &[&[bytes_of(&manager_pubkey), bytes_of(&nonce)]]
                    )?;

                    if !skip_replace {
                        
                        //updating new ClientOrderId and placing Order Again
                        invoke_signed(
                            &place_perp_order2(
                                mango_prog_ai.key,
                                mango_group_ai.key,
                                mango_account_ai.key,
                                fund_account_ai.key,
                                mango_cache_ai.key,
                                perp_accs[i*4].key,
                                perp_accs[(i*4) + 1].key,
                                perp_accs[(i*4) + 2].key,
                                perp_accs[(i*4) + 3].key, 
                                Some(referrer_mango_account_ai.key),
                                &open_orders_accs,
                                side,
                                price,
                                new_max_base_quantity,
                                new_max_quote_quantity,
                                new_client_order_id, 
                                OrderType::Limit,
                                reduce_only,
                                Some(expiry_timestamp),
                                limit
                            )?,
                                &[
                                    mango_prog_ai.clone(),
                                    mango_group_ai.clone(),
                                    mango_account_ai.clone(),
                                    fund_account_ai.clone(),
                                    mango_cache_ai.clone(),
                                    perp_accs[i*4].clone(),
                                    perp_accs[i*4 + 1].clone(),
                                    perp_accs[i*4 + 2].clone(),
                                    perp_accs[i*4 + 3].clone(),
                                    referrer_mango_account_ai.clone(),
                                    default_ai.clone(), 
                                ],
                            &[&[&manager_pubkey.as_ref(), bytes_of(&nonce)]]
                        )?;
                    }

                    fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;
                },
            }

        }

        investor_data.share = U64F64!(0);
        Ok(())
    }    

    pub fn swap(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        swap_index: u8,
        data: Data
    ) -> Result<(), ProgramError> {

        let accounts_iter = &mut accounts.iter();
        let platform_state_ai = next_account_info(accounts_iter)?;
        let fund_account_ai = next_account_info(accounts_iter)?;
        let manager_ai = next_account_info(accounts_iter)?;

        let platform_data = PlatformData::load_checked(platform_state_ai, program_id)?;
        check!(platform_data.is_initialized(), ProgramError::InvalidAccountData);
        
        let mut fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;
        check!(fund_data.is_initialized(), ProgramError::InvalidAccountData);

        // if invalid fund_state_acc
        // although other signers cannot chnage some others fundState so error will be thrown
        // still be better if we add checks (will need to pass manager acc)
        check!(manager_ai.is_signer, ProgramError::MissingRequiredSignature);
        check_eq!(fund_data.manager_account, *manager_ai.key);
        
        check!(swap_index < 2, ProgramError::InvalidArgument);
        
        let nonce = fund_data.signer_nonce;
        drop(fund_data);
        let (source_info, dest_info) = match swap_index {
            0 => swap_instruction_raydium(&data, nonce, accounts)?,
            1 => swap_instruction_orca(&data, nonce, accounts)?,
            _ => return Err(ProgramError::InvalidArgument)
        };

        let source_index = platform_data.get_token_index(&source_info.mint, swap_index);
        let dest_index = platform_data.get_token_index(&dest_info.mint, swap_index);
        msg!("source_index:: {:?} ", source_index);
        msg!("dest index:: {:?}", dest_index);

        msg!("source mint:: {:?}", source_info.mint);
        msg!("dest mint:: {:?}", dest_info.mint);

        fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;

        // owner checks
        check_eq!(source_info.owner, fund_data.fund_pda);
        check_eq!(dest_info.owner, fund_data.fund_pda);

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

    pub fn change_fund_privacy(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> Result<(), ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let fund_account_ai = next_account_info(accounts_iter)?;
        let manager_ai = next_account_info(accounts_iter)?;
        let mut fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;
        check!(manager_ai.is_signer, FundError::IncorrectSignature);
        check_eq!(fund_data.manager_account, *manager_ai.key);
        fund_data.is_private = !(fund_data.is_private);
        Ok(())
    }

    // manager Performance Fee Claim
    pub fn claim (
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> Result<(), ProgramError> {
        const NUM_FIXED:usize = 11;
        let accounts = array_ref![accounts, 0, NUM_FIXED];

        let [
            platform_ai,
            fund_account_ai,
            mango_account_ai,
            mango_group_ai,
            mango_cache_ai,
            mango_prog_ai,
            manager_ai,
            fund_btoken_ai,
            manager_btoken_ai,
            investin_btoken_ai,
            token_prog_ai
        ] = accounts;

        let mut fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;
        let platform_data = PlatformData::load_checked(platform_ai, program_id)?;
        // check if manager signed the tx
        check!(manager_ai.is_signer, FundError::IncorrectSignature);
        check_eq!(fund_data.manager_account, *manager_ai.key);
        // let (perp_pnl, usdc_deposits) = get_mango_valuation(
        //     &fund_data,
        //     &mango_account_ai,
        //     &mango_group_ai,
        //     &mango_cache_ai,
        //     &mango_prog_ai
        // )?;
        // let mango_val = U64F64::from_fixed(usdc_deposits.checked_add(perp_pnl).unwrap());
        update_amount_and_performance(
            &platform_data,
            &mut fund_data,
            true
        )?;

        let performance_fee_manager: u64 = U64F64::to_num(U64F64::from_num(fund_data.performance_fee)
        .checked_mul(U64F64::from_num(90)).unwrap()
        .checked_div(U64F64::from_num(100)).unwrap());

        let performance_fee_investin: u64 = U64F64::to_num(U64F64::from_num(fund_data.performance_fee)
        .checked_div(U64F64::from_num(10)).unwrap());
        check_eq!(platform_data.investin_vault, *investin_btoken_ai.key);
        let nonce = fund_data.signer_nonce;
        let transfer_instruction = spl_token::instruction::transfer(
            token_prog_ai.key,
            fund_btoken_ai.key,
            manager_btoken_ai.key,
            fund_account_ai.key,
            &[fund_account_ai.key],
            performance_fee_manager
        )?;
        let transfer_accs = [
            fund_btoken_ai.clone(),
            manager_btoken_ai.clone(),
            fund_account_ai.clone(),
            token_prog_ai.clone()
        ];
        
        drop(fund_data);
        let signer_seeds = [&*manager_ai.key.as_ref(), bytes_of(&nonce)];
        invoke_signed(&transfer_instruction, &transfer_accs, &[&signer_seeds])?;

        let transfer_instruction = spl_token::instruction::transfer(
            token_prog_ai.key,
            fund_btoken_ai.key,
            investin_btoken_ai.key,
            fund_account_ai.key,
            &[fund_account_ai.key],
            performance_fee_investin
        )?;
        let transfer_accs = [
            fund_btoken_ai.clone(),
            investin_btoken_ai.clone(),
            fund_account_ai.clone(),
            token_prog_ai.clone()
        ];
        invoke_signed(&transfer_instruction, &transfer_accs, &[&signer_seeds])?;
        msg!("Transfer Complete");

        fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;

        fund_data.tokens[0].balance = parse_token_account(&fund_btoken_ai)?.amount;
        fund_data.performance_fee = U64F64!(0);
        
        update_amount_and_performance(
            &platform_data,
            &mut fund_data,
            false
        )?;

        Ok(())

    }



    pub fn flush_debts (
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        index: u8,
        count: u8
    ) -> Result<(), ProgramError> {

        let accounts_iter = &mut accounts.iter();
        let platform_state_ai = next_account_info(accounts_iter)?;
        let fund_account_ai = next_account_info(accounts_iter)?;
        let manager_ai = next_account_info(accounts_iter)?;
        let vault_ai = next_account_info(accounts_iter)?;
        // let pda_man_ai = next_account_info(accounts_iter)?;
        let token_prog_ai = next_account_info(accounts_iter)?;

        let platform_data = PlatformData::load_checked(platform_state_ai, program_id)?;
        let mut fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;

        check_eq!(manager_ai.is_signer, true);
        check_eq!(fund_data.manager_account, *manager_ai.key);

        check_eq!(fund_data.tokens[index as usize].is_active, true);
        check_eq!(fund_data.tokens[index as usize].vault, *vault_ai.key);

        let token_mint = platform_data.token_list[fund_data.tokens[index as usize].index[fund_data.tokens[index as usize].mux as usize] as usize].mint;
        
        let nonce = fund_data.signer_nonce;
        
        let mut cumulative_debt = 0u64;
        drop(fund_data); 
        for i in 0..count {
            let investor_state_ai = next_account_info(accounts_iter)?;
            let investor_token_ai = next_account_info(accounts_iter)?;

            let mut investor_data = InvestorData::load_mut_checked(investor_state_ai, program_id)?;
            let mint_1 = platform_data.token_list[investor_data.token_indexes[index as usize] as usize].mint;

            // validation checks
            check_eq!(investor_data.manager, *manager_ai.key);
            check_eq!(parse_token_account(investor_token_ai)?.owner, investor_data.owner);
            check_eq!(token_mint, mint_1);

            invoke_signed(
                &(spl_token::instruction::transfer(
                    token_prog_ai.key,
                    vault_ai.key,
                    investor_token_ai.key,
                    fund_account_ai.key,
                    &[fund_account_ai.key],
                    investor_data.token_debts[index as usize]
                ))?,
                &[
                    vault_ai.clone(),
                    investor_token_ai.clone(),
                    fund_account_ai.clone(),
                    token_prog_ai.clone()
                ],
                &[&[&*manager_ai.key.as_ref(), bytes_of(&nonce)]]
            )?;
            cumulative_debt += investor_data.token_debts[index as usize];
            investor_data.token_debts[index as usize] = 0;
            investor_data.token_indexes[index as usize] = 0;

        }
        fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;
        fund_data.tokens[index as usize].debt = fund_data.tokens[index as usize].debt.checked_sub(cumulative_debt).unwrap();
        fund_data.tokens[index as usize].balance = parse_token_account(vault_ai)?.amount;

        Ok(())
    }

    
    
    pub fn admin_control(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        intialize_platform: u8,
        freeze_platform: u8,
        unfreeze_platform: u8,
        change_vault: u8,
        freeze_fund: u8,
        unfreeze_fund: u8,
        change_min_amount: u64,
        change_perf_fee: u64
    ) -> Result<(), ProgramError> {
        let accounts_iter = &mut accounts.iter();
        
        let platform_state_ai = next_account_info(accounts_iter)?;
        let investin_admin_ai = next_account_info(accounts_iter)?;
        let investin_vault_ai = next_account_info(accounts_iter)?;
        let mint_ai = next_account_info(accounts_iter)?;
        
        let mut platform_data = PlatformData::load_mut_checked(platform_state_ai, program_id)?;
        check!(investin_admin_ai.is_signer, FundError::IncorrectSignature);
        
        //check_eq!(investin_admin::ID, *investin_admin_ai.key); REVERT-MAINNET
        
        if intialize_platform == 1 {
            
            // check_eq!(platform_data.is_initialized(), false);  REVERT-MAINNET
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
        platform_data.investin_admin = *investin_admin_ai.key;
        platform_data.investin_vault = *investin_vault_ai.key;
        
        // add USDC as base token
        let mint_info = Mint::unpack(&mint_ai.data.borrow())?;
        platform_data.token_list[0].mint = *mint_ai.key;
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
        check!(*investin_vault_ai.key != Pubkey::default(), ProgramError::InvalidArgument);
        platform_data.investin_vault = *investin_vault_ai.key;
    }
    if freeze_fund == 1 {
        let fund_account_ai = next_account_info(accounts_iter)?;
        let mut fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;
        fund_data.is_initialized = false;
    }
    if unfreeze_fund == 1 {
        let fund_account_ai = next_account_info(accounts_iter)?;
        let mut fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;
        fund_data.is_initialized = true;
    }
    if change_min_amount > 0 {
        let fund_account_ai = next_account_info(accounts_iter)?;
        let mut fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;
        fund_data.min_amount = change_min_amount;
    }
    if change_perf_fee > 0 {
        let fund_account_ai = next_account_info(accounts_iter)?;
        let mut fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;
        fund_data.performance_fee_percentage = U64F64::from_num(change_perf_fee / 100);
    }
    
    Ok(())
}

pub fn init_mango_reimbursement(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> Result<(), ProgramError> {
    const NUM_FIXED: usize = 7;
    let accounts = array_ref![accounts, 0, NUM_FIXED];
    let [
        fund_pda_ai,
        mango_reimbursement_program_ai,
        group_ai,
        reimbursement_account_ai,
        payer_ai,
        system_program_ai,
        sysvar_rent_ai,
    ] = accounts;

    assert_eq!(*system_program_ai.key, solana_program::system_program::id(), "System Program ID mismatch");
    

    let mut fund_data = FundAccount::load_mut_checked(fund_pda_ai, program_id)?;
    fund_data.reimbursement_init = true;
    let (manager_account, signer_nonce) = (fund_data.manager_account, fund_data.signer_nonce);

    let signer_seeds = [
        &manager_account.as_ref(),
        bytes_of(&signer_nonce),
    ];
    drop(fund_data);

    
    invoke_signed(
        &create_reimbursement_account(
            mango_reimbursement_program_ai.key, 
            group_ai.key, 
            reimbursement_account_ai.key, 
            fund_pda_ai.key, 
            payer_ai.key
        )?, 
        accounts, 
        &[&signer_seeds]
    )?;

    Ok(())
}

pub fn mango_reimbursement(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    token_index: usize, 
    index_into_table: usize, 
) -> Result<(), ProgramError> {
    const NUM_FIXED: usize = 12;
    let accounts = array_ref![accounts, 0, NUM_FIXED];
    let [
        fund_pda_ai,
        mango_reimbursement_program_ai,
        group_ai, 
        vault_ai, 
        fund_usdc_vault_ai, 
        reimbursement_account_ai, 
        claim_mint_token_account_ai, 
        claim_mint_ai, 
        table_ai,
        token_program_ai,
        system_program_ai,
        sysvar_rent_ai,
    ] = accounts;

    assert_eq!(*system_program_ai.key, solana_program::system_program::id(), "System Program ID mismatch");
    assert_eq!(*token_program_ai.key, spl_token::id(), "Token Program ID mismatch");

    let fund_data = FundAccount::load_checked(fund_pda_ai, program_id)?;
    assert_eq!(*fund_usdc_vault_ai.key, fund_data.tokens[0].vault);
    let (manager_account, signer_nonce) = (fund_data.manager_account, fund_data.signer_nonce);

    let signer_seeds = [
        &manager_account.as_ref(),
        bytes_of(&signer_nonce),
    ];
    drop(fund_data);

    
    invoke_signed(
        &reimburse(
            mango_reimbursement_program_ai.key, 
            group_ai.key, 
            vault_ai.key, 
            fund_usdc_vault_ai.key, 
            reimbursement_account_ai.key, 
            fund_pda_ai.key, 
            fund_pda_ai.key, 
            claim_mint_token_account_ai.key, 
            claim_mint_ai.key, 
            table_ai.key, 
            token_index, 
            index_into_table, 
            true
        )?,
        accounts, 
        &[&signer_seeds]
    )?;

    let fund_usdc_vault_data = parse_token_account(fund_usdc_vault_ai)?;
    let mut fund_data = FundAccount::load_mut_checked(fund_pda_ai, program_id)?;
    fund_data.tokens[0].balance = fund_usdc_vault_data.amount;
    fund_data.reimbursement_claimed = true;

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
            FundInstruction::Initialize { min_amount, performance_fee_percentage, no_of_tokens, is_private } => {
                msg!("FundInstruction::Initialize");
                return Self::initialize(program_id, accounts, min_amount, performance_fee_percentage, no_of_tokens, is_private);
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
                msg!("FundInstruction::InvestorWithdrawFromFund");
                return Self::withdraw_from_fund(program_id, accounts);
            }
            FundInstruction::InvestorWithdrawSettleFunds => {
                msg!("FundInstruction::InvestorWithdrawSettle");
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
            // FundInstruction::MangoInitialize  => {
            //     msg!("FundInstruction::MangoInitialize");
            //     // return mango_init_mango_account(program_id, accounts);
            // }
            FundInstruction::AdminControl { intialize_platform,
                freeze_platform, unfreeze_platform, change_vault, freeze_fund, unfreeze_fund,
                change_min_amount, change_perf_fee } => {
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
                    change_perf_fee
                );
            }
            // FundInstruction::MangoDeposit { token_slot_index, mango_token_index, quantity} => {
            //     msg!("FundInstruction::MangoDeposit");
            //     return;
            //     //  mango_deposit(program_id, accounts, token_slot_index, mango_token_index, quantity);
            // }
            // FundInstruction::MangoPlacePerpOrder { perp_market_id, side, price, quantity, reduce_only } => {
            //     msg!("FundInstruction::MangoPlacePerpOrder");
            //     return ;
            //     // mango_place_perp_order(program_id, accounts, perp_market_id , side, price, quantity, reduce_only);
            // }
            // FundInstruction::MangoPlacePerpOrder2 { 
            //     perp_market_id,
            //     price,
            //     max_base_quantity,
            //     max_quote_quantity,
            //     client_order_id,
            //     expiry_timestamp,
            //     side,
            //     order_type,
            //     reduce_only,
            //     limit 
            // } => {
            //     msg!("FundInstruction::MangoPlacePerpOrder2");
            //     return mango_place_perp_order2(
            //         program_id,
            //         accounts,
            //         perp_market_id,
            //         side,
            //         price,
            //         max_base_quantity,
            //         max_quote_quantity,
            //         client_order_id,
            //         order_type,
            //         reduce_only,
            //         expiry_timestamp,
            //         limit
            //     );
            // }
            // FundInstruction::CancelPerpOrder { client_order_id} => {
            //     msg!("FundInstruction::CancelPerpOrder");
            //     return mango_cancel_perp_order(program_id, accounts, client_order_id);
            // }
            // FundInstruction::WithdrawProcessLimitOrders => {
            //     msg!("FundInstruction::WithdrawProcessLimitOrders");
            //     return Self::withdraw_process_limit_orders(program_id, accounts);
            // }
            // FundInstruction::MangoRemovePerpIndex {perp_market_id} => {
            //     msg!("FundInstruction::MangoRemovePerpIndex");
            //     return mango_remove_perp_index(program_id, accounts, perp_market_id);
            // }
            // FundInstruction::MangoPlaceSpotOrder { side, price,trade_size } => {
            //     msg!("FundInstruction::MangoPlaceSpotOrder");
            //     return mango_place_spot_order2(program_id, accounts, side, price, trade_size);
            // }
            // FundInstruction::MangoWithdraw { token_slot_index, mango_token_index, quantity } => {
            //     msg!("FundInstruction::MangoWithdraw");
            //     return mango_withdraw(program_id, accounts, token_slot_index, mango_token_index, quantity);
            // }
            // FundInstruction::MangoWithdrawInvestor => {
            //     msg!("FundInstruction::MangoWithdrawInvestor");
            //     return mango_withdraw_investor(program_id, accounts);
            // }
            // FundInstruction::MangoWithdrawInvestorPlaceOrder { price } => {
                //     msg!("FundInstruction::MangoWithdrawInvestorPlaceOrder");
                //     return mango_withdraw_investor_place_order(program_id, accounts, price);
                // }
            FundInstruction::ChangeFundPrivacy => {
                msg!("FundInstruction::ChangeFundPrivacy");
                return Self::change_fund_privacy(program_id, accounts);
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
            FundInstruction::JupiterSwap => {
                msg!("FundInstruction::JupiterSwap");
                let (&_op, op_data) = array_refs![data, 1; ..;];
                return jup_swap(program_id, accounts, op_data);
            }
            FundInstruction::SetSwapGuard {token_in_fund_slot, token_out_fund_slot, amount_in} => {
                msg!("FundInstruction::SetSwapGuard");
                return set_swap_guard(program_id, accounts, token_in_fund_slot, token_out_fund_slot, amount_in);
            }
            FundInstruction::CheckSwapGuard => {
                msg!("FundInstruction::CheckSwapGuard");
                return check_swap_guard(program_id, accounts);
            }
            FundInstruction::InitOpenOrderAccounts => {
                msg!("FundInstruction::InitOpenOrders");
                let (&_op, op_data) = array_refs![data, 1; ..;];
                return init_open_order_accs(program_id, accounts, op_data);
            }
            FundInstruction::FriktionDeposit { deposit_amount} => {
                msg!("FundInstruction::FriktionDeposit");
                return friktion_deposit(program_id, accounts, deposit_amount);
            }
            FundInstruction::ReadFriktion => {
                msg!("FundInstruction::ReadFriktion");
                return read_friktion_data(program_id, accounts);
            }
            FundInstruction::FriktionDeposit0 { deposit_amount } => {
                msg!("FundInstruction::FriktionDeposit");
                return friktion_deposit(program_id, accounts, deposit_amount)
            }
            FundInstruction::FriktionCancelPendingDeposit => {
                msg!("FundInstruction::FriktionCancelPendingDeposit");
                return friktion_cancel_pending_deposit(program_id, accounts)
            }
            FundInstruction::FriktionWithdraw { withdraw_amount } => {
                msg!("FundInstruction::FriktionWithdraw");
                return friktion_withdraw(program_id, accounts)
            }
            FundInstruction::FriktionCancelPendingWithdrawal => {
                msg!("FundInstruction::FriktionCancelPendingWithdrawal");
                return friktion_cancel_pending_withdrawal(program_id, accounts)
            }
            FundInstruction::FriktionClaimPendingDeposit => {
                msg!("FundInstruction::FriktionClaimPendingDeposit");
                return friktion_claim_pending_deposit(program_id, accounts)
            }
            FundInstruction::FriktionClaimPendingWithdrawal => {
                msg!("FundInstruction::FriktionClaimPendingWithdrawal");
                return friktion_claim_pending_withdrawal(program_id, accounts)
            }
            FundInstruction::UpdateFriktionValue => {
                msg!("FundInstruction::FriktionUpdateValue");
                return update_friktion_value(program_id, accounts)
            }
            FundInstruction::FriktionAddToFund { ul_token_slot } => {
                msg!("FundInstruction::FriktionAddToFund");
                return friktion_add_to_fund(program_id, accounts, ul_token_slot);
            }
            FundInstruction::FriktionRemoveFromFund => {
                msg!("FundInstruction::FriktionRemoveFromFund");
                return friktion_remove_from_fund(program_id, accounts)
            }
            FundInstruction::FriktionInvestorWithdrawUL => {
                msg!("FundInstruction::FriktionInvestorWithdrawUL");
                return friktion_investor_withdraw_ul(program_id, accounts)
            }
            FundInstruction::FriktionInvestorWithdrawUL2 => {
                msg!("FundInstruction::FriktionInvestorWithdrawUL_2");
                return friktion_investor_withdraw_ul_2(program_id, accounts)
            }
            FundInstruction::FriktionInvestorWithdrawFTokens => {
                msg!("FundInstruction::FriktionInvestorWithdrawFTokens");
                return  friktion_investor_withdraw_ftokens(program_id, accounts)
            }
            FundInstruction::InitReimbursement => {
                msg!("FundInstruction::InitReimbursement");
                return Self::init_mango_reimbursement(program_id, accounts);
            }
            FundInstruction::Reimburse {token_index, index_into_table} => {
                msg!("FundInstruction::Reimburse");
                return Self::mango_reimbursement(program_id, accounts, token_index, index_into_table);
            }
        }
    }
}

pub fn create_reimbursement_account(
    program_id: &Pubkey,
    group_pk: &Pubkey,
    reimbursement_account_pk: &Pubkey,
    mango_account_owner_pk: &Pubkey,
    payer_pk: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let accounts = vec![
        AccountMeta::new_readonly(*group_pk, false),
        AccountMeta::new(*reimbursement_account_pk, false),
        AccountMeta::new_readonly(*mango_account_owner_pk, false),
        AccountMeta::new(*payer_pk, true),
        AccountMeta::new_readonly(solana_program::system_program::ID, false),
        AccountMeta::new_readonly(solana_program::sysvar::rent::ID, false)
    ];

    let mut ix_data = Vec::<u8>::new();
    ix_data.extend(CREATE_REIMBURSEMENT_ACCOUNT_OPCODE.to_be_bytes().to_vec());
    Ok(Instruction { program_id: *program_id, accounts, data: ix_data })
}

pub fn reimburse(
    program_id: &Pubkey,
    group_pk: &Pubkey,
    vault_pk: &Pubkey,
    token_account_pk: &Pubkey,
    reimbursement_account_pk: &Pubkey,
    mango_account_owner_pk: &Pubkey,
    signer_pk: &Pubkey,
    claim_mint_token_account_pk: &Pubkey,
    claim_mint_pk: &Pubkey,
    table_pk: &Pubkey,
    token_index: usize,
    index_into_table: usize,
    transfer_claim: bool,
) -> Result<Instruction, ProgramError> {
    let accounts = vec![
        AccountMeta::new_readonly(*group_pk, false),
        AccountMeta::new(*vault_pk, false),
        AccountMeta::new(*token_account_pk, false),
        AccountMeta::new(*reimbursement_account_pk, false),
        AccountMeta::new_readonly(*mango_account_owner_pk, false),
        AccountMeta::new(*signer_pk, true),
        AccountMeta::new(*claim_mint_token_account_pk, false),
        AccountMeta::new(*claim_mint_pk, false),
        AccountMeta::new_readonly(*table_pk, false),
        AccountMeta::new_readonly(spl_token::ID, false),
        AccountMeta::new_readonly(solana_program::system_program::ID, false),
        AccountMeta::new_readonly(solana_program::sysvar::rent::ID, false)
    ];

    let mut ix_data = Vec::<u8>::new();
    ix_data.extend(REIMBURSE_OPCODE.to_be_bytes().to_vec());
    ix_data.extend_from_slice(&token_index.to_le_bytes());
    ix_data.extend_from_slice(&index_into_table.to_le_bytes());
    ix_data.extend_from_slice(&[transfer_claim as u8]);

    Ok(Instruction { program_id: *program_id, accounts, data: ix_data })
}


// calculate prices, get fund valuation and performance
pub fn update_amount_and_performance(
    platform_data: &PlatformData,
    fund_data: &mut FundAccount,
    // mango_val: U64F64,
    update_perf: bool
) -> Result<(), ProgramError> {
    // let mut usdc_deposits: I80F48 = ZERO_I80F48;
    // let mut token_deposits: I80F48 = ZERO_I80F48;
    // msg!("UA&P");
    let mut fund_val = ZERO_U64F64;
    
    // add USDC balance (not decimal adjusted)
    fund_val = fund_val.checked_add(U64F64::from_num(fund_data.tokens[0].balance.checked_sub(fund_data.tokens[0].debt).unwrap())).unwrap();
    // msg!("USDC: {:?}", U64F64::from_num(fund_data.tokens[0].balance - fund_data.tokens[0].debt));
    // Calculate prices for all tokens with balances
    msg!("Timestamp:: {:?}", Clock::get()?.unix_timestamp);
    for i in 1..NUM_TOKENS {

        // dont update if token balance == 0
        if (fund_data.tokens[i].balance.checked_sub(fund_data.tokens[i].debt).unwrap()) == 0 { continue; }
        
        // get last mux
        // get index of token
        let token_info = platform_data.token_list[fund_data.tokens[i].index[fund_data.tokens[i].mux as usize] as usize];
        
        if Clock::get()?.unix_timestamp.checked_sub(token_info.last_updated).unwrap() > 100 {
            msg!("{} price not up-to-date", i);
            return Err(FundError::PriceStaleInAccount.into())
        }
        // calculate price in terms of base token
        let mut val: U64F64 = U64F64::from_num(fund_data.tokens[i].balance.checked_sub(fund_data.tokens[i].debt).unwrap())
        .checked_mul(token_info.pool_price).unwrap();

         if token_info.pc_index != 0 {
             let underlying_token_info = platform_data.token_list[token_info.pc_index as usize];
             if Clock::get()?.unix_timestamp - underlying_token_info.last_updated > 100 {
                msg!("{} base price not up-to-date", i);
                return Err(FundError::PriceStaleInAccount.into())
            }
             val = val.checked_mul(underlying_token_info.pool_price).unwrap();
         }
        fund_val = fund_val.checked_add(val).unwrap();
    }

    if fund_data.friktion_vault.is_active {

        let friktion_ul_token_info = platform_data.token_list[fund_data.tokens[fund_data.friktion_vault.ul_token_slot as usize].index[fund_data.tokens[fund_data.friktion_vault.ul_token_slot as usize].mux as usize] as usize];
            
            if Clock::get()?.unix_timestamp.checked_sub(fund_data.friktion_vault.last_updated).unwrap() > 100 {
                msg!("FKV not up-to-date...");
                return Err(FundError::PriceStaleInAccount.into())
            }
            // calculate price in terms of base token
            let mut val: U64F64 = U64F64::from_num(fund_data.friktion_vault.ul_token_balance.checked_sub(fund_data.friktion_vault.ul_token_debt).unwrap())
            .checked_mul(friktion_ul_token_info.pool_price).unwrap();
    
             if friktion_ul_token_info.pc_index != 0 {
                 let underlying_token_info = platform_data.token_list[friktion_ul_token_info.pc_index as usize];
                 if Clock::get()?.unix_timestamp.checked_sub(underlying_token_info.last_updated).unwrap() > 100 {
                    msg!("ul price not up-to-date.. aborting");
                    return Err(FundError::PriceStaleInAccount.into())
                }
                 val = val.checked_mul(underlying_token_info.pool_price).unwrap();
             }
    
        fund_val = fund_val.checked_add(val).unwrap();
    }
    

    
    if update_perf {
        let mut perf = U64F64::from_num(fund_data.prev_performance);
        // only case where performance is not updated:
        // when no investments and no performance fee for manager
        if fund_data.number_of_active_investments != 0 || fund_data.performance_fee != ZERO_U64F64 {
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
    Ok(())
}

// pub fn get_mango_valuation(
//     fund_data: &FundAccount,
//     mango_account_ai: &AccountInfo,
//     mango_group_ai: &AccountInfo,
//     mango_cache_ai: &AccountInfo,
//     mango_prog_ai: &AccountInfo,
// ) -> Result<(I80F48, I80F48), ProgramError> {
//     let mut perp_pnl: I80F48 = ZERO_I80F48;
//     let mut usdc_deposits: I80F48 = ZERO_I80F48;
//     // let mut token_deposits: I80F48 = ZERO_I80F48;
//     // let mut token_deposits_val: I80F48 = ZERO_I80F48;
//     // msg!("GMV");
//     if(fund_data.mango_positions.mango_account != Pubkey::default()){
        
//         let mango_group = MangoGroup::load_checked(mango_group_ai, mango_prog_ai.key)?;
//         let mango_account = MangoAccount::load_checked(mango_account_ai, mango_prog_ai.key, mango_group_ai.key)?;
//         let mango_cache = MangoCache::load_checked(mango_cache_ai, mango_prog_ai.key, &mango_group)?;
//         // account for native USDC deposits
//         usdc_deposits  = mango_account.get_native_deposit(&mango_cache.root_bank_cache[QUOTE_INDEX], QUOTE_INDEX)?
//         .checked_sub(mango_account.get_native_borrow(&mango_cache.root_bank_cache[QUOTE_INDEX], QUOTE_INDEX)?).unwrap();
       
//         // .checked_sub(I80F48::from_num(fund_data.mango_positions.investor_debts[0])).unwrap();
//         let fund_debts = I80F48::from_num(fund_data.mango_positions.investor_debts[0]);
//         if usdc_deposits > fund_debts {
//             usdc_deposits = usdc_deposits.checked_sub(fund_debts).unwrap();
//         } else {
//             msg!("Investor Debts Exceeded Deposits... Rekt");
//         }

//         //     fund_data.mango_positions.investor_debts[0] = 0;
//         // }

//         // let dti = fund_data.mango_positions.deposit_index as usize;
//         // //Check if deposit_index is valid
//         // if(dti < QUOTE_INDEX){
//         //     token_deposits = mango_account.get_native_deposit(&mango_cache.root_bank_cache[dti], dti)?.checked_sub(I80F48::from_num(fund_data.mango_positions.investor_debts[1])).unwrap();
//         //     token_deposits_val = token_deposits.checked_mul(mango_cache.price_cache[dti].price).unwrap();
//         //     // msg!("rootbank.di {:?}", mango_cache.root_bank_cache[dti].deposit_index);
//         // }
//         for i in 0..3 {
//             let market_index = fund_data.mango_positions.perp_markets[i] as usize;
//             if(market_index == u8::MAX as usize){
//                 continue;
//             }
//             // Calculate pnl for perp account
//             let (base_val, quote_val) = mango_account.perp_accounts[market_index].get_val(&mango_group.perp_markets[market_index],
//                 &mango_cache.perp_market_cache[market_index], mango_cache.price_cache[market_index].price)?;

//             perp_pnl = perp_pnl.checked_add(base_val.checked_add(quote_val).unwrap()).unwrap();
//             // fund_val = fund_val.checked_add(U64F64::from_num(0)).unwrap();
//         }
//     } else {
//     msg!("NO MANGO ACCOUNT");
//     }
//     // msg!("mango_val done");
//     Ok((perp_pnl, usdc_deposits))
// }

// pub fn adjust_mango_pnl(usdc: &mut I80F48, token: &mut I80F48, token_val: &mut I80F48, perp_pnl: I80F48) -> Result<(), ProgramError> {
//     if perp_pnl < ZERO_I80F48 {
//             let mut pnl_adj_ratio = ONE_I80F48.checked_add(perp_pnl.checked_div((*usdc).checked_add(*token_val).unwrap()).unwrap()).unwrap();
//             msg!("pnl_adj: {:?} ", pnl_adj_ratio);
//             *usdc = (*usdc).checked_mul(pnl_adj_ratio).unwrap();
//             *token = (*token).checked_mul(pnl_adj_ratio).unwrap();
//             *token_val = (*token_val).checked_mul(pnl_adj_ratio).unwrap();
//             msg!("usdc {:?}, tok {:?}, tv {:?}", *usdc, *token, *token_val);
//         } else {
//             *usdc = (*usdc).checked_add(perp_pnl).unwrap();
//         }
//     Ok(())
// }

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
    // fund_data: &FundAccount,
    nonce: u8,
    accounts: &[AccountInfo]
) -> Result<(Account, Account), ProgramError>{
    let accounts = array_ref![accounts, 0, 22];
    let [
        _platform_state_ai,
        _fund_account_ai,
        manager_ai,
        pool_prog_ai,
        token_prog_ai,
        amm_id,
        amm_authority,
        amm_open_orders,
        amm_target_orders,
        pool_coin_token_ai,
        pool_pc_token_ai,
        dex_prog_ai,
        dex_market_ai,
        bids_ai,
        asks_ai,
        event_queue_ai,
        coin_vault_ai,
        pc_vault_ai,
        signer_ai,
        source_token_ai,
        dest_token_ai,
        owner_token_ai
    ] = accounts;

    // check program id
    check_eq!(*pool_prog_ai.key, raydium_id::ID);

    invoke_signed(
        &(Instruction::new_with_borsh(
            *pool_prog_ai.key,
            &data,
            vec![
                AccountMeta::new_readonly(*token_prog_ai.key, false),
                AccountMeta::new(*amm_id.key, false),
                AccountMeta::new(*amm_authority.key, false),
                AccountMeta::new(*amm_open_orders.key, false),
                AccountMeta::new(*amm_target_orders.key, false),
                AccountMeta::new(*pool_coin_token_ai.key, false),
                AccountMeta::new(*pool_pc_token_ai.key, false),
                AccountMeta::new_readonly(*dex_prog_ai.key, false),
                AccountMeta::new(*dex_market_ai.key, false),
                AccountMeta::new(*bids_ai.key, false),
                AccountMeta::new(*asks_ai.key, false),
                AccountMeta::new(*event_queue_ai.key, false),
                AccountMeta::new(*coin_vault_ai.key, false),
                AccountMeta::new(*pc_vault_ai.key, false),
                AccountMeta::new(*signer_ai.key, false),
                AccountMeta::new(*source_token_ai.key, false),
                AccountMeta::new(*dest_token_ai.key, false),
                AccountMeta::new(*owner_token_ai.key, true)
            ],
        )),
        &[
            token_prog_ai.clone(),
            amm_id.clone(),
            amm_authority.clone(),
            amm_open_orders.clone(),
            amm_target_orders.clone(),
            pool_coin_token_ai.clone(),
            pool_pc_token_ai.clone(),
            dex_prog_ai.clone(),
            dex_market_ai.clone(),
            bids_ai.clone(),
            asks_ai.clone(),
            event_queue_ai.clone(),
            coin_vault_ai.clone(),
            pc_vault_ai.clone(),
            signer_ai.clone(),
            source_token_ai.clone(),
            dest_token_ai.clone(),
            owner_token_ai.clone(),
        ],
        &[&[&*manager_ai.key.as_ref(), bytes_of(&nonce)]]
    )?;
    msg!("swap instruction done");


    let source_info = parse_token_account(source_token_ai)?;
    let dest_info = parse_token_account(dest_token_ai)?;

    Ok((source_info, dest_info))
}

pub fn swap_instruction_orca(
    data: &Data,
    nonce: u8,
    accounts: &[AccountInfo]
) -> Result<(Account, Account), ProgramError>{

    let accounts = array_ref![accounts, 0, 13];
    let [
        _platform_state_ai,
        fund_account_ai,
        manager_ai,
        orca_prog_id,
        swap_ai,
        swap_authority,
        user_source,
        pool_source,
        pool_dest,
        user_dest,
        pool_mint,
        fee_account,
        token_prog_ai
    ] = accounts;
    
    
    // check program id
    check_eq!(*orca_prog_id.key, orca_id::ID);
    
    invoke_signed(
        &(Instruction::new_with_borsh(
            *orca_prog_id.key,
            &data,
            vec![
                AccountMeta::new_readonly(*swap_ai.key, false),
                AccountMeta::new_readonly(*swap_authority.key, false),
                AccountMeta::new_readonly(*fund_account_ai.key, true),
                AccountMeta::new(*user_source.key, false),
                AccountMeta::new(*pool_source.key, false),
                AccountMeta::new(*pool_dest.key, false),
                AccountMeta::new(*user_dest.key, false),
                AccountMeta::new(*pool_mint.key, false),
                AccountMeta::new(*fee_account.key, false),
                AccountMeta::new_readonly(*token_prog_ai.key, false)
            ],
        )),
        &[
            swap_ai.clone(),
            swap_authority.clone(),
            fund_account_ai.clone(),
            user_source.clone(),
            pool_source.clone(),
            pool_dest.clone(),
            user_dest.clone(),
            pool_mint.clone(),
            fee_account.clone(),
            token_prog_ai.clone(),
        ],
        &[&[&*manager_ai.key.as_ref(), bytes_of(&nonce)]]
    )?;
    msg!("swap instruction done");

    let source_info = parse_token_account(user_source)?;
    let dest_info = parse_token_account(user_dest)?;
    
    Ok((source_info, dest_info))
}

pub fn get_share(
    fund_data: &mut FundAccount,
    investor_data: &mut InvestorData,
) -> Result<U64F64, ProgramError> {
    let perf_share = U64F64::from_num(fund_data.prev_performance)
    .checked_div(U64F64::from_num(investor_data.start_performance)).unwrap();

    let actual_amount: u64 = investor_data.amount;

    let mut investment_return = U64F64::from_num(actual_amount)
    .checked_mul(perf_share).unwrap();

    // check if withdraw exceed
    // check!(amount <= U64F64::to_num(total_share), ProgramError::InsufficientFunds);

    // in case of profit
    if investment_return > actual_amount {
        let profit = investment_return
        .checked_sub(U64F64::from_num(actual_amount)).unwrap();

        let performance_fee = profit.checked_mul(fund_data.performance_fee_percentage).unwrap()
        .checked_div(U64F64::from_num(100)).unwrap();

        investment_return = investment_return.checked_sub(performance_fee).unwrap();

        fund_data.performance_fee = fund_data.performance_fee.checked_add(performance_fee).unwrap();
        
        // investment_return = U64F64::from_num(profit)
        // .checked_mul(
        //     (U64F64::from_num(100).checked_sub(fund_data.performance_fee_percentage).unwrap())
        //     .checked_div(U64F64::from_num(100)).unwrap()
        //     ).unwrap()
        // .checked_add(U64F64::from_num(actual_amount)).unwrap();

        // fund_data.performance_fee = U64F64::to_num(U64F64::from_num(fund_data.performance_fee)
        // .checked_add(U64F64::from_num(profit)
        // .checked_mul(
        //     U64F64::from_num(fund_data.performance_fee_percentage)
        //     .checked_div(U64F64::from_num(100)).unwrap()
        // ).unwrap()).unwrap()
        // );
    }

    let share = U64F64::from_num(investment_return)
    .checked_div(fund_data.total_amount).unwrap();

    Ok(share)
}
pub fn get_perp_vals(
    fund_data: &FundAccount,
    mango_account_ai: &AccountInfo,
    mango_prog_ai: &AccountInfo,
    mango_group_ai: &AccountInfo,
)-> Result<[i64; 4], ProgramError> {
    let mut perp_vals: [i64; 4] = [0; 4];
    let mango_account = MangoAccount::load_checked(mango_account_ai, mango_prog_ai.key, mango_group_ai.key)?;
    for i in 0..3 {
        let fpi = fund_data.mango_positions.perp_markets[i];
        if fpi != u8::MAX {
            perp_vals[i] = mango_account.perp_accounts[fpi as usize].base_position;
        }
    }
    Ok(perp_vals)
}


// pub fn update_settle_amounts(
//     fund_data: &mut FundAccount,
//     investor_data: &mut InvestorData,
//     usdc_before: I80F48,
//     usdc_after: I80F48,
//     tokens_before: I80F48,
//     tokens_after: I80F48
// )-> Result<(), ProgramError> {
//     msg!("USA");
//     let usdc_diff = (usdc_before.checked_sub(I80F48::from_fixed(investor_data.margin_debt[0])).unwrap()).checked_sub(usdc_after).unwrap();
//     if usdc_diff > 0 {
//         investor_data.margin_debt[0] = investor_data.margin_debt[0].checked_sub(U64F64::from_fixed(usdc_diff)).unwrap();
//         fund_data.mango_positions.investor_debts[0] = fund_data.mango_positions.investor_debts[0].checked_sub(I80F48::to_num(usdc_diff)).unwrap();
//     }
//     let token_diff = (tokens_before.checked_sub(I80F48::from_fixed(investor_data.margin_debt[1])).unwrap()).checked_sub(tokens_after).unwrap();
//     if token_diff > 0 {
//         investor_data.margin_debt[1] = investor_data.margin_debt[1].checked_sub(U64F64::from_fixed(token_diff)).unwrap();
//         fund_data.mango_positions.investor_debts[1] = fund_data.mango_positions.investor_debts[1].checked_sub(I80F48::to_num(token_diff)).unwrap();
//     }
//     Ok(())
// }


pub fn close_investor_account (
    investor_ai: &AccountInfo,
    investor_state_ai: &AccountInfo
)-> Result<(), ProgramError> {

    let dest_starting_lamports = investor_ai.lamports();
    **investor_ai.lamports.borrow_mut() = dest_starting_lamports
            .checked_add(investor_state_ai.lamports())
            .ok_or(ProgramError::AccountBorrowFailed)?;
    **investor_state_ai.lamports.borrow_mut() = 0;

    Ok(())
}