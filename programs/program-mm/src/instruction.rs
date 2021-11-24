use arrayref::{array_ref, array_refs};
use mango::matching::{OrderType, Side};
use num_enum::TryFromPrimitive;


#[repr(C)]
#[derive(Clone)]
pub enum FundInstruction {

    /// Accounts expected
    /// 0. [WRITE]  Fund State Account
    /// 1. [SIGNER] Manager Wallet Account
    /// 2. []       Fund PDA Account
    /// 3. []       Fund Vault USDC Account
    /// 4. []       Fund Vault MNGO Account
    /// 4. []       Mango Group Account
    /// 5. []       Mango Account
    /// 6. []       Mango Prog Account

    Initialize {
        min_amount: u64,
        min_return: u64,
        performance_fee_percentage: u64
        // perp_market_index: u8
    },

    /// 0. [WRITE]  Fund State Account 
    /// 1. [WRITE]  Investor State Account
    /// 2. [SIGNER] Investor Wallet Account
    /// 3. []       Investor Base Token Account
    /// 4. []       Fund Vault Account
    ///             mango_prog_ai - Mango Prog account
    /// 6. []       Mango Group Account
    /// 7. []       Mango Account
    /// 8. []       Mango Cache Account
    /// 9. []       Token Program
    InvestorDeposit {
        amount: u64
    },

    
    /// 0.  [writable]  fund_state_acc - Fund State Account
    /// 1.  [signer]    investor_state_acc - Inv State Account
    /// 2.  []          investor_acc - Investor Wallet
    /// 3.  []  fund_vault_acc - Fund Vault
    /// 4.  []          mango_prog_acc - Mango Program Account
    /// 
    /// 0. `[]` mango_group_ai - MangoGroup
    /// 1. `[writable]` mango_account_ai - the MangoAccount of owner
    /// 2. `[signer]` owner_ai - owner of MangoAccount (fund_pda_acc)
    /// 3. `[]` mango_cache_ai - MangoCache for this MangoGroup
    /// 4. `[writable]` perp_market_ai
    /// 5. `[writable]` bids_ai - bids account for this PerpMarket
    /// 6. `[writable]` asks_ai - asks account for this PerpMarket
    /// 7. `[writable]` event_queue_ai - EventQueue for this PerpMarket
    /// 
    /// 4. `[read]` usdc_root_bank_ai,     -
    /// 5. `[write]` usdc_node_bank_ai,     -
    /// 6. `[write]` usdc_vault_ai,         -
    /// 7. `[write]` inv_token_acc, -
    /// 8. `[read]` signer_ai,        -
    /// 9. `[read]` token_prog_ai,    -
    /// 10.`[read]` default_acc, (for open orders)
    /// 
    /// 
    InvestorWithdraw,

    /// 0.  [writable]  fund_state_acc - Fund State Account
    /// 1.  [writable]  investor_state_acc - Investor State Account
    /// 2.  []          investor_acc - Investor Account
    /// 3.  []          mango_prog_acc - Mango Program Account
    /// 4.  []          investor_mngo_acc - Investor Mango Account
    /// 
    /// 0. `[]` mango_group_ai - MangoGroup that this mango account is for
    /// 1. `[]` mango_cache_ai - MangoCache
    /// 2. `[writable]` mango_account_ai - MangoAccount
    /// 3. `[signer]` owner_ai - MangoAccount owner (fund_pda_acc)
    /// 4. `[]` perp_market_ai - PerpMarket
    /// 5. `[writable]` mngo_perp_vault_ai
    /// 6. `[]` mngo_root_bank_ai
    /// 7. `[writable]` mngo_node_bank_ai
    /// 8. `[writable]` mngo_bank_vault_ai
    /// 9. `[]` signer_ai - Group Signer Account
    /// 10. `[]` token_prog_ai - SPL Token program id
    /// 11. `[]` default_ai - SPL Token program id

    InvestorHarvestMngo,

    /// 0.  [writable]  fund_state_ai - Fund State Account
    /// 1.  [writable]  manager_ai - Manager Account
    /// 2.  []          mango_prog_ai - Mango Program Account
    /// 3.  []          man_mngo_ai - Manager Mango Account
    /// 
    /// 0. `[]` mango_group_ai - MangoGroup that this mango account is for
    /// 1. `[]` mango_cache_ai - MangoCache
    /// 2. `[writable]` mango_account_ai - MangoAccount
    /// 3. `[signer]` owner_ai - MangoAccount owner (fund_pda_acc)
    /// 4. `[]` perp_market_ai - PerpMarket
    /// 5. `[writable]` mngo_perp_vault_ai
    /// 6. `[]` mngo_root_bank_ai
    /// 7. `[writable]` mngo_node_bank_ai
    /// 8. `[writable]` mngo_bank_vault_ai
    /// 9. `[]` signer_ai - Group Signer Account
    /// 10. `[]` token_prog_ai - SPL Token program id
    /// 11. `[]` default_ai - SPL Token program id
    ManagerHarvestMngo,


