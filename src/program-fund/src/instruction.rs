use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
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

    // AdminControl{

    // }

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

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct Data {
    pub instr: u8,
    pub amount_in: u64,
    pub min_amount_out: u64
}
