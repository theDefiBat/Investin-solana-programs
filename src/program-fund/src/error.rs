
use bytemuck::Contiguous;
use num_enum::IntoPrimitive;
use num_derive::FromPrimitive;
use solana_program::program_error::ProgramError;
use thiserror::Error;

pub type FundResult<T = ()> = Result<T, FundError>;


#[derive(Error, Debug, PartialEq, Eq)]
pub enum FundError {
    #[error(transparent)]
    ProgramError(#[from] ProgramError),
    #[error("{fund_error_code}; line:{line}")]
    FundErrorCode { fund_error_code: FundErrorCode, line: u32 },
}
//#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq, IntoPrimitive)]
#[repr(u32)]
pub enum FundErrorCode {

    #[error("FundAccount is Already Initialised")]
    FundAccountAlreadyInit,

    #[error("InvestorAccount is Already Initialised")]
    InvestorAccountAlreadyInit,

    #[error("Invorrect signature")]
    IncorrectSignature,

    #[error("Incorrect program id passed")]
    IncorrectProgramId,

    #[error("Incorrect PDA passed")]
    IncorrectPDA,

    #[error("Invalid Token Accounts passed")]
    InvalidTokenAccount,

    /// Invalid instruction
    #[error("Invalid Instruction")]
    InvalidInstruction,

    /// Amount less than minimum Amount
    #[error("Amount less than minimum amount")]
    InvalidAmount,

    /// Investor Mismatch
    #[error("Investor Mismatch")]
    InvestorMismatch,

    /// Manager Mismatch
    #[error("Manager Mismatch")]
    ManagerMismatch,
}

impl From<FundError> for ProgramError {
    fn from(e: FundError) -> Self {
        match e {
            FundError::ProgramError(pe) => pe,
            FundError::FundErrorCode {
                fund_error_code,
                line: _
            } => ProgramError::Custom(fund_error_code.into()),
        }
    }
}

#[inline]
#[inline]
pub fn check_assert(
    cond: bool,
    fund_error_code:
    FundErrorCode,
    line: u32,
) -> FundResult<()> {
    if cond {
        Ok(())
    } else {
        Err(FundError::FundErrorCode { fund_error_code, line })
    }
}
