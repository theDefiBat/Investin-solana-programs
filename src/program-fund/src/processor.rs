use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::bytes_of;
use fixed::types::U64F64;
use solana_program::{
    account_info::AccountInfo,
    msg,
    instruction:: {AccountMeta, Instruction},
    program_error::ProgramError,
    program_pack::{Pack, IsInitialized},
    pubkey::Pubkey,
    program::{invoke, invoke_signed},
};

use arrayref::{array_ref, array_refs};

use spl_token::state::{Account, Mint};

use crate::error::{check_assert, FundError, FundErrorCode};
use crate::instruction::{FundInstruction, Data};
use crate::state::{NUM_TOKENS, MAX_INVESTORS, FundData, InvestorData, PlatformData};

macro_rules! check {
    ($cond:expr, $err:expr) => {
        check_assert($cond, $err, line!())
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
        let mut platform_data = PlatformData::try_from_slice(&platform_acc.data.borrow())?;
        let mut fund_data = FundData::try_from_slice(&fund_state_acc.data.borrow())?;

        //  check if already init
        check!(!fund_data.is_initialized(), FundErrorCode::FundAccountAlreadyInit);

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
            fund_data.tokens[i].decimals = mint.decimals;
            fund_data.tokens[i].balance = 0;
        }

        fund_data.decimals = 6;
        fund_data.min_amount = min_amount;
        fund_data.min_return = min_return;
        fund_data.performance_fee_percentage = performance_fee_percentage;

        fund_data.total_amount = 0; 
        fund_data.prev_performance = 1000000;
        fund_data.number_of_active_investments = 0;

        // get nonce for signing later
        let (_pda, nonce) = Pubkey::find_program_address(&[&*manager_acc.key.as_ref()], program_id);
        fund_data.signer_nonce = nonce;
        fund_data.is_initialized = true;

        msg!("Serialising data");
        fund_data.serialize(&mut *fund_state_acc.data.borrow_mut());

        // update platform_data
        platform_data.fund_managers[platform_data.no_of_active_funds as usize] = *manager_acc.key;
        platform_data.no_of_active_funds += 1;
        
        platform_data.serialize(&mut *platform_acc.data.borrow_mut());

        msg!("fund state:: {:?}", fund_data);
        msg!("investor state:: {:?}", platform_data);

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

        let mut fund_data = FundData::try_from_slice(&fund_state_acc.data.borrow())?;
        let mut investor_data = InvestorData::try_from_slice(&investor_state_acc.data.borrow())?;

        // check if fund state acc passed is initialised
        check!(!fund_data.is_initialized(), FundErrorCode::FundAccountAlreadyInit);

        // TODO: check if pda_man_acc is derived from fund_data.manager_account
        // TODO: check if router_btoken_acc is derived from pda_inv_acc

        // check if amount deposited is more than the minimum amount for the fund
        check!(amount >= fund_data.min_amount, FundErrorCode::InvalidAmount);

        // check if investor has signed the transaction
        check!(investor_acc.is_signer, FundErrorCode::IncorrectSignature);
        // TODO: check if investor state account is derived from its address

        // check if investor_state_account is already initialised
        check!(investor_data.is_initialized(), FundErrorCode::InvestorAccountAlreadyInit);

        check!(*token_prog_acc.key == spl_token::id(), FundErrorCode::IncorrectProgramId);
        
        // get nonce for signing later
        let (_pda, nonce) = Pubkey::find_program_address(&[&*investor_acc.key.as_ref()], program_id);

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
        invoke(&deposit_instruction, &deposit_accs);

        msg!("Deposit done..");

        investor_data.is_initialized = true;
        investor_data.owner = *investor_acc.key;
        investor_data.amount = amount;
        investor_data.start_performance = 0;
        // Store manager's PDA
        investor_data.manager = *pda_man_acc.key;
        investor_data.serialize(&mut *investor_state_acc.data.borrow_mut());
        
        fund_data.amount_in_router += amount;
        fund_data.investors[fund_data.no_of_investments as usize] = *investor_acc.key;
        fund_data.no_of_investments += 1;
        fund_data.serialize(&mut *fund_state_acc.data.borrow_mut());

        msg!("fund state:: {:?}", fund_data);
        msg!("investor state:: {:?}", investor_data);
        
        Ok(())
    }

    // manager transfer
    pub fn transfer (
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> Result<(), ProgramError> {

        const NUM_FIXED:usize = 9;
        let accounts = array_ref![accounts, 0, NUM_FIXED + 2*(NUM_TOKENS-1) + MAX_INVESTORS];

        let (
            fixed_accs,
            pool_accs,
            investor_state_accs
        ) = array_refs![accounts, NUM_FIXED, 2*(NUM_TOKENS-1), MAX_INVESTORS];

        let [
            platform_acc,
            fund_state_acc,
            manager_acc,
            router_btoken_acc,
            fund_btoken_acc,
            manager_btoken_acc,
            investin_btoken_acc,
            pda_router_acc,
            token_prog_acc
        ] = fixed_accs;

        let platform_data = PlatformData::try_from_slice(&platform_acc.data.borrow())?;
        let mut fund_data = FundData::try_from_slice(&fund_state_acc.data.borrow())?;

        // check if manager signed the tx
        check!(manager_acc.is_signer, FundErrorCode::IncorrectProgramId);

        // check if router PDA matches
        check!(*pda_router_acc.key == platform_data.router, FundErrorCode::IncorrectPDA);

        msg!("Calculating transfer amount");
        let transferable_amount: u64 = U64F64::to_num(U64F64::from_num(fund_data.amount_in_router)
        .checked_mul(U64F64::from_num(98)).unwrap()
        .checked_div(U64F64::from_num(100)).unwrap());

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
        );
      
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
        );
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
        );
        
        msg!("Transfers completed");
        let tc = parse_token_account(fund_btoken_acc)?;
        
        fund_data.tokens[0].balance += tc.amount;
        fund_data.number_of_active_investments = fund_data.no_of_investments;
        fund_data.amount_in_router = 0;
        fund_data.serialize(&mut *fund_state_acc.data.borrow_mut());

        if fund_data.number_of_active_investments > 0 {
            update_amount_and_performance(fund_state_acc, pool_accs);
        }
        else {
            fund_data.total_amount = transferable_amount;
            fund_data.prev_performance = 1000000;
            fund_data.serialize(&mut *fund_state_acc.data.borrow_mut());
        }

        let in_queue = fund_data.no_of_investments - fund_data.number_of_active_investments;
        for i in 0..in_queue {

            let investor_state_acc = &investor_state_accs[i as usize];
            let mut investor_data = InvestorData::try_from_slice(&investor_state_acc.data.borrow())?;
            if investor_data.start_performance == 0 {
                investor_data.start_performance = fund_data.prev_performance;
                investor_data.serialize(&mut *investor_state_acc.data.borrow_mut());
            }
        }        
        msg!("fund state:: {:?}", fund_data);  

        Ok(())
    }
    // investor withdraw
    pub fn withdraw (
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64
    ) -> Result<(), ProgramError> {


        const NUM_FIXED:usize = 10;
        let accounts = array_ref![accounts, 0, NUM_FIXED + 4*NUM_TOKENS - 2];

        let (
            fixed_accs,
            inv_token_accs,
            fund_token_accs,
            pool_accs
        ) = array_refs![accounts, NUM_FIXED, NUM_TOKENS, NUM_TOKENS, 2*(NUM_TOKENS-1)];

        let [
            platform_acc,
            fund_state_acc,
            investor_state_acc,
            investor_acc,
            router_btoken_acc,
            manager_btoken_acc,
            investin_btoken_acc,
            pda_man_acc,
            pda_router_acc,
            token_prog_acc
        ] = fixed_accs;
        // check if investor has signed the transaction
        check!(investor_acc.is_signer, FundErrorCode::IncorrectSignature);

        // TODO: check if manager_btoken_acc and investin_btoken_acc is correct from states
        let platform_data = PlatformData::try_from_slice(&platform_acc.data.borrow())?;
        let mut fund_data = FundData::try_from_slice(&fund_state_acc.data.borrow())?;
        let mut investor_data = InvestorData::try_from_slice(&investor_state_acc.data.borrow())?;

        // Manager has not transferred to vault
        if investor_data.start_performance == 0 {
            invoke_signed(
                &(spl_token::instruction::transfer(
                    token_prog_acc.key,
                    router_btoken_acc.key,
                    inv_token_accs[0].key,
                    pda_router_acc.key,
                    &[pda_router_acc.key],
                    investor_data.amount
                ))?,
                &[
                    router_btoken_acc.clone(),
                    inv_token_accs[0].clone(),
                    pda_router_acc.clone(),
                    token_prog_acc.clone()
                ],
                &[&["router".as_ref(), bytes_of(&platform_data.router_nonce)]]
            );

        } else {
            update_amount_and_performance(fund_state_acc, pool_accs);

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
            let profit = U64F64::from_num(investment_return)
            .checked_sub(U64F64::from_num(actual_amount)).unwrap();
            let performance: u64 = U64F64::to_num(profit.checked_div(U64F64::from_num(actual_amount)).unwrap()
            .checked_mul(U64F64::from_num(100)).unwrap());
            if performance >= fund_data.min_return {

                investment_return = U64F64::from_num(profit)
                .checked_mul(
                    (U64F64::from_num(100).checked_sub(U64F64::from_num(fund_data.performance_fee_percentage)).unwrap())
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
                // let performanceFeeManager = U64F64::to_num(U64F64::from_num(performanceFee).checked_mul(U64F64::from_num(90)).unwrap()
                //     .checked_div(U64F64::from_num(100)).unwrap());
                
                // let transfer_instruction = spl_token::instruction::transfer(
                //     token_prog_acc.key,
                //     fund_token_accs[0].key,
                //     manager_btoken_acc.key,
                //     pda_man_acc.key,
                //     &[pda_man_acc.key],
                //     performanceFeeManager
                // )?;
                // let withdraw_accs = [
                //     fund_token_accs[0].clone(),
                //     manager_btoken_acc.clone(),
                //     pda_man_acc.clone(),
                //     token_prog_acc.clone()
                // ];
                // let signer_seeds = [fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)];
                // invoke_signed(&transfer_instruction, &withdraw_accs, &[&signer_seeds])?;

                // let platformFee = U64F64::to_num(U64F64::from_num(performanceFee)
                // .checked_div(U64F64::from_num(10)).unwrap());

                // let transfer_instruction = spl_token::instruction::transfer(
                //     token_prog_acc.key,
                //     fund_token_accs[0].key,
                //     investin_btoken_acc.key,
                //     pda_man_acc.key,
                //     &[pda_man_acc.key],
                //     platformFee
                // )?;
                // let withdraw_accs = [
                //     fund_token_accs[0].clone(),
                //     investin_btoken_acc.clone(),
                //     pda_man_acc.clone(),
                //     token_prog_acc.clone()
                // ];
                // let signer_seeds = [fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)];
                // invoke_signed(&transfer_instruction, &withdraw_accs, &[&signer_seeds])?;

            }

            let share = U64F64::from_num(investment_return)
            .checked_div(U64F64::from_num(fund_data.total_amount)).unwrap();

            for i in 0..NUM_TOKENS {
                let withdraw_amount = U64F64::from_num(fund_data.tokens[i].balance)
                .checked_mul(share).unwrap();
                let withdraw_rounded = U64F64::to_num(withdraw_amount);
            
                if withdraw_rounded < 100 {
                continue;
                }

                msg!("Invoking withdraw instruction");
                invoke_signed(
                    &(spl_token::instruction::transfer(
                        token_prog_acc.key,
                        fund_token_accs[i].key,
                        inv_token_accs[i].key,
                        pda_man_acc.key,
                        &[pda_man_acc.key],
                        withdraw_rounded
                    ))?,
                    &[
                        fund_token_accs[i].clone(),
                        inv_token_accs[i].clone(),
                        pda_man_acc.clone(),
                        token_prog_acc.clone()
                    ],
                    &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
                );
                msg!("withdraw instruction done");

                fund_data.tokens[i].balance = parse_token_account(&fund_token_accs[i])?.amount;
            }
            fund_data.number_of_active_investments -= 1;
        }
        
        investor_data.amount = 0;
        investor_data.start_performance = 0;
        investor_data.serialize(&mut *investor_state_acc.data.borrow_mut());
        
        fund_data.no_of_investments -= 1;
        fund_data.serialize(&mut *fund_state_acc.data.borrow_mut());
        update_amount_and_performance(fund_state_acc, pool_accs);

        msg!("fund state:: {:?}", fund_data);
        msg!("investor state:: {:?}", investor_data);
        
        Ok(())
    }

    pub fn swap(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        data: Data
    ) -> Result<(), ProgramError> {
        
        const NUM_FIXED: usize = 20;
        let accounts = array_ref![accounts, 0, NUM_FIXED];
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

        let mut fund_data = FundData::try_from_slice(&fund_state_acc.data.borrow())?;

        msg!("swap instruction call");
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
                    AccountMeta::new(*dex_prog_acc.key, true),
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
                owner_token_acc.clone()
            ],
            &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
        );
        msg!("swap instruction done");
        
        // update balances on fund_state_acc for source & dest
        let source_info = parse_token_account(source_token_acc)?;
        let dest_info = parse_token_account(dest_token_acc)?;

        for i in 0..NUM_TOKENS {
            if fund_data.tokens[i].mint == source_info.mint {
                fund_data.tokens[i].balance = source_info.amount;
            }
            if fund_data.tokens[i].mint == dest_info.mint {
                fund_data.tokens[i].balance = dest_info.amount;
            }
        }
        fund_data.serialize(&mut *fund_state_acc.data.borrow_mut());
        Ok(())    
    }

    // manager Performance Fee Claim
    pub fn claim (
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> Result<(), ProgramError> {
        const NUM_FIXED:usize = 7;
        let accounts = array_ref![accounts, 0, NUM_FIXED];
        let [
            fund_state_acc,
            manager_acc,
            fund_btoken_acc,
            manager_btoken_acc,
            investin_btoken_acc,
            pda_man_acc,
            token_prog_acc
        ] = accounts;

        let mut fund_data = FundData::try_from_slice(&fund_state_acc.data.borrow())?;

        // check if manager signed the tx
        check!(manager_acc.is_signer, FundErrorCode::IncorrectSignature);

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
        invoke_signed(&transfer_instruction, &transfer_accs, &[&signer_seeds]);

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
        invoke_signed(&transfer_instruction, &transfer_accs, &[&signer_seeds]);

        msg!("Transfer Complete");
        
        fund_data.performance_fee = 0;
        fund_data.serialize(&mut *fund_state_acc.data.borrow_mut());
        Ok(())

    }

    // instruction processor
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        data: &[u8]
    ) -> Result<(), ProgramError> {
        msg!("Program Entrypoint");
        let instruction = FundInstruction::try_from_slice(data)?;
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
            FundInstruction::InvestorWithdraw { amount } => {
                msg!("FundInstruction::InvestorWithdraw");
                return Self::withdraw(program_id, accounts, amount);
            }
            FundInstruction::Swap { data } => {
                msg!("FundInstruction::Swap");
                return Self::swap(program_id, accounts, data);
            }
            FundInstruction::ClaimPerformanceFee {} => {
                msg!("FundInstruction::ClaimPerformanceFee");
                return Self::claim(program_id, accounts);
            }
            _ => {
                Ok(())
            }
        }
    }
}

