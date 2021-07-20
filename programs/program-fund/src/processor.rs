use std::mem::size_of;
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
use crate::state::{NUM_TOKENS, MAX_INVESTORS, FundData, InvestorData, TokenInfo, PlatformData, PriceAccount};
use crate::state::Loadable;
use crate::mango_utils::*;
use mango::state::{MarginAccount, MangoGroup, NUM_MARKETS, load_open_orders, Loadable as MangoLoadable};

macro_rules! check {
    ($cond:expr, $err:expr) => {
        if !($cond) {
            return Err(($err).into())
        }
    }
}

// macro_rules! check_eq {
//     ($x:expr, $y:expr, $err:expr) => {
//         check_assert($x == $y, $err, line!())
//     }
// }

pub struct Fund {}

impl Fund {
    // Fund Initialize
    pub fn initialize (
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        min_amount: u64,
        min_return: u64,
        performance_fee_percentage: u64
    ) -> Result<(), ProgramError> {

        const NUM_FIXED:usize = 4;
        let accounts = array_ref![accounts, 0, NUM_FIXED + NUM_TOKENS];
        let (
            fixed_accs,
            mint_accs
        ) = array_refs![accounts, NUM_FIXED, NUM_TOKENS];

        let [
            platform_acc,
            fund_state_acc,
            manager_acc,
            fund_btoken_acc,
        ] = fixed_accs;

        // TODO: Add check that base_account is derived from manager_account
        let mut platform_data = PlatformData::load_mut(platform_acc)?;
        let mut fund_data = FundData::load_mut(fund_state_acc)?;

        //  check if already init
        check!(!fund_data.is_initialized(), FundError::FundAccountAlreadyInit);

        // check if platform_data is init
        if !platform_data.is_initialized() {
            let (router_pda, nonce) = 
                Pubkey::find_program_address(&["router".as_ref()], program_id
            );
            platform_data.router = router_pda;
            platform_data.router_nonce = nonce;
            
            platform_data.is_initialized = true;
            platform_data.no_of_active_funds = 0;
        }

        // save manager's wallet address
        fund_data.manager_account = *manager_acc.key;

        // TODO: Add check that base_account mint is USDT only
        // TODO: check that mint_accs[0] is USDT
        // token data for base token USDT
        fund_data.tokens[0].vault = *fund_btoken_acc.key;

        // whitelisted tokens
        for i in 0..NUM_TOKENS {
            let mint_acc = &mint_accs[i];
            let mint = Mint::unpack(&mint_acc.data.borrow())?;
            fund_data.tokens[i].mint = *mint_acc.key;
            fund_data.tokens[i].decimals = mint.decimals.into();
            fund_data.tokens[i].balance = 0;
        }

        fund_data.decimals = 6;
        fund_data.min_amount = min_amount;
        fund_data.min_return = U64F64::from_num(min_return / 100);
        fund_data.performance_fee_percentage = U64F64::from_num(performance_fee_percentage / 100);

        fund_data.total_amount = 0; 
        fund_data.prev_performance = U64F64!(1.00);
        fund_data.number_of_active_investments = 0;
        fund_data.mango_positions[0].margin_account = Pubkey::default();
        fund_data.mango_positions[1].margin_account = Pubkey::default();


        // get nonce for signing later
        let (_pda, nonce) = Pubkey::find_program_address(&[&*manager_acc.key.as_ref()], program_id);
        fund_data.signer_nonce = nonce;
        fund_data.is_initialized = true;

        // update platform_data
        platform_data.no_of_active_funds += 1;

        Ok(())
    }

