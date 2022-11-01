// use bytemuck::bytes_of;
// use fixed::types::I80F48;
// use fixed::types::U64F64;
// use fixed_macro::types::I80F48;
// use mango::instruction::cancel_perp_order;
// use mango::instruction::cancel_perp_order_by_client_id;
// use solana_program::{
//     account_info::AccountInfo,
//     msg,
//     instruction:: {AccountMeta, Instruction},
//     program_error::ProgramError,
//     program_pack::Pack,
//     pubkey::Pubkey,
//     program::invoke_signed,
//     rent::Rent,
// };
// use num_enum::TryFromPrimitive;
// use std::convert::TryFrom;
// use std::convert::TryInto;
// use std::cell::RefMut;
// use std::i128;

// use fixed_macro::types::U64F64;
// pub const ONE_U64F64: U64F64 = U64F64!(1);
// pub const ZERO_U64F64: U64F64 = U64F64!(0);

// use std::num::NonZeroU64;


// use arrayref::{array_ref, array_refs};

// use crate::error::FundError;
// use crate::state::MAX_LIMIT_ORDERS;
// use crate::state::{MAX_INVESTORS_WITHDRAW, NUM_MARGIN, FundAccount, InvestorData};
// use crate::state::Loadable;
// use crate::processor::{ parse_token_account, close_investor_account };
// use serum_dex::state::MarketState;
// use serum_dex::instruction::{NewOrderInstructionV3, SelfTradeBehavior};
// use serum_dex::matching::{Side as SerumSide, OrderType as SerumOrderType};
// use mango::state::{MangoAccount, MangoGroup, MangoCache, MAX_PAIRS, QUOTE_INDEX};
// // use mango::state::Loadable as OtherLoadable;
// use mango::instruction::{deposit, withdraw, place_perp_order, place_perp_order2, init_spot_open_orders, settle_pnl, MangoInstruction};

// use mango::matching::{Side, OrderType};
// use spl_token::state::Account;

// macro_rules! check {
//     ($cond:expr, $err:expr) => {
//         if !($cond) {
//             return Err(($err).into())
//         }
//     }
// }

// macro_rules! check_eq {
//     ($x:expr, $y:expr) => {
//         if ($x != $y) {
//             return Err(FundError::Default.into())
//         }
//     }
// }

// // pub fn place_spot_order2(
// //     program_id: &Pubkey,
// //     mango_group_pk: &Pubkey,
// //     mango_account_pk: &Pubkey,
// //     owner_pk: &Pubkey,
// //     mango_cache_pk: &Pubkey,
// //     dex_prog_pk: &Pubkey,
// //     spot_market_pk: &Pubkey,
// //     bids_pk: &Pubkey,
// //     asks_pk: &Pubkey,
// //     dex_request_queue_pk: &Pubkey,
// //     dex_event_queue_pk: &Pubkey,
// //     dex_base_pk: &Pubkey,
// //     dex_quote_pk: &Pubkey,
// //     base_root_bank_pk: &Pubkey,
// //     base_node_bank_pk: &Pubkey,
// //     base_vault_pk: &Pubkey,
// //     quote_root_bank_pk: &Pubkey,
// //     quote_node_bank_pk: &Pubkey,
// //     quote_vault_pk: &Pubkey,
// //     signer_pk: &Pubkey,
// //     dex_signer_pk: &Pubkey,
// //     msrm_or_srm_vault_pk: &Pubkey,
// //     open_orders_pks: &[Pubkey],
// //     order: serum_dex::instruction::NewOrderInstructionV3,
// // ) -> Result<Instruction, ProgramError> {
// //     msg!("Calling Mango...");
// //     let mut accounts = vec![
// //         AccountMeta::new_readonly(*mango_group_pk, false),
// //         AccountMeta::new(*mango_account_pk, false),
// //         AccountMeta::new_readonly(*owner_pk, true),
// //         AccountMeta::new_readonly(*mango_cache_pk, false),
// //         AccountMeta::new_readonly(*dex_prog_pk, false),
// //         AccountMeta::new(*spot_market_pk, false),
// //         AccountMeta::new(*bids_pk, false),
// //         AccountMeta::new(*asks_pk, false),
// //         AccountMeta::new(*dex_request_queue_pk, false),
// //         AccountMeta::new(*dex_event_queue_pk, false),
// //         AccountMeta::new(*dex_base_pk, false),
// //         AccountMeta::new(*dex_quote_pk, false),
// //         AccountMeta::new_readonly(*base_root_bank_pk, false),
// //         AccountMeta::new(*base_node_bank_pk, false),
// //         AccountMeta::new(*base_vault_pk, false),
// //         AccountMeta::new_readonly(*quote_root_bank_pk, false),
// //         AccountMeta::new(*quote_node_bank_pk, false),
// //         AccountMeta::new(*quote_vault_pk, false),
// //         AccountMeta::new_readonly(spl_token::ID, false),
// //         AccountMeta::new_readonly(*signer_pk, false),
// //         AccountMeta::new_readonly(*dex_signer_pk, false),
// //         AccountMeta::new_readonly(*msrm_or_srm_vault_pk, false),
// //     ];

// //     accounts.extend(open_orders_pks.iter().map(
// //         |pk| 
// //         if *pk == Pubkey::default(){
// //             AccountMeta::new_readonly(*pk, false)
// //         } else {
// //             AccountMeta::new(*pk, false)
// //         })
// //     );

// //     let instr = MangoInstruction::PlaceSpotOrder2 { order };
// //     let data = instr.pack();

// //     Ok(Instruction { program_id: *program_id, accounts, data })
// // }

// pub fn init_mango_account(
//     program_id: &Pubkey,
//     mango_group_pk: &Pubkey,
//     mango_account_pk: &Pubkey,
//     owner_pk: &Pubkey,
// ) -> Result<Instruction, ProgramError> {
//     let accounts = vec![
//         AccountMeta::new_readonly(*mango_group_pk, false),
//         AccountMeta::new(*mango_account_pk, false),
//         AccountMeta::new_readonly(*owner_pk, true),
//     ];

//     let instr = MangoInstruction::InitMangoAccount;
//     let data = instr.pack();
//     Ok(Instruction { program_id: *program_id, accounts, data })
// }

