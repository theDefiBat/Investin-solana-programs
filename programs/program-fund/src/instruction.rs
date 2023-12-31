use arrayref::{array_ref, array_refs};
use borsh::BorshSerialize;
use mango::matching::{Side, OrderType};
use num_enum::TryFromPrimitive;

#[repr(C)]
#[derive(Clone)]
pub enum FundInstruction {

    /// Accounts expected
    /// 0. [WRITE]  Platform State Account
    /// 1. [WRITE]  Fund State Account (derived from FA)
    /// 2. [SIGNER] Manager Wallet Account
    /// 3. []       Fund Base Token Account
    /// 4..4+NUM_TOKENS [] Token mints to be whitelisted

    Initialize {
        min_amount: u64,
        performance_fee_percentage: u64,
        no_of_tokens: u8,
        is_private: bool
    },

    /// 0. [WRITE]  Fund State Account (derived from FA)
    /// 1. [WRITE]  Investor State Account (derived from IPDA)
    /// 2. [SIGNER] Investor Wallet Account
    /// 3. []       Investor Base Token Account
    /// 4. []       Router Base Token Account (derived)
    /// 5. []       PDA of Manager (Fund Address)
    /// 6. []       Token Program
    InvestorDeposit {
        amount: u64,
        index: u8 // index of the investor slot
    },

    /// 0. []       Platform State Account
    /// 1. [WRITE]  Fund State Account
    /// 2. [READ]   Price Account
    ///             Open orders account
    ///             mango oracle account
    /// 3. [READ]   Clock Sysvar Account
    /// 4. [SIGNER] Manager Wallet Account
    /// 5. []       Router Base Token Account
    /// 6. []       Fund Base Token Account
    /// 7. []       Manager Base Token Account
    /// 8. []       Investin Base Token Account
    /// 9. []       PDA of Router
    /// 10. []       Token Program
    /// 11..11+MAX_INVESTORS Investor State Accounts for the fund
    ManagerTransfer,
    
    /// 0. [WRITE]  Platform State Account
    /// 1. [WRITE]  Fund State Account (derived from FA)
    /// 2. [WRITE]  Investor State Account (derived from IPDA)
    /// 3. [SIGNER] Investor Wallet Account
    /// 4. []       Router Base Token Account
    /// 5. []       PDA of Manager
    /// 6. []       PDA of Router
    /// 7. []       Token Program
    /// 8..8+NUM_TOKENS []  Investor Token Accounts
    /// 8+NUM_TOKENS.. 8+2*NUM_TOKENS  Fund Token Accounts
    InvestorWithdrawFromFund,

            // platform_ai,
            // fund_account_ai, [write]
            // investor_state_ai, [write]
            // investor_ai,
            // mango_account_ai,
            // mango_group_ai,
            // mango_cache_ai,
            // mango_prog_ai,
            // referrer_mango_account_ai,
            // default_ai,
            // ..4 X [perp_market_ai,     // write default_ai if no perp market for i^th index
            //        bids_ai,            // write default_ai if no perp market for i^th index
            //        asks_ai,            // write default_ai if no perp market for i^th index
            //        event_queue_ai,]   //write default_ai if no perp market for i^th index
            // 
    InvestorWithdrawSettleFunds,

    /// 0. [WRITE] Fund State Account
    /// 1. [] Raydium Pool Program
    /// 2. [] Token Program
    /// 3..17 [] Raydium/Serum Accounts
    /// 18. [] Source Token Account
    /// 19. [] Destination Token Account
    /// 20. [] PDA of Manager
    Swap {
        swap_index: u8,
        data: Data
    },

    /// 0. [WRITE] Fund State Account (derived from FA)
    /// 1. [READ]   Price Account
    /// 4. [READ]   Mango Group Account (for valuation)
    /// 5. [READ]   Margin Account (for valuation)
    ///             Open Orders account
    ///             Mango oracle account
    /// 2. [READ]   Clock Sysvar Account
    /// 3. [SIGNER] Manager Wallet Account
    /// 4. []       Fund Base Token Account
    /// 5. []       Manager Base Token Account
    /// 6. []       Investin Base Token Account
    /// 7. []       PDA of Manager
    /// 8. []       Token Program
    ClaimPerformanceFee,

