// use crate::id;

// #[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum FundInstruction {
    /// Accounts expected
    /// 0. [WRITE] Fund Account
    /// 1. [SIGNER] Investor Account
    /// 2. [] Fund Token Account
    /// 3. [] Investor Token Account
    /// 3. [] Token Program

    DepositLamports {
        amount: u64
    },

    /// Accounts expected
    /// 0. [WRITE] Fund Account
    /// 1. [SIGNER] Investor Account
    /// 2. [] Fund Token Account
    /// 3. [] Investor Token Account
    /// 3. [] Token Program

    WithdrawLamports {
        amount: u64
    }
}