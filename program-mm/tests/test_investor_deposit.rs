#![cfg(feature="test-bpf")]

mod helpers;
use assert_matches::*;

use helpers::*;
use solana_program_test::*;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Signer, Keypair},
    transaction::Transaction,
    account::Account,
};
use std::str::FromStr;
use std::str;
use solana_program::account_info::AccountInfo;
use solana_program::program::invoke;

use mm::{
    entrypoint::process_instruction,
    instruction:: {init_mm_fund, investor_deposit},
    state::InvestorData,
};

#[tokio::test]
async fn test_mm_fund() {

    let program_id = Pubkey::new_unique();
    let system_program_account = Pubkey::from_str("11111111111111111111111111111111").unwrap();

    let mut test = ProgramTest::new(
        "Investin Market Making Fund",
        program_id,
        processor!(process_instruction),
    );


    // limit to track compute unit increase
    // test.set_bpf_compute_max_units(50_000);

    // setup admin account
    let admin = Keypair::new();
    test.add_account(admin.pubkey(), Account::new(u32::MAX as u64, 0, &system_program_account));

    let usdc_mint = add_usdc_mint(&mut test);

    let (fund_pda, _signer_nonce) = Pubkey::find_program_address(&[&admin.pubkey().to_bytes()], &program_id);

    let rent = Rent::get()?;        
    investor_size = size_of::<InvestorData>();

    invoke(
        &solana_program::system_instruction::create_account(
            &admin.key,
            &investor_state_account.key,
            rent.minimum_balance(investor_size).max(1),
            investor_size as u64,
            &program_id,
        ),
        &[admin.clone(), investor_state_account.clone(), system_program_account.clone()],
    )?;

    // setup fund_pda token accounts
    let fund_usdc_vault = add_token_account(
        &mut test,
        fund_pda,
        usdc_mint.pubkey,
        0,
    );

    let mango_v3_program_id = Pubkey::from_str("mv3ekLzLbnVPNxjSKvqBpU3ZeZXPQdEC3bp5MDEBG68").unwrap();

    let mango_group_account = Pubkey::from_str("98pjRuQjK3qA6gXts96PqZT4Ze5QmnCmt3QYjhbUSPue").unwrap();

    let accountNum = 0;

    let mango_account_ai = Pubkey::find_program_address([
        mango_group_account.to_bytes(),
        fund_pda.to_bytes(),
        accountNum,
    ],
    &mango_v3_program_id);

    let delegate = Pubkey::from_str("HcikBBJaAUTZXyqHQYHv46NkvwXVigkk2CuQgGuNQEnX").unwrap();
    let system_program_account = Pubkey::from_str("11111111111111111111111111111111").unwrap();
    let spl_token_program_account = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap();

    let min_amount = 100 * 1000000;
    let performance_fee_bps = 20 * 100;

    let initial_amount = 100 * 1000000;

    // setup investor account
    let investor = Keypair::new();
    test.add_account(investor.pubkey(), Account::new(u32::MAX as u64, 0, &system_program_account));

    let investor_usdc_account = add_token_account(
        &mut test,
        investor.pubkey(),
        usdc_mint.pubkey,
        initial_amount,
    );

    let deposit_amount = 50 * 1000000;

    let (mut banks_client, payer, recent_blockhash) = test.start().await;

    let rent = banks_client.get_rent().await.unwrap();
    let mut transaction = Transaction::new_with_payer(
        &[
            init_mm_fund(
                &program_id,
                &admin.pubkey(),
                &fund_pda,
                &fund_usdc_vault,
                &mango_v3_program_id,
                &mango_group_account,
                &mango_account_ai,
                &delegate,
                &system_program_account,
                min_amount,
                performance_fee_bps,
            ).unwrap(),
            investor_deposit(
                &program_id,
                &fund_pda,
                &investor_state_account,
                &investor.pubkey(),
                &investor_usdc_account.pubkey,
                &fund_usdc_vault.pubkey,
                &spl_token_program_account,
                deposit_amount,
            ).unwrap(),
        ],
        Some(&payer.pubkey()),
    );

    let recent_blockhash = banks_client.get_recent_blockhash().await.unwrap();
    transaction.sign(
        &[&payer, &admin, &investor],
        recent_blockhash,
    );
    assert_matches!(banks_client.process_transaction(transaction).await, Ok(()));

     // Test expected amount is deducted from investor wallet
     let final_investor_balance = get_token_balance(&mut banks_client, investor_usdc_account.pubkey).await;
     assert_eq!(final_investor_balance, initial_amount - deposit_amount);

     let fund_vault_balance = get_token_balance(&mut banks_client, fund_usdc_vault.pubkey).await;
     assert_eq!(fund_vault_balance, deposit_amount);
}