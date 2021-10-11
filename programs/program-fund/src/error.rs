use num_derive::FromPrimitive;
use solana_program::program_error::ProgramError;
use thiserror::Error;


#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum FundError {

    #[error("IVN || FundAccount is Already Initialised")]
    FundAccountAlreadyInit,

    #[error("IVN || InvestorAccount is Already Initialised")]
    InvestorAccountAlreadyInit,

    #[error("IVN || Invorrect signature")]
    IncorrectSignature,

    #[error("IVN || Incorrect program id passed")]
    IncorrectProgramId,

    #[error("IVN || Incorrect PDA passed")]
    IncorrectPDA,

    #[error("IVN || Invalid Token Accounts passed")]
    InvalidTokenAccount,

    #[error("IVN || Invalid State Accounts passed")]
    InvalidStateAccount,

    /// Invalid instruction
    #[error("IVN || Invalid Instruction")]
    InvalidInstruction,

    /// Amount less than minimum Amount
    #[error("IVN || Amount less than minimum amount")]
    InvalidAmount,

    /// Investor Mismatch
    #[error("IVN || Investor Mismatch")]
    InvestorMismatch,

    /// Manager Mismatch
    #[error("IVN || Manager Mismatch")]
    ManagerMismatch,

    /// Maximum Number of Depositors at a time reached
    #[error("IVN || Wait for Manager Transfer")]
    DepositLimitReached,

    #[error("IVN || Stale price in account")]
    PriceStaleInAccount,

    #[error("IVN || Invalid Margin Instruction State")]
    InvalidMangoState,

    #[error("IVN || Invalid Index passed")]
    InvalidIndex,

    #[error("IVN || Default Error")]
    Default
}

impl From<FundError> for ProgramError {
    fn from(e: FundError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
