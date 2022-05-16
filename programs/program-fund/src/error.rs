use flux_aggregator::error;
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

    ///a Manager Mismatch
    #[error("Manager Mismatch")]
    ManagerMismatch,

    ///b Maximum Number of Depositors at a time reached
    #[error("Wait for Manager Transfer")]
    DepositLimitReached,

    //c
    #[error("Stale price in account")]
    PriceStaleInAccount,

    //d
    #[error("Invalid Mango Instruction State")]
    InvalidMangoState,

    //e
    #[error("Mango Account Not Initialized")]
    MangoNotInitialized,

    //f
    #[error("Default Error")]
    Default,

    //10
    #[error("Fund is Private")]
    PrivateFund,

    //11
    #[error("minAmountOut invalidated")]
    MinAmountFailed,

     //12
     #[error("limit Still in Processing")]
     LimitOrderProcessing,
     
     //13
     #[error("Claim pending Deposit on Friktion")]
     UnclaimedPendingDeposit,
     //13
     #[error("Claim pending Withdrawal on Friktion")]
     UnclaimedPendingwithdrawal,

     #[error("Incorrect Friktion Vault")]
     IncorrectFriktionVault,

     #[error("Incorrect Friktion Vault")]
     InvestorIndexError

}

impl From<FundError> for ProgramError {
    fn from(e: FundError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
