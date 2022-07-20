#![cfg(feature="test-bpf")]

use std::convert::TryInto;
use std::mem::size_of;
use std::str::FromStr;
use std::borrow::Borrow;
use assert_matches::*;

use std::cell::{Ref, RefMut};
use solana_program::pubkey::Pubkey;
use solana_program::account_info::AccountInfo;
use solana_program::program_error::ProgramError;
use bytemuck::{from_bytes, from_bytes_mut, Pod, Zeroable, bytes_of, Contiguous};

// use bytemuck::{bytes_of, Contiguous};
use fixed::types::U64F64;
use flux_aggregator::borsh_state::BorshState;
use flux_aggregator::borsh_utils;
use flux_aggregator::state::{Aggregator, AggregatorConfig, Answer};
use safe_transmute::{self, to_bytes::transmute_one_to_bytes};
use serum_dex::state::{AccountFlag, MarketState, ToAlignedBytes};
use solana_program::program_option::COption;
use solana_program::program_pack::Pack;
// use solana_program::pubkey::Pubkey;
use solana_program::pubkey::PubkeyError;
use solana_program::clock::{Clock, UnixTimestamp};
use solana_program::sysvar;
use bincode::deserialize;
use solana_program_test::{BanksClient, ProgramTest, ProgramTestContext, find_file, read_file};
use solana_sdk::{
    account::Account,
    account_info::IntoAccountInfo,
    instruction::Instruction,
    signature::{Keypair, Signer},
    transaction::Transaction,
    signer::keypair::read_keypair_file
};
use spl_token::state::{Account as Token, AccountState, Mint};
use switchboard_v2::AggregatorAccountData;

pub const NULL_PUBKEY: &str = "nu11111111111111111111111111111111111111111";

pub const SOL_PYTH_PRODUCT: &str = "3Mnn2fX6rQyUsyELYms1sBJyChWofzSNRoqYzvgMVz5E";
pub const SOL_PYTH_PRICE: &str = "J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix";
pub const SOL_SWITCHBOARD_FEED: &str = "AdtRGGhmqvom3Jemp5YNrxd9q9unX36BZk1pujkkXijL";
pub const SOL_SWITCHBOARDV2_FEED: &str = "GvDMxPzN1sCj7L26YDK2HnMRXEQmQ2aemov8YBtPS7vR";

pub const SRM_PYTH_PRODUCT: &str = "6MEwdxe4g1NeAF9u6KDG14anJpFsVEa2cvr5H6iriFZ8";
pub const SRM_PYTH_PRICE: &str = "992moaMQKs32GKZ9dxi8keyM2bUmbrwBZpK4p2K6X5Vs";
pub const SRM_SWITCHBOARD_FEED: &str = "BAoygKcKN7wk8yKzLD6sxzUQUqLvhBV1rjMA4UJqfZuH";
pub const SRM_SWITCHBOARDV2_FEED: &str = "CUgoqwiQ4wCt6Tthkrgx5saAEpLBjPCdHshVa4Pbfcx2";

pub const USDC_MINT: &str = "8FRFC6MoGGkMFQwngccyu69VnYbzykGeez7ignHVAFSN";

/// Mainnet program id for Switchboard v2.
pub mod switchboard_v2_mainnet {
    solana_program::declare_id!("SW1TCH7qEPTdLsDHRgPuMQjbQxKdH2aBStViMFnt64f");
}

/// Devnet program id for Switchboard v2.
pub mod switchboard_v2_devnet {
    solana_program::declare_id!("2TfB33aLaneQb5TNVwyDz3jSZXS6jdW2ARw1Dgf84XCG");
}


pub fn gen_signer_seeds<'a>(nonce: &'a u64, acc_pk: &'a Pubkey) -> [&'a [u8]; 2] {
    [acc_pk.as_ref(), bytes_of(nonce)]
}

trait AddPacked {
    fn add_packable_account<T: Pack>(
        &mut self,
        pubkey: Pubkey,
        amount: u64,
        data: &T,
        owner: &Pubkey,
    );
}

impl AddPacked for ProgramTest {
    fn add_packable_account<T: Pack>(
        &mut self,
        pubkey: Pubkey,
        amount: u64,
        data: &T,
        owner: &Pubkey,
    ) {
        let mut account = Account::new(amount, T::get_packed_len(), owner);
        data.pack_into_slice(&mut account.data);
        self.add_account(pubkey, account);
    }
}

