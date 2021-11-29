use bytemuck::bytes_of;
use fixed::types::I80F48;

use solana_program::{
    account_info::AccountInfo,
    msg,
    instruction:: {AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    program::invoke_signed,
};

use arrayref::array_ref;

use crate::error::FundError;
use crate::state::FundData;
use crate::processor::{ parse_token_account};

use mango::state::{MangoAccount, MangoGroup, MangoCache, MAX_PAIRS};
//use mango::state::Loadable as OtherLoadable;
use mango::instruction::{deposit, withdraw, place_perp_order, cancel_perp_order_by_client_id, MangoInstruction};
use mango::matching::{Side, OrderType};

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


pub fn mango_deposit (
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    quantity: u64
) -> Result<(), ProgramError>
{
    const NUM_FIXED: usize = 12;
    let accounts = array_ref![accounts, 0, NUM_FIXED];
    let [
        fund_state_acc,
        manager_acc, // or delegate
        mango_prog_ai,
        mango_group_ai,         // read
        mango_account_ai,       // write
        fund_pda_acc,               // read
        mango_cache_ai,         // read
        root_bank_ai,           // read
        node_bank_ai,           // write
        vault_ai,               // write
        token_prog_ai,          // read
        owner_token_account_ai, // write
    ] = accounts;

    let mut fund_data = FundData::load_mut_checked(fund_state_acc, program_id)?;

    check!(fund_data.is_initialized, ProgramError::InvalidAccountData);
    check!(manager_acc.is_signer, ProgramError::MissingRequiredSignature);
    
    // check_eq!(fund_data.manager_account, *manager_acc.key);
    check!((fund_data.manager_account == *manager_acc.key), FundError::ManagerMismatch);

    // check fund vault
    check_eq!(fund_data.vault_key, *owner_token_account_ai.key); 
    
    invoke_signed(
        &deposit(mango_prog_ai.key, mango_group_ai.key, mango_account_ai.key, fund_pda_acc.key,
            mango_cache_ai.key, root_bank_ai.key, node_bank_ai.key, vault_ai.key, owner_token_account_ai.key, quantity)?,
        &[
            mango_prog_ai.clone(),
            mango_group_ai.clone(),
            mango_account_ai.clone(),
            fund_pda_acc.clone(),
            mango_cache_ai.clone(),
            root_bank_ai.clone(),
            node_bank_ai.clone(),
            vault_ai.clone(),
            owner_token_account_ai.clone(),
            token_prog_ai.clone()
        ],
        &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
    )?;

    msg!("invoke done");

    fund_data.vault_balance = parse_token_account(owner_token_account_ai)?.amount;

    Ok(())
}

pub fn mango_place_perp_order (
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    perp_market_id: u8,
    side: Side,
    price: i64,
    quantity: i64,
    client_order_id: u64,
    order_type: OrderType,
) -> Result<(), ProgramError>

{
    const NUM_FIXED: usize = 12;
    let accounts = array_ref![accounts, 0, NUM_FIXED];

    let [
        fund_state_acc,
        manager_acc,
        mango_prog_ai,
        mango_group_ai,     // read
        mango_account_ai,   // write
        fund_pda_acc,           // read, signer
        mango_cache_ai,     // read
        perp_market_ai,     // write
        bids_ai,            // write
        asks_ai,            // write
        event_queue_ai,    // write
        default_acc,
    ] = accounts;

    let fund_data = FundData::load_mut_checked(fund_state_acc, program_id)?;

    check!(manager_acc.is_signer, ProgramError::MissingRequiredSignature);

    // check_eq!(fund_data.manager_account, *manager_acc.key);
    check!((fund_data.manager_account == *manager_acc.key), FundError::ManagerMismatch);
    
    let open_orders_accs = [Pubkey::default(); MAX_PAIRS];
    invoke_signed(
        &place_perp_order(mango_prog_ai.key,
            mango_group_ai.key, mango_account_ai.key, fund_pda_acc.key,
            mango_cache_ai.key,perp_market_ai.key, bids_ai.key, asks_ai.key, event_queue_ai.key, &open_orders_accs,
            side, price, quantity, client_order_id, order_type, true)?,
        &[
            mango_prog_ai.clone(),
            mango_group_ai.clone(),
            mango_account_ai.clone(),
            fund_pda_acc.clone(),
            perp_market_ai.clone(),
            mango_cache_ai.clone(),
            bids_ai.clone(),
            asks_ai.clone(),
            default_acc.clone(),
            event_queue_ai.clone()
        ],
        &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
    )?;
    Ok(())
}

pub fn mango_cancel_perp_by_id (
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    client_order_id: u64,
    invalid_id_ok: bool
) -> Result<(), ProgramError>

{
    const NUM_FIXED: usize = 9;
    let accounts = array_ref![accounts, 0, NUM_FIXED];

    let [
        fund_state_acc,
        manager_acc,
        mango_prog_ai,
        mango_group_ai,     // read
        mango_account_ai,   // write
        fund_pda_acc,           // read, signer
        perp_market_ai,     // write
        bids_ai,            // write
        asks_ai, 
    ] = accounts;


    let fund_data = FundData::load_mut_checked(fund_state_acc, program_id)?;

    check!(manager_acc.is_signer, ProgramError::MissingRequiredSignature);
    check!((fund_data.manager_account == *manager_acc.key), FundError::ManagerMismatch);

    invoke_signed(
        &cancel_perp_order_by_client_id(mango_prog_ai.key, mango_group_ai.key, mango_account_ai.key, fund_pda_acc.key,
            perp_market_ai.key, bids_ai.key, asks_ai.key, client_order_id, invalid_id_ok)?,
        &[
            mango_prog_ai.clone(),
            mango_group_ai.clone(),
            mango_account_ai.clone(),
            fund_pda_acc.clone(),
            perp_market_ai.clone(),
            bids_ai.clone(),
            asks_ai.clone(),
        ],
        &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
    )?;
    Ok(())
}

pub fn mango_withdraw (
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    quantity: u64
) -> Result<(), ProgramError>
{
    const NUM_FIXED: usize = 14;
    let accounts = array_ref![accounts, 0, NUM_FIXED];
    let [
        fund_state_ai,
        manager_ai,
        mango_prog_ai,

        mango_group_ai,     // read
        mango_account_ai,   // write
        fund_pda_ai,           // read
        mango_cache_ai,     // read
        root_bank_ai,       // read
        node_bank_ai,       // write
        vault_ai,           // write
        fund_vault_ai,   // write
        signer_ai,          // read
        token_prog_ai,      // read
        default_ai
    ] = accounts;

    let mut fund_data = FundData::load_mut_checked(fund_state_ai, program_id)?;

    check!(fund_data.is_initialized, ProgramError::InvalidAccountData);
    check!(manager_ai.is_signer, ProgramError::MissingRequiredSignature);

    // check_eq!(fund_data.manager_account, *manager_ai.key);
    check!((fund_data.manager_account == *manager_ai.key), FundError::ManagerMismatch);

    // check fund vault
    check_eq!(fund_data.vault_key, *fund_vault_ai.key); 
    
    // withdraw USDC from mango account
    let open_orders_accs = [Pubkey::default(); MAX_PAIRS];
    invoke_signed(
        &withdraw(mango_prog_ai.key, mango_group_ai.key, mango_account_ai.key, fund_pda_ai.key,
            mango_cache_ai.key, root_bank_ai.key, node_bank_ai.key, vault_ai.key, fund_vault_ai.key,
            signer_ai.key, &open_orders_accs, quantity, false)?,
        &[
            mango_prog_ai.clone(),
            mango_group_ai.clone(),
            mango_account_ai.clone(),
            fund_pda_ai.clone(),
            mango_cache_ai.clone(),
            root_bank_ai.clone(),
            node_bank_ai.clone(),
            vault_ai.clone(),
            fund_vault_ai.clone(),
            signer_ai.clone(),
            default_ai.clone(),
            token_prog_ai.clone()
        ],
        &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
    )?;

    msg!("invoke done");

    fund_data.vault_balance = parse_token_account(fund_vault_ai)?.amount;

    Ok(())
}

pub fn get_mngo_accrued(
    mango_account_ai: &AccountInfo,
    mango_group_ai: &AccountInfo,
    mango_cache_ai: &AccountInfo,
    mango_prog_ai: &AccountInfo,
    mngo_root_bank_ai: &AccountInfo
) -> Result<u64, ProgramError> {
    let mango_group = MangoGroup::load_checked(mango_group_ai, mango_prog_ai.key)?;
    let mango_account = MangoAccount::load_checked(mango_account_ai, mango_prog_ai.key, mango_group_ai.key)?;
    let mango_cache = MangoCache::load_checked(mango_cache_ai, mango_prog_ai.key, &mango_group)?;
    
    let mngo_index = mango_group.find_root_bank_index(mngo_root_bank_ai.key).unwrap();

    let mngo_deposits  = mango_account.get_native_deposit(&mango_cache.root_bank_cache[mngo_index], mngo_index)?;
    msg!("mngo accrued in account:: {:?}", mngo_deposits);
    Ok(I80F48::to_num(mngo_deposits))
}

pub fn init_mango_account(
    program_id: &Pubkey,
    mango_group_pk: &Pubkey,
    mango_account_pk: &Pubkey,
    owner_pk: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let accounts = vec![
        AccountMeta::new_readonly(*mango_group_pk, false),
        AccountMeta::new(*mango_account_pk, false),
        AccountMeta::new_readonly(*owner_pk, true),
    ];

    let instr = MangoInstruction::InitMangoAccount;
    let data = instr.pack();
    Ok(Instruction { program_id: *program_id, accounts, data })
}



pub fn redeem_mngo(
    program_id: &Pubkey,
    mango_group_pk: &Pubkey,
    mango_cache_pk: &Pubkey,
    mango_account_pk: &Pubkey,
    owner_pk: &Pubkey,
    perp_market_pk: &Pubkey,
    mngo_perp_vault_pk: &Pubkey,
    mngo_root_bank_pk: &Pubkey,
    mngo_node_bank_pk: &Pubkey,
    mngo_bank_vault_pk: &Pubkey,
    signer_pk: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let accounts = vec![
        AccountMeta::new_readonly(*mango_group_pk, false),
        AccountMeta::new_readonly(*mango_cache_pk, false),
        AccountMeta::new(*mango_account_pk, false),
        AccountMeta::new_readonly(*owner_pk, true),
        AccountMeta::new_readonly(*perp_market_pk, false),
        AccountMeta::new(*mngo_perp_vault_pk, false),
        AccountMeta::new_readonly(*mngo_root_bank_pk, false),
        AccountMeta::new(*mngo_node_bank_pk, false),
        AccountMeta::new(*mngo_bank_vault_pk, false),
        AccountMeta::new_readonly(*signer_pk, false),
        AccountMeta::new_readonly(spl_token::ID, false),
    ];

    let instr = MangoInstruction::RedeemMngo { };
    let data = instr.pack();
    Ok(Instruction { program_id: *program_id, accounts, data })
}