    // investor deposit
    pub fn deposit (
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64
    ) -> Result<(), ProgramError> {
        const NUM_FIXED:usize = 7;
        let accounts = array_ref![accounts, 0, NUM_FIXED];

        let [
            fund_state_acc,
            investor_state_acc,
            investor_acc,
            investor_btoken_acc,

            router_btoken_acc,
            pda_man_acc,
            token_prog_acc
        ] = accounts;

        let mut fund_data = FundData::load_mut(fund_state_acc)?;
        let mut investor_data = InvestorData::load_mut(investor_state_acc)?;

        // check if fund state acc passed is initialised
        check!(fund_data.is_initialized(), FundError::InvalidStateAccount);

        let depositors: u64 = U64F64::to_num(U64F64::from_num(fund_data.no_of_investments).checked_sub(U64F64::from_num(fund_data.number_of_active_investments)).unwrap());

        check!(depositors < 10, FundError::DepositLimitReached);

        // TODO: check if pda_man_acc is derived from fund_data.manager_account
        // TODO: check if router_btoken_acc is derived from pda_inv_acc

        // check if amount deposited is more than the minimum amount for the fund
        check!(amount >= fund_data.min_amount, FundError::InvalidAmount);

        // check if investor has signed the transaction
        check!(investor_acc.is_signer, FundError::IncorrectSignature);
        // TODO: check if investor state account is derived from its address
        // check_owner(investor_state_acc, investor_acc.key);

        // check if investor_state_account is already initialised
        //check!(investor_data.is_initialized(), FundError::InvestorAccountAlreadyInit);
        
        if !investor_data.is_initialized {
            investor_data.is_initialized = true;
            investor_data.owner = *investor_acc.key;
            // Store manager's PDA
            investor_data.manager = fund_data.manager_account;
        }

        // dont update queue if previous deposit already in router
        if investor_data.amount_in_router == 0 {
            // calculate waiting queue index
            let index = fund_data.no_of_investments - fund_data.number_of_active_investments;
            fund_data.investors[index as usize] = *investor_state_acc.key;
            fund_data.no_of_investments += 1;
        }

        investor_data.amount_in_router += amount;

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
        
        fund_data.amount_in_router += amount;
    
        Ok(())
    }