// pub mod mango_v3_id {
//     use solana_program::declare_id;
//     // #[cfg(feature = "devnet")]
//     // declare_id!("4skJ85cdxQAFVKbcGgfun8iZPL7BadVYXG3kGEGkufqA");
//     // #[cfg(not(feature = "devnet"))]
//     declare_id!("mv3ekLzLbnVPNxjSKvqBpU3ZeZXPQdEC3bp5MDEBG68");
// }

// pub fn mango_init_mango_account(
//     program_id: &Pubkey,
//     accounts: &[AccountInfo],
// ) -> Result<(), ProgramError> {
//     const NUM_FIXED: usize = 5;
//     let accounts = array_ref![accounts, 0, NUM_FIXED];

//     let [
//         fund_account_ai,
//         manager_ai,
//         mango_prog_ai,
//         mango_group_ai,
//         mango_account_ai,
//     ] = accounts;

//     let mut fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;
//     //Check for Mango v3 ID 
//     check_eq!(*mango_prog_ai.key, mango_v3_id::ID);
//     check!(manager_ai.is_signer, ProgramError::MissingRequiredSignature);
//     check_eq!(*mango_account_ai.owner, mango_v3_id::ID);
//     check_eq!(fund_data.manager_account, *manager_ai.key);
//     check_eq!(fund_data.mango_positions.mango_account, Pubkey::default());
//     fund_data.mango_positions.mango_account = *mango_account_ai.key;
//     let nonce = fund_data.signer_nonce;
//     drop(fund_data);
//     invoke_signed(
//         &init_mango_account(mango_prog_ai.key, mango_group_ai.key, mango_account_ai.key, fund_account_ai.key)?,
//         &[
//             mango_prog_ai.clone(),
//             mango_group_ai.clone(),
//             mango_account_ai.clone(),
//             fund_account_ai.clone()
//         ],
//         &[&[&*manager_ai.key.as_ref(), bytes_of(&nonce)]]
//         )?;    
//     Ok(())
// }

// pub fn mango_deposit(
//     program_id: &Pubkey,
//     accounts: &[AccountInfo],
//     token_slot_index: u8,
//     mango_token_index: u8,
//     quantity: u64
// ) -> Result<(), ProgramError> {
//     const NUM_FIXED: usize = 11;
//     let accounts = array_ref![accounts, 0, NUM_FIXED];
//     let [
//         fund_account_ai,
//         manager_ai, // or delegate
//         mango_prog_ai,
//         mango_group_ai,         // read
//         mango_account_ai,       // write
//         mango_cache_ai,         // read
//         root_bank_ai,           // read
//         node_bank_ai,           // write
//         vault_ai,               // write
//         token_prog_ai,          // read
//         owner_token_account_ai, // write
//     ] = accounts;

//     let mut fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;
    
//     check_eq!(fund_data.tokens[token_slot_index as usize].vault, *owner_token_account_ai.key); 
//     check_eq!(*mango_prog_ai.key, mango_v3_id::ID);
//     check!(fund_data.mango_positions.mango_account != Pubkey::default(), FundError::MangoNotInitialized);
//     check_eq!(*mango_account_ai.key, fund_data.mango_positions.mango_account);
//     // check_eq!(mango_group.tokens[mango_token_index].root_bank, )   
//     check_eq!(mango_token_index as usize, QUOTE_INDEX);
//     // if(mango_token_index as usize != QUOTE_INDEX){
//     //     check!(fund_data.mango_positions.deposit_index == mango_token_index || 
//     //         fund_data.mango_positions.deposit_index == u8::MAX, FundError::InvalidMangoState);
//     //     fund_data.mango_positions.deposit_index = mango_token_index;
//     //     fund_data.tokens[token_slot_index as usize].is_on_mango = 1;
//     // }
//     check!(fund_data.is_initialized, ProgramError::InvalidAccountData);
//     check!(manager_ai.is_signer, ProgramError::MissingRequiredSignature);
    
//     // check_eq!(fund_data.manager_account, *manager_ai.key);
//     check!((fund_data.manager_account == *manager_ai.key), FundError::ManagerMismatch);

//     // // check fund vault
//     // check_eq!(fund_data.vault_key, *owner_token_account_ai.key); 
//     let nonce = fund_data.signer_nonce;
//     drop(fund_data);
//     invoke_signed(
//         &deposit(mango_prog_ai.key, mango_group_ai.key, mango_account_ai.key, fund_account_ai.key,
//             mango_cache_ai.key, root_bank_ai.key, node_bank_ai.key, vault_ai.key, owner_token_account_ai.key, quantity)?,
//         &[
//             mango_prog_ai.clone(),
//             mango_group_ai.clone(),
//             mango_account_ai.clone(),
//             fund_account_ai.clone(),
//             mango_cache_ai.clone(),
//             root_bank_ai.clone(),
//             node_bank_ai.clone(),
//             vault_ai.clone(),
//             owner_token_account_ai.clone(),
//             token_prog_ai.clone()
//         ],
//         &[&[&*manager_ai.key.as_ref(), bytes_of(&nonce)]]
//     )?;

//     msg!("invoke done");

//     let token_info = parse_token_account(owner_token_account_ai)?;
//     fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;
//     fund_data.tokens[token_slot_index as usize].balance = token_info.amount;
//     check!(fund_data.tokens[token_slot_index as usize].balance >= fund_data.tokens[token_slot_index as usize].debt, ProgramError::InsufficientFunds);

//     Ok(())
// }

// pub fn mango_place_perp_order(
//     program_id: &Pubkey,
//     accounts: &[AccountInfo],
//     perp_market_id: u8,
//     side: Side,
//     mut price: i64,
//     quantity: i64, 
//     mut reduce_only: bool
// ) -> Result<(), ProgramError> {
//     const NUM_FIXED: usize = 12;
//     let accounts = array_ref![accounts, 0, NUM_FIXED];

//     let [
//         fund_account_ai,
//         manager_ai,
//         mango_prog_ai,
//         mango_group_ai,     // read
//         mango_account_ai,   // write
//         mango_cache_ai,     // read
//         perp_market_ai,     // write
//         bids_ai,            // write
//         asks_ai,            // write
//         event_queue_ai,    // write
//         referrer_mango_account_ai,
//         default_ai,
//     ] = accounts;