    // fund_state_ai,
    // manager_ai,
    // manager_btoken_ai,
    // fund_vault_ai,
    // mango_prog_ai,
    // mango_group_ai,
    // mango_cache_ai,
    // mango_account_ai,
    // fund_pda_ai,
    // token_prog_ai
    ClaimPerformanceFee,


    /// Proxy to Deposit instruction on Mango
    /// 
    /// 0.  [writable]  fund_state_acc - Fund State Account
    /// 1.  [signer]    manager_acc - Manager Account to sign
    /// 2.  []          mango_prog_acc - Mango Program Account
    /// 
    /// 0. `[]` mango_group_ai - MangoGroup that this mango account is for
    /// 1. `[writable]` mango_account_ai - the mango account for this user
    /// 2. `[signer]` owner_ai - Solana account of owner of the mango account (fund_pda_acc)
    /// 3. `[]` mango_cache_ai - MangoCache
    /// 4. `[]` root_bank_ai - RootBank owned by MangoGroup
    /// 5. `[writable]` node_bank_ai - NodeBank owned by RootBank
    /// 6. `[writable]` vault_ai - TokenAccount owned by MangoGroup
    /// 7. `[]` token_prog_ai - acc pointed to by SPL token program id
    /// 8. `[writable]` owner_token_account_ai - TokenAccount owned by user which will be sending the funds
    MangoDeposit {
        quantity: u64
    },

    /// Proxy to Withdraw instruction on Mango
    /// 
    /// 0. `[write]`  fund_state_acc - Fund State Account
    /// 1. `[signer]`    manager_acc - Manager Account to sign
    /// 2. `[]`          mango_prog_acc - Mango Program Account
    /// 
    /// 0. `[read]` mango_group_ai,   -
    /// 1. `[write]` mango_account_ai, -
    /// 2. `[read]` owner_ai,         - (fund_pda_acc)
    /// 3. `[read]` mango_cache_ai,   -
    /// 4. `[read]` root_bank_ai,     -
    /// 5. `[write]` node_bank_ai,     -
    /// 6. `[write]` vault_ai,         -
    /// 7. `[write]` token_account_ai, -
    /// 8. `[read]` signer_ai,        -
    /// 9. `[read]` token_prog_ai,    -
    /// 10.`[read]` default_acc, (for open orders)
    MangoWithdraw {
        quantity: u64
    },


    /// Proxy to PlacePerpOrder instruction on Mango
    ///
    /// Accounts expected by this instruction (19 + 2 * NUM_MARKETS):
    ///
    /// 0.  [writable]  fund_state_acc - Fund State Account
    /// 1.  [signer]    manager_acc - Manager Account to sign
    /// 2.  []          mango_prog_acc - Mango Program Account
    /// 
    /// 0. `[]` mango_group_ai - MangoGroup
    /// 1. `[writable]` mango_account_ai - the MangoAccount of owner
    /// 2. `[signer]` owner_ai - owner of MangoAccount (fund_pda_acc)
    /// 3. `[]` mango_cache_ai - MangoCache for this MangoGroup
    /// 4. `[writable]` perp_market_ai
    /// 5. `[writable]` bids_ai - bids account for this PerpMarket
    /// 6. `[writable]` asks_ai - asks account for this PerpMarket
    /// 7. `[writable]` event_queue_ai - EventQueue for this PerpMarket
    MangoPlacePerpOrder {
        price: i64,
        quantity: i64,
        client_order_id: u64,
        side: Side,
        /// Can be 0 -> LIMIT, 1 -> IOC, 2 -> PostOnly
        order_type: OrderType,
    },
    
    /// Settle all funds from serum dex open orders into MarginAccount positions
    ///
    /// 0.  [writable]  fund_state_acc - Fund State Account
    /// 1.  [signer]    manager_acc - Manager Account to sign
    /// 2.  []          mango_prog_acc - Mango Program Account
    /// 
    /// 0. `[]` mango_group_ai - MangoGroup
    /// 1. `[writable]` mango_account_ai - the MangoAccount of owner
    /// 2. `[signer]` owner_ai - owner of MangoAccount (fund_pda_acc)
    /// 3. `[writable]` perp_market_ai
    /// 4. `[writable]` bids_ai - bids account for this PerpMarket
    /// 5. `[writable]` asks_ai - asks account for this PerpMarket
    MangoCancelPerpById {
        client_order_id: u64,
        invalid_id_ok: bool
    },

