use crate::id;

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum FundInstruction {
    /// Accounts expected
    /// 0. [WRITE] Fund Account
    /// 1. [SIGNER] Investor Account

    DepositLamports {
        amount: u64
    },

    WithdrawLamports {
        amount: u64
    }
}