//     let mut fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;
//     //Check for Mango v3 ID 
//     check_eq!(*mango_prog_ai.key, mango_v3_id::ID);
//     check!(manager_ai.is_signer, ProgramError::MissingRequiredSignature);

//     // check_eq!(fund_data.manager_account, *manager_ai.key);
//     check!((fund_data.manager_account == *manager_ai.key), FundError::ManagerMismatch);
//     let fund_perp_market_index = fund_data.get_mango_perp_index(perp_market_id);
//     if fund_perp_market_index == None {
//         let new_fund_perp_market_index = fund_data.get_mango_perp_index(u8::MAX).unwrap();
//         fund_data.mango_positions.perp_markets[new_fund_perp_market_index] = perp_market_id;
//     }
//     let open_orders_accs = [Pubkey::default(); MAX_PAIRS];
//     let nonce = fund_data.signer_nonce;
//     if perp_market_id == 13 {
//         reduce_only = true;
//         if price == 0 {
//             if side == Side::Bid{
//                 price = i64::MAX;
//             } else {
//                 price = 1;
//             }
//         }
//     }
//     drop(fund_data);
//     invoke_signed(
//         &place_perp_order(mango_prog_ai.key,
//             mango_group_ai.key, mango_account_ai.key, fund_account_ai.key,
//             mango_cache_ai.key,perp_market_ai.key, bids_ai.key, asks_ai.key, event_queue_ai.key, Some(referrer_mango_account_ai.key), &open_orders_accs,
//             side, price, quantity, 0, OrderType::ImmediateOrCancel, reduce_only)?,
//         &[
//             mango_prog_ai.clone(),
//             mango_group_ai.clone(),
//             mango_account_ai.clone(),
//             fund_account_ai.clone(),
//             mango_cache_ai.clone(),
//             perp_market_ai.clone(),
//             bids_ai.clone(),
//             asks_ai.clone(),
//             event_queue_ai.clone(),
//             referrer_mango_account_ai.clone(),
//             default_ai.clone(), 
//         ],
//         &[&[&*manager_ai.key.as_ref(), bytes_of(&nonce)]]
//     )?;

//     let mango_group_data = MangoGroup::load_checked(mango_group_ai, mango_prog_ai.key)?;
//     check_eq!(mango_group_data.perp_markets[perp_market_id as usize].perp_market, *perp_market_ai.key);
//     Ok(())

// }


// //currently only for Limit Orders
// pub fn mango_place_perp_order2(
//     program_id: &Pubkey,
//     accounts: &[AccountInfo],
//     perp_market_id: u8,
//     side: Side,
//     price: i64,
//     max_base_quantity: i64,
//     max_quote_quantity: i64,
//     client_order_id: u64,
//     _order_type: OrderType,
//     reduce_only: bool,
//     expiry_timestamp: u64,
//     limit: u8,
// ) -> Result<(), ProgramError> {

//     msg!("yooyo: perpId-{:?} side- {:?}, p-{},bq-{}, qq-{}, coi-{}, et{:?}, l-{} ", perp_market_id,side,price,max_base_quantity, max_quote_quantity, client_order_id, expiry_timestamp, limit );

//     check!(client_order_id > 0, FundError::InvalidInstruction);

//     // disAllow clientOrderid = 0

//     const NUM_FIXED: usize = 12;
//     let accounts = array_ref![accounts, 0, NUM_FIXED];

//     let [
//         fund_account_ai,
//         manager_ai,
//         mango_prog_ai,
//         mango_group_ai,     // read
//         mango_account_ai,   // write
//         mango_cache_ai,     // read
//         perp_market_ai,     // write
//         bids_ai,            // write
//         asks_ai,            // write
//         event_queue_ai,    // write
//         referrer_mango_account_ai,
//         default_ai,
//     ] = accounts;

//     let mut fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;
//     //Check for Mango v3 ID 
//     check_eq!(*mango_prog_ai.key, mango_v3_id::ID);
//     check!(manager_ai.is_signer, ProgramError::MissingRequiredSignature);

//     // check_eq!(fund_data.manager_account, *manager_ai.key);
//     check!((fund_data.manager_account == *manager_ai.key), FundError::ManagerMismatch);
//     let fund_perp_market_index = fund_data.get_mango_perp_index(perp_market_id);
//     if fund_perp_market_index == None {
//         let new_fund_perp_market_index = fund_data.get_mango_perp_index(u8::MAX).unwrap(); //getting empty Index
//         fund_data.mango_positions.perp_markets[new_fund_perp_market_index] = perp_market_id;
//     }
//     let open_orders_accs = [Pubkey::default(); MAX_PAIRS];
//     let nonce = fund_data.signer_nonce;

//     // first check all slots and clear if some already executed
//     for i in 0..MAX_LIMIT_ORDERS {
//         msg!("check sl-{:?}",i);
//         if  fund_data.limit_orders[i].client_order_id == 0 {
//             msg!("ignore 0");
//             continue;
//         }
//         // let limit_order_slot = fund_data.find_slot_by_client_id(client_order_id).unwrap();
//         //check if order has already execueted
//         let mango_account = MangoAccount::load_checked(mango_account_ai, mango_prog_ai.key, mango_group_ai.key)?;
//         let valid =  mango_account.find_order_with_client_id(fund_data.limit_orders[i].perp_market_id as usize,fund_data.limit_orders[i].client_order_id);
//             match valid {
//                 None => {
//                     msg!("order already executed");
//                     //order already executed
//                     fund_data.limit_orders[i].client_order_id = 0; 
//                 }
//                 _ => {
//                     // move on still listed
//                     msg!("still listed");
//                 }
//             }
//     }

//     //get free slot or Panic
//     let free_slot = fund_data.find_slot_by_client_id(0).unwrap();

//     // update structs 
//     fund_data.limit_orders[free_slot].price = price;
//     fund_data.limit_orders[free_slot].max_base_quantity = max_base_quantity;
//     fund_data.limit_orders[free_slot].max_quote_quantity = max_quote_quantity;
//     fund_data.limit_orders[free_slot].client_order_id = client_order_id;
//     fund_data.limit_orders[free_slot].perp_market_id = perp_market_id;
//     fund_data.limit_orders[free_slot].side = side;
//     fund_data.limit_orders[free_slot].expiry_timestamp = expiry_timestamp;
//     fund_data.limit_orders[free_slot].reduce_only = reduce_only;
//     fund_data.limit_orders[free_slot].limit = limit;