    // manager transfer
    pub fn transfer (
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> Result<(), ProgramError> {

        const NUM_FIXED:usize = 15;
        let accounts = array_ref![accounts, 0, NUM_FIXED + MAX_INVESTORS];

        let (
            fixed_accs,
            investor_state_accs
        ) = array_refs![accounts, NUM_FIXED, MAX_INVESTORS];

        let [
            platform_acc,
            fund_state_acc,
            price_acc,
            mango_group_acc,
            margin_acc,
            open_orders_acc,
            oracle_acc,
            clock_sysvar_acc,
            manager_acc,
            router_btoken_acc,
            fund_btoken_acc,
            manager_btoken_acc,
            investin_btoken_acc,
            pda_router_acc,
            token_prog_acc
        ] = fixed_accs;

        let platform_data = PlatformData::load(platform_acc)?;
        let mut fund_data = FundData::load_mut(fund_state_acc)?;

        let mut margin_equity = U64F64!(0);
        if fund_data.no_of_margin_positions > 0 && fund_data.mango_positions[0].state != 0 {
            let mango_group_data = MangoGroup::load(mango_group_acc)?;
            let margin_data = MarginAccount::load(margin_acc)?;
            let token_index = fund_data.mango_positions[0].margin_index as usize;
            let (equity, coll) = get_equity_and_coll_ratio(token_index, &mango_group_data, &margin_data, oracle_acc, open_orders_acc)?;
            msg!("equity and coll:: {:?} {:?}", equity, coll);
            let debt_valuation = coll.checked_mul(U64F64::from_num(fund_data.mango_positions[0].investor_debt)).unwrap();
            margin_equity = equity.checked_sub(debt_valuation).unwrap();
        }

        // check if manager signed the tx
        check!(manager_acc.is_signer, FundError::IncorrectProgramId);

        // check if router PDA matches
        check!(*pda_router_acc.key == platform_data.router, FundError::IncorrectPDA);

        // check_owner(router_btoken_acc, pda_router_acc.key);

        msg!("Calculating transfer amount");
        let transferable_amount: u64 = U64F64::to_num(U64F64::from_num(fund_data.amount_in_router)
        .checked_mul(U64F64::from_num(98)).unwrap()
        .checked_div(U64F64::from_num(100)).unwrap());

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
      
        msg!("Management Fee Transfer");
        let management_fee: u64 = U64F64::to_num(U64F64::from_num(fund_data.amount_in_router)
        .checked_div(U64F64::from_num(100)).unwrap());

        invoke_signed(
            &(spl_token::instruction::transfer(
                token_prog_acc.key,
                router_btoken_acc.key,
                manager_btoken_acc.key,
                pda_router_acc.key,
                &[pda_router_acc.key],
                management_fee
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
        let protocol_fee: u64 = U64F64::to_num(U64F64::from_num(fund_data.amount_in_router)
        .checked_div(U64F64::from_num(100)).unwrap());

        invoke_signed(
            &(spl_token::instruction::transfer(
                token_prog_acc.key,
                router_btoken_acc.key,
                investin_btoken_acc.key,
                pda_router_acc.key,
                &[pda_router_acc.key],
                protocol_fee
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
        
        // dont update performance; just amount
        update_amount_and_performance(&mut fund_data, &price_acc, &clock_sysvar_acc, margin_equity, true)?;

        let in_queue = fund_data.no_of_investments - fund_data.number_of_active_investments;
        for i in 0..in_queue {

            let investor_state_acc = &investor_state_accs[i as usize];
            let mut investor_data = InvestorData::load_mut(investor_state_acc)?;
            if investor_data.start_performance == 0 {
                investor_data.start_performance = fund_data.prev_performance;
            }
            if investor_data.amount_in_router != 0 {
                let investment_return: u64 = U64F64::to_num(U64F64::from_num(investor_data.amount)
                .checked_mul(
                    U64F64::from_num(fund_data.prev_performance).checked_div(U64F64::from_num(investor_data.start_performance)).unwrap()
                ).unwrap());
        
                investor_data.amount = U64F64::to_num(U64F64::from_num(investment_return)
                .checked_add(U64F64::from_num(investor_data.amount_in_router)).unwrap());
                investor_data.start_performance = fund_data.prev_performance;
                
                investor_data.amount_in_router = 0;
            }
        }
        
        fund_data.tokens[0].balance = parse_token_account(&fund_btoken_acc)?.amount;
        // dont update performance now
        update_amount_and_performance(&mut fund_data, &price_acc, &clock_sysvar_acc, margin_equity, true)?;
        fund_data.number_of_active_investments = fund_data.no_of_investments;
        fund_data.amount_in_router = 0;

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
        // check if investor has signed the transaction
        check!(investor_acc.is_signer, FundError::IncorrectSignature);

        // TODO: check if manager_btoken_acc and investin_btoken_acc is correct from states
        let platform_data = PlatformData::load(platform_acc)?;
        let mut fund_data = FundData::load_mut(fund_state_acc)?;
        let mut investor_data = InvestorData::load_mut(investor_state_acc)?;

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

        } 

        if investor_data.has_withdrawn == true {
            for i in 0..NUM_TOKENS {
                // TODO:: check if fund_debt on inv_acc <= fund_debt on fund
                if investor_data.fund_debt[i] < 10 {
                    continue;
                }
                invoke_signed(
                    &(spl_token::instruction::transfer(
                        token_prog_acc.key,
                        fund_token_accs[i].key,
                        inv_token_accs[i].key,
                        pda_man_acc.key,
                        &[pda_man_acc.key],
                        investor_data.fund_debt[i]
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
                fund_data.tokens[i].debt -= investor_data.fund_debt[i];
                investor_data.fund_debt[i] = 0;
            }
        }
        investor_data.amount = 0;
        investor_data.start_performance = U64F64!(0);
        investor_data.amount_in_router = 0;
        investor_data.has_withdrawn = false;
        
        fund_data.no_of_investments -= 1;        
        Ok(())
    }

    pub fn withdraw_settle(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> Result<(), ProgramError> {

        const NUM_FIXED:usize = 9;
        let accounts = array_ref![accounts, 0, NUM_FIXED];

        let [
            fund_state_acc,
            investor_state_acc,
            investor_acc,
            price_acc,
            mango_group_acc,
            margin_acc,
            open_orders_acc,
            oracle_acc,
            clock_sysvar_acc
        ] = accounts;

        let mut fund_data = FundData::load_mut(fund_state_acc)?;
        let mut investor_data = InvestorData::load_mut(investor_state_acc)?;
        let mango_group_data = MangoGroup::load(mango_group_acc)?;
        let margin_data = MarginAccount::load(margin_acc)?;

        check!(investor_data.owner == *investor_acc.key, ProgramError::MissingRequiredSignature);
        check!(investor_acc.is_signer, ProgramError::MissingRequiredSignature);

        // TODO::  check if has_withdrawn == false
        let mut coll_ratio = U64F64!(1);
        let mut margin_equity = U64F64!(0);
        if fund_data.no_of_margin_positions > 0 && fund_data.mango_positions[0].state != 0 {
            let token_index = fund_data.mango_positions[0].margin_index as usize;
            let (equity, coll) = get_equity_and_coll_ratio(token_index, &mango_group_data, &margin_data, oracle_acc, open_orders_acc)?;
            msg!("equity and coll:: {:?} {:?}", equity, coll);
            let debt_valuation = coll.checked_mul(U64F64::from_num(fund_data.mango_positions[0].investor_debt)).unwrap();
            margin_equity = equity.checked_sub(debt_valuation).unwrap();
            coll_ratio = coll;
        }

        if investor_data.amount != 0 && investor_data.start_performance != 0 {
            update_amount_and_performance(&mut fund_data, &price_acc, &clock_sysvar_acc, margin_equity, true)?;
            let share = get_share(&mut fund_data, &mut investor_data)?;
            for i in 0..NUM_TOKENS {
                let withdraw_amount: u64 = U64F64::to_num(U64F64::from_num(fund_data.tokens[i].balance)
                .checked_mul(share).unwrap());
                investor_data.fund_debt[i] = withdraw_amount;
                fund_data.tokens[i].debt = withdraw_amount;
            }
            // active margin trade
            if margin_equity > 0 && fund_data.mango_positions[0].state != 0 {
                let margin_share: u64 = U64F64::to_num(margin_equity.checked_mul(share).unwrap()
                    .checked_div(coll_ratio).unwrap()
                );
                investor_data.margin_debt = margin_share;
                investor_data.margin_position_id = fund_data.mango_positions[0].position_id as u64;
                fund_data.mango_positions[0].investor_debt += margin_share;
            }
            fund_data.number_of_active_investments -= 1;
            investor_data.has_withdrawn = true;
            update_amount_and_performance(&mut fund_data, &price_acc, &clock_sysvar_acc, margin_equity, false)?;
        }
        Ok(())
    }

    pub fn swap(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        data: Data
    ) -> Result<(), ProgramError> {
                
        let accounts_iter = &mut accounts.iter();
        // Get the first account
        let fund_state_acc = next_account_info(accounts_iter)?;        
        let mut fund_data = FundData::load_mut(fund_state_acc)?;

        let (source_info, dest_info) = swap_instruction(&data, &fund_data, accounts)?;


        for i in 0..NUM_TOKENS {
            if fund_data.tokens[i].mint == source_info.mint {
                fund_data.tokens[i].balance = source_info.amount;
            }
            if fund_data.tokens[i].mint == dest_info.mint {
                fund_data.tokens[i].balance = dest_info.amount;
            }
            check!(fund_data.tokens[i].balance >= fund_data.tokens[i].debt, ProgramError::InsufficientFunds);
        }
        Ok(())
    }

    // manager Performance Fee Claim
    pub fn claim (
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> Result<(), ProgramError> {
        const NUM_FIXED:usize = 13;
        let accounts = array_ref![accounts, 0, NUM_FIXED];

        let [
            fund_state_acc,
            price_acc,
            mango_group_acc,
            margin_acc,
            open_orders_acc,
            oracle_acc,
            clock_sysvar_acc,
            manager_acc,
            fund_btoken_acc,
            manager_btoken_acc,
            investin_btoken_acc,
            pda_man_acc,
            token_prog_acc
        ] = accounts;

        let mut fund_data = FundData::load_mut(fund_state_acc)?;
        let mango_group_data = MangoGroup::load(mango_group_acc)?;
        let margin_data = MarginAccount::load(margin_acc)?;

        let mut margin_equity = U64F64!(0);
        if fund_data.no_of_margin_positions > 0 && fund_data.mango_positions[0].state != 0 {
            let token_index = fund_data.mango_positions[0].margin_index as usize;
            let (equity, coll) = get_equity_and_coll_ratio(token_index, &mango_group_data, &margin_data, oracle_acc, open_orders_acc)?;
            let debt_valuation = coll.checked_mul(U64F64::from_num(fund_data.mango_positions[0].investor_debt)).unwrap();
            margin_equity = equity.checked_sub(debt_valuation).unwrap();
        }
        // check if manager signed the tx
        check!(manager_acc.is_signer, FundError::IncorrectSignature);

        update_amount_and_performance(&mut fund_data, &price_acc, &clock_sysvar_acc, margin_equity, true)?;

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

        update_amount_and_performance(&mut fund_data, &price_acc, &clock_sysvar_acc, margin_equity, false)?;
        
        Ok(())

    }

    pub fn admin_control (
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        platform_is_initialized: u8,
        fund_is_initialized: u8,
        fund_min_amount: u64,
        fund_min_return: u64,
        fund_performance_fee_percentage: u64
    ) -> Result<(), ProgramError> {
        let accounts_iter = &mut accounts.iter();

        let platform_state_acc = next_account_info(accounts_iter)?;
        let investin_acc = next_account_info(accounts_iter)?;

        let mut platform_data = PlatformData::load_mut(platform_state_acc)?;
        check!(investin_acc.is_signer, FundError::IncorrectSignature);

        // freeze the platform
        if platform_is_initialized == 0 {
            platform_data.is_initialized = false;
        }
        // init
        else if !platform_data.is_initialized && platform_is_initialized == 1 {
            let (router_pda, nonce) = 
                Pubkey::find_program_address(&["router".as_ref()], program_id
            );
            platform_data.router = router_pda;
            platform_data.router_nonce = nonce;

            platform_data.is_initialized = true;
            platform_data.no_of_active_funds = 0;
            platform_data.investin_admin = *investin_acc.key;
        } else {
            let fund_state_acc = next_account_info(accounts_iter)?;
            let mut fund_data = FundData::load_mut(fund_state_acc)?;
            if fund_is_initialized == 0 {
                fund_data.is_initialized = false;
            }
            fund_data.min_amount = fund_min_amount;
            fund_data.min_return = U64F64::from_num(fund_min_return / 100);
            fund_data.performance_fee_percentage = U64F64::from_num(fund_performance_fee_percentage / 100);
        }

        Ok(())
    }

    // pub fn TestingDeposit (
    //     program_id: &Pubkey,
    //     accounts: &[AccountInfo],
    //     amount: u64
    // ) -> Result<(), ProgramError> {
    //     const NUM_FIXED:usize = 8;
    //     let accounts = array_ref![accounts, 0, NUM_FIXED];
    //     let [
    //         fund_state_acc,
    //         price_acc,
    //         clock_sysvar_acc,
    //         manager_acc,
    //         fund_btoken_acc,
    //         manager_btoken_acc,
    //         pda_man_acc,
    //         token_prog_acc
    //     ] = accounts;

    //     let mut fund_data = FundData::load_mut(fund_state_acc)?;

    //     // check if manager signed the tx
    //     // check!(&manager_acc.is_signer, FundError::IncorrectSignature);

    //     msg!("Invoking transfer instructions");
    //     let deposit_instruction = spl_token::instruction::transfer(
    //         token_prog_acc.key,
    //         manager_btoken_acc.key,
    //         fund_btoken_acc.key,
    //         manager_acc.key,
    //         &[&manager_acc.key],
    //         amount
    //     )?;
    //     let deposit_accs = [
    //         manager_btoken_acc.clone(),
    //         fund_btoken_acc.clone(),
    //         manager_acc.clone(),
    //         token_prog_acc.clone()
    //     ];
    //     invoke(&deposit_instruction, &deposit_accs)?;

    //     fund_data.tokens[0].balance = parse_token_account(&fund_btoken_acc)?.amount;
    //     update_amount_and_performance(&mut fund_data, &price_acc, &clock_sysvar_acc, true).unwrap();

    //     Ok(())
    // }

    // pub fn TestingWithdraw (
    //     program_id: &Pubkey,
    //     accounts: &[AccountInfo],
    //     amount: u64
    // ) -> Result<(), ProgramError> {
    //     const NUM_FIXED:usize = 8;
    //     let accounts = array_ref![accounts, 0, NUM_FIXED];
    //     let [
    //         fund_state_acc,
    //         price_acc,
    //         clock_sysvar_acc,
    //         manager_acc,
    //         fund_btoken_acc,
    //         manager_btoken_acc,
    //         pda_man_acc,
    //         token_prog_acc
    //     ] = accounts;

    //     let mut fund_data = FundData::load_mut(fund_state_acc)?;

    //     // check if manager signed the tx
    //     // check!(&manager_acc.is_signer, FundErrorCode::IncorrectSignature);

    //     msg!("Invoking transfer instructions");
    //     let transfer_instruction = spl_token::instruction::transfer(
    //         token_prog_acc.key,
    //         fund_btoken_acc.key,
    //         manager_btoken_acc.key,
    //         pda_man_acc.key,
    //         &[pda_man_acc.key],
    //         amount
    //     )?;
    //     let transfer_accs = [
    //         fund_btoken_acc.clone(),
    //         manager_btoken_acc.clone(),
    //         pda_man_acc.clone(),
    //         token_prog_acc.clone()
    //     ];
    //     let signer_seeds = [fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)];
    //     invoke_signed(&transfer_instruction, &transfer_accs, &[&signer_seeds])?;

    //     fund_data.tokens[0].balance = parse_token_account(&fund_btoken_acc)?.amount;
    //     update_amount_and_performance(&mut fund_data, &price_acc, &clock_sysvar_acc, true)?;

    //     Ok(())
    // }

    // instruction processor
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        data: &[u8]
    ) -> Result<(), ProgramError> {
        let instruction = FundInstruction::unpack(data).ok_or(ProgramError::InvalidInstructionData)?;
        match instruction {
            FundInstruction::Initialize { min_amount, min_return, performance_fee_percentage } => {
                msg!("FundInstruction::Initialize");
                return Self::initialize(program_id, accounts, min_amount, min_return, performance_fee_percentage);
            }
            FundInstruction::InvestorDeposit { amount } => {
                msg!("FundInstruction::InvestorDeposit");
                return Self::deposit(program_id, accounts, amount);
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
            FundInstruction::Swap { data } => {
                msg!("FundInstruction::Swap");
                return Self::swap(program_id, accounts, data);
            }
            FundInstruction::ClaimPerformanceFee {} => {
                msg!("FundInstruction::ClaimPerformanceFee");
                return Self::claim(program_id, accounts);
            }
            FundInstruction::MangoInitialize  => {
                msg!("FundInstruction::MangoInitialize");
                return mango_init_margin_account(program_id, accounts);
            }
            FundInstruction::AdminControl {platform_is_initialized, fund_is_initialized, fund_min_amount, fund_min_return, fund_performance_fee_percentage} => {
                msg!("FundInstruction::AdminControl");
                return Self::admin_control(program_id, accounts, platform_is_initialized, fund_is_initialized, fund_min_amount, fund_min_return, fund_performance_fee_percentage);
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
            _ => {
                Ok(())
            }
        }
    }
}

pub fn find_index (
    price_data: &PriceAccount,
    mint: Pubkey
) -> Result<usize, ProgramError> {
    for i in 0..price_data.count {
        if price_data.prices[i as usize].token_mint == mint {
            return Ok(i as usize)
        }
    }
    return Err(ProgramError::InvalidArgument)
}

// calculate prices, get fund valuation and performance
pub fn update_amount_and_performance(
    fund_data: &mut FundData,
    price_acc: &AccountInfo,
    clock_sysvar_acc: &AccountInfo,
    margin_equity: U64F64,
    update_perf: bool
) -> Result<(), ProgramError> {
    
    msg!("called update_amount_and_performance");

    // get price account info
    let price_data = PriceAccount::load_mut(price_acc)?;
    let clock = &Clock::from_account_info(clock_sysvar_acc)?;

    // add USDT balance (not decimal adjusted)
    let mut fund_val = U64F64::from_num(fund_data.tokens[0].balance - fund_data.tokens[0].debt).
        checked_add(margin_equity).unwrap();
    msg!("margin_equity:: {:?}", margin_equity);
    // Calculate prices for all tokens with balances
    for i in 0..(NUM_TOKENS-1) {

        // dont update if token balance == 0
        if fund_data.tokens[i+1].balance == 0 { continue; }

        // get index of token
        let index = find_index(&price_data, fund_data.tokens[i+1].mint)?;

        if clock.unix_timestamp - price_data.prices[index].last_updated > 100 {
            msg!("price not up-to-date.. aborting");
            return Err(FundError::PriceStaleInAccount.into())
        }
        // calculate price in terms of base token
        let val: U64F64 = U64F64::from_num(fund_data.tokens[i+1].balance - fund_data.tokens[i+1].debt)
        .checked_mul(U64F64::from_num(price_data.prices[index].token_price)).unwrap()
        .checked_div(U64F64::from_num(10u64.pow(fund_data.tokens[i+1].decimals as u32))).unwrap();

        fund_val = fund_val.checked_add(val).unwrap();
    }
    if update_perf {
        let mut perf = U64F64::from_num(fund_data.prev_performance);
        // only case where performance is not updated:
        // when no investments and no performance fee for manager
        if fund_data.number_of_active_investments != 0 || fund_data.performance_fee != 0 {
            perf = fund_val.checked_div(U64F64::from_num(fund_data.total_amount)).unwrap()
            .checked_mul(U64F64::from_num(fund_data.prev_performance)).unwrap();
        }
        // adjust for manager performance fee
        fund_data.performance_fee = U64F64::to_num(U64F64::from_num(perf)
            .checked_div(U64F64::from_num(fund_data.prev_performance)).unwrap()
            .checked_mul(U64F64::from_num(fund_data.performance_fee)).unwrap());
        fund_data.prev_performance = U64F64::to_num(perf);
    }
    
    fund_data.total_amount = U64F64::to_num(fund_val);
    
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

    for i in 0..NUM_MARKETS {
        //check_eq_default!(&mango_group.oracles[i], oracle_accs[i].key)?;

        // TODO store this info in MangoGroup, first make sure it cannot be changed by solink
        let quote_adj = U64F64::from_num(
            10u64.pow(quote_decimals.checked_sub(mango_group.oracle_decimals[token_index]).unwrap() as u32)
        );

        let answer = flux_aggregator::read_median(oracle_acc)?; // this is in USD cents

        let value = U64F64::from_num(answer.median);

        let base_adj = U64F64::from_num(10u64.pow(mango_group.mint_decimals[i] as u32));
        price = quote_adj
            .checked_div(base_adj).unwrap()
            .checked_mul(value).unwrap();
    }
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
        .checked_add(U64F64::from_num(open_orders.native_pc_total + open_orders.referrer_rebates_accrued)).unwrap()
        .checked_mul(price).unwrap()
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

pub fn swap_instruction(
    data: &Data,
    fund_data: &FundData,
    accounts: &[AccountInfo]
) -> Result<(Account, Account), ProgramError>{
    let accounts = array_ref![accounts, 0, 20];
    let [
        fund_state_acc,
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
                AccountMeta::new(*token_prog_acc.key, false),
                AccountMeta::new(*amm_id.key, false),
                AccountMeta::new(*amm_authority.key, false),
                AccountMeta::new(*amm_open_orders.key, false),
                AccountMeta::new(*amm_target_orders.key, false),
                AccountMeta::new(*pool_coin_token_acc.key, false),
                AccountMeta::new(*pool_pc_token_acc.key, false),
                AccountMeta::new(*dex_prog_acc.key, false),
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

    Ok((source_info, dest_info))
}

pub fn get_share(
    fund_data: &mut FundData,
    investor_data: &mut InvestorData,
) -> Result<U64F64, ProgramError> {
    let perf_share = U64F64::from_num(fund_data.prev_performance)
    .checked_div(U64F64::from_num(investor_data.start_performance)).unwrap();

    msg!("performance: {:?}", perf_share);

    let actual_amount: u64 = U64F64::to_num(U64F64::from_num(investor_data.amount)
    .checked_mul(U64F64::from_num(98)).unwrap()
    .checked_div(U64F64::from_num(100)).unwrap());

    let mut investment_return = U64F64::from_num(actual_amount)
    .checked_mul(perf_share).unwrap();

    // check if withdraw exceed
    // check!(amount <= U64F64::to_num(total_share), ProgramError::InsufficientFunds);

    // in case of profit
    if investment_return > actual_amount {
        let profit = U64F64::from_num(investment_return)
        .checked_sub(U64F64::from_num(actual_amount)).unwrap();
        let performance: u64 = U64F64::to_num(profit.checked_div(U64F64::from_num(actual_amount)).unwrap()
        .checked_mul(U64F64::from_num(1000000)).unwrap());
        // if performance exceeds min return; update manager performance fees
        if performance >= fund_data.min_return {
            investment_return = U64F64::from_num(profit)
            .checked_mul(
                (U64F64::from_num(1000000).checked_sub(U64F64::from_num(fund_data.performance_fee_percentage)).unwrap())
                .checked_div(U64F64::from_num(1000000)).unwrap()
                ).unwrap()
            .checked_add(U64F64::from_num(actual_amount)).unwrap();

            fund_data.performance_fee = U64F64::to_num(U64F64::from_num(fund_data.performance_fee)
            .checked_add(U64F64::from_num(profit)
            .checked_mul(
                U64F64::from_num(fund_data.performance_fee_percentage)
                .checked_div(U64F64::from_num(1000000)).unwrap()
            ).unwrap()).unwrap()
            );
        }
    }

    let share = U64F64::from_num(investment_return)
    .checked_div(U64F64::from_num(fund_data.total_amount)).unwrap();

    Ok(share)
}