    /// 0. [WRITE] Platform State Account
    /// 1. [SIGNER] investin Wallet Account 
    /// 2. []       Fund state Account / 2. []     Base Token Mint Address
    AdminControl{
        intialize_platform: u8,
        freeze_platform: u8,
        unfreeze_platform: u8,
        change_vault: u8,
        freeze_fund: u8,
        unfreeze_fund: u8,
        change_min_amount: u64,
        change_perf_fee: u64
    },

    /// Initialize a mango account for a user
    ///
    /// Accounts expected by this instruction (4):
    ///
    /// 0. `[]` mango_group_ai - MangoGroup that this mango account is for
    /// 1. `[writable]` mango_account_ai - the mango account data
    /// 2. `[signer]` owner_ai - Solana account of owner of the mango account
    /// 3. `[]` rent_ai - Rent sysvar account
    // MangoInitialize,

    /// Proxy to Deposit instruction on Mango
    /// 
    /// fund_account_ai,
    /// manager_ai,  
    /// mango_prog_ai,
    /// mango_group_ai,         // read
    /// mango_account_ai,       // write
    /// mango_cache_ai,         // read
    /// root_bank_ai,           // read
    /// node_bank_ai,           // write
    /// vault_ai,               // write
    /// token_prog_ai,          // read
    /// owner_token_account_ai, // write
    // MangoDeposit {
    //     token_slot_index: u8,
    //     mango_token_index: u8,
    //     quantity: u64
    // },

    /// Place an order on the Serum Dex and settle funds from the open orders account
    ///
    /// Accounts expected by this instruction (19 + 2 * NUM_MARKETS):
    /// ProgramId
    /// fund_state_acc,
    /// manager_acc,
    /// mango_prog_ai,
    /// mango_group_ai,     // read
    /// mango_account_ai,   // write
    /// mango_cache_ai,     // read
    /// perp_market_ai,     // write
    /// bids_ai,            // write
    /// asks_ai,            // write
    /// event_queue_ai,    // write
    /// referrer_mango_account_ai, //write
    /// default_acc,
    // MangoPlacePerpOrder { //Only Market Orders
    //     perp_market_id: u8,
    //     side: Side,
    //     price: i64,
    //     quantity: i64,
    //     reduce_only: bool
    // },

    /// Place an order on the Serum Dex and settle funds from the open orders account
    ///
    /// Accounts expected by this instruction (19 + 2 * NUM_MARKETS):
    /// ProgramId
    /// fund_state_acc,
    /// manager_acc,
    /// mango_prog_ai,
    /// mango_group_ai,     // read
    /// mango_account_ai,   // write
    /// mango_cache_ai,     // read
    /// perp_market_ai,     // write
    /// bids_ai,            // write
    /// asks_ai,            // write
    /// event_queue_ai,    // write
    /// default_acc,
    // MangoPlacePerpOrder2  {
    //     // perp Market
    //     perp_market_id: u8,

    //     /// Price in quote lots per base lots.
    //     ///
    //     /// Effect is based on order type, it's usually
    //     /// - fill orders on the book up to this price or
    //     /// - place an order on the book at this price.
    //     ///
    //     /// Ignored for Market orders and potentially adjusted for PostOnlySlide orders.
    //     price: i64,

    //     /// Max base lots to buy/sell.
    //     max_base_quantity: i64,

    //     /// Max quote lots to pay/receive (not taking fees into account).
    //     max_quote_quantity: i64,

    //     /// Arbitrary user-controlled order id.
    //     client_order_id: u64,

    //     /// Timestamp of when order expires
    //     ///
    //     /// Send 0 if you want the order to never expire.
    //     /// Timestamps in the past mean the instruction is skipped.
    //     /// Timestamps in the future are reduced to now + 255s.
    //     expiry_timestamp: u64,

    //     side: Side,

    //     /// Can be 0 -> LIMIT, 1 -> IOC, 2 -> PostOnly, 3 -> Market, 4 -> PostOnlySlide
    //     order_type: OrderType,

    //     reduce_only: bool,

    //     /// Maximum number of orders from the book to fill.
    //     ///
    //     /// Use this to limit compute used during order matching.
    //     /// When the limit is reached, processing stops and the instruction succeeds.
    //     limit: u8,
    // },