    ///  Proxy to RedeemMngo instruction
    /// 
    ///  Redeem the mngo_accrued in a PerpAccount for MNGO in MangoAccount deposits
    ///
    /// Accounts expected by this instruction (11):
    /// 
    /// 0. []   fund_state_acc
    /// 1. []   mango_group_acc
    /// 
    /// 0. `[]` mango_group_ai - MangoGroup that this mango account is for
    /// 1. `[]` mango_cache_ai - MangoCache
    /// 2. `[writable]` mango_account_ai - MangoAccount
    /// 3. `[signer]` owner_ai - MangoAccount owner
    /// 4. `[]` perp_market_ai - PerpMarket
    /// 5. `[writable]` mngo_perp_vault_ai
    /// 6. `[]` mngo_root_bank_ai
    /// 7. `[writable]` mngo_node_bank_ai
    /// 8. `[writable]` mngo_bank_vault_ai
    /// 9. `[]` signer_ai - Group Signer Account
    /// 10. `[]` token_prog_ai - SPL Token program id
    RedeemMngo,

    // Add delegate to call place/cancel mango instructions
    /// 0. []   fund_state_ai - Fund state account
    /// 1. []   manager_ai - Manager Account
    /// 2. []   delegate_ai - Delegate account
    AddDelegate

    // Add perp market to the fund
    /// 0. [] fund_state_ai
    /// 
    AddPerpMarket
}

impl FundInstruction {
    pub fn unpack(input: &[u8]) -> Option<Self> {
        let (&op, data) = array_refs![input, 4; ..;];
        let op = u32::from_le_bytes(op);
        Some(match op {
            0 => {
                let data = array_ref![data, 0, 8 + 8 + 8 + 1];
                let (
                    min_amount,
                    min_return,
                    performance_fee_percentage,
                    perp_market_index
                ) = array_refs![data, 8, 8, 8, 1];

                FundInstruction::Initialize {
                    min_amount: u64::from_le_bytes(*min_amount),
                    min_return: u64::from_le_bytes(*min_return),
                    performance_fee_percentage: u64::from_le_bytes(*performance_fee_percentage),
                    perp_market_index: u8::from_le_bytes(*perp_market_index),
                }
            },
            1 => {
                let amount = array_ref![data, 0, 8];
                FundInstruction::InvestorDeposit {
                    amount: u64::from_le_bytes(*amount)
                }
            },
            2 => {
                FundInstruction::InvestorWithdraw
            },
            3 => {
                FundInstruction::InvestorHarvestMngo
            },
            4 => {
                FundInstruction::ManagerHarvestMngo
            },
            5 => {
                FundInstruction::ClaimPerformanceFee
            },
            6 => {
                let quantity = array_ref![data, 0, 8];
                FundInstruction::MangoDeposit{
                    quantity: u64::from_le_bytes(*quantity)
                }
            },
            7 => {
                let quantity = array_ref![data, 0, 8];
                FundInstruction::MangoWithdraw{
                    quantity: u64::from_le_bytes(*quantity)
                }
            },
            8 => {
                let data_arr = array_ref![data, 0, 27];
                let (perp_market_id, price, quantity, client_order_id, side, order_type) =
                array_refs![data_arr, 1, 8, 8, 8, 1, 1];
                FundInstruction::MangoPlacePerpOrder {
                    perp_market_id: u8::from_le_bytes(*perp_market_id),
                    price: i64::from_le_bytes(*price),
                    quantity: i64::from_le_bytes(*quantity),
                    client_order_id: u64::from_le_bytes(*client_order_id),
                    side: Side::try_from_primitive(side[0]).ok()?,
                    order_type: OrderType::try_from_primitive(order_type[0]).ok()?,
                }
            },
            9 => {
                let data_arr = array_ref![data, 0, 9];
                let (client_order_id, invalid_id_ok) = array_refs![data_arr, 8, 1];

                FundInstruction::MangoCancelPerpById {
                    client_order_id: u64::from_le_bytes(*client_order_id),
                    invalid_id_ok: invalid_id_ok[0] != 0,
                }
            },
            10 => {
                FundInstruction::RedeemMngo
            },
            11 => {
                FundInstruction::AddDelegate
            },
            _ => { return None; }
        })
    }
}