//     drop(fund_data);

//     invoke_signed(
//         &place_perp_order2(mango_prog_ai.key,
//             mango_group_ai.key, mango_account_ai.key, fund_account_ai.key,
//             mango_cache_ai.key,perp_market_ai.key, bids_ai.key, asks_ai.key, event_queue_ai.key, 
//             Some(referrer_mango_account_ai.key), &open_orders_accs,
//             side, price, max_base_quantity, max_quote_quantity, client_order_id, OrderType::Limit, reduce_only, Some(expiry_timestamp),limit)?,
//             &[
//                 mango_prog_ai.clone(),
//                 mango_group_ai.clone(),
//                 mango_account_ai.clone(),
//                 fund_account_ai.clone(),
//                 mango_cache_ai.clone(),
//                 perp_market_ai.clone(),
//                 bids_ai.clone(),
//                 asks_ai.clone(),
//                 event_queue_ai.clone(),
//                 referrer_mango_account_ai.clone(),
//                 default_ai.clone(), 
//             ],
//         &[&[&*manager_ai.key.as_ref(), bytes_of(&nonce)]]
//     )?;

    
//     let mango_group_data = MangoGroup::load_checked(mango_group_ai, mango_prog_ai.key)?;
//     check_eq!(mango_group_data.perp_markets[perp_market_id as usize].perp_market, *perp_market_ai.key);
//     Ok(())

// }

// pub fn mango_cancel_perp_order(
//     program_id: &Pubkey,
//     accounts: &[AccountInfo],
//     client_order_id: u64,
// ) -> Result<(), ProgramError> {
//     msg!("client_order_id: {:?}",client_order_id);
//     check!(client_order_id > 0, FundError::InvalidInstruction);

//     const NUM_FIXED: usize = 8;
//     let accounts = array_ref![accounts, 0, NUM_FIXED];

//     let [
//         fund_account_ai,
//         manager_ai,
//         mango_prog_ai,
//         mango_group_ai,     // read
//         mango_account_ai,   // write
//         perp_market_ai,     // write
//         bids_ai,            // write
//         asks_ai,            // write
//     ] = accounts;

//     let mut fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;
//     //Check for Mango v3 ID 
//     check_eq!(*mango_prog_ai.key, mango_v3_id::ID);
//     check!(manager_ai.is_signer, ProgramError::MissingRequiredSignature);
//     let nonce = fund_data.signer_nonce;

//     // clear data and clientOrderid = 0
//     let limit_order_slot = fund_data.find_slot_by_client_id(client_order_id).unwrap();
//     msg!("limit_order_slot {:?}",limit_order_slot);

//     //check if order has already execueted
//     let mango_account = MangoAccount::load_checked(mango_account_ai, mango_prog_ai.key, mango_group_ai.key)?;
//     let valid = mango_account.find_order_with_client_id(fund_data.limit_orders[limit_order_slot].perp_market_id as usize,client_order_id);
//     drop(mango_account);

//         match valid {
//             Some(_) => {
//                 fund_data.limit_orders[limit_order_slot].client_order_id = 0;
//                 fund_data.limit_orders[limit_order_slot].max_base_quantity = 0 as i64;
//                 fund_data.limit_orders[limit_order_slot].max_quote_quantity = 0 as i64;
//                 fund_data.limit_orders[limit_order_slot].perp_market_id = 0 as u8;
//                 fund_data.limit_orders[limit_order_slot].side = Side::Bid;
//                 fund_data.limit_orders[limit_order_slot].expiry_timestamp = 0 as u64;
//                 fund_data.limit_orders[limit_order_slot].reduce_only = false;

//                 drop(fund_data);
    
//                 invoke_signed(
//                     &cancel_perp_order_by_client_id(mango_prog_ai.key,
//                         mango_group_ai.key, mango_account_ai.key, fund_account_ai.key,
//                         perp_market_ai.key, bids_ai.key, asks_ai.key,client_order_id ,false)?,
//                     &[
//                         mango_prog_ai.clone(),
//                         mango_group_ai.clone(),
//                         mango_account_ai.clone(),
//                         fund_account_ai.clone(),
//                         perp_market_ai.clone(),
//                         bids_ai.clone(),
//                         asks_ai.clone(), 
//                     ],
//                     &[&[&*manager_ai.key.as_ref(), bytes_of(&nonce)]]
//                 )?;
//             }
//             None => {
//                 //order already executed
//                 fund_data.limit_orders[limit_order_slot].client_order_id = 0; 
//                 //clear data if needed
//                 fund_data.limit_orders[limit_order_slot].max_base_quantity = 0 as i64;
//                 fund_data.limit_orders[limit_order_slot].max_quote_quantity = 0 as i64;
//                 fund_data.limit_orders[limit_order_slot].perp_market_id = 0 as u8;
//                 fund_data.limit_orders[limit_order_slot].side = Side::Bid;
//                 fund_data.limit_orders[limit_order_slot].expiry_timestamp = 0 as u64;
//                 fund_data.limit_orders[limit_order_slot].reduce_only = false;
//                 msg!("order already executed");
//             }
//         }

    

   
//     Ok(())

// }

// // deprecated
// pub fn mango_cancel_perp_by_order_id(
//     program_id: &Pubkey,
//     accounts: &[AccountInfo],
//     order_id: i128,
// ) -> Result<(), ProgramError> {
//     msg!("order_id: {:?}",order_id);
//     const NUM_FIXED: usize = 8;
//     let accounts = array_ref![accounts, 0, NUM_FIXED];

//     let [
//         fund_account_ai,
//         manager_ai,
//         mango_prog_ai,
//         mango_group_ai,     // read
//         mango_account_ai,   // write
//         perp_market_ai,     // write
//         bids_ai,            // write
//         asks_ai,            // write
//     ] = accounts;

//     let mut fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;
//     //Check for Mango v3 ID 
//     check_eq!(*mango_prog_ai.key, mango_v3_id::ID);
//     check!(manager_ai.is_signer, ProgramError::MissingRequiredSignature);
//     let nonce = fund_data.signer_nonce;
//     drop(fund_data);
    