    // CancelPerpOrder {
    //     client_order_id: u64
    // },

    // WithdrawProcessLimitOrders,
    
    /// Settle all funds from serum dex open orders into MarginAccount positions
    ///
    /// Accounts expected by this instruction (14):
    /// 0.  [writable]  fund_Account_acc - Fund State Account
    /// 1.  [signer]    manager_acc - Manager Account to sign
    /// 3.  []          mango_prog_acc - Mango Program Account
    /// 
    /// 0. `[writable]` mango_group_acc - MangoGroup that this margin account is for
    /// 2. `[writable]` margin_account_acc - MarginAccount
    /// 3. `[]` clock_acc - Clock sysvar account
    /// 4. `[]` dex_prog_acc - program id of serum dex
    /// 5  `[writable]` spot_market_acc - dex MarketState account
    /// 6  `[writable]` open_orders_acc - open orders for this market for this MarginAccount
    /// 7. `[]` signer_acc - MangoGroup signer key
    /// 8. `[writable]` dex_base_acc - base vault for dex MarketState
    /// 9. `[writable]` dex_quote_acc - quote vault for dex MarketState
    /// 10. `[writable]` base_vault_acc - MangoGroup base vault acc
    /// 11. `[writable]` quote_vault_acc - MangoGroup quote vault acc
    /// 12. `[]` dex_signer_acc - dex Market signer account
    /// 13. `[]` spl token program
    // MangoRemovePerpIndex {
    //     perp_market_id: u8
    // },

    /// Withdraw funds that were deposited earlier.
    ///
    /// Accounts expected by this instruction (8 + 2 * NUM_MARKETS):
    ///
    /// 0.  [writable]  fund_account_acc - Fund State Account
    /// 1.  []          price_acc - Aggregator Price Account
    /// 1.  [signer]    manager_acc - Manager Account to sign
    /// 3.  []          mango_prog_acc - Mango Program Account
    /// 
    /// 0. `[writable]` mango_group_acc - MangoGroup that this margin account is for
    /// 1. `[writable]` margin_account_acc - the margin account for this user
    /// 3. `[writable]` token_account_acc - TokenAccount owned by user which will be receiving the funds
    /// 4. `[writable]` vault_acc - TokenAccount owned by MangoGroup which will be sending
    /// 5. `[]` signer_acc - acc pointed to by signer_key
    /// 6. `[]` token_prog_acc - acc pointed to by SPL token program id
    /// 7. `[]` clock_acc - Clock sysvar account
    /// 8..8+NUM_MARKETS `[]` open_orders_accs - open orders for each of the spot market
    /// 8+NUM_MARKETS..8+2*NUM_MARKETS `[]`
    ///     oracle_accs - flux aggregator feed accounts
    // MangoClosePosition {
    //     price: u64 // remove later
    // },

    /// Withdraw funds from Mango
    ///
    /// Accounts expected by this instruction: 
    ///
    /// fund_account_ai,
    // manager_ai,
    // mango_prog_ai,

    // mango_group_ai,     // read
    // mango_account_ai,   // write
    // mango_cache_ai,     // read
    // root_bank_ai,       // read
    // node_bank_ai,       // write
    // vault_ai,           // write
    // fund_token_ai,       // write
    // signer_ai,          // read
    // token_prog_ai,      // read
    // default_ai
    // MangoWithdraw {
    //     token_slot_index: u8,
    //     mango_token_index: u8,
    //     quantity: u64
    // },

    /// Withdraw funds that were deposited earlier.
    // fund_account_ai,      //write
    // investor_state_ai,  //write
    // investor_ai,        //signer
    // mango_prog_ai,      //
    // mango_group_ai,     // read
    // mango_account_ai,   // write
    // mango_cache_ai,     // read
    // usdc_root_bank_ai,       // read
    // usdc_node_bank_ai,       // write
    // usdc_vault_ai,           // write
    // usdc_investor_token_ai,   // write
    // token_root_bank_ai,       // read
    // token_node_bank_ai,       // write
    // token_vault_ai,           // write
    // token_investor_token_ai,   // write
    // signer_ai,          // read
    // token_prog_ai,      // read
    // default_ai
    // MangoWithdrawInvestor,