// calculate prices, get fund valuation and performance
pub fn update_amount_and_performance(
    fund_state_acc: &AccountInfo,
    pool_accs: &[AccountInfo]
) -> Result<(), ProgramError> {

    let mut fund_data = FundData::try_from_slice(&fund_state_acc.data.borrow())?;
    
    // add USDT balance (not decimal adjusted)
    let mut fund_val = U64F64::from_num(fund_data.tokens[0].balance);
    // Calculate prices for all tokens with balances
    for i in 0..(NUM_TOKENS-1) {
        let coin_acc = &pool_accs[i];
        let pc_acc = &pool_accs[i+1];
        let coin_data = Account::unpack(&coin_acc.data.borrow())?;
        let pc_data = Account::unpack(&pc_acc.data.borrow())?;

        // match for token pair sequence
        check!(coin_data.mint == fund_data.tokens[i+1].mint, FundErrorCode::InvalidTokenAccount);
        check!(pc_data.mint == fund_data.tokens[0].mint, FundErrorCode::InvalidTokenAccount);
        
        // get reserves ratio
        let pc_res = U64F64::from_num(pc_data.amount);
        let coin_res = U64F64::from(coin_data.amount);

        // calculate price in terms of base token
        let val: U64F64 = pc_res
        .checked_div(coin_res).unwrap()
        .checked_mul(U64F64::from_num(fund_data.tokens[i+1].balance)).unwrap();

        fund_val = fund_val.checked_add(val).unwrap();
    }

    let perf = fund_val.checked_div(U64F64::from_num(fund_data.total_amount)).unwrap()
    .checked_mul(U64F64::from_num(fund_data.prev_performance)).unwrap();
    
    fund_data.prev_performance = U64F64::to_num(perf);
    fund_data.total_amount = U64F64::to_num(fund_val);

    fund_data.serialize(&mut *fund_state_acc.data.borrow_mut());

    Ok(())
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
