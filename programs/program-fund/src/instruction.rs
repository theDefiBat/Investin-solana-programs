use arrayref::{array_ref, array_refs};
use borsh::BorshSerialize;
use num_enum::TryFromPrimitive;

use std::convert::TryInto;
use std::num::NonZeroU64;
use fixed::types::U64F64;


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
        min_return: u64,
        performance_fee_percentage: u64
    },

    /// 0. [WRITE]  Fund State Account (derived from FA)
    /// 1. [WRITE]  Investor State Account (derived from IPDA)
    /// 2. [SIGNER] Investor Wallet Account
    /// 3. []       Investor Base Token Account
    /// 4. []       Router Base Token Account (derived)
    /// 5. []       PDA of Manager (Fund Address)
    /// 6. []       Token Program
    InvestorDeposit {
        amount: u64
    },

    /// 0. []       Platform State Account
    /// 1. [WRITE]  Fund State Account
    /// 2. [READ]   Price Account
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
    /// 3. [READ]   Price Account
    /// 4. [READ]   Clock Sysvar Account
    /// 5. [SIGNER] Investor Wallet Account
    /// 6. []       Router Base Token Account
    /// 7. []       PDA of Manager
    /// 8. []       PDA of Router
    /// 9. []       Token Program
    /// 10..10+NUM_TOKENS []  Investor Token Accounts
    /// 10+NUM_TOKENS.. 10+2*NUM_TOKENS  Fund Token Accounts
    InvestorWithdraw {
        amount: u64
    },

    /// 0. [WRITE] Fund State Account
    /// 1. [] Raydium Pool Program
    /// 2. [] Token Program
    /// 3..17 [] Raydium/Serum Accounts
    /// 18. [] Source Token Account
    /// 19. [] Destination Token Account
    /// 20. [] PDA of Manager
    Swap {
        data: Data
    },

    /// 0. [WRITE] Fund State Account (derived from FA)
    /// 1. [READ]   Price Account
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
        platform_is_initialized: u8,
        fund_is_initialized: u8,
        fund_min_amount: u64,
        fund_min_return: u64,
        fund_performance_fee_percentage: u64
    },

    /// Initialize a margin account for a user
    ///
    /// Accounts expected by this instruction (4):
    /// 0.  []  fund_state_acc - Fund State Account
    /// 1.  [signer]  manager_acc - Manager Account
    /// 2.  []  fund_pda_acc - Fund PDA Account
    /// 3.  []  mango_prog_acc - Mango Program Account
    /// 
    /// 0. `[]` mango_group_acc - MangoGroup that this margin account is for
    /// 1. `[writable]` margin_account_acc - the margin account data
    /// 3. `[]` rent_acc - Rent sysvar account
    MangoInitialize,

    /// Proxy to Deposit instruction on Mango
    /// 
    /// 0.  [writable]  fund_state_acc - Fund State Account
    /// 1.  [signer]    manager_acc - Manager Account to sign
    /// 2.  []          fund_pda_acc - Fund PDA Account
    /// 2.  []          mango_prog_acc - Mango Program Account
    /// 
    /// 3. `[writable]` mango_group_acc - MangoGroup that this margin account is for
    /// 4. `[writable]` margin_account_acc - the margin account for this user
    /// 6. `[writable]` token_account_acc - TokenAccount owned by user which will be sending the funds
    /// 7. `[writable]` vault_acc - TokenAccount owned by MangoGroup
    /// 8. `[]` token_prog_acc - acc pointed to by SPL token program id
    /// 9. `[]` clock_acc - Clock sysvar account
    MangoDeposit {
        quantity: u64
    },

    /// Place an order on the Serum Dex and settle funds from the open orders account
    ///
    /// Accounts expected by this instruction (19 + 2 * NUM_MARKETS):
    ///
    /// 0.  [writable]  fund_state_acc - Fund State Account
    /// 1.  [signer]    manager_acc - Manager Account to sign
    /// 2.  []          fund_pda_acc - Fund PDA Account
    /// 3.  []          mango_prog_acc - Mango Program Account
    /// 
    /// 1. `[writable]` mango_group_acc - MangoGroup that this margin account is for
    /// 3. `[writable]` margin_account_acc - MarginAccount
    /// 4. `[]` clock_acc - Clock sysvar account
    /// 5. `[]` dex_prog_acc - program id of serum dex
    /// 6. `[writable]` spot_market_acc - serum dex MarketState
    /// 7. `[writable]` dex_request_queue_acc - serum dex request queue for this market
    /// 8. `[writable]` dex_event_queue - serum dex event queue for this market
    /// 9. `[writable]` bids_acc - serum dex bids for this market
    /// 10. `[writable]` asks_acc - serum dex asks for this market
    /// 11. `[writable]` base_vault_acc - mango vault for base currency
    /// 12. `[writable]` quote_vault_acc - mango vault for quote currency
    /// 13. `[]` signer_acc - mango signer key
    /// 14. `[writable]` dex_base_acc - serum dex market's vault for base (coin) currency
    /// 15. `[writable]` dex_quote_acc - serum dex market's vault for quote (pc) currency
    /// 16. `[]` spl token program
    /// 17. `[]` the rent sysvar
    /// 18. `[writable]` srm_vault_acc - MangoGroup's srm_vault used for fee reduction
    /// 19. `[]` dex_signer_acc - signer for serum dex MarketState
    /// 20..19+NUM_MARKETS `[writable]` open_orders_accs - open orders for each of the spot market
    /// 19+NUM_MARKETS..19+2*NUM_MARKETS `[]`
    ///     oracle_accs - flux aggregator feed accounts
    MangoPlaceAndSettle {
        order: serum_dex::instruction::NewOrderInstructionV3
    },


    /// Place an order on the Serum Dex and settle funds from the open orders account
    ///
    /// Accounts expected by this instruction (19 + 2 * NUM_MARKETS):
    ///
    /// 0.  [writable]  fund_state_acc - Fund State Account
    /// 1.  [signer]    manager_acc - Manager Account to sign
    /// 2.  []          fund_pda_acc - Fund PDA Account
    /// 3.  []          mango_prog_acc - Mango Program Account
    /// 
    /// 0. `[writable]` mango_group_acc - MangoGroup that this margin account is for
    /// 2. `[writable]` margin_account_acc - MarginAccount
    /// 3. `[]` clock_acc - Clock sysvar account
    /// 4. `[]` dex_prog_acc - program id of serum dex
    /// 5. `[writable]` spot_market_acc - serum dex MarketState
    /// 6. `[writable]` dex_request_queue_acc - serum dex request queue for this market
    /// 7. `[writable]` dex_event_queue - serum dex event queue for this market
    /// 8. `[writable]` bids_acc - serum dex bids for this market
    /// 9. `[writable]` asks_acc - serum dex asks for this market
    /// 10. `[writable]` vault_acc - mango's vault for this currency (quote if buying, base if selling)
    /// 11. `[]` signer_acc - mango signer key
    /// 12. `[writable]` dex_base_acc - serum dex market's vault for base (coin) currency
    /// 13. `[writable]` dex_quote_acc - serum dex market's vault for quote (pc) currency
    /// 14. `[]` spl token program
    /// 15. `[]` the rent sysvar
    /// 16. `[writable]` srm_vault_acc - MangoGroup's srm_vault used for fee reduction
    /// 17..17+NUM_MARKETS `[writable]` open_orders_accs - open orders for each of the spot market
    /// 17+NUM_MARKETS..17+2*NUM_MARKETS `[]`
    ///     oracle_accs - flux aggregator feed accounts
    MangoPlaceOrder {
        order: serum_dex::instruction::NewOrderInstructionV3
    },
    
