use num_derive::FromPrimitive;
use solana_program::program_error::ProgramError;
use thiserror::Error;


#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum FundError {
    //0
    #[error("FundAccount is Already Initialised")]
    FundAccountAlreadyInit,

    //1
    #[error("InvestorAccount is Already Initialised")]
    InvestorAccountAlreadyInit,

    //2
    #[error("Invorrect signature")]
    IncorrectSignature,

    //3
    #[error("Incorrect program id passed")]
    IncorrectProgramId,

    //4
    #[error("Incorrect PDA passed")]
    IncorrectPDA,

    //5
    #[error("Invalid Token Accounts passed")]
    InvalidTokenAccount,

    //6
    #[error("Invalid State Accounts passed")]
    InvalidStateAccount,

    ///7 Invalid instruction
    #[error("Invalid Instruction")]
    InvalidInstruction,

    ///8 Amount less than minimum Amount
    #[error("Amount less than minimum amount")]
    InvalidAmount,

    ///9 Investor Mismatch
    #[error("Investor Mismatch")]
    InvestorMismatch,

    ///10 Manager Mismatch
    #[error("Manager Mismatch")]
    ManagerMismatch,

    ///11 Maximum Number of Depositors at a time reached
    #[error("Wait for Manager Transfer")]
    DepositLimitReached,

    //12
    #[error("Stale price in account")]
    PriceStaleInAccount,

    //13
    #[error("Invalid Margin Instruction State")]
    InvalidMangoState,

    //14
    #[error("Default Error")]
    Default
}

impl From<FundError> for ProgramError {
    fn from(e: FundError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