fn gen_signer_key(
    nonce: u64,
    acc_pk: &Pubkey,
    program_id: &Pubkey,
) -> Result<Pubkey, PubkeyError> {
    let seeds = gen_signer_seeds(&nonce, acc_pk);
    Pubkey::create_program_address(&seeds, program_id)
}

pub fn create_signer_key_and_nonce(program_id: &Pubkey, acc_pk: &Pubkey) -> (Pubkey, u64) {

    for i in 0..=u64::MAX_VALUE {
        if let Ok(pk) = gen_signer_key(i, acc_pk, program_id) {
            return (pk, i);
        }
    }
    panic!("Could not generate signer key");

}

pub struct TestTokenAccount {
    pub pubkey: Pubkey,
}

pub fn add_token_account(test: &mut ProgramTest, owner: Pubkey, mint: Pubkey, initial_balance: u64) -> TestTokenAccount {
    let pubkey = Pubkey::new_unique();
    test.add_packable_account(
        pubkey,
        u32::MAX as u64,
        &Token {
            mint: mint,
            owner: owner,
            amount: initial_balance,
            state: AccountState::Initialized,
            ..Token::default()
        },
        &spl_token::id(),
    );
    TestTokenAccount { pubkey }
}

#[allow(dead_code)]  // Compiler complains about this even tho it is used
pub async fn get_token_balance(banks_client: &mut BanksClient, pubkey: Pubkey) -> u64 {
    let token: Account = banks_client.get_account(pubkey).await.unwrap().unwrap();

    spl_token::state::Account::unpack(&token.data[..])
        .unwrap()
        .amount
}

pub struct TestMint {
    pub pubkey: Pubkey,
    pub authority: Keypair,
    pub decimals: u8,
}


pub fn add_mint(test: &mut ProgramTest, decimals: u8) -> TestMint {
    let authority = Keypair::new();
    let pubkey = Pubkey::new_unique();
    test.add_packable_account(
        pubkey,
        u32::MAX as u64,
        &Mint {
            is_initialized: true,
            mint_authority: COption::Some(authority.pubkey()),
            decimals,
            ..Mint::default()
        },
        &spl_token::id(),
    );
    TestMint {
        pubkey,
        authority,
        decimals,
    }
}

pub fn add_usdc_mint(test: &mut ProgramTest) -> TestMint {
    let authority = Keypair::new();
    let pubkey = Pubkey::from_str(USDC_MINT).unwrap();
    let decimals = 6;
    test.add_packable_account(
        pubkey,
        u32::MAX as u64,
        &Mint {
            is_initialized: true,
            mint_authority: COption::Some(authority.pubkey()),
            decimals,
            ..Mint::default()
        },
        &spl_token::id(),
    );
    TestMint {
        pubkey,
        authority,
        decimals,
    }
}

pub struct TestOracle {
    pub pyth_product_pubkey: Pubkey,
    pub pyth_price_pubkey: Pubkey,
    pub switchboard_feed_pubkey: Pubkey,
    pub price: Decimal,
}

pub fn add_sol_oracle(test: &mut ProgramTest) -> TestOracle {
    add_oracle(
        test,
        Pubkey::from_str(SOL_PYTH_PRODUCT).unwrap(),
        Pubkey::from_str(SOL_PYTH_PRICE).unwrap(),
        Pubkey::from_str(SOL_SWITCHBOARD_FEED).unwrap(),
        // Set SOL price to $20
        Decimal::from_integer(20u64),
    )
}

pub fn add_sol_oracle_switchboardv2(test: &mut ProgramTest) -> TestOracle {
    add_oracle(
        test,
        Pubkey::from_str(NULL_PUBKEY).unwrap(),
        Pubkey::from_str(NULL_PUBKEY).unwrap(),
        Pubkey::from_str(SOL_SWITCHBOARDV2_FEED).unwrap(),
        // Set SOL price to $20
        Decimal::from_integer(20u64),
    )
}

