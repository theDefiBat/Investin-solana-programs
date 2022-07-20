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

use mm::{
    entrypoint::process_instruction,
    instruction:: {init_mm_fund},
};

#[tokio::test]
async fn test_mm_fund() {

    let program_id = Pubkey::new_unique();

    let mut test = ProgramTest::new(
        "mm",
        program_id,
        processor!(process_instruction),
    );


    // limit to track compute unit increase
    // test.set_bpf_compute_max_units(50_000);

    let system_program_account = Pubkey::from_str("11111111111111111111111111111111").unwrap();

    // setup admin account
    let admin = Keypair::new();
    test.add_account(admin.pubkey(), Account::new(u32::MAX as u64, 0, &system_program_account));

    let usdc_mint = add_usdc_mint(&mut test);

    let (fund_pda, _signer_nonce) = Pubkey::find_program_address(&[&admin.pubkey().to_bytes()], &program_id);

    // setup fund_pda token accounts
    let fund_usdc_vault = add_token_account(
        &mut test,
        fund_pda,
        usdc_mint.pubkey,
        0,
    );

    let mango_v3_program_id = Pubkey::from_str("mv3ekLzLbnVPNxjSKvqBpU3ZeZXPQdEC3bp5MDEBG68").unwrap();

    let mango_group_account = Pubkey::from_str("98pjRuQjK3qA6gXts96PqZT4Ze5QmnCmt3QYjhbUSPue").unwrap();

    let accountNum:u64 = 0;

    let mango_account_ai = Pubkey::find_program_address([
        mango_group_account.to_bytes(),
        fund_pda.to_bytes(),
        accountNum,
    ],
    &mango_v3_program_id);

    let delegate = Pubkey::from_str("HcikBBJaAUTZXyqHQYHv46NkvwXVigkk2CuQgGuNQEnX").unwrap();
    let system_program_account = Pubkey::from_str("11111111111111111111111111111111").unwrap();

    let min_amount = 100 * 1000000;
    let performance_fee_bps = 20 * 100;

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
        ],
        Some(&payer.pubkey()),
    );

    let recent_blockhash = banks_client.get_recent_blockhash().await.unwrap();
    transaction.sign(
        &[&payer, &admin],
        recent_blockhash,
    );
    assert_matches!(banks_client.process_transaction(transaction).await, Ok(()));
}