    /// Place an order on the Serum Dex and settle funds from the open orders account
    ///fund_account_ai,
    // manager_ai,
    // mango_prog_ai,
    // mango_group_ai,         // read
    // mango_account_ai,       // write
    // mango_cache_ai,         // read
    // dex_prog_ai,            // read
    // spot_market_ai,         // write
    // bids_ai,                // write
    // asks_ai,                // write
    // dex_request_queue_ai,   // write
    // dex_event_queue_ai,     // write
    // dex_base_ai,            // write
    // dex_quote_ai,           // write
    // base_root_bank_ai,      // read
    // base_node_bank_ai,      // write
    // base_vault_ai,          // write
    // quote_root_bank_ai,     // read
    // quote_node_bank_ai,     // write
    // quote_vault_ai,         // write
    // token_prog_ai,          // read
    // signer_ai,              // read
    // dex_signer_ai,          // read
    // msrm_or_srm_vault_ai,   // read
    /// +NUM_MARKETS `[writable]` open_orders_accs - open orders for each of the spot market
    ///
    // MangoPlaceSpotOrder {
    //     side: u8, // 1 for sell, 0 for buy
    //     price: u64, // remove later
    //     trade_size: u64 // trade amount
    // },
    

    /// Settle all funds from serum dex open orders into MarginAccount positions
    ///
    /// Accounts expected by this instruction (14):
    /// 0.  [writable]  fund_account_acc - Fund State Account
    /// 1.              investor_state_acc
    /// 1.  [signer]    investor_acc - Investor Account to sign
    /// 3.  []          mango_prog_acc - Mango Program Account
    /// 
    /// 0. `[writable]` mango_group_acc - MangoGroup that this margin account is for
    /// 2. `[writable]` margin_account_acc - MarginAccount
    /// 3. `[]` clock_acc - Clock sysvar account
    /// 4. `[]` dex_prog_acc - program id of serum dex
    /// 5  `[writable]` spot_market_acc - dex MarketState account
    /// 6  `[writable]` open_orders_acc - open orders for this market for this MarginAccount
    /// 7. `[]` signer_acc - MangoGroup signer key
    /// 8. `[writable]` dex_base_acc - base vault for dex MarketState
    /// 9. `[writable]` dex_quote_acc - quote vault for dex MarketState
    /// 10. `[writable]` base_vault_acc - MangoGroup base vault acc
    /// 11. `[writable]` quote_vault_acc - MangoGroup quote vault acc
    /// 12. `[]` dex_signer_acc - dex Market signer account
    /// 13. `[]` spl token program
    // MangoWithdrawInvestorSettle,

    /// Accounts Expected
    /// 0. [WRITE] Platform Account
    /// 1. [READ] CLOCK SYSVAR account
    /// 2. [SIGNER] Investin Admin Account
    /// 3. [READ]   Token Mint Account
    /// 4. []   Pool Token Account
    /// 5. []   Pool Base Token Account
    /// ............
    /// N. 
    /// 
    
    ChangeFundPrivacy,

    AddTokenToWhitelist {
        token_id: u8,
        pc_index: u8
    },

    /// Accounts Expected
    /// 0. [WRITE] Platform Account
    /// 1. [READ] CLOCK SYSVAR account
    /// 2. [READ]   Pool Token Account
    /// 3. [READ]   Pool Base Token Account
    /// ......
    UpdateTokenPrices {
        count: u8 // count of tokens
    },

    // Platform Account
    // Fund state account
    // Token mint account
    AddTokenToFund {
        index: u8 // index of slot
    },

    // Platform Account
    // Fund State account
    // Token mint account
    RemoveTokenFromFund {
        index: u8 // index of slot
    },

    FriktionInvestorWithdrawUL2,

    FlushDebts {
        index: u8,
        count: u8
    },

    SetSwapGuard {
        token_in_fund_slot: u8, 
        token_out_fund_slot: u8,
        amount_in: u64
    },

    FriktionDeposit {
        deposit_amount: u64
    },

    FriktionDeposit0 {
        deposit_amount: u64
    },

    FriktionWithdraw {
        withdraw_amount: u64
    },