//     invoke_signed(
//         &cancel_perp_order(mango_prog_ai.key,
//             mango_group_ai.key, mango_account_ai.key, fund_account_ai.key,
//             perp_market_ai.key, bids_ai.key, asks_ai.key,order_id ,false)?,
//         &[
//             mango_prog_ai.clone(),
//             mango_group_ai.clone(),
//             mango_account_ai.clone(),
//             fund_account_ai.clone(),
//             perp_market_ai.clone(),
//             bids_ai.clone(),
//             asks_ai.clone(), 
//         ],
//         &[&[&*manager_ai.key.as_ref(), bytes_of(&nonce)]]
//     )?;

   
//     Ok(())

// }


// pub fn mango_remove_perp_index(
//     program_id: &Pubkey,
//     accounts: &[AccountInfo],
//     perp_market_id: u8
// ) -> Result<(), ProgramError> {
//     const NUM_FIXED: usize = 6;
//     let accounts = array_ref![accounts, 0, NUM_FIXED];

//     let [
//         fund_account_ai,
//         manager_ai,
//         mango_prog_ai,
//         mango_group_ai,     // read
//         mango_account_ai,   // write
//         mango_cache_ai,     // write
//     ] = accounts;

//     let mut fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;
//     //Check for Mango v3 ID 
//     check_eq!(*mango_prog_ai.key, mango_v3_id::ID);
//     check!(manager_ai.is_signer, ProgramError::MissingRequiredSignature);
//     check_eq!(*mango_account_ai.key, fund_data.mango_positions.mango_account); //Add this chreck elsewhere
//     // check_eq!(fund_data.manager_account, *manager_ai.key);
//     check!((fund_data.manager_account == *manager_ai.key), FundError::ManagerMismatch);

//     let mango_group = MangoGroup::load_checked(mango_group_ai, mango_prog_ai.key)?;
//     let mango_account = MangoAccount::load_checked(mango_account_ai, mango_prog_ai.key, mango_group_ai.key)?;
//     let mango_cache = MangoCache::load_checked(mango_cache_ai, mango_prog_ai.key, &mango_group)?;
    
//     let (base_val, quote_val) = mango_account.perp_accounts[perp_market_id as usize].get_val(&mango_group.perp_markets[perp_market_id as usize],
//         &mango_cache.perp_market_cache[perp_market_id as usize], mango_cache.price_cache[perp_market_id as usize].price)?;

//     let perp_pnl = base_val.checked_add(quote_val).unwrap();
    
//     let base_pos = mango_account.perp_accounts[perp_market_id as usize].base_position;

//     check!(perp_pnl == I80F48!(0) && base_pos == I80F48!(0), ProgramError::InsufficientFunds);

//     //Check No pending limit orders
//     for i in 0..MAX_LIMIT_ORDERS {
//         if fund_data.limit_orders[i].perp_market_id == perp_market_id {
//         let valid =  mango_account.find_order_with_client_id(fund_data.limit_orders[i].perp_market_id as usize,fund_data.limit_orders[i].client_order_id);
//         check!(valid == None, FundError::LimitOrderProcessing);

//         }
//     }
//     let fund_perp_makret_index = fund_data.get_mango_perp_index(perp_market_id).unwrap();
//     fund_data.mango_positions.perp_markets[fund_perp_makret_index as usize] = u8::MAX;
    
//     //Settle PnL to be executed right after place_perp_order...

//     Ok(())

// }

// //TODO::Update!!!
// // pub fn mango_settle_pnl(
// //     program_id: &Pubkey,
// //     accounts: &[AccountInfo],
// //     perp_market_id: u8
// // ) -> Result<(), ProgramError>
// // {
// //     const NUM_FIXED: usize = 9;
// //     let accounts = array_ref![accounts, 0, NUM_FIXED];

// //     let [
// //         fund_account_ai,
// //         manager_ai,
// //         mango_prog_ai,
// //         mango_group_ai,
// //         mango_account_a_ai,
// //         mango_account_b_ai,
// //         mango_cache_ai,
// //         root_bank_ai,
// //         node_bank_ai,
// //     ] = accounts;
// //     let mut fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;
// //     // check!(manager_ai.is_signer, ProgramError::MissingRequiredSignature);
// //     // check!((fund_data.manager_account == *manager_ai.key), FundError::ManagerMismatch);
// //     invoke_signed(
// //         &settle_pnl(mango_prog_ai.key, mango_group_ai.key, mango_account_a_ai.key, mango_account_a_ai.key, 
// //             mango_cache_ai.key, root_bank_ai.key, node_bank_ai.key, perp_market_id as usize)?,
// //         &[
// //             mango_prog_ai.clone(),
// //             mango_group_ai.clone(),
// //             mango_account_a_ai.clone(),
// //             mango_account_b_ai.clone(),
// //             mango_cache_ai.clone(),
// //             root_bank_ai.clone(),
// //             node_bank_ai.clone()
// //         ],
// //         &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
// //     )?;
// //     Ok(())
// // }


// pub fn mango_withdraw(
//     program_id: &Pubkey,
//     accounts: &[AccountInfo],
//     token_slot_index: u8,
//     mango_token_index: u8,
//     quantity: u64
// ) -> Result<(), ProgramError>
// {
//     const NUM_FIXED: usize = 13;
//     let accounts = array_ref![accounts, 0, NUM_FIXED];
//     let [
//         fund_account_ai,
//         manager_ai,
//         mango_prog_ai,
//         mango_group_ai,     // read
//         mango_account_ai,   // write
//         mango_cache_ai,     // read
//         root_bank_ai,       // read
//         node_bank_ai,       // write
//         vault_ai,           // write
//         fund_token_ai,   // write
//         signer_ai,          // read
//         token_prog_ai,      // read
//         default_ai
//     ] = accounts;

//     let mut fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;

//     check!(fund_data.is_initialized, ProgramError::InvalidAccountData);
//     check!(manager_ai.is_signer, ProgramError::MissingRequiredSignature);
//     check!(fund_data.mango_positions.mango_account != Pubkey::default(), FundError::MangoNotInitialized);
//     //Check for Mango v3 ID 
//     check_eq!(*mango_prog_ai.key, mango_v3_id::ID);

