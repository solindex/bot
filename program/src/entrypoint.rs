use num_traits::FromPrimitive;
use solana_program::{
    account_info::AccountInfo, decode_error::DecodeError, entrypoint, entrypoint::ProgramResult,
    msg, program_error::PrintProgramError, pubkey::Pubkey,
};

use crate::{error::BonfidaBotError, processor::Processor};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Entrypoint");
    if let Err(error) = Processor::process_instruction(program_id, accounts, instruction_data) {
        // catch the error so we can print it
        error.print::<BonfidaBotError>();
        return Err(error);
    }
    Ok(())
}

impl PrintProgramError for BonfidaBotError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            BonfidaBotError::InvalidInstruction => msg!("Error: Invalid instruction!"),
            BonfidaBotError::Overflow => msg!("Error: Arithmetic operation overflow!"),
            BonfidaBotError::LockedOperation => msg!("Error: Operation is locked in the current pool state!"),
            BonfidaBotError::NotEnoughFIDA => msg!("Error: Pool must contain a minimum amount of FIDA tokens"),
            BonfidaBotError::OperationTooSmall => msg!("Error: Operation was too small")
        }
    }
}
