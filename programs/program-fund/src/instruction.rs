use arrayref::{array_ref, array_refs};
use borsh::BorshSerialize;
use mango::matching::{Side};
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
        min_return: u64,
        performance_fee_percentage: u64,
        no_of_tokens: u8
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

    /// 0. [WRITE]  Fund State Account (derived from FA)
    /// 1. [WRITE]  Investor State Account (derived from IPDA)
    /// 2. [SIGNER] investor wallet account
    /// 3. [READ]   Price Account
    /// 4. [READ]   Mango Group Account (for valuation)
    /// 5. [READ]   Margin Account (for valuation)
    /// 6. []       Open orders account
    /// 7  []       mango oracle account
    /// 8. [READ]   Clock Sysvar Account
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
        change_min_return: u64,
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
    MangoInitialize,

    /// Proxy to Deposit instruction on Mango
    /// 
    /// fund_state_ai,
    /// manager_ai,  
    /// fund_pda_ai, 
    /// mango_prog_ai,
    /// mango_group_ai,         // read
    /// mango_account_ai,       // write
    /// mango_cache_ai,         // read
    /// root_bank_ai,           // read
    /// node_bank_ai,           // write
    /// vault_ai,               // write
    /// token_prog_ai,          // read
    /// owner_token_account_ai, // write
    MangoDeposit {
        token_slot_index: u8,
        mango_token_index: u8,
        quantity: u64
    },

    /// Place an order on the Serum Dex and settle funds from the open orders account
    ///
    /// Accounts expected by this instruction (19 + 2 * NUM_MARKETS):
    /// ProgramId
    /// fund_state_acc,
    /// manager_acc,
    /// mango_prog_ai,
    /// mango_group_ai,     // read
    /// mango_account_ai,   // write
    /// fund_pda_acc,           // read, signer
    /// mango_cache_ai,     // read
    /// perp_market_ai,     // write
    /// bids_ai,            // write
    /// asks_ai,            // write
    /// event_queue_ai,    // write
    /// default_acc,
    MangoPlacePerpOrder { //Only Market Orders
        perp_market_id: u8,
        side: Side,
        quantity: i64
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
    MangoSettlePnL {
        perp_market_id: u8,
    },

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
    // MangoClosePosition {
    //     price: u64 // remove later
    // },

    /// Withdraw funds from Mango
    ///
    /// Accounts expected by this instruction: 
    ///
    /// fund_state_ai,
    // manager_ai,
    // mango_prog_ai,

    // mango_group_ai,     // read
    // mango_account_ai,   // write
    // fund_pda_ai,           // read
    // mango_cache_ai,     // read
    // root_bank_ai,       // read
    // node_bank_ai,       // write
    // vault_ai,           // write
    // fund_token_ai,       // write
    // signer_ai,          // read
    // token_prog_ai,      // read
    // default_ai
    MangoWithdraw {
        token_slot_index: u8,
        mango_token_index: u8,
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
    // MangoWithdrawInvestor,

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
    // MangoWithdrawInvestorPlaceOrder {
    //     price: u64
    // },

    /// Settle all funds from serum dex open orders into MarginAccount positions
    ///
    /// Accounts expected by this instruction (14):
    /// 0.  [writable]  fund_state_acc - Fund State Account
    /// 1.              investor_state_acc
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

    FlushDebts {
        index: u8,
        count: u8
    }

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
                let data = array_ref![data, 0, 8 + 8 + 8 + 1];
                let (
                    min_amount,
                    min_return,
                    performance_fee_percentage,
                    no_of_tokens
                ) = array_refs![data, 8, 8, 8, 1];

                FundInstruction::Initialize {
                    min_amount: u64::from_le_bytes(*min_amount),
                    min_return: u64::from_le_bytes(*min_return),
                    performance_fee_percentage: u64::from_le_bytes(*performance_fee_percentage),
                    no_of_tokens: u8::from_le_bytes(*no_of_tokens),
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
                let data = array_ref![data, 0, 6 + 8 + 8 + 8];
                let (
                    intialize_platform,
                    freeze_platform,
                    unfreeze_platform,
                    change_vault,
                    freeze_fund,
                    unfreeze_fund,
                    change_min_amount,
                    change_min_return,
                    change_perf_fee
                ) = array_refs![data, 1, 1, 1, 1, 1, 1, 8, 8, 8];

                FundInstruction::AdminControl {
                    intialize_platform: u8::from_le_bytes(*intialize_platform),
                    freeze_platform: u8::from_le_bytes(*freeze_platform),
                    unfreeze_platform: u8::from_le_bytes(*unfreeze_platform),
                    change_vault: u8::from_le_bytes(*change_vault),
                    freeze_fund: u8::from_le_bytes(*freeze_fund),
                    unfreeze_fund: u8::from_le_bytes(*unfreeze_fund),
                    change_min_amount: u64::from_le_bytes(*change_min_amount),
                    change_min_return: u64::from_le_bytes(*change_min_return),
                    change_perf_fee: u64::from_le_bytes(*change_perf_fee)
                }
            },
            8 => {
                FundInstruction::MangoInitialize
            },
            9 => {
                let data = array_ref![data, 0, 1 + 1 + 8];

                let (
                    token_slot_index,
                    mango_token_index,
                    quantity
                ) = array_refs![data, 1, 1, 8];

                FundInstruction::MangoDeposit{
                    token_slot_index: u8::from_le_bytes(*token_slot_index),
                    mango_token_index: u8::from_le_bytes(*mango_token_index),
                    quantity: u64::from_le_bytes(*quantity)
                }
            },
            10 => {
                let data_arr = array_ref![data, 0, 1 + 1 + 8];
                let (perp_market_id, side, quantity) =
                array_refs![data_arr, 1, 1, 8];
                FundInstruction::MangoPlacePerpOrder {
                    perp_market_id: u8::from_le_bytes(*perp_market_id),
                    side: Side::try_from_primitive(side[0]).ok()?,
                    quantity: i64::from_le_bytes(*quantity)
                }
            },
            11 => {
                let perp_market_id = array_ref![data, 0, 1];
                FundInstruction::MangoSettlePnL {
                    perp_market_id: u8::from_le_bytes(*perp_market_id)
                }
            },
            // 12 => {
            //     let price = array_ref![data, 0, 8];
            //     FundInstruction::MangoClosePosition {
            //         price: u64::from_le_bytes(*price),
            //     }
            // },
            13 => {
                let data_arr = array_ref![data, 0, 1 + 1 + 8];
                let (token_slot_index, mango_token_index, quantity) = array_refs![data_arr, 1, 1, 8];
                FundInstruction::MangoWithdraw {
                    token_slot_index: u8::from_le_bytes(*token_slot_index),
                    mango_token_index: u8::from_le_bytes(*mango_token_index),
                    quantity: u64::from_le_bytes(*quantity)
                }
            },
            // 14 => {
            //     FundInstruction::MangoWithdrawInvestor
            // },
            // 15 => {
            //     let price = array_ref![data, 0, 8];
            //     FundInstruction::MangoWithdrawInvestorPlaceOrder {
            //         price: u64::from_le_bytes(*price),
            //     }
            // },
            // 16 => {
            //     FundInstruction::MangoWithdrawInvestorSettle
            // },
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
            _ => { return None; }
        })
    }
}