//! Program entrypoint for runtime verification proofs of original spl token implmentation

use {
    crate::processor::Processor,
    solana_account_info::AccountInfo,
    solana_program_error::{ProgramResult},
    solana_pubkey::Pubkey,
    spl_token_interface::error::TokenError,
};

solana_program_entrypoint::entrypoint!(process_instruction);

/// Process an instruction, edited to call RV proof harnesses
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let result = inner_process_instruction(program_id, accounts, instruction_data);

    if let Err(ref _error) = result {
        // Log the error
        // msg!(_error.to_str::<TokenError>()); // Removing for less dependencies
    }

    result
}

/// A runtime verification cheatcode to set the instruction discriminator.
/// TODO: Currently calling assert for concrete testing but needs backend support in K.
fn cheatcode_set_descriminator(discriminator: u8, instruction_data: &[u8]) {
    assert_eq!(discriminator, instruction_data[0]);
}

/// Inner instruction processor that dispatches to proof harnesses
fn inner_process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let [discriminator, _rest @ ..] = instruction_data else {
        return Err(TokenError::InvalidInstruction.into());
    };

    match *discriminator {
        // 3 - Transfer
        3 => {
            test_process_transfer(
                program_id,
                // accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?,
                accounts, // TODO Daniel: Change type
                instruction_data.first_chunk().ok_or(TokenError::InvalidInstruction)?,
            )
        }
        // For all other instructions, just call the regular processor
        _ => {
            Processor::process(program_id, accounts, instruction_data)
        }
    }
}

/// program_id               // Token Program ID
/// accounts[0]              // Source Info
/// accounts[1]              // Destination Info
/// accounts[2]              // Authority Info
/// instruction_data[0]      // Discriminator 3 (Transfer)
/// instruction_data[1..9]   // Little Endian Bytes of u64 amount
#[inline(never)]
fn test_process_transfer(
    program_id: &Pubkey,
    // accounts: &[AccountInfo; 3],
    accounts: &[AccountInfo], // TODO Daniel: Change type
    instruction_data: &[u8; 9],
) -> ProgramResult {
    // Set descriminator and program id to concrete value
    cheatcode_set_descriminator(3, instruction_data);
    // cheatcode_set_program_id(program_id);

    // Strip discriminator so instruction data is equivalent p-token harness
    let instruction_data_with_discriminator = &instruction_data.clone();
    let instruction_data: &[u8; 8] = instruction_data.last_chunk().unwrap();

    //-Initial State-----------------------------------------------------------

    //-Process Instruction-----------------------------------------------------
    let result = Processor::process(program_id, accounts, instruction_data_with_discriminator);

    //-Assert Postconditions---------------------------------------------------

    // Ensure instruction_data was not mutated
    assert_eq!(*instruction_data, instruction_data_with_discriminator[1..]);

    result
}
