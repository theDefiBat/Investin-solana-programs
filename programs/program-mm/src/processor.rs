use bytemuck::bytes_of;
use fixed::types::U64F64;
use fixed_macro::types::U64F64;
use fixed::traits::FromFixed;

use fixed::types::I80F48;
use fixed_macro::types::I80F48;

use solana_program::{
    account_info::AccountInfo,
    msg,
    program_error::ProgramError,
    program_pack::{Pack, IsInitialized},
    pubkey::Pubkey,
    program::{invoke, invoke_signed},
};

use arrayref::array_ref;

use spl_token::state::Account;

use crate::error::FundError;
use crate::instruction::FundInstruction;
use crate::state::{FundData, InvestorData};
use crate::mango_utils::*;

use mango::state::{MangoAccount, MangoGroup, MangoCache, PerpMarket, MAX_PAIRS, QUOTE_INDEX};
use mango::instruction::{ cancel_all_perp_orders, withdraw, place_perp_order, consume_events };
use mango::ids::mngo_token;
use mango::matching::{Side, OrderType, Book};

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

pub mod investin_vault {
    use solana_program::declare_id;
    // set investin admin
    declare_id!("2C7AtpEbcdfmDzh5g4cFBzCXbgZJmxhY2bWPMi7QKqBH");
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
        perp_market_index: u8
    ) -> Result<(), ProgramError> {

        const NUM_FIXED:usize = 8;
        let accounts = array_ref![accounts, 0, NUM_FIXED];

        let [
            fund_state_ai,
            manager_ai,
            fund_pda_ai,
            fund_vault_ai,
            fund_mngo_vault_ai,
            mango_group_ai,
            mango_account_ai,
            mango_prog_ai,
        ] = accounts;

        let mut fund_data = FundData::load_mut_checked(fund_state_ai, program_id)?;

        check!(min_return >= 500, ProgramError::InvalidArgument);
        check!(min_amount >= 10000000, ProgramError::InvalidArgument);
        check!(performance_fee_percentage >= 100 && performance_fee_percentage <= 4000, ProgramError::InvalidArgument);
        check!(perp_market_index > 0 && (perp_market_index as usize) < MAX_PAIRS, ProgramError::InvalidArgument);

        // check for manager's signature
        check!(manager_ai.is_signer, ProgramError::MissingRequiredSignature);
        // save manager's wallet address
        fund_data.manager_account = *manager_ai.key;

        // get nonce for signing later
        let (pda, nonce) = Pubkey::find_program_address(&[&*manager_ai.key.as_ref()], program_id);
        fund_data.fund_pda = pda;
        fund_data.signer_nonce = nonce;

        // check for ownership of vault
        let fund_vault = parse_token_account(fund_vault_ai)?;
        let fund_mngo_vault = parse_token_account(fund_mngo_vault_ai)?;

        check_eq!(fund_vault.owner, fund_data.fund_pda);
        check_eq!(fund_mngo_vault.owner, fund_data.fund_pda);
        check_eq!(&fund_mngo_vault.mint, &mngo_token::ID); // check for mngo mint

        fund_data.vault_key = *fund_vault_ai.key;
        fund_data.mngo_vault_key = *fund_mngo_vault_ai.key;
        fund_data.vault_balance = 0;

        // Init Mango account for the fund
        invoke_signed(
            &init_mango_account(mango_prog_ai.key, mango_group_ai.key, mango_account_ai.key, fund_pda_ai.key)?,
            &[
                mango_prog_ai.clone(),
                mango_group_ai.clone(),
                mango_account_ai.clone(),
                fund_pda_ai.clone(),
            ],
            &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
        )?;
        fund_data.mango_account = *mango_account_ai.key;
        fund_data.delegate = Pubkey::default();

        fund_data.min_amount = min_amount;
        fund_data.min_return = U64F64::from_num(min_return / 100);
        fund_data.performance_fee_percentage = U64F64::from_num(performance_fee_percentage / 100);

        fund_data.total_amount = U64F64!(0); 
        fund_data.prev_performance = U64F64!(1.00);
        fund_data.no_of_investments = 0;
        fund_data.mngo_per_share = U64F64!(0);
        fund_data.deposits = 0;
        fund_data.mngo_accrued = 0;
        fund_data.total_mngo_accrued = 0;
        fund_data.mngo_manager = 0;
        fund_data.perp_market_index = perp_market_index;

        fund_data.is_initialized = true;

        Ok(())
    }

    // investor deposit
    pub fn deposit (
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64
    ) -> Result<(), ProgramError> {
        const NUM_FIXED:usize = 11;
        let accounts = array_ref![accounts, 0, NUM_FIXED];

        let [
            fund_state_ai,
            investor_state_ai,
            investor_ai,
            investor_btoken_ai,
            fund_vault_ai,
            investin_vault_ai,
            mango_prog_ai,
            mango_group_ai,
            mango_account_ai,
            mango_cache_ai,
            token_prog_ai
        ] = accounts;

        let mut fund_data = FundData::load_mut_checked(fund_state_ai, program_id)?;
        let mut investor_data = InvestorData::load_mut_checked(investor_state_ai, program_id)?;

        // check if fund state acc passed is initialised
        check!(fund_data.is_initialized(), FundError::InvalidStateAccount);
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

        update_amount_and_performance(&mut fund_data, mango_account_ai, mango_group_ai,
            mango_cache_ai, mango_prog_ai, true)?;
        
        // check for transfers
        check!(*token_prog_ai.key == spl_token::id(), FundError::IncorrectProgramId);
        check_eq!(fund_data.vault_key, *fund_vault_ai.key);
        check_eq!(investin_vault::ID, *investin_vault_ai.key);

        let protocol_fees = U64F64::to_num(U64F64::from_num(amount).checked_mul(U64F64!(0.005)).unwrap());

        let adj_amount = amount.checked_sub(protocol_fees).unwrap();
        invoke(
            &(spl_token::instruction::transfer(
                token_prog_ai.key,
                investor_btoken_ai.key,
                investin_vault_ai.key,
                investor_ai.key,
                &[&investor_ai.key],
                protocol_fees
            ))?,
            &[
                investor_btoken_ai.clone(),
                investin_vault_ai.clone(),
                investor_ai.clone(),
                token_prog_ai.clone()
            ]
        )?;

        msg!("Depositing tokens..");
        let deposit_instruction = spl_token::instruction::transfer(
            token_prog_ai.key,
            investor_btoken_ai.key,
            fund_vault_ai.key,
            investor_ai.key,
            &[&investor_ai.key],
            adj_amount
        )?;
        let deposit_accs = [
            investor_btoken_ai.clone(),
            fund_vault_ai.clone(),
            investor_ai.clone(),
            token_prog_ai.clone()
        ];
        invoke(&deposit_instruction, &deposit_accs)?;

        // update balances
        fund_data.vault_balance = parse_token_account(fund_vault_ai)?.amount;
        fund_data.total_amount = fund_data.total_amount.checked_add(U64F64::from_num(adj_amount)).unwrap();
        fund_data.deposits = fund_data.deposits.checked_add(adj_amount).unwrap();
        fund_data.no_of_investments += 1;

        // update investor acc
        investor_data.amount = adj_amount;
        investor_data.start_performance = fund_data.prev_performance;
        investor_data.mngo_debt = fund_data.mngo_per_share;
        msg!("Deposit done..");
        
        Ok(())
    }

    // investor withdraw
    pub fn investor_withdraw(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> Result<(), ProgramError> {

        const NUM_FIXED:usize = 20;
        let accounts = array_ref![accounts, 0, NUM_FIXED];

        let [
            fund_state_ai,
            investor_state_ai,
            investor_ai,
            fund_vault_ai,
            mango_prog_ai,

            mango_group_ai,
            mango_account_ai,
            fund_pda_ai,
            mango_cache_ai,
            perp_market_ai,
            bids_ai,
            asks_ai,
            event_queue_ai,

            root_bank_ai,
            node_bank_ai,
            vault_ai,
            inv_token_ai,
            signer_ai,
            token_prog_ai,
            default_ai,
        ] = accounts;

        let mut fund_data = FundData::load_mut_checked(fund_state_ai, program_id)?;
        let mut investor_data = InvestorData::load_mut_checked(investor_state_ai, program_id)?;

        check!(investor_ai.is_signer, FundError::IncorrectSignature);
        check_eq!(investor_data.owner, *investor_ai.key);
        check_eq!(investor_data.manager, fund_data.manager_account);

        fund_data.vault_balance = parse_token_account(fund_vault_ai)?.amount;
        let (mango_deposits, pnl) = 
            update_amount_and_performance(&mut fund_data, mango_account_ai, mango_group_ai, mango_cache_ai, mango_prog_ai, true)?;
        let share = get_share(&mut fund_data, &mut investor_data)?;
        msg!("share:: {:?}", share);
        let mut withdrawable_amount = U64F64::to_num(share.checked_mul(fund_data.total_amount).unwrap());
        
        if fund_data.no_of_investments == 1 {
            withdrawable_amount += 1;
        }

        msg!("withdrawable_amount:: {:?}", withdrawable_amount);
        // if free USDC available, transfer whole amount from vault
        if fund_data.vault_balance >= withdrawable_amount {
            invoke_signed(
                &(spl_token::instruction::transfer(
                    token_prog_ai.key,
                    fund_vault_ai.key,
                    inv_token_ai.key,
                    fund_pda_ai.key,
                    &[fund_pda_ai.key],
                    withdrawable_amount
                ))?,
                &[
                    fund_vault_ai.clone(),
                    inv_token_ai.clone(),
                    fund_pda_ai.clone(),
                    token_prog_ai.clone()
                ],
                &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
            )?;
        }
        else {
            let vault_transfer = U64F64::to_num(share.checked_mul(U64F64::from_num(fund_data.vault_balance)).unwrap());
            invoke_signed(
                &(spl_token::instruction::transfer(
                    token_prog_ai.key,
                    fund_vault_ai.key,
                    inv_token_ai.key,
                    fund_pda_ai.key,
                    &[fund_pda_ai.key],
                    vault_transfer
                ))?,
                &[
                    fund_vault_ai.clone(),
                    inv_token_ai.clone(),
                    fund_pda_ai.clone(),
                    token_prog_ai.clone()
                ],
                &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
            )?;

            // cancel all perp orders
            invoke_signed(
                &cancel_all_perp_orders(mango_prog_ai.key, mango_group_ai.key, mango_account_ai.key, fund_pda_ai.key,
                    perp_market_ai.key, bids_ai.key, asks_ai.key, 10)?,
                &[
                    mango_prog_ai.clone(),
                    mango_group_ai.clone(),
                    mango_account_ai.clone(),
                    fund_pda_ai.clone(),
                    perp_market_ai.clone(),
                    bids_ai.clone(),
                    asks_ai.clone(),
                ],
                &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
            )?;

            // no spot orders
            let open_orders_accs = [Pubkey::default(); MAX_PAIRS];

            msg!("calling perp_withdraw");
            let (side, price, quantity) = get_perp_withdraw(share, fund_data.perp_market_index as usize,
                mango_account_ai, mango_group_ai, mango_cache_ai, perp_market_ai, bids_ai, asks_ai, mango_prog_ai)?;
            // close perp positions according to share ratio (if any)
            msg!("side:: {:?}, price:: {:?}, qty:: {:?}", side, price, quantity);
            if quantity > 0 {
                invoke_signed(
                    &place_perp_order(mango_prog_ai.key,
                        mango_group_ai.key, mango_account_ai.key, fund_pda_ai.key,
                        mango_cache_ai.key,perp_market_ai.key, bids_ai.key, asks_ai.key, event_queue_ai.key, &open_orders_accs,
                        side, price, quantity, 0, OrderType::ImmediateOrCancel, true)?,
                    &[
                        mango_prog_ai.clone(),
                        mango_group_ai.clone(),
                        mango_account_ai.clone(),
                        fund_pda_ai.clone(),
                        perp_market_ai.clone(),
                        mango_cache_ai.clone(),
                        bids_ai.clone(),
                        asks_ai.clone(),
                        default_ai.clone(),
                        event_queue_ai.clone()
                    ],
                    &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
                )?;

            }

            let withdraw_mango = get_mango_withdraw(share, mango_deposits, fund_data.perp_market_index as usize,
                pnl, mango_account_ai, mango_group_ai, mango_cache_ai, perp_market_ai, mango_prog_ai)?;

            msg!("withdrawing from mango:{:?}", withdraw_mango);
            invoke_signed(
                &withdraw(mango_prog_ai.key, mango_group_ai.key, mango_account_ai.key, fund_pda_ai.key,
                    mango_cache_ai.key, root_bank_ai.key, node_bank_ai.key, vault_ai.key, inv_token_ai.key,
                    signer_ai.key, &open_orders_accs, withdraw_mango - 1, false)?,
                &[
                    mango_prog_ai.clone(),
                    mango_group_ai.clone(),
                    mango_account_ai.clone(),
                    fund_pda_ai.clone(),
                    mango_cache_ai.clone(),
                    root_bank_ai.clone(),
                    node_bank_ai.clone(),
                    vault_ai.clone(),
                    inv_token_ai.clone(),
                    signer_ai.clone(),
                    default_ai.clone(),
                    token_prog_ai.clone()
                ],
                &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
            )?;
        }
        fund_data.total_amount = fund_data.total_amount.checked_mul(U64F64!(1) - share).unwrap();
        fund_data.deposits = fund_data.deposits.checked_sub(investor_data.amount).unwrap();
        fund_data.vault_balance = parse_token_account(fund_vault_ai)?.amount;
        fund_data.no_of_investments -= 1;

        // close investor account on exit
        close_investor_account(investor_ai, investor_state_ai)?;
        Ok(())
    }


    pub fn investor_harvest_mngo (
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> Result<(), ProgramError> {
        const NUM_FIXED:usize = 18;
        let accounts = array_ref![accounts, 0, NUM_FIXED];

        let [
            fund_state_ai,
            investor_state_ai,
            investor_ai,
            mango_prog_ai,
            fund_mngo_vault_ai,
            inv_mngo_ai,

            mango_group_ai,
            mango_cache_ai,
            mango_account_ai,
            fund_pda_ai,
            perp_market_ai,
            mngo_perp_vault_ai,
            mngo_root_bank_ai,
            mngo_node_bank_ai,
            mngo_bank_vault_ai,
            signer_ai,
            token_prog_ai,
            default_ai
        ] = accounts;

        let mut fund_data = FundData::load_mut_checked(fund_state_ai, program_id)?;
        let mut investor_data = InvestorData::load_mut_checked(investor_state_ai, program_id)?;

        check!(investor_ai.is_signer, FundError::IncorrectSignature);
        check_eq!(investor_data.owner, *investor_ai.key);
        check_eq!(investor_data.manager, fund_data.manager_account);

        // check mngo vault
        check_eq!(fund_data.mngo_vault_key, *fund_mngo_vault_ai.key);

        // redeem all mango accrued to mango account
        invoke_signed(
            &redeem_mngo(mango_prog_ai.key, mango_group_ai.key,
                mango_cache_ai.key,
                mango_account_ai.key,
                fund_pda_ai.key,
                perp_market_ai.key,
                mngo_perp_vault_ai.key,
                mngo_root_bank_ai.key,
                mngo_node_bank_ai.key,
                mngo_bank_vault_ai.key,
                signer_ai.key,
            )?,
            &[
                mango_prog_ai.clone(),
                mango_group_ai.clone(),
                mango_cache_ai.clone(),
                mango_account_ai.clone(),
                fund_pda_ai.clone(),
                perp_market_ai.clone(),
                mngo_perp_vault_ai.clone(),
                mngo_root_bank_ai.clone(),
                mngo_node_bank_ai.clone(),
                mngo_bank_vault_ai.clone(),
                signer_ai.clone(),
                token_prog_ai.clone()
            ],
            &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
        )?;

        // get mngo to withdraw to mngo vault
        let mut mngo_delta = get_mngo_accrued(mango_account_ai, mango_group_ai, mango_cache_ai, mango_prog_ai, mngo_root_bank_ai)?;   
        let open_orders_accs = [Pubkey::default(); MAX_PAIRS];
        invoke_signed(
            &withdraw(mango_prog_ai.key, mango_group_ai.key, mango_account_ai.key, fund_pda_ai.key,
                mango_cache_ai.key, mngo_root_bank_ai.key, mngo_node_bank_ai.key, mngo_bank_vault_ai.key, fund_mngo_vault_ai.key,
                signer_ai.key, &open_orders_accs, mngo_delta, false)?,
            &[
                mango_prog_ai.clone(),
                mango_group_ai.clone(),
                mango_account_ai.clone(),
                fund_pda_ai.clone(),
                mango_cache_ai.clone(),
                mngo_root_bank_ai.clone(),
                mngo_node_bank_ai.clone(),
                mngo_bank_vault_ai.clone(),
                fund_mngo_vault_ai.clone(),
                signer_ai.clone(),
                default_ai.clone(),
                token_prog_ai.clone()
            ],
            &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
        )?;
        fund_data.mngo_accrued = parse_token_account(fund_mngo_vault_ai)?.amount;
        fund_data.total_mngo_accrued = fund_data.total_mngo_accrued.checked_add(mngo_delta).unwrap();

        // update manager share on every redeem
        let man_share = U64F64::to_num(U64F64::from_num(mngo_delta).checked_mul(fund_data.performance_fee_percentage / 100).unwrap());
        fund_data.mngo_manager = fund_data.mngo_manager.checked_add(man_share).unwrap();

        // rest gets distributed to investors
        mngo_delta = mngo_delta.checked_sub(man_share).unwrap();
        // update mngo per share values
        fund_data.mngo_per_share = fund_data.mngo_per_share.checked_add(
            U64F64::from_num(mngo_delta).checked_div(U64F64::from_num(fund_data.deposits)).unwrap()
        ).unwrap();

        // mngo due to investor
        let inv_mngo_share = fund_data.mngo_per_share.checked_sub(investor_data.mngo_debt).unwrap();
        let inv_mngo = U64F64::to_num(inv_mngo_share.checked_mul(U64F64::from_num(investor_data.amount)).unwrap());

        msg!("investor mngo:: {:?}", inv_mngo);
        invoke_signed(
            &(spl_token::instruction::transfer(
                token_prog_ai.key,
                fund_mngo_vault_ai.key,
                inv_mngo_ai.key,
                fund_pda_ai.key,
                &[fund_pda_ai.key],
                inv_mngo
            ))?,
            &[
                fund_mngo_vault_ai.clone(),
                inv_mngo_ai.clone(),
                fund_pda_ai.clone(),
                token_prog_ai.clone()
            ],
            &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
        )?;

        fund_data.mngo_accrued = parse_token_account(fund_mngo_vault_ai)?.amount;
        investor_data.mngo_debt = fund_data.mngo_per_share;

        Ok(())
    }

    pub fn manager_harvest_mngo (
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> Result<(), ProgramError> {
        const NUM_FIXED:usize = 17;
        let accounts = array_ref![accounts, 0, NUM_FIXED];

        let [
            fund_state_ai,
            manager_ai,
            mango_prog_ai,
            fund_mngo_vault_ai,
            man_mngo_ai,

            mango_group_ai,
            mango_cache_ai,
            mango_account_ai,
            fund_pda_ai,
            perp_market_ai,
            mngo_perp_vault_ai,
            mngo_root_bank_ai,
            mngo_node_bank_ai,
            mngo_bank_vault_ai,
            signer_ai,
            token_prog_ai,
            default_ai
        ] = accounts;

        let mut fund_data = FundData::load_mut_checked(fund_state_ai, program_id)?;

        check!(manager_ai.is_signer, ProgramError::MissingRequiredSignature);
        check_eq!(fund_data.manager_account, *manager_ai.key);

        // check mngo vault
        check_eq!(fund_data.mngo_vault_key, *fund_mngo_vault_ai.key);

        // redeem all mango accrued to mango account
        invoke_signed(
            &redeem_mngo(mango_prog_ai.key, mango_group_ai.key,
                mango_cache_ai.key,
                mango_account_ai.key,
                fund_pda_ai.key,
                perp_market_ai.key,
                mngo_perp_vault_ai.key,
                mngo_root_bank_ai.key,
                mngo_node_bank_ai.key,
                mngo_bank_vault_ai.key,
                signer_ai.key,
            )?,
            &[
                mango_prog_ai.clone(),
                mango_group_ai.clone(),
                mango_cache_ai.clone(),
                mango_account_ai.clone(),
                fund_pda_ai.clone(),
                perp_market_ai.clone(),
                mngo_perp_vault_ai.clone(),
                mngo_root_bank_ai.clone(),
                mngo_node_bank_ai.clone(),
                mngo_bank_vault_ai.clone(),
                signer_ai.clone(),
                token_prog_ai.clone()
            ],
            &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
        )?;

        // get mngo to withdraw to mngo vault
        let mut mngo_delta = get_mngo_accrued(mango_account_ai, mango_group_ai, mango_cache_ai, mango_prog_ai, mngo_root_bank_ai)?;   
        let open_orders_accs = [Pubkey::default(); MAX_PAIRS];
        invoke_signed(
            &withdraw(mango_prog_ai.key, mango_group_ai.key, mango_account_ai.key, fund_pda_ai.key,
                mango_cache_ai.key, mngo_root_bank_ai.key, mngo_node_bank_ai.key, mngo_bank_vault_ai.key, fund_mngo_vault_ai.key,
                signer_ai.key, &open_orders_accs, mngo_delta, false)?,
            &[
                mango_prog_ai.clone(),
                mango_group_ai.clone(),
                mango_account_ai.clone(),
                fund_pda_ai.clone(),
                mango_cache_ai.clone(),
                mngo_root_bank_ai.clone(),
                mngo_node_bank_ai.clone(),
                mngo_bank_vault_ai.clone(),
                fund_mngo_vault_ai.clone(),
                signer_ai.clone(),
                default_ai.clone(),
                token_prog_ai.clone()
            ],
            &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
        )?;
        fund_data.mngo_accrued = parse_token_account(fund_mngo_vault_ai)?.amount;
        fund_data.total_mngo_accrued = fund_data.total_mngo_accrued.checked_add(mngo_delta).unwrap();

        // update manager share on every redeem
        let man_share = U64F64::to_num(U64F64::from_num(mngo_delta).checked_mul(fund_data.performance_fee_percentage / 100).unwrap());
        fund_data.mngo_manager = fund_data.mngo_manager.checked_add(man_share).unwrap();

        // rest gets distributed to investors
        mngo_delta = mngo_delta.checked_sub(man_share).unwrap();
        if fund_data.deposits != 0 {
        // update mngo per share values
            fund_data.mngo_per_share = fund_data.mngo_per_share.checked_add(
                U64F64::from_num(mngo_delta).checked_div(U64F64::from_num(fund_data.deposits)).unwrap()
            ).unwrap();
        }
        msg!("manager mngo due:: {:?}", fund_data.mngo_manager);
        invoke_signed(
            &(spl_token::instruction::transfer(
                token_prog_ai.key,
                fund_mngo_vault_ai.key,
                man_mngo_ai.key,
                fund_pda_ai.key,
                &[fund_pda_ai.key],
                fund_data.mngo_manager
            ))?,
            &[
                fund_mngo_vault_ai.clone(),
                man_mngo_ai.clone(),
                fund_pda_ai.clone(),
                token_prog_ai.clone()
            ],
            &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
        )?;

        fund_data.mngo_accrued = parse_token_account(fund_mngo_vault_ai)?.amount;
        fund_data.mngo_manager = 0;
        Ok(())
    }

    // manager perf fee claim (non-mango)
    pub fn claim (
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> Result<(), ProgramError> {
        const NUM_FIXED:usize = 10;
        let accounts = array_ref![accounts, 0, NUM_FIXED];

        let [
            fund_state_ai,
            manager_ai,
            manager_btoken_ai,
            fund_vault_ai,
            mango_prog_ai,
            mango_group_ai,
            mango_cache_ai,
            mango_account_ai,
            fund_pda_ai,
            token_prog_ai
        ] = accounts;


        let mut fund_data = FundData::load_mut_checked(fund_state_ai, program_id)?;

        check!(manager_ai.is_signer, FundError::IncorrectSignature);
        check_eq!(fund_data.manager_account, *manager_ai.key);

        let _dep = update_amount_and_performance(&mut fund_data, mango_account_ai, mango_group_ai, mango_cache_ai, mango_prog_ai, true)?;

        msg!("Invoking transfer instructions");
        let performance_fee_manager: u64 = U64F64::to_num(fund_data.performance_fee);

        let transfer_instruction = spl_token::instruction::transfer(
            token_prog_ai.key,
            fund_vault_ai.key,
            manager_btoken_ai.key,
            fund_pda_ai.key,
            &[fund_pda_ai.key],
            performance_fee_manager
        )?;
        let transfer_accs = [
            fund_vault_ai.clone(),
            manager_btoken_ai.clone(),
            fund_pda_ai.clone(),
            token_prog_ai.clone()
        ];
        let signer_seeds = [fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)];
        invoke_signed(&transfer_instruction, &transfer_accs, &[&signer_seeds])?;

        msg!("Transfer Complete");

        fund_data.vault_balance = parse_token_account(&fund_vault_ai)?.amount;
        fund_data.total_amount = fund_data.total_amount.checked_sub(U64F64::from_num(performance_fee_manager)).unwrap();
        fund_data.performance_fee = U64F64!(0);
        
        Ok(())

    }

    pub fn redeem_mngo_accrued (
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> Result<(), ProgramError> {
        const NUM_FIXED:usize = 15;
        let accounts = array_ref![accounts, 0, NUM_FIXED];

        let [
            fund_state_ai,
            fund_mngo_vault_ai,
            mango_prog_ai,
            mango_group_ai,
            mango_cache_ai,
            mango_account_ai,
            fund_pda_ai,
            perp_market_ai,
            mngo_perp_vault_ai,
            mngo_root_bank_ai,
            mngo_node_bank_ai,
            mngo_bank_vault_ai,
            signer_ai,
            token_prog_ai,
            default_ai
        ] = accounts;

        let mut fund_data = FundData::load_mut_checked(fund_state_ai, program_id)?;

         // check mngo vault
         check_eq!(fund_data.mngo_vault_key, *fund_mngo_vault_ai.key);

         // redeem all mango accrued to mango account
         invoke_signed(
             &redeem_mngo(mango_prog_ai.key, mango_group_ai.key,
                 mango_cache_ai.key,
                 mango_account_ai.key,
                 fund_pda_ai.key,
                 perp_market_ai.key,
                 mngo_perp_vault_ai.key,
                 mngo_root_bank_ai.key,
                 mngo_node_bank_ai.key,
                 mngo_bank_vault_ai.key,
                 signer_ai.key,
             )?,
             &[
                 mango_prog_ai.clone(),
                 mango_group_ai.clone(),
                 mango_cache_ai.clone(),
                 mango_account_ai.clone(),
                 fund_pda_ai.clone(),
                 perp_market_ai.clone(),
                 mngo_perp_vault_ai.clone(),
                 mngo_root_bank_ai.clone(),
                 mngo_node_bank_ai.clone(),
                 mngo_bank_vault_ai.clone(),
                 signer_ai.clone(),
                 token_prog_ai.clone()
             ],
             &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
         )?;
 
         // get mngo to withdraw to mngo vault
         let mut mngo_delta = get_mngo_accrued(mango_account_ai, mango_group_ai, mango_cache_ai, mango_prog_ai, mngo_root_bank_ai)?;   
         let open_orders_accs = [Pubkey::default(); MAX_PAIRS];
         invoke_signed(
             &withdraw(mango_prog_ai.key, mango_group_ai.key, mango_account_ai.key, fund_pda_ai.key,
                 mango_cache_ai.key, mngo_root_bank_ai.key, mngo_node_bank_ai.key, mngo_bank_vault_ai.key, fund_mngo_vault_ai.key,
                 signer_ai.key, &open_orders_accs, mngo_delta, false)?,
             &[
                 mango_prog_ai.clone(),
                 mango_group_ai.clone(),
                 mango_account_ai.clone(),
                 fund_pda_ai.clone(),
                 mango_cache_ai.clone(),
                 mngo_root_bank_ai.clone(),
                 mngo_node_bank_ai.clone(),
                 mngo_bank_vault_ai.clone(),
                 fund_mngo_vault_ai.clone(),
                 signer_ai.clone(),
                 default_ai.clone(),
                 token_prog_ai.clone()
             ],
             &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
         )?;
         fund_data.mngo_accrued = parse_token_account(fund_mngo_vault_ai)?.amount;
         fund_data.total_mngo_accrued = fund_data.total_mngo_accrued.checked_add(mngo_delta).unwrap();
 
         // update manager share on every redeem
         let man_share = U64F64::to_num(U64F64::from_num(mngo_delta).checked_mul(fund_data.performance_fee_percentage / 100).unwrap());
         fund_data.mngo_manager = fund_data.mngo_manager.checked_add(man_share).unwrap();
 
         // rest gets distributed to investors
         mngo_delta = mngo_delta.checked_sub(man_share).unwrap();
         if fund_data.deposits != 0 {
            // update mngo per share values
            fund_data.mngo_per_share = fund_data.mngo_per_share.checked_add(
                U64F64::from_num(mngo_delta).checked_div(U64F64::from_num(fund_data.deposits)).unwrap()
            ).unwrap();
        }
        
        Ok(())
    }

    pub fn add_delegate (
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> Result<(), ProgramError> {
        const NUM_FIXED:usize = 3;
        let accounts = array_ref![accounts, 0, NUM_FIXED];

        let [
            fund_state_ai,
            manager_ai,
            delegate_ai
        ] = accounts;

        let mut fund_data = FundData::load_mut_checked(fund_state_ai, program_id)?;

        check!(manager_ai.is_signer, FundError::IncorrectSignature);
        check_eq!(fund_data.manager_account, *manager_ai.key);

        fund_data.delegate = *delegate_ai.key;
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
            FundInstruction::Initialize { min_amount, min_return, performance_fee_percentage, perp_market_index } => {
                msg!("FundInstruction::Initialize");
                return Self::initialize(program_id, accounts, min_amount, min_return, performance_fee_percentage, perp_market_index);
            }
            FundInstruction::InvestorDeposit { amount } => {
                msg!("FundInstruction::InvestorDeposit");
                return Self::deposit(program_id, accounts, amount);
            }
            FundInstruction::InvestorWithdraw => {
                msg!("FundInstruction::InvestorWithdraw");
                return Self::investor_withdraw(program_id, accounts);
            }
            FundInstruction::InvestorHarvestMngo => {
                msg!("FundInstruction::InvestorHarvestMngo");
                return Self::investor_harvest_mngo(program_id, accounts);
            }
            FundInstruction::ManagerHarvestMngo => {
                msg!("FundInstruction::ManagerHarvestMngo");
                return Self::manager_harvest_mngo(program_id, accounts);
            }
            FundInstruction::ClaimPerformanceFee => {
                msg!("FundInstruction::ClaimPerformanceFee");
                return Self::claim(program_id, accounts);
            }
            FundInstruction::MangoDeposit { quantity } => {
                msg!("FundInstruction::MangoDeposit");
                return mango_deposit(program_id, accounts, quantity);
            }
            FundInstruction::MangoWithdraw { quantity } => {
                msg!("FundInstruction::MangoWithdraw");
                return mango_withdraw(program_id, accounts, quantity);
            }
            FundInstruction::MangoPlacePerpOrder { 
                side,
                price,
                quantity,
                client_order_id,
                order_type, } => {
                msg!("FundInstruction::MangoPlaceOrder");
                return mango_place_perp_order(program_id,
                    accounts,
                    side,
                    price,
                    quantity,
                    client_order_id,
                    order_type);
            }
            FundInstruction::MangoCancelPerpById { client_order_id, invalid_id_ok: _ } => {
                msg!("FundInstruction::MangoCancelPerpById");
                return mango_cancel_perp_by_id(program_id, accounts, client_order_id, invalid_id_ok);
            }
            FundInstruction::RedeemMngo => {
                msg!("FundInstruction::RedeemMngo");
                return Self::redeem_mngo_accrued(program_id, accounts);
            }
            FundInstruction::AddDelegate => {
                msg!("FundInstruction::AddDelegate");
                return Self::add_delegate(program_id, accounts);
            }
        }
    }
}


// calculate prices, get fund valuation and performance
pub fn get_perp_withdraw(
    share_ratio: U64F64,
    market_index: usize,
    mango_account_ai: &AccountInfo,
    mango_group_ai: &AccountInfo,
    mango_cache_ai: &AccountInfo,
    perp_market_ai: &AccountInfo,
    bids_ai: &AccountInfo,
    asks_ai: &AccountInfo,
    mango_prog_ai: &AccountInfo
) -> Result<(Side, i64, i64), ProgramError> {

    let mango_group = MangoGroup::load_checked(mango_group_ai, mango_prog_ai.key)?;
    let mango_account = MangoAccount::load_checked(mango_account_ai, mango_prog_ai.key, mango_group_ai.key)?;
    let mut perp_market = PerpMarket::load_mut_checked(perp_market_ai, mango_prog_ai.key, mango_group_ai.key)?;
    let mut book = Book::load_checked(mango_prog_ai.key, bids_ai, asks_ai, &perp_market)?;

    let a = mango_account.perp_accounts[market_index].base_position;
    msg!("base pos:: {:?}", a);
    let mut b:i64 = I80F48::to_num(I80F48::from_fixed(share_ratio).checked_mul(I80F48::from_num(a)).unwrap());
 
    let mut side;
    let mut price;
    let mut quantity;

    if a > 0 {
        side = Side::Ask;
        quantity = b;
        price = I80F48::to_num(I80F48::from_num(book.get_best_bid_price().unwrap()).checked_mul(I80F48::from_num(0.95)).unwrap()) 
    }
    else {
        side = Side::Bid;
        quantity = -b;
        price = I80F48::to_num(I80F48::from_num(book.get_best_ask_price().unwrap()).checked_mul(I80F48::from_num(1.05)).unwrap())
    }

    if b == a {
        Ok((side, price, quantity))
    }
    else {
        Ok((side, price, quantity + 1))
    }
}

// calculate amount to withdraw from mango account
pub fn get_mango_withdraw(
    share_ratio: U64F64,
    native_deposits: I80F48,
    market_index: usize,
    before_pnl: I80F48,
    mango_account_ai: &AccountInfo,
    mango_group_ai: &AccountInfo,
    mango_cache_ai: &AccountInfo,
    perp_market_ai: &AccountInfo,
    mango_prog_ai: &AccountInfo
) -> Result<u64, ProgramError> {

    let mango_group = MangoGroup::load_checked(mango_group_ai, mango_prog_ai.key)?;
    let mango_account = MangoAccount::load_checked(mango_account_ai, mango_prog_ai.key, mango_group_ai.key)?;
    let mango_cache = MangoCache::load_checked(mango_cache_ai, mango_prog_ai.key, &mango_group)?;
    let root_bank_cache = &mango_cache.root_bank_cache[QUOTE_INDEX];

    let (base_val, quote_val) = mango_account.perp_accounts[market_index].get_val(&mango_group.perp_markets[market_index], &mango_cache.perp_market_cache[market_index],
        mango_cache.price_cache[market_index].price)?;
    let pnl = base_val.checked_add(quote_val).unwrap();
    msg!("pnl after:: {:?}", pnl);

    let pnl_diff = before_pnl.checked_sub(pnl).unwrap();
    let mut withdraw_share = I80F48::from_fixed(share_ratio).checked_mul(native_deposits).unwrap();

    // only if pnl goes down after
    if pnl_diff > 0 {
        withdraw_share = withdraw_share.checked_sub(pnl_diff).unwrap();
    }
    Ok(I80F48::to_num(withdraw_share))
}

// calculate prices, get fund valuation and performance
pub fn update_amount_and_performance(
    fund_data: &mut FundData,
    mango_account_ai: &AccountInfo,
    mango_group_ai: &AccountInfo,
    mango_cache_ai: &AccountInfo,
    mango_prog_ai: &AccountInfo,
    update_perf: bool
) -> Result<(I80F48, I80F48), ProgramError> {
    
    let market_index = fund_data.perp_market_index as usize;

    let mango_group = MangoGroup::load_checked(mango_group_ai, mango_prog_ai.key)?;
    let mango_account = MangoAccount::load_checked(mango_account_ai, mango_prog_ai.key, mango_group_ai.key)?;
    let mango_cache = MangoCache::load_checked(mango_cache_ai, mango_prog_ai.key, &mango_group)?;
    
    let root_bank_cache = &mango_cache.root_bank_cache[QUOTE_INDEX];

    let mut fund_val = I80F48::from_num(fund_data.vault_balance); // add balance in fund vault

    // account for native USDC deposits
    let mut native_deposits  = mango_account.get_native_deposit(root_bank_cache, QUOTE_INDEX)?;
    fund_val = fund_val.checked_add(native_deposits).unwrap();

    // Calculate pnl for perp account
    let (base_val, quote_val) = mango_account.perp_accounts[market_index].get_val(&mango_group.perp_markets[market_index],
        &mango_cache.perp_market_cache[market_index], mango_cache.price_cache[market_index].price)?;

    let pnl = base_val.checked_add(quote_val).unwrap();
    msg!("pnl before:: {:?}", base_val + quote_val);

    fund_val = fund_val.checked_add(pnl).unwrap();
   
    if update_perf {
        let mut perf = U64F64::from_num(fund_data.prev_performance);
        // only case where performance is not updated:
        // when no investments and no performance fee for manager
        if fund_data.no_of_investments != 0 || fund_data.performance_fee != 0 {
            perf = U64F64::from_fixed(fund_val).checked_div(fund_data.total_amount).unwrap()
            .checked_mul(U64F64::from_num(fund_data.prev_performance)).unwrap();
        }
        // adjust for manager performance fee
        fund_data.performance_fee = U64F64::to_num(U64F64::from_num(perf)
            .checked_div(U64F64::from_num(fund_data.prev_performance)).unwrap()
            .checked_mul(U64F64::from_num(fund_data.performance_fee)).unwrap());
        fund_data.prev_performance = perf;
    }
    
    fund_data.total_amount = U64F64::from_fixed(fund_val);
    
    msg!("updated amount: {:?}", fund_data.total_amount);
    msg!("updated perf {:?}", fund_data.prev_performance);
    
    Ok((native_deposits, pnl))
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