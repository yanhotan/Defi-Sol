// smart-contracts/src/lib.rs
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

mod error;
mod instructions;
mod processor;
mod state;
mod utils;

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("SOL Stake-Lend protocol instruction");
    
    if let Err(error) = instructions::process_instruction(program_id, accounts, instruction_data) {
        // Program errors get logged automatically
        return Err(error);
    }
    
    Ok(())
}