pub fn add_usdc_oracle(test: &mut ProgramTest) -> TestOracle {
    add_oracle(
        test,
        // Mock with SRM since Pyth doesn't have USDC yet
        Pubkey::from_str(SRM_PYTH_PRODUCT).unwrap(),
        Pubkey::from_str(SRM_PYTH_PRICE).unwrap(),
        Pubkey::from_str(SRM_SWITCHBOARD_FEED).unwrap(),
        // Set USDC price to $1
        Decimal::from_integer(1u64),
    )
}

pub fn add_usdc_oracle_switchboardv2(test: &mut ProgramTest) -> TestOracle {
    add_oracle(
        test,
        // Mock with SRM since Pyth doesn't have USDC yet
        Pubkey::from_str(NULL_PUBKEY).unwrap(),
        Pubkey::from_str(NULL_PUBKEY).unwrap(),
        Pubkey::from_str(SRM_SWITCHBOARDV2_FEED).unwrap(),
        // Set USDC price to $1
        Decimal::from_integer(1u64),
    )
}

pub fn add_oracle(
    test: &mut ProgramTest,
    pyth_product_pubkey: Pubkey,
    pyth_price_pubkey: Pubkey,
    switchboard_feed_pubkey: Pubkey,
    price: Decimal,
) -> TestOracle {
    let oracle_program_id = read_keypair_file("tests/fixtures/oracle_program_id.json").unwrap();

    if pyth_price_pubkey.to_string() != NULL_PUBKEY {
        // Add Pyth product account
        test.add_account_with_file_data(
            pyth_product_pubkey,
            u32::MAX as u64,
            oracle_program_id.pubkey(),
            &format!("{}.bin", pyth_product_pubkey.to_string()),
        );
    }
    if pyth_price_pubkey.to_string() != NULL_PUBKEY {
        // Add Pyth price account after setting the price
        let filename = &format!("{}.bin", pyth_price_pubkey.to_string());
        let mut pyth_price_data = read_file(find_file(filename).unwrap_or_else(|| {
            panic!("Unable to locate {}", filename);
        }));

        let mut pyth_price = pyth::load_mut::<pyth::Price>(pyth_price_data.as_mut_slice()).unwrap();

        let decimals = 10u64
            .checked_pow(pyth_price.expo.checked_abs().unwrap().try_into().unwrap())
            .unwrap();

        pyth_price.valid_slot = 0;
        pyth_price.agg.price = price
            .to_u64()
            // .unwrap()
            .checked_mul(decimals)
            .unwrap()
            .try_into()
            .unwrap();

        test.add_account(
            pyth_price_pubkey,
            Account {
                lamports: u32::MAX as u64,
                data: pyth_price_data,
                owner: oracle_program_id.pubkey(),
                executable: false,
                rent_epoch: 0,
            },
        );
    }

    // Add Switchboard price feed account after setting the price
    let filename2 = &format!("{}.bin", switchboard_feed_pubkey.to_string());
    // mut and set data here later
    let mut switchboard_feed_data = read_file(find_file(filename2).unwrap_or_else(|| {
        panic!("Unable tod locate {}", filename2);
    }));

    let is_v2 = switchboard_feed_pubkey.to_string() == SOL_SWITCHBOARDV2_FEED;
        // || switchboard_feed_pubkey.to_string() == SRM_SWITCHBOARDV2_FEED;
    if is_v2 {
        // let mut_switchboard_feed_data = &mut switchboard_feed_data[8..];
        let agg_state =
            bytemuck::from_bytes_mut::<AggregatorAccountData>(&mut switchboard_feed_data[8..]);
        agg_state.latest_confirmed_round.round_open_slot = 0;
        test.add_account(
            switchboard_feed_pubkey,
            Account {
                lamports: u32::MAX as u64,
                data: switchboard_feed_data,
                owner: switchboard_v2_devnet::id(),
                executable: false,
                rent_epoch: 0,
            },
        );
    } else {
        test.add_account(
            switchboard_feed_pubkey,
            Account {
                lamports: u32::MAX as u64,
                data: switchboard_feed_data,
                owner: oracle_program_id.pubkey(),
                executable: false,
                rent_epoch: 0,
            },
        );
    }

    TestOracle {
        pyth_product_pubkey,
        pyth_price_pubkey,
        switchboard_feed_pubkey,
        price,
    }
}