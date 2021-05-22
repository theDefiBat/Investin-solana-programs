use num_derive::FromPrimitive;
use solana_program::program_error::ProgramError;
use thiserror::Error;

pub type FundResult<T = ()> = Result<T, FundError>;

#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum FundError {
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
        ProgramError::Custom(e as u32)
    }
}

#[inline]
pub fn check_assert(
    cond: bool,
    error: ProgramError
) -> Result<(), ProgramError> {
    if cond {
        Ok(())
    } else {
        Err(error)
    }
}