//     check_eq!(fund_data.tokens[token_slot_index as usize].vault, *fund_token_ai.key);

    
//     // check_eq!(mango_group.tokens[mango_token_index].root_bank, )
//     check!((fund_data.manager_account == *manager_ai.key), FundError::ManagerMismatch);
    
//     // withdraw USDC from mango account
//     let open_orders_accs = [Pubkey::default(); MAX_PAIRS];
//     let nonce = fund_data.signer_nonce;
//     drop(fund_data);
//     invoke_signed(
//         &withdraw(mango_prog_ai.key, mango_group_ai.key, mango_account_ai.key, fund_account_ai.key,
//             mango_cache_ai.key, root_bank_ai.key, node_bank_ai.key, vault_ai.key, fund_token_ai.key,
//             signer_ai.key, &open_orders_accs, quantity, false)?,
//         &[
//             mango_prog_ai.clone(),
//             mango_group_ai.clone(),
//             mango_account_ai.clone(),
//             fund_account_ai.clone(),
//             mango_cache_ai.clone(),
//             root_bank_ai.clone(),
//             node_bank_ai.clone(),
//             vault_ai.clone(),
//             fund_token_ai.clone(),
//             signer_ai.clone(),
//             default_ai.clone(),
//             token_prog_ai.clone()
//         ],
//         &[&[&*manager_ai.key.as_ref(), bytes_of(&nonce)]]
//     )?;

//     msg!("invoke done");
//     fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;
//     let dest_info = parse_token_account(fund_token_ai)?;
//     check_eq!(dest_info.owner, fund_data.fund_pda);
//     fund_data.tokens[token_slot_index as usize].balance = dest_info.amount;
//     // let mango_group = MangoGroup::load_checked(mango_group_ai, mango_prog_ai.key)?;
//     let mango_group = MangoGroup::load_checked(mango_group_ai, mango_prog_ai.key)?;
//     let mango_account = MangoAccount::load_checked(mango_account_ai, mango_prog_ai.key, mango_group_ai.key)?;
//     let mango_cache = MangoCache::load_checked(mango_cache_ai, mango_prog_ai.key, &mango_group)?;
//     // account for native USDC deposits
//     let usdc_deposits  = mango_account.get_native_deposit(&mango_cache.root_bank_cache[QUOTE_INDEX], QUOTE_INDEX)?;
    
//     //.checked_sub(mango_account.get_native_borrow(&mango_cache.root_bank_cache[QUOTE_INDEX], QUOTE_INDEX)?).unwrap();
//     check!(I80F48::from_num(fund_data.mango_positions.investor_debts[0]) <= usdc_deposits, ProgramError::InsufficientFunds);
//     // .checked_sub(I80F48::from_num(fund_data.mango_positions.investor_debts[0])).unwrap();
//     // if mango_token_index as usize != QUOTE_INDEX {
//     //     check!(deposits_after >= fund_data.mango_positions.investor_debts[1] , FundError::InvalidAmount);
//     //     if deposits_after < 0.00001 {
//     //         fund_data.mango_positions.deposit_index = u8::MAX;
//     //         fund_data.tokens[token_slot_index as usize].is_on_mango = 0;
//     //     }
//     // } else {
//         // check!(deposits_after >= fund_data.mango_positions.investor_debts[0] , FundError::InvalidAmount);
//     // }
    
//     // fund_data.tokens[0].balance = parse_token_account(fund_token_ai)?.amount;

//     Ok(())
// }

// // pub fn mango_place_spot_order2(
// //     program_id: &Pubkey,
// //     accounts: &[AccountInfo],
// //     side: u8,
// //     price: u64,
// //     trade_size: u64,
// //     call_init: bool
// // ) -> Result<(), ProgramError> {
// //     const NUM_FIXED: usize = 26;
// //     let fixed_ais = array_ref![accounts, 0, NUM_FIXED];
// //     let [
// //         fund_account_ai,
// //         manager_ai,
// //         mango_prog_ai,
// //         mango_group_ai,         // read
// //         mango_account_ai,       // write
// //         mango_cache_ai,         // read
// //         dex_prog_ai,            // read
// //         spot_market_ai,         // write
// //         bids_ai,                // write
// //         asks_ai,                // write
// //         dex_request_queue_ai,   // write
// //         dex_event_queue_ai,     // write
// //         dex_base_ai,            // write
// //         dex_quote_ai,           // write
// //         base_root_bank_ai,      // read
// //         base_node_bank_ai,      // write
// //         base_vault_ai,          // write
// //         quote_root_bank_ai,     // read
// //         quote_node_bank_ai,     // write
// //         quote_vault_ai,         // write
// //         token_prog_ai,          // read
// //         signer_ai,              // read
// //         dex_signer_ai,          // read
// //         msrm_or_srm_vault_ai,   // read
// //         packed_open_orders_ais,
// //         rent_ai
// //     ] = fixed_ais;

// //     let fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;
// //     check!(fund_data.is_initialized, ProgramError::InvalidAccountData);
// //     // check!(manager_ai.is_signer, ProgramError::MissingRequiredSignature);
// //     check!(fund_data.mango_positions.mango_account != Pubkey::default(), FundError::MangoNotInitialized);
// //     //Check for Mango v3 ID 
// //     check_eq!(*mango_prog_ai.key, mango_v3_id::ID);
// //     //TODO Check for qotue params to match USDC
// //     //drop(fund_data);
// //     if call_init {
// //         invoke_signed(
// //             &init_spot_open_orders(
// //                 mango_prog_ai.key,
// //                 mango_group_ai.key, 
// //                 mango_account_ai.key, 
// //                 fund_account_ai.key, 
// //                 dex_prog_ai.key,
// //                 packed_open_orders_ais.key, 
// //                 spot_market_ai.key, 
// //                 signer_ai.key
// //             )?,
// //             &[
// //                 mango_prog_ai.clone(),
// //                 mango_group_ai.clone(),
// //                 mango_account_ai.clone(),
// //                 fund_account_ai.clone(),
// //                 dex_prog_ai.clone(),
// //                 packed_open_orders_ais.clone(),
// //                 spot_market_ai.clone(),
// //                 signer_ai.clone(),
// //                 rent_ai.clone()
// //             ], 
// //             &[&[&*manager_ai.key.as_ref(), bytes_of(&fund_data.signer_nonce)]]
// //         )?;
// //     }
    