    JupiterSwap,
    CheckSwapGuard,
    InitOpenOrderAccounts,
    ReadFriktion,
    FriktionCancelPendingDeposit,
    FriktionCancelPendingWithdrawal,
    FriktionClaimPendingDeposit,
    FriktionClaimPendingWithdrawal,
    UpdateFriktionValue,
    FriktionAddToFund {
        ul_token_slot: u8
    },
    FriktionRemoveFromFund,
    FriktionInvestorWithdrawUL,
    FriktionInvestorWithdrawFTokens,
    InitReimbursement,

    Reimburse{
        token_index: usize, 
        index_into_table: usize, 
    },
}


#[derive(Clone, BorshSerialize)]
pub struct Data {
    pub instr: u8,
    pub amount_in: u64,
    pub min_amount_out: u64
}

impl FundInstruction {
    pub fn unpack(input: &[u8]) -> Option<Self> {
        let (&op, data) = array_refs![input, 1; ..;];
        let op = u8::from_le_bytes(op);
        Some(match op {
            0 => {
                let data = array_ref![data, 0, 8 + 8 + 1 + 1];
                let (
                    min_amount,
                    performance_fee_percentage,
                    no_of_tokens,
                    is_private
                ) = array_refs![data, 8, 8, 1, 1];
                let is_private = match is_private {
                    [0] => false,
                    [1] => true,
                    _ => return None,
                };
                FundInstruction::Initialize {
                    min_amount: u64::from_le_bytes(*min_amount),
                    performance_fee_percentage: u64::from_le_bytes(*performance_fee_percentage),
                    no_of_tokens: u8::from_le_bytes(*no_of_tokens),
                    is_private
                }
            },
            1 => {
                let data = array_ref![data, 0, 8 + 1];
                let (
                    amount,
                    index
                ) = array_refs![data, 8, 1];

                FundInstruction::InvestorDeposit {
                    amount: u64::from_le_bytes(*amount),
                    index: u8::from_le_bytes(*index)
                }
            },
            2 => {
                FundInstruction::ManagerTransfer
            },
            3 => {
                FundInstruction::InvestorWithdrawFromFund
            },
            4 => {
                FundInstruction::InvestorWithdrawSettleFunds
            }
            5 => {
                let data = array_ref![data, 0, 1 + 1 + 8 + 8];
                let (
                    swap_index,
                    instruction,
                    amount_in,
                    min_amount_out
                ) = array_refs![data, 1, 1, 8, 8];

                FundInstruction::Swap {
                    swap_index: u8::from_le_bytes(*swap_index),
                    data: Data {
                        instr: u8::from_le_bytes(*instruction),
                        amount_in: u64::from_le_bytes(*amount_in),
                        min_amount_out: u64::from_le_bytes(*min_amount_out)
                    }
                }
            },
            6 => {
                FundInstruction::ClaimPerformanceFee
            },
            7 => {
                let data = array_ref![data, 0, 6 + 8 + 8];
                let (
                    intialize_platform,
                    freeze_platform,
                    unfreeze_platform,
                    change_vault,
                    freeze_fund,
                    unfreeze_fund,
                    change_min_amount,
                    change_perf_fee
                ) = array_refs![data, 1, 1, 1, 1, 1, 1, 8, 8];
                FundInstruction::AdminControl {
                    intialize_platform: u8::from_le_bytes(*intialize_platform),
                    freeze_platform: u8::from_le_bytes(*freeze_platform),
                    unfreeze_platform: u8::from_le_bytes(*unfreeze_platform),
                    change_vault: u8::from_le_bytes(*change_vault),
                    freeze_fund: u8::from_le_bytes(*freeze_fund),
                    unfreeze_fund: u8::from_le_bytes(*unfreeze_fund),
                    change_min_amount: u64::from_le_bytes(*change_min_amount),
                    change_perf_fee: u64::from_le_bytes(*change_perf_fee)
                }
            },
            // 8 => {
            //     FundInstruction::MangoInitialize
            // },
            // 9 => {
            //     let data = array_ref![data, 0, 1 + 1 + 8];

            //     let (
            //         token_slot_index,
            //         mango_token_index,
            //         quantity
            //     ) = array_refs![data, 1, 1, 8];

            //     FundInstruction::MangoDeposit{
            //         token_slot_index: u8::from_le_bytes(*token_slot_index),
            //         mango_token_index: u8::from_le_bytes(*mango_token_index),
            //         quantity: u64::from_le_bytes(*quantity)
            //     }
            // },
            // 10 => {
            //     let data_arr = array_ref![data, 0, 1 + 1 + 8 + 8 + 1];
            //     let (perp_market_id, side, price, quantity, reduce_only) =
            //     array_refs![data_arr, 1, 1, 8, 8, 1];
            //     let reduce_only = match reduce_only {
            //         [0] => false,
            //         [1] => true,
            //         _ => return None,
            //     };
            //     FundInstruction::MangoPlacePerpOrder {
            //         perp_market_id: u8::from_le_bytes(*perp_market_id),
            //         side: Side::try_from_primitive(side[0]).ok()?,
            //         price: i64::from_le_bytes(*price),
            //         quantity: i64::from_le_bytes(*quantity),
            //         reduce_only
            //     }
            // },
            // 11 => {
            //     let perp_market_id = array_ref![data, 0, 1];
            //     FundInstruction::MangoRemovePerpIndex {
            //         perp_market_id: u8::from_le_bytes(*perp_market_id)
            //     }
            // },
            // 12 => {
            //     let data_arr = array_ref![data, 0, 17];
            //     let (
            //         side,
            //         price,
            //         trade_size
            //     ) = array_refs![data_arr, 1, 8, 8];
            //     FundInstruction::MangoPlaceSpotOrder {
            //         side: u8::from_le_bytes(*side),
            //         price: u64::from_le_bytes(*price),
            //         trade_size: u64::from_le_bytes(*trade_size),
            //     }
            // },
            // 13 => {
            //     let data_arr = array_ref![data, 0, 1 + 1 + 8];
            //     let (token_slot_index, mango_token_index, quantity) = array_refs![data_arr, 1, 1, 8];
            //     FundInstruction::MangoWithdraw {
            //         token_slot_index: u8::from_le_bytes(*token_slot_index),
            //         mango_token_index: u8::from_le_bytes(*mango_token_index),
            //         quantity: u64::from_le_bytes(*quantity)
            //     }
            // },
            // 14 => {
            //     FundInstruction::MangoWithdrawInvestor
            // },
            // 15 => {
            //     let price = array_ref![data, 0, 8];
            //     FundInstruction::MangoWithdrawInvestorPlaceOrder {
            //         price: u64::from_le_bytes(*price),
            //     }
            // },
            16 => {
                FundInstruction::ChangeFundPrivacy
            },
            17 => {
                let data = array_ref![data, 0, 2];
                let (
                    token_id,
                    pc_index
                ) = array_refs![data, 1, 1]; 
                FundInstruction::AddTokenToWhitelist {
                    token_id: u8::from_le_bytes(*token_id),
                    pc_index: u8::from_le_bytes(*pc_index)
                }
            },
            18 => {
                let count = array_ref![data, 0, 1];
                FundInstruction::UpdateTokenPrices {
                    count: u8::from_le_bytes(*count)
                }
            },
            // TODO:: remove redunant instruction
            19 => {
                let count = array_ref![data, 0, 1];
                FundInstruction::UpdateTokenPrices {
                    count: u8::from_le_bytes(*count)
                }
            },
            20 => {
                let index = array_ref![data, 0, 1];
                FundInstruction::AddTokenToFund {
                    index: u8::from_le_bytes(*index)
                }
            },
            21 => {
                let index = array_ref![data, 0, 1];
                FundInstruction::RemoveTokenFromFund{
                    index: u8::from_le_bytes(*index)
                }
            },
            22 => {
                let data = array_ref![data, 0, 2];
                let (
                    index,
                    count
                ) = array_refs![data, 1, 1]; 
                FundInstruction::FlushDebts{
                    index: u8::from_le_bytes(*index),
                    count: u8::from_le_bytes(*count)
                }
            }

            23 => {
                FundInstruction::JupiterSwap
            }

            24 => {
                let deposit_amount = array_ref![data, 0, 8];
                FundInstruction::FriktionDeposit{
                    deposit_amount: u64::from_le_bytes(*deposit_amount)
                }
            }

            25 => {
                let data = array_ref![data, 0, 1 + 1 + 8];
                let (
                    token_in_fund_slot,
                    token_out_fund_slot,
                    amount_in
                ) = array_refs![data, 1, 1, 8]; 
                FundInstruction::SetSwapGuard{
                    token_in_fund_slot: u8::from_le_bytes(*token_in_fund_slot),
                    token_out_fund_slot: u8::from_le_bytes(*token_out_fund_slot),
                    amount_in: u64::from_le_bytes(*amount_in)
                }
            }

            26 => {
                FundInstruction::CheckSwapGuard
            }

            27 => {
                FundInstruction::InitOpenOrderAccounts
            }
            
            // 28 => {
            //     let data_arr = array_ref![data, 0, 1 + 8 + 8 + 8 + 8 + 8 + 1 + 1 + 1 + 1 ];
            //     let (
            //         perp_market_id,
            //         price,
            //         max_base_quantity,
            //         max_quote_quantity,
            //         client_order_id,
            //         expiry_timestamp,
            //         side,
            //         order_type,
            //         reduce_only,
            //         limit,
            //     ) = array_refs![data_arr, 1, 8, 8, 8, 8, 8, 1, 1, 1, 1];

            //     FundInstruction::MangoPlacePerpOrder2 { 
            //         perp_market_id: u8::from_le_bytes(*perp_market_id),
            //         price: i64::from_le_bytes(*price),
            //         max_base_quantity: i64::from_le_bytes(*max_base_quantity),
            //         max_quote_quantity: i64::from_le_bytes(*max_quote_quantity),
            //         client_order_id: u64::from_le_bytes(*client_order_id),
            //         expiry_timestamp: u64::from_le_bytes(*expiry_timestamp),
            //         side: Side::try_from_primitive(side[0]).ok()?, 
            //         order_type: OrderType::try_from_primitive(order_type[0]).ok()?, 
            //         reduce_only: reduce_only[0] != 0,
            //         limit: u8::from_le_bytes(*limit),
            //     }     
            // }

            // 29 => {
            //     let order_id = array_ref![data, 0, 8];
            //     FundInstruction::CancelPerpOrder {
            //         client_order_id: u64::from_le_bytes(*order_id)
            //     }
            // }
            // 30 => {
            //     FundInstruction::WithdrawProcessLimitOrders
            // }
            
            33 => {
                FundInstruction::ReadFriktion
            }

            34 => {
                let deposit_amount = array_ref![data, 0, 8];
                FundInstruction::FriktionDeposit0{
                    deposit_amount: u64::from_le_bytes(*deposit_amount)
                }
            }

            35 => {
                FundInstruction::FriktionCancelPendingDeposit

            }
            36 => {
                let withdraw_amount = array_ref![data, 0, 8];
                FundInstruction::FriktionWithdraw{
                    withdraw_amount: u64::from_le_bytes(*withdraw_amount)
                }
            }
            37 => {
                FundInstruction::FriktionCancelPendingWithdrawal
            }
            38 => {
                FundInstruction::FriktionClaimPendingDeposit
            }
            39 => {
                FundInstruction::FriktionClaimPendingWithdrawal
            }
            40 => {
                FundInstruction::UpdateFriktionValue
            }
            41 => {
                let ul_token_slot = array_ref![data, 0, 1];
                FundInstruction::FriktionAddToFund {
                    ul_token_slot: u8::from_le_bytes(*ul_token_slot)
                }
            }
            42 => {
                FundInstruction::FriktionRemoveFromFund
            }
            43 => {
                FundInstruction::FriktionInvestorWithdrawUL
            }
            44 => {
                FundInstruction::FriktionInvestorWithdrawFTokens
            }
            45 => {
                FundInstruction::FriktionInvestorWithdrawUL2
            }
            46 => FundInstruction::InitReimbursement,
            
            47 => {
                let data = array_ref![data, 0, 8 + 8 ];
                let (token_index, index_into_table) = array_refs![data, 8, 8];

                FundInstruction::Reimburse { 
                    token_index: usize::from_le_bytes(*token_index), 
                    index_into_table: usize::from_le_bytes(*index_into_table), 
                }
            }
            


            _ => { return None; }
        })
    }
}