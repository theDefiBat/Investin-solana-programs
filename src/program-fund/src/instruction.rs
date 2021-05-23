use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum FundInstruction {

    /// Accounts expected
    /// 0. [WRITE]  Fund State Account (derived from FA)
    /// 1. [SIGNER] Manager Wallet Account
    /// 2. []       Fund Base Token Account
    /// 3..3+NUM_TOKENS [] Token mints to be whitelisted
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

    /// 0. [WRITE]  Fund State Account
    /// 1. [WRITE]  Investor State Account
    /// 2. [SIGNER] Manager Wallet Account
    /// 3. []       Router Base Token Account
    /// 4. []       Fund Base Token Account
    /// 5. []       Manager Base Token Account
    /// 6. []       Investin Base Token Account
    /// 7. []       PDA of Router
    /// 8. []       Token Program
    /// 9..7+2*NUM_TOKENS-1 Pool Token Accounts for each pair
    ManagerTransfer,
    
    /// 0. [WRITE]  Fund State Account (derived from FA)    
    /// 1. [WRITE]  Investor State Account (derived from IPDA)
    /// 2. [SIGNER] Investor Wallet Account
    /// 3. []       Router Base Token Account
    /// 4. []       Manager Base Token Account
    /// 5. []       PDA of Router
    /// 6. []       PDA of Manager
    /// 7. []       Token Program
    /// 8..6+NUM_TOKENS []  Investor Token Accounts
    /// 6+NUM_TOKENS.. 6+2*NUM_TOKENS  Fund Token Accounts
    /// 6+2*NUM_TOKENS..6+4*NUM_TOKENS Pool Token Accounts for each pair
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
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct Data {
    pub instr: u8,
    pub amount_in: u64,
    pub min_amount_out: u64
}