// //     let coin_lots = convert_size_to_lots(spot_market_ai, dex_prog_ai.key, trade_size, false)?;
// //     msg!("coin_lots:: {:?} ", coin_lots);

// //     let pc_qty = convert_size_to_lots(spot_market_ai, dex_prog_ai.key, trade_size * price, true)?;
// //     msg!("pc_qty:: {:?}", pc_qty);
// //     let fee_rate:U64F64 = U64F64!(0.0022); // fee_bps = 22; BASE

// //     let exact_fee: u64 = U64F64::to_num(fee_rate.checked_mul(U64F64::from_num(pc_qty)).unwrap());

// //     let pc_qty_including_fees = pc_qty + exact_fee;
// //     msg!("pc_qty:: {:?}", pc_qty_including_fees);

// //     let order_side = serum_dex::matching::Side::try_from_primitive(side.try_into().unwrap()).unwrap();
// //     // let mut open_orders_accs = [Pubkey::default(); MAX_PAIRS];
// //     // open_orders_accs[3] = *packed_open_orders_ais[3].key;
// //     let order: NewOrderInstructionV3 = NewOrderInstructionV3 {
// //         side: order_side,
// //         limit_price: NonZeroU64::new(price).unwrap(),
// //         max_coin_qty: NonZeroU64::new(coin_lots).unwrap(),
// //         max_native_pc_qty_including_fees: NonZeroU64::new(pc_qty_including_fees).unwrap(),
// //         self_trade_behavior: SelfTradeBehavior::AbortTransaction,
// //         order_type: SerumOrderType::ImmediateOrCancel,
// //         client_order_id: 1,
// //         limit: 65535,
// //     };

// //         invoke_signed(
// //             &place_spot_order2(
// //                 mango_prog_ai.key,
// //                 mango_group_ai.key, 
// //                 mango_account_ai.key, 
// //                 fund_account_ai.key, 
// //                 mango_cache_ai.key, 
// //                 dex_prog_ai.key, 
// //                 spot_market_ai.key, 
// //                 bids_ai.key, 
// //                 asks_ai.key, 
// //                 dex_request_queue_ai.key, 
// //                 dex_event_queue_ai.key, 
// //                 dex_base_ai.key, 
// //                 dex_quote_ai.key, 
// //                 base_root_bank_ai.key, 
// //                 base_node_bank_ai.key, 
// //                 base_vault_ai.key, 
// //                 quote_root_bank_ai.key, 
// //                 quote_node_bank_ai.key, 
// //                 quote_vault_ai.key, 
// //                 signer_ai.key, 
// //                 dex_signer_ai.key, 
// //                 msrm_or_srm_vault_ai.key, 
// //                 &[*packed_open_orders_ais.key],
// //                 order)?,
// //             &[
// //                 mango_prog_ai.clone(),
// //                 mango_group_ai.clone(), 
// //                 mango_account_ai.clone(), 
// //                 fund_account_ai.clone(), 
// //                 mango_cache_ai.clone(), 
// //                 dex_prog_ai.clone(), 
// //                 spot_market_ai.clone(), 
// //                 bids_ai.clone(), 
// //                 asks_ai.clone(), 
// //                 dex_request_queue_ai.clone(), 
// //                 dex_event_queue_ai.clone(), 
// //                 dex_base_ai.clone(), 
// //                 dex_quote_ai.clone(), 
// //                 base_root_bank_ai.clone(), 
// //                 base_node_bank_ai.clone(), 
// //                 base_vault_ai.clone(), 
// //                 quote_root_bank_ai.clone(), 
// //                 quote_node_bank_ai.clone(), 
// //                 quote_vault_ai.clone(), 
// //                 signer_ai.clone(), 
// //                 dex_signer_ai.clone(), 
// //                 msrm_or_srm_vault_ai.clone(),
// //                 packed_open_orders_ais.clone(),
// //             ],
// //             &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
// //         )?;

// //     Ok(())
// // }

// pub fn mango_withdraw_investor(
//     program_id: &Pubkey,
//     accounts: &[AccountInfo],
// ) -> Result<(), ProgramError> {
//     const NUM_FIXED: usize = 18;
//     let accounts = array_ref![accounts, 0, NUM_FIXED];
//     let [
//         fund_account_ai,      //write
//         investor_state_ai,  //write
//         investor_ai,        //signer
//         mango_prog_ai,      //
//         mango_group_ai,     // read
//         mango_account_ai,   // write
//         mango_cache_ai,     // read
//         usdc_root_bank_ai,       // read
//         usdc_node_bank_ai,       // write
//         usdc_vault_ai,           // write
//         usdc_investor_token_ai,   // write
//         token_root_bank_ai,       // read
//         token_node_bank_ai,       // write
//         token_vault_ai,           // write
//         token_investor_token_ai,   // write
//         signer_ai,          // read
//         token_prog_ai,      // read
//         default_ai
//     ] = accounts;
//     let mut fund_data = FundAccount::load_mut_checked(fund_account_ai, program_id)?;
//     let mut investor_data = InvestorData::load_mut_checked(investor_state_ai, program_id)?;

