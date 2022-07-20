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
use solana_program::entrypoint::ProgramResult;

use mm::{
    entrypoint::process_instruction,
    instruction:: {init_mm_fund, investor_deposit, process_deposit},
};

#[tokio::test]
async fn test_mm_fund() {

    let program_id = Pubkey::new_unique();
    let mango_v3_program_id = Pubkey::new_unique();
    let system_program_account = Pubkey::from_str("11111111111111111111111111111111").unwrap();

    let mut test = ProgramTest::new(
        "Investin Market Making Fund",
        program_id,
        processor!(process_instruction),
    );

    test.add_program("mango", mango_v3_program_id, processor!(process_mango_instruction));


    // limit to track compute unit increase
    // test.set_bpf_compute_max_units(50_000);

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

    // let mango_v3_program_id = Pubkey::from_str("mv3ekLzLbnVPNxjSKvqBpU3ZeZXPQdEC3bp5MDEBG68").unwrap();

    let mango_group_account = Pubkey::from_str("98pjRuQjK3qA6gXts96PqZT4Ze5QmnCmt3QYjhbUSPue").unwrap();
    let mango_cache_account = Pubkey::from_str("8mFQbdXsFXt3R3cu3oSNS3bDZRwJRP18vyzd9J278J9z").unwrap();
    let root_bank_account = Pubkey::from_str("HUBX4iwWEUK5VrXXXcB7uhuKrfT4fpu2T9iZbg712JrN").unwrap();
    let node_bank_account = Pubkey::from_str("J2Lmnc1e4frMnBEJARPoHtfpcohLfN67HdK1inXjTFSM").unwrap();
    // let vault_ai = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey('8FRFC6MoGGkMFQwngccyu69VnYbzykGeez7ignHVAFSN'), mango_group_ai, transaction);
    let vault_account = add_token_account(
        &mut test,
        mango_group_account,
        usdc_mint.pubkey,
        0,
    );
    
    let accountNum = 0;

    let (mango_account_ai, mango_bump) = Pubkey::find_program_address([
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
    let deposit_amount = 50 * 1000000;

    // setup investor account
    let investor = Keypair::new();
    test.add_account(investor.pubkey(), Account::new(u32::MAX as u64, 0, &system_program_account));

    let investor_usdc_account = add_token_account(
        &mut test,
        investor.pubkey(),
        usdc_mint.pubkey,
        initial_amount,
    );

    let (mut banks_client, payer, recent_blockhash) = test.start().await;

    let rent = banks_client.get_rent().await.unwrap();
    
    // investor deposit
    {
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
    // todo: investor state account
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

    // Process deposit
    {
        let mut transaction = Transaction::new_with_payer(
            &[
                process_deposit(
                    &program_id,
                    &fund_pda,
                    &admin.pubkey(),
                    &mango_v3_program_id,
                    &mango_group_account,
                    &mango_account_ai,
                    &mango_cache_account,
                    &root_bank_account,
                    &node_bank_account,
                    &vault_account, //todo
                    &spl_token_program_account,
                    &fund_usdc_vault.pubkey,
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

        // Test expected amount is deducted from fund vault
        let final_fund_balance = get_token_balance(&mut banks_client, fund_usdc_vault.pubkey).await;
        assert_eq!(final_fund_balance, 0);

        let mango_account_balance = get_token_balance(&mut banks_client, mango_account_ai).await;
        assert_eq!(mango_account_balance, deposit_amount);
    }
}

fn process_mango_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    Ok(mango::processor::Processor::process(program_id, accounts, instruction_data)?)
}