/// Settle all funds from serum dex open orders into MarginAccount positions
    ///
    /// Accounts expected by this instruction (14):
    /// 0.  [writable]  fund_state_acc - Fund State Account
    /// 1.  [signer]    manager_acc - Manager Account to sign
    /// 2.  []          fund_pda_acc - Fund PDA Account
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
    MangoSettleFunds,

    /// Withdraw funds that were deposited earlier.
    ///
    /// Accounts expected by this instruction (8 + 2 * NUM_MARKETS):
    ///
    /// 0.  [writable]  fund_state_acc - Fund State Account
    /// 1.  []          price_acc - Aggregator Price Account
    /// 1.  [signer]    manager_acc - Manager Account to sign
    /// 2.  []          fund_pda_acc - Fund PDA Account
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
    MangoWithdrawToFund {
        quantity: u64
    },

    /// Withdraw funds that were deposited earlier.
    ///
    /// Accounts expected by this instruction (11 + 2 * NUM_TOKENS + 2 * NUM_MARKETS):
    ///
    /// 0.  [writable]  fund_state_acc - Fund State Account
    /// 1.  []          inv_state_acc - Investor State Account
    /// 2.  [signer]    investor_acc - Investor Account
    /// 3.  []          fund_pda_acc- Fund PDA Account
    
    /// 5.  []          mango_prog_acc - Mango Program Account
    /// 
    /// 0. `[writable]` mango_group_acc - MangoGroup that this margin account is for
    /// 1. `[writable]` margin_account_acc - the margin account for this user
    /// 3. `[writable]` token_account_acc - TokenAccount owned by user which will be receiving the funds
    /// 4. `[writable]` vault_acc - TokenAccount owned by MangoGroup which will be sending
    /// 2. `[]` signer_acc - acc pointed to by signer_key
    /// 3. `[]` token_prog_acc - acc pointed to by SPL token program id
    /// 4. `[]` clock_acc - Clock sysvar account
    /// 
    /// 16..19 (NUM_MARKETS) `[]` open_orders_accs - open orders for each of the spot market
    /// 20..23 (NUM_MARKETS) `[]` oracle_accs - flux aggregator feed accounts
    MangoWithdrawInvestor {
        token_index: usize
    },

    /// Place an order on the Serum Dex and settle funds from the open orders account
    ///
    /// Accounts expected by this instruction (19 + 2 * NUM_MARKETS):
    ///
    /// 0.  [writable]  fund_state_acc - Fund State Account
    /// 1.  []          inv_state_acc - Investor State Account
    /// 1.  [signer]    investor_acc - Manager Account to sign
    /// 2.  []          fund_pda_acc - Fund PDA Account
    /// 3.  []          mango_prog_acc - Mango Program Account
    /// 
    /// 0. `[writable]` mango_group_acc - MangoGroup that this margin account is for
    /// 2. `[writable]` margin_account_acc - MarginAccount
    /// 3. `[]` clock_acc - Clock sysvar account
    /// 4. `[]` dex_prog_acc - program id of serum dex
    /// 5. `[writable]` spot_market_acc - serum dex MarketState
    /// 6. `[writable]` dex_request_queue_acc - serum dex request queue for this market
    /// 7. `[writable]` dex_event_queue - serum dex event queue for this market
    /// 8. `[writable]` bids_acc - serum dex bids for this market
    /// 9. `[writable]` asks_acc - serum dex asks for this market
    /// 10. `[writable]` vault_acc - mango's vault for this currency (quote if buying, base if selling)
    /// 11. `[]` signer_acc - mango signer key
    /// 12. `[writable]` dex_base_acc - serum dex market's vault for base (coin) currency
    /// 13. `[writable]` dex_quote_acc - serum dex market's vault for quote (pc) currency
    /// 14. `[]` spl token program
    /// 15. `[]` the rent sysvar
    /// 16. `[writable]` srm_vault_acc - MangoGroup's srm_vault used for fee reduction
    /// 17..17+NUM_MARKETS `[writable]` open_orders_accs - open orders for each of the spot market
    /// 17+NUM_MARKETS..17+2*NUM_MARKETS `[]`
    ///     oracle_accs - flux aggregator feed accounts
    MangoWithdrawInvestorPlaceOrder {
        order: serum_dex::instruction::NewOrderInstructionV3
    },

    /// Settle all funds 
    ///
    /// Accounts expected by this instruction (14):
    /// 0.  [writable]  fund_state_acc - Fund State Account
    /// 1.  [signer]    investor_acc - Investor Account to sign
    /// 2.  []          fund_pda_acc - Fund PDA Account
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
    MangoWithdrawInvestorSettle {
        token_index: usize
    },

    /// Place an order on the Serum Dex and settle funds from the open orders account
    ///
    /// Accounts expected by this instruction (19 + 2 * NUM_MARKETS):
    ///
    /// 0.  [writable]  fund_state_acc - Fund State Account
    /// 1.  []          inv_state_acc - Investor State Account
    /// 1.  [signer]    investor_acc - Manager Account to sign
    /// 2.  []          fund_pda_acc - Fund PDA Account
    /// 3.  []          mango_prog_acc - Mango Program Account
    /// 
    /// 0. `[writable]` mango_group_acc - MangoGroup that this margin account is for
    /// 2. `[writable]` margin_account_acc - MarginAccount
    /// 3. `[]` clock_acc - Clock sysvar account
    /// 4. `[]` dex_prog_acc - program id of serum dex
    /// 5. `[writable]` spot_market_acc - serum dex MarketState
    /// 6. `[writable]` dex_request_queue_acc - serum dex request queue for this market
    /// 7. `[writable]` dex_event_queue - serum dex event queue for this market
    /// 8. `[writable]` bids_acc - serum dex bids for this market
    /// 9. `[writable]` asks_acc - serum dex asks for this market
    /// 10. `[writable]` vault_acc - mango's vault for this currency (quote if buying, base if selling)
    /// 11. `[]` signer_acc - mango signer key
    /// 12. `[writable]` dex_base_acc - serum dex market's vault for base (coin) currency
    /// 13. `[writable]` dex_quote_acc - serum dex market's vault for quote (pc) currency
    /// 12. `[]` dex_signer_acc - dex Market signer account
    /// 14. `[]` spl token program
    /// 15. `[]` the rent sysvar
    /// 10. `[writable]` base_vault_acc - MangoGroup base vault acc
    /// 11. `[writable]` quote_vault_acc - MangoGroup quote vault acc
    /// 16. `[writable]` srm_vault_acc - MangoGroup's srm_vault used for fee reduction
    /// 17..17+NUM_MARKETS `[writable]` open_orders_accs - open orders for each of the spot market
    /// 17+NUM_MARKETS..17+2*NUM_MARKETS `[]`
    ///     oracle_accs - flux aggregator feed accounts
    MangoWithdrawInvestorPlaceAndSettle {
        order: serum_dex::instruction::NewOrderInstructionV3,
        token_index: usize
    },




    /// 0. [WRITE]  Fund State Account (derived from FA)
    /// 1. [READ]   Price Account
    /// 2. [READ]   Clock Sysvar Account
    /// 3. [SIGNER] Manager Wallet Account 
    /// 4. []       Fund Base Token Account
    /// 5. []       Manager Base Token Account
    /// 6. []       PDA of manager
    /// 7. []       Token Program
    TestingDeposit{
        amount: u64
    },

    /// 0. [WRITE]  Fund State Account (derived from FA)
    /// 1. [READ]   Price Account
    /// 2. [READ]   Clock Sysvar Account
    /// 3. [SIGNER] Manager Wallet Account 
    /// 4. []       Fund Base Token Account
    /// 5. []       Manager Base Token Account
    /// 6. []       PDA of manager
    /// 7. []       Token Program
    TestingWithdraw{
        amount: u64,
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
                let data = array_ref![data, 0, 8 + 8 + 8];
                let (
                    min_amount,
                    min_return,
                    performance_fee_percentage
                ) = array_refs![data, 8, 8, 8];

                FundInstruction::Initialize {
                    min_amount: u64::from_le_bytes(*min_amount),
                    min_return: u64::from_le_bytes(*min_return),
                    performance_fee_percentage: u64::from_le_bytes(*performance_fee_percentage),
                }
            },
            1 => {
                let amount = array_ref![data, 0, 8];
                FundInstruction::InvestorDeposit {
                    amount: u64::from_le_bytes(*amount)
                }
            },
            2 => {
                FundInstruction::ManagerTransfer
            },
            3 => {
                let amount = array_ref![data, 0, 8];
                FundInstruction::InvestorWithdraw {
                    amount: u64::from_le_bytes(*amount)
                }
            },
            4 => {
                let data = array_ref![data, 0, 1 + 8 + 8];
                let (
                    instruction,
                    amount_in,
                    min_amount_out
                ) = array_refs![data, 1, 8, 8];

                FundInstruction::Swap {
                    data: Data {
                        instr: u8::from_le_bytes(*instruction),
                        amount_in: u64::from_le_bytes(*amount_in),
                        min_amount_out: u64::from_le_bytes(*min_amount_out)
                    }
                }
            },
            5 => {
                FundInstruction::ClaimPerformanceFee
            },
            6 => {
                let data = array_ref![data, 0, 1 + 1 + 8 + 8 + 8];
                let (
                    platform_is_initialized,
                    fund_is_initialized,
                    fund_min_amount,
                    fund_min_return,
                    fund_performance_fee_percentage
                ) = array_refs![data, 1, 1, 8, 8, 8];

                FundInstruction::AdminControl {
                    platform_is_initialized: u8::from_le_bytes(*platform_is_initialized),
                    fund_is_initialized: u8::from_le_bytes(*fund_is_initialized),
                    fund_min_amount: u64::from_le_bytes(*fund_min_amount),
                    fund_min_return: u64::from_le_bytes(*fund_min_return),
                    fund_performance_fee_percentage: u64::from_le_bytes(*fund_performance_fee_percentage)
                }
            },
            7 => {
                FundInstruction::MangoInitialize
            },
            8 => {
                let quantity = array_ref![data, 0, 8];
                FundInstruction::MangoDeposit{
                    quantity: u64::from_le_bytes(*quantity)
                }
            },
            9 => {
                let data_arr = array_ref![data, 0, 46];
                let order = unpack_dex_new_order_v3(data_arr)?;
                FundInstruction::MangoPlaceAndSettle {
                    order
                }
            },
            10 => {
                let data_arr = array_ref![data, 0, 46];
                let order = unpack_dex_new_order_v3(data_arr)?;
                FundInstruction::MangoPlaceOrder {
                    order
                }
            },
            11 => {
                FundInstruction::MangoSettleFunds
            },
            12 => {
                let quantity = array_ref![data, 0, 8];
                FundInstruction::MangoWithdrawToFund{
                    quantity: u64::from_le_bytes(*quantity)
                }
            },
            13 => {
                let token_index = array_ref![data, 0, 8];
                FundInstruction::MangoWithdrawInvestor{
                    token_index: usize::from_le_bytes(*token_index)
                }
            },
            14 => {
                let data_arr = array_ref![data, 0, 46];
                let order = unpack_dex_new_order_v3(data_arr)?;
                FundInstruction::MangoWithdrawInvestorPlaceOrder {
                    order
                }
            },
            15 => {
                let token_index = array_ref![data, 0, 8];
                FundInstruction::MangoWithdrawInvestorSettle{
                    token_index: usize::from_le_bytes(*token_index)
                }
            },
            16 => {
                let data_arr = array_ref![data, 0, 46 + 8];
                let (
                    order,
                    token_index 
                ) = array_refs![data_arr, 46, 8];
                let order = unpack_dex_new_order_v3(order)?;
                FundInstruction::MangoWithdrawInvestorPlaceAndSettle {
                    order,
                    token_index: usize::from_le_bytes(*token_index)
                }
            },
            17 => {
                let amount = array_ref![data, 0, 8];
                FundInstruction::TestingDeposit {
                    amount: u64::from_le_bytes(*amount)
                }
            },
            18 => {
                let amount = array_ref![data, 0, 8];
                FundInstruction::TestingWithdraw {
                    amount: u64::from_le_bytes(*amount)
                }
            }
            _ => { return None; }
        })
    }
}