//     check!(investor_data.owner == *investor_ai.key, ProgramError::MissingRequiredSignature);
//     check!(investor_ai.is_signer, ProgramError::MissingRequiredSignature);
//     check_eq!(investor_data.manager, fund_data.manager_account);
//     check!(investor_data.has_withdrawn == true && investor_data.withdrawn_from_margin == false, FundError::InvalidStateAccount);
//     check_eq!(*mango_prog_ai.key, mango_v3_id::ID);
//     fund_data.mango_positions.investor_debts[0] = fund_data.mango_positions.investor_debts[0].checked_sub(U64F64::to_num(investor_data.margin_debt[0])).unwrap();
//     let manager_account = fund_data.manager_account;
//     let nonce = fund_data.signer_nonce;
//     let open_orders_accs = [Pubkey::default(); MAX_PAIRS];
//     let usdc_quantity:u64 =  U64F64::to_num(investor_data.margin_debt[0]);
//     msg!("usdc {:?}", usdc_quantity);
//     drop(fund_data);
//     if usdc_quantity > 0 {
//         invoke_signed(
//             &withdraw(mango_prog_ai.key, mango_group_ai.key, mango_account_ai.key, fund_account_ai.key,
//                 mango_cache_ai.key, usdc_root_bank_ai.key, usdc_node_bank_ai.key, usdc_vault_ai.key, usdc_investor_token_ai.key,
//                 signer_ai.key, &open_orders_accs, usdc_quantity, false)?,
//             &[
//                 mango_prog_ai.clone(),
//                 mango_group_ai.clone(),
//                 mango_account_ai.clone(),
//                 fund_account_ai.clone(),
//                 mango_cache_ai.clone(),
//                 usdc_root_bank_ai.clone(),
//                 usdc_node_bank_ai.clone(),
//                 usdc_vault_ai.clone(),
//                 usdc_investor_token_ai.clone(),
//                 signer_ai.clone(),
//                 default_ai.clone(),
//                 token_prog_ai.clone()
//             ],
//             &[&[bytes_of(&manager_account), bytes_of(&nonce)]]
//         )?;
//     }

//     // let token_quantity:u64 =  U64F64::to_num(investor_data.margin_debt[1]);
//     // if token_quantity > 0 {
//     //     invoke_signed(
//     //         &withdraw(mango_prog_ai.key, mango_group_ai.key, mango_account_ai.key, fund_account_ai.key,
//     //             mango_cache_ai.key, token_root_bank_ai.key, token_node_bank_ai.key, token_vault_ai.key, token_investor_token_ai.key,
//     //             signer_ai.key, &open_orders_accs, token_quantity, false)?,
//     //         &[
//     //             mango_prog_ai.clone(),
//     //             mango_group_ai.clone(),
//     //             mango_account_ai.clone(),
//     //             fund_account_ai.clone(),
//     //             mango_cache_ai.clone(),
//     //             token_root_bank_ai.clone(),
//     //             token_node_bank_ai.clone(),
//     //             token_vault_ai.clone(),
//     //             token_investor_token_ai.clone(),
//     //             signer_ai.clone(),
//     //             default_ai.clone(),
//     //             token_prog_ai.clone()
//     //         ],
//     //         &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
//     //     )?;
//     // }
    
//     msg!("invoke done");
//     // fund_data.mango_positions.investor_debts[1] = fund_data.mango_positions.investor_debts[1].checked_sub(U64F64::to_num(investor_data.margin_debt[1])).unwrap();
//     investor_data.margin_debt = [ZERO_U64F64; 2];
//     investor_data.withdrawn_from_margin = true;
//     Ok(())
// }

// // pub fn convert_size_to_lots(
// //     spot_market_acc: &AccountInfo,
// //     dex_program_id: &Pubkey,
// //     size: u64,
// //     pc: bool
// // ) -> Result <u64, ProgramError> {
// //     let market = MarketState::load(spot_market_acc, dex_program_id)?;
// //     if pc {
// //         Ok(size * market.pc_lot_size / market.coin_lot_size)
// //     }
// //     else {
// //         Ok(size / market.coin_lot_size)
// //     }
// // }

// // pub fn convert_size_to_lots(
// //     spot_market_acc: &AccountInfo,
// //     dex_program_id: &Pubkey,
// //     size: u64,
// //     pc: bool
// // ) -> Result <u64, ProgramError> {
// //     let market = MarketState::load(spot_market_ai, dex_program_id)?;
// //     if pc {
// //         Ok(size * market.pc_lot_size / market.coin_lot_size)
// //     }
// //     else {
// //         Ok(size / market.coin_lot_size)
// //     }
// // }

// // fn get_withdraw_lots(
// //     spot_market_acc: &AccountInfo,
// //     dex_program_id: &Pubkey,
// //     size: u64,
// //     side: u8,
// // ) -> Result <u64, ProgramError> {
// //     let market = MarketState::load(spot_market_ai, dex_program_id)?;
// //     Ok((size / market.coin_lot_size) + side as u64)
// // }

// // pub fn get_investor_withdraw_lots(
// //     spot_market_acc: &AccountInfo,
// //     dex_program_id: &Pubkey,
// //     size: u64,
// //     pos_size: u64,
// //     side: u8
// // ) -> Result <u64, ProgramError> {
// //     let market = MarketState::load(spot_market_ai, dex_program_id)?;
// //     //if size + market.coin_lot_size > pos_size {
// //     if (pos_size - size) / market.coin_lot_size == 0 {
// //         Ok((size / market.coin_lot_size) + side as u64) // same as manager close case
// //     }
// //     else {
// //         Ok((size / market.coin_lot_size) + 1)
// //     }
// //     // Ok((size / market.coin_lot_size) + side as u64)
// // }

// // pub fn update_investor_debts(
// //     fund_data: &FundAccount,
// //     investor_accs: &[AccountInfo],
// //     withdraw_amount: u64,
// //     index: usize
// // ) -> Result<(u64, U64F64), ProgramError> {
    
// //     let mut debts: u64 = 0;
// //     let mut debts_share = U64F64!(0);

// //     for i in 0..MAX_INVESTORS_WITHDRAW {
// //         if *investor_accs[i].key == Pubkey::default() {
// //             continue;
// //         }
// //         let mut investor_data = InvestorData::load_mut(&investor_accs[i])?;
// //         if investor_data.margin_position_id[index] == fund_data.mango_positions[index].position_id as u64 {
// //             // update
// //             let debt_valuation: u64 = U64F64::to_num(U64F64::from_num(withdraw_amount)
// //             .checked_mul(investor_data.margin_debt[index] / fund_data.mango_positions[index].share_ratio).unwrap());
// //             debts += debt_valuation;
// //             debts_share += investor_data.margin_debt[index] / fund_data.mango_positions[index].share_ratio;

// //             // update investor debts; add to USDC debt
// //             investor_data.margin_debt[index] = U64F64!(0);
// //             investor_data.token_debts[0] += debt_valuation;
// //             investor_data.has_withdrawn = true;
// //             investor_data.withdrawn_from_margin = false;
// //             investor_data.margin_position_id[index] = 0; // remove position id
// //         }
// //     }
// //     Ok((debts, debts_share))
// // }




