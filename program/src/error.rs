use num_derive::FromPrimitive;
use solana_program::{decode_error::DecodeError, program_error::ProgramError};
use thiserror::Error;

/// Errors that may be returned by the Token vesting program.
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum BonfidaBotError {
    // Invalid instruction
    #[error("Invalid Instruction")]
    InvalidInstruction,
    #[error("Arithmetic operation overflow")]
    Overflow,
    #[error("Operation is locked in the current pool state")]
    LockedOperation,
    #[error("Not enough FIDA in account.")]
    NotEnoughFIDA,
    #[error("Operation too small.")]
    OperationTooSmall,
}

impl From<BonfidaBotError> for ProgramError {
    fn from(e: BonfidaBotError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for BonfidaBotError {
    fn type_of() -> &'static str {
        "BonfidaBotError"
    }
}