fn unpack_dex_new_order_v3(data: &[u8; 46]) -> Option<serum_dex::instruction::NewOrderInstructionV3> {
    let (
        &side_arr,
        &price_arr,
        &max_coin_qty_arr,
        &max_native_pc_qty_arr,
        &self_trade_behavior_arr,
        &otype_arr,
        &client_order_id_bytes,
        &limit_arr,
    ) = array_refs![data, 4, 8, 8, 8, 4, 4, 8, 2];

    let side = serum_dex::matching::Side::try_from_primitive(u32::from_le_bytes(side_arr).try_into().ok()?).ok()?;
    let limit_price = NonZeroU64::new(u64::from_le_bytes(price_arr))?;
    let max_coin_qty = NonZeroU64::new(u64::from_le_bytes(max_coin_qty_arr))?;
    let max_native_pc_qty_including_fees =
        NonZeroU64::new(u64::from_le_bytes(max_native_pc_qty_arr))?;
    let self_trade_behavior = serum_dex::instruction::SelfTradeBehavior::try_from_primitive(
        u32::from_le_bytes(self_trade_behavior_arr)
            .try_into()
            .ok()?,
    )
        .ok()?;
    let order_type = serum_dex::matching::OrderType::try_from_primitive(u32::from_le_bytes(otype_arr).try_into().ok()?).ok()?;
    let client_order_id = u64::from_le_bytes(client_order_id_bytes);
    let limit = u16::from_le_bytes(limit_arr);

    Some(serum_dex::instruction::NewOrderInstructionV3 {
        side,
        limit_price,
        max_coin_qty,
        max_native_pc_qty_including_fees,
        self_trade_behavior,
        order_type,
        client_order_id,
        limit,
    })
}