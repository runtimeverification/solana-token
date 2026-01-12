// The harnesses have blocks with the same error conidition
// beside each other which clippy doesn't like, but that is
// but that is preferable for clarity currently.
#![allow(clippy::if_same_then_else)]
// Code that is guarded from arithmetic overflow in both the
//  harness logic and by K protecting from UB is flagged
// by clippy
#![allow(clippy::arithmetic_side_effects)]
// Also note that there are some other inlined clippy bypasses
// in the harnesses that should be acknowledged

use {
    crate::processor::*,
    pinocchio::{
        account_info::AccountInfo,
        no_allocator, nostd_panic_handler, program_entrypoint,
        program_error::{ProgramError, ToStr},
        pubkey::Pubkey,
        sysvars::Sysvar,
        ProgramResult,
    },
    pinocchio_token_interface::{
        error::TokenError,
        program::ID as PROGRAM_ID,
        state::{account_state::AccountState, Initializable, Transmutable},
    },
};

/// Macros to abstract API differences between spl-token and p-token.
/// spl-token AccountInfo has fields (.key, .owner), p-token has methods
/// (.key(), .owner()). spl-token wrappers have methods (.mint(), .decimals()),
/// p-token has fields (.mint, .decimals).
macro_rules! key {
    ($acc:expr) => {
        $acc.key()
    };
}
macro_rules! owner {
    ($acc:expr) => {
        $acc.owner()
    };
}
macro_rules! mint {
    ($acc:expr) => {
        $acc.mint
    };
}
macro_rules! decimals {
    ($m:expr) => {
        $m.decimals
    };
}
macro_rules! account_owner {
    ($acc:expr) => {
        $acc.owner
    };
}
/// Cheatcode macros to abstract naming differences.
macro_rules! cheatcode_mint {
    ($acc:expr) => {
        cheatcode_is_mint($acc)
    };
}
macro_rules! cheatcode_account {
    ($acc:expr) => {
        cheatcode_is_account($acc)
    };
}
/// Process call macro to abstract the processor call difference.
/// p-token calls process_mint_to_checked directly.
macro_rules! call_process_mint_to_checked {
    ($accounts:expr, $instruction_data:expr) => {
        process_mint_to_checked($accounts, $instruction_data)
    };
}
macro_rules! call_process_approve {
    ($accounts:expr, $instruction_data:expr) => {
        process_approve($accounts, $instruction_data)
    };
}
macro_rules! cheatcode_multisig {
    ($acc:expr) => {
        cheatcode_is_multisig($acc)
    };
}
macro_rules! call_process_revoke {
    ($accounts:expr) => {
        process_revoke($accounts)
    };
}
macro_rules! call_process_transfer {
    ($accounts:expr, $instruction_data:expr) => {
        process_transfer($accounts, $instruction_data)
    };
}
macro_rules! call_process_mint_to {
    ($accounts:expr, $instruction_data:expr) => {
        process_mint_to($accounts, $instruction_data)
    };
}
macro_rules! call_process_burn {
    ($accounts:expr, $instruction_data:expr) => {
        process_burn($accounts, $instruction_data)
    };
}
/// Macros for inner_test_validate_owner
macro_rules! is_signer {
    ($acc:expr) => {
        $acc.is_signer()
    };
}
macro_rules! is_owned_by_program {
    ($acc:expr) => {
        $acc.is_owned_by(&pinocchio_token_interface::program::ID)
    };
}
macro_rules! multisig_signers {
    ($multisig:expr) => {
        $multisig.signers
    };
}
macro_rules! multisig_m {
    ($multisig:expr) => {
        $multisig.m
    };
}
macro_rules! multisig_n {
    ($multisig:expr) => {
        $multisig.n
    };
}
macro_rules! is_valid_signer_index {
    ($n:expr) => {
        Multisig::is_valid_signer_index($n)
    };
}
macro_rules! call_process_initialize_multisig {
    ($accounts:expr, $instruction_data:expr) => {
        process_initialize_multisig($accounts, $instruction_data)
    };
}
macro_rules! call_process_initialize_multisig2 {
    ($accounts:expr, $instruction_data:expr) => {
        process_initialize_multisig2($accounts, $instruction_data)
    };
}
macro_rules! get_rent_sysvar {
    () => {
        pinocchio::sysvars::rent::Rent::get().unwrap()
    };
}
macro_rules! assert_pubkey_from_slice {
    ($actual:expr, $slice:expr) => {{
        assert_eq!($actual, $slice);
    }};
}
macro_rules! call_process_set_authority {
    ($accounts:expr, $instruction_data:expr) => {
        process_set_authority($accounts, $instruction_data)
    };
}
/// Macros for test_process_initialize_mint_freeze
macro_rules! cheatcode_rent {
    ($acc:expr) => {
        cheatcode_is_rent($acc)
    };
}
macro_rules! rent_id {
    () => {
        &pinocchio::sysvars::rent::RENT_ID
    };
}
macro_rules! mint_decimals {
    ($mint:expr) => {
        $mint.decimals
    };
}
macro_rules! assert_mint_authority {
    ($mint:expr, $slice:expr) => {
        assert_eq!($mint.mint_authority().unwrap(), $slice);
    };
}
macro_rules! assert_freeze_authority {
    ($mint:expr, $slice:expr) => {
        assert_eq!($mint.freeze_authority().unwrap(), $slice);
    };
}
macro_rules! call_process_initialize_mint {
    ($accounts:expr, $instruction_data:expr) => {
        process_initialize_mint($accounts, $instruction_data)
    };
}
macro_rules! call_process_initialize_mint_no_freeze {
    ($accounts:expr, $instruction_data:expr) => {
        process_initialize_mint($accounts, $instruction_data)
    };
}
macro_rules! call_process_initialize_mint2 {
    ($accounts:expr, $instruction_data:expr) => {
        process_initialize_mint2($accounts, $instruction_data)
    };
}
macro_rules! call_process_initialize_mint2_no_freeze {
    ($accounts:expr, $instruction_data:expr) => {
        process_initialize_mint2($accounts, $instruction_data)
    };
}
/// Macros for test_process_initialize_account
macro_rules! native_mint_id {
    () => {
        &pinocchio_token_interface::native_mint::ID
    };
}
macro_rules! program_id {
    () => {
        &pinocchio_token_interface::program::ID
    };
}
macro_rules! account_info_owner {
    ($account:expr) => {
        $account.owner()
    };
}
macro_rules! call_process_amount_to_ui_amount {
    ($accounts:expr, $instruction_data:expr) => {
        process_amount_to_ui_amount($accounts, $instruction_data)
    };
}
macro_rules! call_process_ui_amount_to_amount {
    ($accounts:expr, $instruction_data:expr) => {
        process_ui_amount_to_amount($accounts, $instruction_data)
    };
}
macro_rules! call_process_transfer_checked {
    ($accounts:expr, $instruction_data:expr) => {
        process_transfer_checked($accounts, $instruction_data)
    };
}
macro_rules! call_process_burn_checked {
    ($accounts:expr, $instruction_data:expr) => {
        process_burn_checked($accounts, $instruction_data)
    };
}
macro_rules! accounts_equal {
    ($a:expr, $b:expr) => {
        $a == $b
    };
}
macro_rules! account_state_uninitialized {
    () => {
        pinocchio_token_interface::state::account_state::AccountState::Uninitialized
    };
}
macro_rules! account_state_initialized {
    () => {
        pinocchio_token_interface::state::account_state::AccountState::Initialized
    };
}
macro_rules! call_process_initialize_account {
    ($accounts:expr) => {
        process_initialize_account($accounts)
    };
}
/// Macros for test_process_initialize_account2/3
macro_rules! call_process_initialize_account2 {
    ($accounts:expr, $instruction_data:expr) => {
        process_initialize_account2($accounts, $instruction_data)
    };
}
macro_rules! call_process_initialize_account3 {
    ($accounts:expr, $instruction_data:expr) => {
        process_initialize_account3($accounts, $instruction_data)
    };
}
macro_rules! pubkey_bytes {
    () => {
        pinocchio::pubkey::PUBKEY_BYTES
    };
}
macro_rules! assert_account_owner_eq_bytes {
    ($acc:expr, $bytes:expr) => {
        assert_eq!($acc.owner, *$bytes);
    };
}
macro_rules! assert_account_mint_eq_key {
    ($acc:expr, $key:expr) => {
        assert_eq!($acc.mint, *$key);
    };
}
/// Macros for test_process_transfer
macro_rules! account_state_frozen {
    () => {
        pinocchio_token_interface::state::account_state::AccountState::Frozen
    };
}
macro_rules! accounts_eq {
    ($a:expr, $b:expr) => {
        $a == $b
    };
}
macro_rules! call_process_transfer_inner {
    ($accounts:expr, $instruction_data:expr) => {
        process_transfer($accounts, $instruction_data)
    };
}
macro_rules! account_mint {
    ($acc:expr) => {
        $acc.mint
    };
}
macro_rules! owner_is_program {
    ($acc:expr) => {
        $acc.owner() == &pinocchio_token_interface::program::ID
    };
}
macro_rules! call_process_close_account {
    ($accounts:expr) => {
        process_close_account($accounts)
    };
}
macro_rules! incinerator_id {
    () => {
        pinocchio_token_interface::state::account::INCINERATOR_ID
    };
}
macro_rules! call_process_freeze_account {
    ($accounts:expr) => {
        process_freeze_account($accounts)
    };
}
macro_rules! call_process_thaw_account {
    ($accounts:expr) => {
        process_thaw_account($accounts)
    };
}
macro_rules! call_process_sync_native {
    ($accounts:expr) => {
        process_sync_native($accounts)
    };
}
macro_rules! call_process_get_account_data_size {
    ($accounts:expr) => {
        process_get_account_data_size($accounts)
    };
}
macro_rules! call_process_initialize_immutable_owner {
    ($accounts:expr) => {
        process_initialize_immutable_owner($accounts)
    };
}
macro_rules! call_process_approve_checked {
    ($accounts:expr, $instruction_data:expr) => {
        process_approve_checked($accounts, $instruction_data)
    };
}

program_entrypoint!(process_instruction);
// Do not allocate memory.
no_allocator!();
// Use the no_std panic handler.
nostd_panic_handler!();

/// Log an error.
#[cold]
fn log_error(error: &ProgramError) {
    pinocchio::log::sol_log(error.to_str::<TokenError>());
}

/// Process an instruction.
///
/// In the first stage, the entrypoint checks the discriminator of the
/// instruction data to determine whether the instruction is a "batch"
/// instruction or a "regular" instruction. This avoids nesting of "batch"
/// instructions, since it is not sound to have a "batch" instruction inside
/// another "batch" instruction.
#[inline(always)]
pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let [discriminator, remaining @ ..] = instruction_data else {
        return Err(TokenError::InvalidInstruction.into());
    };

    let result = if *discriminator == 255 {
        // 255 - Batch
        #[cfg(feature = "logging")]
        pinocchio::msg!("Instruction: Batch");

        process_batch(accounts, remaining)
    } else {
        inner_process_instruction(accounts, instruction_data)
    };

    result.inspect_err(log_error)
}

/// Process a "regular" instruction.
///
/// The processor of the token program is divided into two parts to reduce the
/// overhead of having a large `match` statement. The first part of the
/// processor handles the most common instructions, while the second part
/// handles the remaining instructions.
///
/// The rationale is to reduce the overhead of making multiple comparisons for
/// popular instructions.
///
/// Instructions on the first part of the inner processor:
///
/// - `0`: `InitializeMint`
/// - `1`: `InitializeAccount`
/// - `3`: `Transfer`
/// - `7`: `MintTo`
/// - `9`: `CloseAccount`
/// - `16`: `InitializeAccount2`
/// - `18`: `InitializeAccount3`
/// - `20`: `InitializeMint2`
#[inline(always)]
pub(crate) fn inner_process_instruction(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    use pinocchio_token_interface::program::ID;

    let [discriminator, instruction_data @ ..] = instruction_data else {
        return Err(TokenError::InvalidInstruction.into());
    };

    match *discriminator {
        // 0 - Test InitializeMint
        0 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Testing Instruction: InitializeMint");

            match instruction_data.len() {
                x if 66 <= x => test_process_initialize_mint_freeze(
                    accounts.first_chunk().unwrap(),
                    instruction_data.first_chunk().unwrap(),
                ),
                x if 34 <= x => test_process_initialize_mint_no_freeze(
                    accounts.first_chunk().unwrap(),
                    instruction_data.first_chunk().unwrap(),
                ),
                _ => panic!("Invalid instruction data length"),
            }
        }
        // 1 - Test InitializeAccount
        1 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Testing Instruction: InitializeAccount");

            test_process_initialize_account(accounts.first_chunk().unwrap())
        }
        // 3 - Test Transfer
        3 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Testing Instruction: Transfer");

            // TODO: Thoroughly test for insufficient account length
            // We should be calling `insufficient_accounts_length_transfer(x)?` in the
            // future
            match accounts.len() {
                x if accounts.len() < 3 => panic!("Invalid amount of accounts for transfer: {x}"),
                _ => (),
            }

            match accounts[2].data_len() {
                Multisig::LEN if accounts[2].is_owned_by(&ID) => test_process_transfer_multisig(
                    accounts.first_chunk().unwrap(),
                    instruction_data.first_chunk().unwrap(),
                ),
                _ => test_process_transfer(
                    accounts.first_chunk().unwrap(),
                    instruction_data.first_chunk().unwrap(),
                ),
            }
        }
        // 7 - MintTo
        7 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: MintTo");

            // TODO: Thoroughly test for insufficient account length
            // We should be calling `insufficient_accounts_length_mint(x)?` in the future
            match accounts.len() {
                x if accounts.len() < 3 => panic!("Invalid amount of accounts for mint: {x}"),
                _ => (),
            }

            match accounts[2].data_len() {
                Multisig::LEN if accounts[2].is_owned_by(&ID) => test_process_mint_to_multisig(
                    accounts.first_chunk().unwrap(),
                    instruction_data.first_chunk().unwrap(),
                ),
                _ => test_process_mint_to(
                    accounts.first_chunk().unwrap(),
                    instruction_data.first_chunk().unwrap(),
                ),
            }
        }
        // 8 - Test Burn
        8 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Testing Instruction: Burn");

            // TODO: Thoroughly test for insufficient account length
            // We should be calling `insufficient_accounts_length_burn(x)?` in the future
            match accounts.len() {
                x if accounts.len() < 3 => panic!("Invalid amount of accounts for burn: {x}"),
                _ => (),
            }

            match accounts[2].data_len() {
                Multisig::LEN if accounts[2].is_owned_by(&ID) => test_process_burn_multisig(
                    accounts.first_chunk().unwrap(),
                    instruction_data.first_chunk().unwrap(),
                ),
                _ => test_process_burn(
                    accounts.first_chunk().unwrap(),
                    instruction_data.first_chunk().unwrap(),
                ),
            }
        }
        // 9 - Test CloseAccount
        9 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Testing Instruction: CloseAccount");

            // TODO: Thoroughly test for insufficient account length
            // We should be calling `insufficient_accounts_length_close_account(x)?` in the
            // future
            match accounts.len() {
                x if accounts.len() < 3 => {
                    panic!("Invalid amount of accounts for close_account: {x}")
                }
                _ => (),
            }

            match accounts[2].data_len() {
                Multisig::LEN if accounts[2].is_owned_by(&ID) => {
                    test_process_close_account_multisig(accounts.first_chunk().unwrap())
                }
                _ => test_process_close_account(accounts.first_chunk().unwrap()),
            }
        }
        // 12 - Test TransferChecked
        12 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Testing Instruction: TransferChecked");

            // TODO: Thoroughly test for insufficient account length
            // We should be calling `insufficient_accounts_length_transfer_checked(x)?` in
            // the future
            match accounts.len() {
                x if accounts.len() < 4 => {
                    panic!("Invalid amount of accounts for transfer_checked: {x}")
                }
                _ => (),
            }

            match accounts[3].data_len() {
                Multisig::LEN if accounts[2].is_owned_by(&ID) => {
                    test_process_transfer_checked_multisig(
                        accounts.first_chunk().unwrap(),
                        instruction_data.first_chunk().unwrap(),
                    )
                }
                _ => test_process_transfer_checked(
                    accounts.first_chunk().unwrap(),
                    instruction_data.first_chunk().unwrap(),
                ),
            }
        }
        // 15 - Test BurnChecked
        15 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Testing Instruction: BurnChecked");

            // TODO: Thoroughly test for insufficient account length
            // We should be calling `insufficient_accounts_length_burn_checked(x)?` in the
            // future
            match accounts.len() {
                x if accounts.len() < 3 => {
                    panic!("Invalid amount of accounts for burn_checked: {x}")
                }
                _ => (),
            }

            match accounts[2].data_len() {
                Multisig::LEN if accounts[2].is_owned_by(&ID) => {
                    test_process_burn_checked_multisig(
                        accounts.first_chunk().unwrap(),
                        instruction_data.first_chunk().unwrap(),
                    )
                }
                _ => test_process_burn_checked(
                    accounts.first_chunk().unwrap(),
                    instruction_data.first_chunk().unwrap(),
                ),
            }
        }
        // 16 - Test InitializeAccount2
        16 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Testing Instruction: InitializeAccount2");

            test_process_initialize_account2(
                accounts.first_chunk().unwrap(),
                instruction_data.first_chunk().unwrap(),
            )
        }
        // 18 - Test InitializeAccount3
        18 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Testing Instruction: InitializeAccount3");

            test_process_initialize_account3(
                accounts.first_chunk().unwrap(),
                instruction_data.first_chunk().unwrap(),
            )
        }
        // 20 - Test InitializeMint2
        20 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Testing Instruction: InitializeMint2");

            match instruction_data.len() {
                x if 66 <= x => test_process_initialize_mint2_freeze(
                    accounts.first_chunk().unwrap(),
                    instruction_data.first_chunk().unwrap(),
                ),
                x if 34 <= x => test_process_initialize_mint2_no_freeze(
                    accounts.first_chunk().unwrap(),
                    instruction_data.first_chunk().unwrap(),
                ),
                _ => panic!("Invalid instruction data length"),
            }
        }
        d => inner_process_remaining_instruction(accounts, instruction_data, d),
    }
}

/// Process a remaining "regular" instruction.
///
/// This function is called by the [`inner_process_instruction`] function if the
/// discriminator does not match any of the common instructions. This function
/// is used to reduce the overhead of having a large `match` statement in the
/// [`inner_process_instruction`] function.
fn inner_process_remaining_instruction(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
    discriminator: u8,
) -> ProgramResult {
    use pinocchio_token_interface::program::ID;

    match discriminator {
        // 2 - InitializeMultisig
        2 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: InitializeMultisig");

            test_process_initialize_multisig(
                accounts.first_chunk().unwrap(),
                instruction_data.first_chunk().unwrap(),
            )
        }
        // 4 - Approve
        4 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: Approve");

            // TODO: Thoroughly test for insufficient account length
            // We should be calling `insufficient_accounts_length_approve(x)?` in the future
            match accounts.len() {
                x if accounts.len() < 3 => panic!("Invalid amount of accounts for approve: {x}"),
                _ => (),
            }

            match accounts[2].data_len() {
                Multisig::LEN if accounts[2].is_owned_by(&ID) => test_process_approve_multisig(
                    accounts.first_chunk().unwrap(),
                    instruction_data.first_chunk().unwrap(),
                ),
                _ => test_process_approve(
                    accounts.first_chunk().unwrap(),
                    instruction_data.first_chunk().unwrap(),
                ),
            }
        }
        // 5 - Revoke
        5 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: Revoke");

            // TODO: Thoroughly test for insufficient account length
            // We should be calling `insufficient_accounts_length_revoke(x)?` in the future
            match accounts.len() {
                x if accounts.len() < 2 => panic!("Invalid amount of accounts for revoke: {x}"),
                _ => (),
            }

            match accounts[1].data_len() {
                Multisig::LEN if accounts[1].is_owned_by(&ID) => {
                    test_process_revoke_multisig(accounts.first_chunk().unwrap())
                }
                _ => test_process_revoke(accounts.first_chunk().unwrap()),
            }
        }
        // 6 - SetAuthority
        6 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: SetAuthority");

            // TODO: Thoroughly test for insufficient account length
            // We should be calling `insufficient_accounts_length_revoke(x)?` in the future
            match accounts.len() {
                x if accounts.len() < 2 => {
                    panic!("Invalid amount of accounts for set_authority: {x}")
                }
                _ => (),
            }

            // Determine if this is an Account or Mint based on data length
            if let Some(first_account) = accounts.first() {
                match first_account.data_len() {
                    Account::LEN => match accounts[1].data_len() {
                        Multisig::LEN if accounts[1].is_owned_by(&ID) => {
                            test_process_set_authority_account_multisig(
                                accounts.first_chunk().unwrap(),
                                instruction_data.first_chunk().unwrap(),
                            )
                        }
                        _ => test_process_set_authority_account(
                            accounts.first_chunk().unwrap(),
                            instruction_data.first_chunk().unwrap(),
                        ),
                    },
                    Mint::LEN => match accounts[1].data_len() {
                        Multisig::LEN if accounts[1].is_owned_by(&ID) => {
                            test_process_set_authority_mint_multisig(
                                accounts.first_chunk().unwrap(),
                                instruction_data.first_chunk().unwrap(),
                            )
                        }
                        _ => test_process_set_authority_mint(
                            accounts.first_chunk().unwrap(),
                            instruction_data.first_chunk().unwrap(),
                        ),
                    },
                    // FIXME: Create proof harness for this
                    _ => panic!("SetAuthority: Unexpected account data length"),
                }
            } else {
                // FIXME: Create proof harness for this
                Err(ProgramError::NotEnoughAccountKeys)
            }
        }
        // 10 - FreezeAccount
        10 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: FreezeAccount");

            // TODO: Thoroughly test for insufficient account length
            // We should be calling `insufficient_accounts_length_freeze_account(x)?` in the
            // future
            match accounts.len() {
                x if accounts.len() < 3 => {
                    panic!("Invalid amount of accounts for freeze_account: {x}")
                }
                _ => (),
            }

            match accounts[2].data_len() {
                Multisig::LEN if accounts[2].is_owned_by(&ID) => {
                    test_process_freeze_account_multisig(accounts.first_chunk().unwrap())
                }
                _ => test_process_freeze_account(accounts.first_chunk().unwrap()),
            }
        }
        // 11 - ThawAccount
        11 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: ThawAccount");

            // TODO: Thoroughly test for insufficient account length
            // We should be calling `insufficient_accounts_length_thaw_account(x)?` in the
            // future
            match accounts.len() {
                x if accounts.len() < 3 => {
                    panic!("Invalid amount of accounts for thaw_account: {x}")
                }
                _ => (),
            }

            match accounts[2].data_len() {
                Multisig::LEN if accounts[2].is_owned_by(&ID) => {
                    test_process_thaw_account_multisig(accounts.first_chunk().unwrap())
                }
                _ => test_process_thaw_account(accounts.first_chunk().unwrap()),
            }
        }
        // 13 - ApproveChecked
        13 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: ApproveChecked");

            // TODO: Thoroughly test for insufficient account length
            // We should be calling `insufficient_accounts_length_approve_checked(x)?` in
            // the future
            match accounts.len() {
                x if accounts.len() < 4 => {
                    panic!("Invalid amount of accounts for approve_checked: {x}")
                }
                _ => (),
            }

            match accounts[3].data_len() {
                Multisig::LEN if accounts[3].is_owned_by(&ID) => {
                    test_process_approve_checked_multisig(
                        accounts.first_chunk().unwrap(),
                        instruction_data.first_chunk().unwrap(),
                    )
                }
                _ => test_process_approve_checked(
                    accounts.first_chunk().unwrap(),
                    instruction_data.first_chunk().unwrap(),
                ),
            }
        }
        // 14 - MintToChecked
        14 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: MintToChecked");

            // TODO: Thoroughly test for insufficient account length
            // We should be calling `insufficient_accounts_length_mint_to_checked(x)?` in
            // the future
            match accounts.len() {
                x if accounts.len() < 3 => {
                    panic!("Invalid amount of accounts for mint_to_checked: {x}")
                }
                _ => (),
            }

            match accounts[2].data_len() {
                Multisig::LEN if accounts[2].is_owned_by(&ID) => {
                    test_process_mint_to_checked_multisig(
                        accounts.first_chunk().unwrap(),
                        instruction_data.first_chunk().unwrap(),
                    )
                }
                _ => test_process_mint_to_checked(
                    accounts.first_chunk().unwrap(),
                    instruction_data.first_chunk().unwrap(),
                ),
            }
        }
        // 17 - SyncNative
        17 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: SyncNative");

            test_process_sync_native(accounts.first_chunk().unwrap())
        }
        // 19 - InitializeMultisig2
        19 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: InitializeMultisig2");

            test_process_initialize_multisig2(
                accounts.first_chunk().unwrap(),
                instruction_data.first_chunk().unwrap(),
            )
        }
        // 21 - GetAccountDataSize
        21 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: GetAccountDataSize");

            test_process_get_account_data_size(accounts.first_chunk().unwrap())
        }
        // 22 - InitializeImmutableOwner
        22 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: InitializeImmutableOwner");

            test_process_initialize_immutable_owner(accounts.first_chunk().unwrap())
        }
        // 23 - AmountToUiAmount
        23 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: AmountToUiAmount");

            test_process_amount_to_ui_amount(
                accounts.first_chunk().unwrap(),
                instruction_data.first_chunk().unwrap(),
            )
        }
        // 24 - UiAmountToAmount
        24 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: UiAmountToAmount");

            test_process_ui_amount_to_amount(
                accounts.first_chunk().unwrap(),
                // instruction_data.first_chunk().unwrap(),
                instruction_data, // Sized won't work
            )
        }
        // 38 - WithdrawExcessLamports
        38 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: WithdrawExcessLamports");

            // TODO: Thoroughly test for insufficient account length
            // We should be calling `insufficient_accounts_length_mint_to_checked(x)?` in
            // the future
            match accounts.len() {
                x if accounts.len() < 3 => {
                    panic!("Invalid amount of accounts for withdraw_excess_lamports: {x}")
                }
                _ => (),
            }

            if let Some(acc) = accounts.first() {
                match acc.data_len() {
                    Account::LEN => match accounts[2].data_len() {
                        Multisig::LEN if accounts[2].is_owned_by(&ID) => {
                            test_process_withdraw_excess_lamports_account_multisig(
                                accounts.first_chunk().unwrap(),
                            )
                        }
                        _ => test_process_withdraw_excess_lamports_account(
                            accounts.first_chunk().unwrap(),
                        ),
                    },
                    Mint::LEN => match accounts[2].data_len() {
                        Multisig::LEN if accounts[2].is_owned_by(&ID) => {
                            test_process_withdraw_excess_lamports_mint_multisig(
                                accounts.first_chunk().unwrap(),
                            )
                        }
                        _ => test_process_withdraw_excess_lamports_mint(
                            accounts.first_chunk().unwrap(),
                        ),
                    },
                    Multisig::LEN => match accounts[2].data_len() {
                        Multisig::LEN if accounts[2].is_owned_by(&ID) => {
                            test_process_withdraw_excess_lamports_multisig_multisig(
                                accounts.first_chunk().unwrap(),
                            )
                        }
                        _ => test_process_withdraw_excess_lamports_multisig(
                            accounts.first_chunk().unwrap(),
                        ),
                    },
                    // FIXME: Need harness for this
                    _other => panic!("withdraw_excess_lamports: Unexpected account data_len"),
                }
            } else {
                // FIXME: need to add harness isntead since instruction still accepts this case
                // and has an error code
                panic!("withdraw_excess_lamports: no accounts provided")
            }
        }
        _ => Err(TokenError::InvalidInstruction.into()),
    }
}

// Cheatcodes to inject AccountInfo assumptions
#[inline(never)]
fn cheatcode_is_account(_: &AccountInfo) {}
#[inline(never)]
fn cheatcode_is_mint(_: &AccountInfo) {}
#[inline(never)]
fn cheatcode_is_multisig(_: &AccountInfo) {} // TODO: implement multisig cheatcode
#[inline(never)]
fn cheatcode_is_rent(_: &AccountInfo) {}

use {
    pinocchio::sysvars::rent::Rent,
    pinocchio_token_interface::state::{
        account::Account, load_mut_unchecked, load_unchecked, mint::Mint, multisig::Multisig,
    },
};

fn get_account(account_info: &AccountInfo) -> &Account {
    unsafe {
        let byte_ptr = account_info.borrow_data_unchecked();
        let acc_ref = load_unchecked::<Account>(byte_ptr).unwrap();
        acc_ref
    }
}

fn get_mint(account_info: &AccountInfo) -> &Mint {
    unsafe {
        let byte_ptr = account_info.borrow_data_unchecked();
        let acc_ref = load_unchecked::<Mint>(byte_ptr).unwrap();
        acc_ref
    }
}

fn get_multisig(account_info: &AccountInfo) -> &Multisig {
    unsafe {
        let byte_ptr = account_info.borrow_data_unchecked();
        let multisig_ref = load_unchecked::<Multisig>(byte_ptr).unwrap();
        multisig_ref
    }
}

fn get_rent(account_info: &AccountInfo) -> &Rent {
    unsafe { Rent::from_bytes_unchecked(account_info.borrow_data_unchecked()) }
}

/// This function encapsulates the specification of validating the signature
/// requirements In particular, code from mod.rs::validate_owner is checked
include!("../../shared/inner_test_validate_owner.rs");

// wrapper to ensure the test below is in the SMIR JSON
#[no_mangle]
pub unsafe extern "C" fn use_tests(acc: &AccountInfo) {
    test_ptoken_domain_data(acc, acc, acc);
}

// special test for basic domain data access
#[inline(never)]
fn test_ptoken_domain_data(acc: &AccountInfo, mint: &AccountInfo, rent: &AccountInfo) {
    cheatcode_is_mint(mint);
    unsafe {
        let test = mint.borrow_mut_data_unchecked();
        let imint = load_mut_unchecked::<Mint>(test);
        let imint = imint.unwrap();
        imint.set_initialized();
    }
    let imint = get_mint(mint);
    assert!(imint.is_initialized().unwrap());

    cheatcode_is_account(acc);
    unsafe {
        let test = acc.borrow_mut_data_unchecked();
        let iacc: Result<&mut Account, _> = load_mut_unchecked(test);
        let iacc = iacc.unwrap();
        iacc.set_native(true);
    }
    let iacc = get_account(acc);
    assert!(iacc.is_native());

    let owner = acc.owner();
    assert!(acc.is_owned_by(owner));
    // QUESTION: is pinocchio::Account ever written to through AccountInfo?

    // test the system's Rent sysvar
    let sysrent = Rent::get().unwrap();
    let rent_collected = 10;
    let (burnt, distributed) = sysrent.calculate_burn(rent_collected);
    assert!(sysrent.burn_percent > 100 || burnt <= rent_collected && distributed <= rent_collected);

    cheatcode_is_rent(rent);
    let prent = unsafe {
        let test = rent.borrow_data_unchecked();
        Rent::from_bytes_unchecked(test)
    };
    // cannot call any functions that use f64 in any way
    // assume burn_percent value <=100 and calculate with it
    let rent_collected = 10;
    let (burnt, distributed) = prent.calculate_burn(rent_collected);
    assert!(prent.burn_percent > 100 || burnt <= rent_collected && distributed <= rent_collected);
}

// Hack Tests For Stable MIR JSON ---------------------------------------------
/// accounts[0] // Mint Info
/// accounts[1] // Rent Sysvar Info
/// instruction_data[0]      // Decimals
/// instruction_data[1..33]  // Mint Authority Pubkey
/// instruction_data[33]     // Freeze Authority Exists? 1 for freeze
/// instruction_data[34..66] // instruction_data[33] == 1 ==> Freeze Authority
/// Pubkey
include!("../../shared/test_process_initialize_mint_freeze.rs");

/// accounts[0] // Mint Info
/// accounts[1] // Rent Sysvar Info
/// instruction_data[0]      // Decimals
/// instruction_data[1..33]  // Mint Authority Pubkey
/// instruction_data[33]     // Freeze Authority Exists? 0 for no freeze
include!("../../shared/test_process_initialize_mint_no_freeze.rs");

/// accounts[0] // New Account Info
/// accounts[1] // Mint Info
/// accounts[2] // Owner Info
/// accounts[3] // Rent Sysvar Info
include!("../../shared/test_process_initialize_account.rs");

/// accounts[0] // Source Info
/// accounts[1] // Destination Info
/// accounts[2] // Authority Info
/// instruction_data[0..8] // Little Endian Bytes of u64 amount
include!("../../shared/test_process_transfer.rs");

/// accounts[0] // Source Info
/// accounts[1] // Destination Info
/// accounts[2] // Authority Info
/// accounts[3..14] // Signers
/// instruction_data[0..8] // Little Endian Bytes of u64 amount
include!("../../shared/test_process_transfer_multisig.rs");

/// accounts[0] // Mint Info
/// accounts[1] // Destination Info
/// accounts[2] // Owner Info
/// instruction_data[0..8] // Little Endian Bytes of u64 amount
include!("../../shared/test_process_mint_to.rs");

/// accounts[0] // Mint Info
/// accounts[1] // Destination Info
/// accounts[2] // Owner Info
/// accounts[3..14] // Signers
/// instruction_data[0..8] // Little Endian Bytes of u64 amount
include!("../../shared/test_process_mint_to_multisig.rs");

/// accounts[0] // Source Info
/// accounts[1] // Mint Info
/// accounts[2] // Authority Info
/// instruction_data[0..8] // Little Endian Bytes of u64 amount
include!("../../shared/test_process_burn.rs");

/// accounts[0] // Source Info
/// accounts[1] // Mint Info
/// accounts[2] // Authority Info
/// accounts[3..14] // Signers
/// instruction_data[0..8] // Little Endian Bytes of u64 amount
include!("../../shared/test_process_burn_multisig.rs");

/// accounts[0] // Source Info
/// accounts[1] // Destination Info
/// accounts[2] // Authority Info
include!("../../shared/test_process_close_account.rs");

/// accounts[0] // Source Info
/// accounts[1] // Destination Info
/// accounts[2] // Authority Info
/// accounts[3..14] // Multisig Signers
include!("../../shared/test_process_close_account_multisig.rs");

/// accounts[0] // Source Info
/// accounts[1] // Mint Info
/// accounts[2] // Destination Info
/// accounts[3] // Authority Info
/// instruction_data[0..9] // Little Endian Bytes of u64 amount, and decimals
include!("../../shared/test_process_transfer_checked.rs");

/// accounts[0] // Source Info
/// accounts[1] // Mint Info
/// accounts[2] // Destination Info
/// accounts[3] // Authority Info
/// accounts[4..15] // Signers
/// instruction_data[0..9] // Little Endian Bytes of u64 amount, and decimals
include!("../../shared/test_process_transfer_checked_multisig.rs");

/// accounts[0] // Source Info
/// accounts[1] // Mint Info
/// accounts[2] // Authority Info
/// instruction_data[0..9] // Little Endian Bytes of u64 amount, and decimals
include!("../../shared/test_process_burn_checked.rs");

/// accounts[0] // Source Info
/// accounts[1] // Mint Info
/// accounts[2] // Authority Info
/// accounts[3..14] // Signers
/// instruction_data[0..9] // Little Endian Bytes of u64 amount, and decimals
include!("../../shared/test_process_burn_checked_multisig.rs");

include!("../../shared/test_process_initialize_account2.rs");

include!("../../shared/test_process_initialize_account3.rs");

include!("../../shared/test_process_initialize_mint2_freeze.rs");

include!("../../shared/test_process_initialize_mint2_no_freeze.rs");

/// accounts[0]   // Multisig Info
/// accounts[1]   // Rent Sysvar Info
/// accounts[2..] // Signers
/// accounts[2..].len() // n
/// instruction_data[1] // m
include!("../../shared/test_process_initialize_multisig.rs");

include!("../../shared/test_process_approve.rs");

include!("../../shared/test_process_approve_multisig.rs");

include!("../../shared/test_process_revoke.rs");

include!("../../shared/test_process_revoke_multisig.rs");

/// accounts[0] // Account Info - Account Case
/// accounts[1] // Authority Info
/// instruction_data[0] // Authority Type (instruction)
/// instruction_data[1] // New Authority Follows (0 -> No, 1 -> Yes)
/// instruction_data[2..34] // New Authority Pubkey
include!("../../shared/test_process_set_authority_account.rs");

/// accounts[0] // Account Info - Account Case
/// accounts[1] // Authority Info
/// accounts[2..13] // Signers
/// instruction_data[0] // Authority Type (instruction)
/// instruction_data[1] // New Authority Follows (0 -> No, 1 -> Yes)
/// instruction_data[2..34] // New Authority Pubkey
include!("../../shared/test_process_set_authority_account_multisig.rs");

/// accounts[0] // Account Info - Mint Case
/// accounts[1] // Authority Info
/// instruction_data[0] // Authority Type (instruction)
include!("../../shared/test_process_set_authority_mint.rs");

include!("../../shared/test_process_set_authority_mint_multisig.rs");

/// accounts[0] // Source Account Info
/// accounts[1] // Mint Info
/// accounts[2] // Authority Info
include!("../../shared/test_process_freeze_account.rs");

/// accounts[0] // Source Account Info
/// accounts[1] // Mint Info
/// accounts[2] // Authority Info
/// accounts[3..13] // Signers
include!("../../shared/test_process_freeze_account_multisig.rs");

/// accounts[0] // Source Account Info
/// accounts[1] // Mint Info
/// accounts[2] // Authority Info
/// accounts[3..13] // Signers
include!("../../shared/test_process_thaw_account.rs");

/// accounts[0] // Source Account Info
/// accounts[1] // Mint Info
/// accounts[2] // Authority Info
/// accounts[3..13] // Signers
include!("../../shared/test_process_thaw_account_multisig.rs");

/// accounts[0] // Source Account Info
/// accounts[1] // Expected Mint Info
/// accounts[2] // Delegate Info
/// accounts[3] // Owner Info
/// instruction_data[0..9] // Little Endian Bytes of u64 amount, and decimals
include!("../../shared/test_process_approve_checked.rs");

/// accounts[0] // Source Account Info
/// accounts[1] // Expected Mint Info
/// accounts[2] // Delegate Info
/// accounts[3] // Owner Info
/// accounts[4..15] // Signers
/// instruction_data[0..9] // Little Endian Bytes of u64 amount, and decimals
include!("../../shared/test_process_approve_checked_multisig.rs");

include!("../../shared/test_process_mint_to_checked.rs");

/// accounts[0] // Mint Info
/// accounts[1] // Destination Info
/// accounts[2] // Owner Info
/// accounts[3..14] // Signers
/// instruction_data[0..9] // Little Endian Bytes of u64 amount, and decimals
include!("../../shared/test_process_mint_to_checked_multisig.rs");

include!("../../shared/test_process_sync_native.rs");

/// accounts[0]   // Multisig Info
/// accounts[1..] // Signers
/// accounts[1..].len() // n
/// instruction_data[1] // m
include!("../../shared/test_process_initialize_multisig2.rs");

/// accounts[0] // Mint Info
include!("../../shared/test_process_get_account_data_size.rs");

include!("../../shared/test_process_initialize_immutable_owner.rs");

include!("../../shared/test_process_amount_to_ui_amount.rs");

include!("../../shared/test_process_ui_amount_to_amount.rs");

/// accounts[0] // Source Account Info (Account)
/// accounts[1] // Destination Info
/// accounts[2] // Authority Info
#[inline(never)]
fn test_process_withdraw_excess_lamports_account(accounts: &[AccountInfo; 3]) -> ProgramResult {
    cheatcode_is_account(&accounts[0]); // Source Account
    cheatcode_is_account(&accounts[1]); // Destination
    cheatcode_is_account(&accounts[2]); // Excluding the multisig case

    //-Initial State-----------------------------------------------------------
    let src_old = get_account(&accounts[0]);
    let src_data_len = accounts[0].data_len();
    let src_account_initialised = src_old.is_initialized();
    let src_account_owner = src_old.owner;
    let src_account_is_native = src_old.is_native();
    let src_init_lamports = accounts[0].lamports();
    let dst_init_lamports = accounts[1].lamports();
    let maybe_multisig_is_initialised = None; // Value set to `None` since authority is an account

    // Note: Rent is a supported sysvar so ProgramError::UnsupportedSysvar should be
    // impossible
    let rent = pinocchio::sysvars::rent::Rent::get().unwrap();
    let minimum_balance = rent.minimum_balance(accounts[0].data_len());

    //-Process Instruction-----------------------------------------------------
    let result = process_withdraw_excess_lamports(accounts);

    //-Assert Postconditions---------------------------------------------------
    if accounts.len() < 3 {
        assert_eq!(result, Err(ProgramError::NotEnoughAccountKeys));
        return result;
    } else {
        assert_eq!(src_data_len, Account::LEN); // established by cheatcode_is_account
        {
            if src_account_initialised.is_err() {
                assert_eq!(result, Err(ProgramError::InvalidAccountData));
                return result;
            } else if !src_account_initialised.unwrap() {
                assert_eq!(result, Err(ProgramError::UninitializedAccount));
                return result;
            } else if src_account_is_native {
                assert_eq!(result, Err(ProgramError::Custom(10)));
                return result;
            }

            // Validate Owner
            inner_test_validate_owner(
                &src_account_owner, // expected_owner
                &accounts[2],       // owner_account_info
                &accounts[3..],     // tx_signers
                maybe_multisig_is_initialised,
                result.clone(),
            )?;

            if src_init_lamports < minimum_balance {
                assert_eq!(result, Err(ProgramError::Custom(0)));
                return result;
            } else if dst_init_lamports
                .checked_add(src_init_lamports - minimum_balance)
                .is_none()
            {
                assert_eq!(result, Err(ProgramError::Custom(14)));
                return result;
            }

            assert_eq!(accounts[0].lamports(), minimum_balance);
            assert_eq!(
                accounts[1].lamports(),
                dst_init_lamports + (src_init_lamports - minimum_balance)
            );
            assert!(result.is_ok())
        }
    }

    result
}

/// accounts[0] // Source Account Info (Account)
/// accounts[1] // Destination Info
/// accounts[2] // Authority Info
/// accounts[3..14] // Signers
#[inline(never)]
fn test_process_withdraw_excess_lamports_account_multisig(
    accounts: &[AccountInfo; 4],
) -> ProgramResult {
    cheatcode_is_account(&accounts[0]); // Source Account
    cheatcode_is_account(&accounts[1]); // Destination
    cheatcode_is_multisig(&accounts[2]); // Authority

    //-Initial State-----------------------------------------------------------
    let src_old = get_account(&accounts[0]);
    let src_data_len = accounts[0].data_len();
    let src_account_initialised = src_old.is_initialized();
    let src_account_owner = src_old.owner;
    let src_account_is_native = src_old.is_native();
    let src_init_lamports = accounts[0].lamports();
    let dst_init_lamports = accounts[1].lamports();
    let maybe_multisig_is_initialised = Some(get_multisig(&accounts[2]).is_initialized());

    // Note: Rent is a supported sysvar so ProgramError::UnsupportedSysvar should be
    // impossible
    let rent = pinocchio::sysvars::rent::Rent::get().unwrap();
    let minimum_balance = rent.minimum_balance(accounts[0].data_len());

    //-Process Instruction-----------------------------------------------------
    let result = process_withdraw_excess_lamports(accounts);

    //-Assert Postconditions---------------------------------------------------
    if accounts.len() < 3 {
        assert_eq!(result, Err(ProgramError::NotEnoughAccountKeys));
        return result;
    } else {
        assert_eq!(src_data_len, Account::LEN); // established by cheatcode_is_account
        {
            if src_account_initialised.is_err() {
                assert_eq!(result, Err(ProgramError::InvalidAccountData));
                return result;
            } else if !src_account_initialised.unwrap() {
                assert_eq!(result, Err(ProgramError::UninitializedAccount));
                return result;
            } else if src_account_is_native {
                assert_eq!(result, Err(ProgramError::Custom(10)));
                return result;
            }

            // Validate Owner
            inner_test_validate_owner(
                &src_account_owner, // expected_owner
                &accounts[2],       // owner_account_info
                &accounts[3..],     // tx_signers
                maybe_multisig_is_initialised,
                result.clone(),
            )?;

            if src_init_lamports < minimum_balance {
                assert_eq!(result, Err(ProgramError::Custom(0)));
                return result;
            } else if u64::MAX - src_init_lamports + minimum_balance < dst_init_lamports {
                assert_eq!(result, Err(ProgramError::Custom(0)));
                return result;
            }

            assert_eq!(accounts[0].lamports(), minimum_balance);
            assert_eq!(
                accounts[1].lamports(),
                dst_init_lamports + src_init_lamports - minimum_balance
            );
            assert!(result.is_ok())
        }
    }

    result
}

/// accounts[0] // Source Account Info (Mint)
/// accounts[1] // Destination Info
/// accounts[2] // Authority Info
#[inline(never)]
fn test_process_withdraw_excess_lamports_mint(accounts: &[AccountInfo; 3]) -> ProgramResult {
    cheatcode_is_mint(&accounts[0]); // Source Account (Mint)
    cheatcode_is_account(&accounts[1]); // Destination
    cheatcode_is_account(&accounts[2]); // Excluding the multisig case

    //-Initial State-----------------------------------------------------------
    let src_old = get_mint(&accounts[0]);
    let src_data_len = accounts[0].data_len();
    let src_mint_initialised = src_old.is_initialized();
    let src_mint_mint_authority = src_old.mint_authority().cloned();
    let src_init_lamports = accounts[0].lamports();
    let dst_init_lamports = accounts[1].lamports();
    let maybe_multisig_is_initialised = None; // Value set to `None` since authority is an account

    // Note: Rent is a supported sysvar so ProgramError::UnsupportedSysvar should be
    // impossible
    let rent = pinocchio::sysvars::rent::Rent::get().unwrap();
    let minimum_balance = rent.minimum_balance(accounts[0].data_len());

    //-Process Instruction-----------------------------------------------------
    let result = process_withdraw_excess_lamports(accounts);

    //-Assert Postconditions---------------------------------------------------
    if accounts.len() < 3 {
        assert_eq!(result, Err(ProgramError::NotEnoughAccountKeys));
        return result;
    } else {
        assert_eq!(src_data_len, Mint::LEN); // established by cheatcode_is_mint
        {
            if src_mint_initialised.is_err() {
                assert_eq!(result, Err(ProgramError::InvalidAccountData));
                return result;
            } else if !src_mint_initialised.unwrap() {
                assert_eq!(result, Err(ProgramError::UninitializedAccount));
                return result;
            } else if src_mint_mint_authority.is_some() {
                // Validate Owner
                inner_test_validate_owner(
                    &src_mint_mint_authority.unwrap(), // expected_owner
                    &accounts[2],                      // owner_account_info
                    &accounts[3..],                    // tx_signers
                    maybe_multisig_is_initialised,
                    result.clone(),
                )?;
            } else if accounts[0] != accounts[2] {
                assert_eq!(result, Err(ProgramError::Custom(15)));
                return result;
            } else if !accounts[2].is_signer() {
                assert_eq!(result, Err(ProgramError::MissingRequiredSignature));
                return result;
            }

            if src_init_lamports < minimum_balance {
                assert_eq!(result, Err(ProgramError::Custom(0)));
                return result;
            } else if dst_init_lamports
                .checked_add(src_init_lamports - minimum_balance)
                .is_none()
            {
                assert_eq!(result, Err(ProgramError::Custom(14)));
                return result;
            }

            assert!(result.is_ok());
            assert_eq!(accounts[0].lamports(), minimum_balance);
            assert_eq!(
                accounts[1].lamports(),
                dst_init_lamports
                    .checked_add(src_init_lamports - minimum_balance)
                    .unwrap()
            );
        }
    }
    result
}

/// accounts[0] // Source Account Info (Mint)
/// accounts[1] // Destination Info
/// accounts[2] // Authority Info
/// accounts[3..14] // Signers
#[inline(never)]
fn test_process_withdraw_excess_lamports_mint_multisig(
    accounts: &[AccountInfo; 4],
) -> ProgramResult {
    cheatcode_is_mint(&accounts[0]); // Source Account (Mint)
    cheatcode_is_account(&accounts[1]); // Destination
    cheatcode_is_multisig(&accounts[2]); // Authority

    //-Initial State-----------------------------------------------------------
    let src_old = get_mint(&accounts[0]);
    let src_data_len = accounts[0].data_len();
    let src_mint_initialised = src_old.is_initialized();
    let src_mint_mint_authority = src_old.mint_authority().cloned();
    let src_init_lamports = accounts[0].lamports();
    let dst_init_lamports = accounts[1].lamports();
    let maybe_multisig_is_initialised = Some(get_multisig(&accounts[2]).is_initialized());

    // Note: Rent is a supported sysvar so ProgramError::UnsupportedSysvar should be
    // impossible
    let rent = pinocchio::sysvars::rent::Rent::get().unwrap();
    let minimum_balance = rent.minimum_balance(accounts[0].data_len());

    //-Process Instruction-----------------------------------------------------
    let result = process_withdraw_excess_lamports(accounts);

    //-Assert Postconditions---------------------------------------------------
    if accounts.len() < 3 {
        assert_eq!(result, Err(ProgramError::NotEnoughAccountKeys));
        return result;
    } else {
        assert_eq!(src_data_len, Mint::LEN); // established by cheatcode_is_mint
        {
            if src_mint_initialised.is_err() {
                assert_eq!(result, Err(ProgramError::InvalidAccountData));
                return result;
            } else if !src_mint_initialised.unwrap() {
                assert_eq!(result, Err(ProgramError::UninitializedAccount));
                return result;
            } else if src_mint_mint_authority.is_some() {
                // Validate Owner
                inner_test_validate_owner(
                    &src_mint_mint_authority.unwrap(), // expected_owner
                    &accounts[2],                      // owner_account_info
                    &accounts[3..],                    // tx_signers
                    maybe_multisig_is_initialised,
                    result.clone(),
                )?;
            } else if accounts[0] != accounts[2] {
                assert_eq!(result, Err(ProgramError::Custom(15)));
                return result;
            } else if !accounts[2].is_signer() {
                assert_eq!(result, Err(ProgramError::MissingRequiredSignature));
                return result;
            } else if src_init_lamports < minimum_balance {
                assert_eq!(result, Err(ProgramError::Custom(0)));
                return result;
            } else if u64::MAX - src_init_lamports + minimum_balance < dst_init_lamports {
                assert_eq!(result, Err(ProgramError::Custom(0)));
                return result;
            }

            assert_eq!(accounts[0].lamports(), minimum_balance);
            assert_eq!(
                accounts[1].lamports(),
                dst_init_lamports + src_init_lamports - minimum_balance
            );
            assert!(result.is_ok())
        }
    }

    result
}

/// accounts[0] // Source Account Info
/// accounts[1] // Destination Info
/// accounts[2] // Authority Info
#[inline(never)]
fn test_process_withdraw_excess_lamports_multisig(accounts: &[AccountInfo; 3]) -> ProgramResult {
    cheatcode_is_multisig(&accounts[0]); // Source Account (Multisig)
    cheatcode_is_account(&accounts[1]); // Destination
    cheatcode_is_account(&accounts[2]); // Excluding the multisig case

    //-Initial State-----------------------------------------------------------
    let src_data_len = accounts[0].data_len();
    let src_init_lamports = accounts[0].lamports();
    let dst_init_lamports = accounts[1].lamports();
    let maybe_multisig_is_initialised = None; // Value set to `None` since authority is an account

    // Note: Rent is a supported sysvar so ProgramError::UnsupportedSysvar should be
    // impossible
    let rent = pinocchio::sysvars::rent::Rent::get().unwrap();
    let minimum_balance = rent.minimum_balance(accounts[0].data_len());

    //-Process Instruction-----------------------------------------------------
    let result = process_withdraw_excess_lamports(accounts);

    //-Assert Postconditions---------------------------------------------------
    if accounts.len() < 3 {
        assert_eq!(result, Err(ProgramError::NotEnoughAccountKeys));
        return result;
    } else if src_data_len != Account::LEN
        && src_data_len != Mint::LEN
        && src_data_len != Multisig::LEN
    {
        assert_eq!(result, Err(ProgramError::Custom(13)));
        return result;
    } else {
        assert_eq!(src_data_len, Multisig::LEN); // established by cheatcode_is_multisig

        // Validate Owner
        inner_test_validate_owner(
            accounts[0].key(), // expected_owner
            &accounts[2],      // owner_account_info
            &accounts[3..],    // tx_signers
            maybe_multisig_is_initialised,
            result.clone(),
        )?;

        if src_init_lamports < minimum_balance {
            assert_eq!(result, Err(ProgramError::Custom(0)));
            return result;
        } else if dst_init_lamports
            .checked_add(src_init_lamports - minimum_balance)
            .is_none()
        {
            assert_eq!(result, Err(ProgramError::Custom(0)));
            return result;
        }

        assert_eq!(accounts[0].lamports(), minimum_balance);
        assert_eq!(
            accounts[1].lamports(),
            dst_init_lamports + (src_init_lamports - minimum_balance)
        );
        assert!(result.is_ok())
    }

    result
}

/// accounts[0] // Source Account Info
/// accounts[1] // Destination Info
/// accounts[2] // Authority Info
/// accounts[3..14] // Signers
#[inline(never)]
fn test_process_withdraw_excess_lamports_multisig_multisig(
    accounts: &[AccountInfo; 4],
) -> ProgramResult {
    cheatcode_is_multisig(&accounts[0]); // Source Account (Multisig)
    cheatcode_is_account(&accounts[1]); // Destination
    cheatcode_is_multisig(&accounts[2]); // Authority

    //-Initial State-----------------------------------------------------------
    let src_data_len = accounts[0].data_len();
    let src_init_lamports = accounts[0].lamports();
    let dst_init_lamports = accounts[1].lamports();
    let maybe_multisig_is_initialised = Some(get_multisig(&accounts[2]).is_initialized());

    // Note: Rent is a supported sysvar so ProgramError::UnsupportedSysvar should be
    // impossible
    let rent = pinocchio::sysvars::rent::Rent::get().unwrap();
    let minimum_balance = rent.minimum_balance(accounts[0].data_len());

    //-Process Instruction-----------------------------------------------------
    let result = process_withdraw_excess_lamports(accounts);

    //-Assert Postconditions---------------------------------------------------
    if accounts.len() < 3 {
        assert_eq!(result, Err(ProgramError::NotEnoughAccountKeys));
        return result;
    } else if src_data_len != Account::LEN
        && src_data_len != Mint::LEN
        && src_data_len != Multisig::LEN
    {
        assert_eq!(result, Err(ProgramError::Custom(13)));
        return result;
    } else {
        assert_eq!(src_data_len, Multisig::LEN); // established by cheatcode_is_multisig

        // Validate Owner
        inner_test_validate_owner(
            accounts[0].key(), // expected_owner
            &accounts[2],      // owner_account_info
            &accounts[3..],    // tx_signers
            maybe_multisig_is_initialised,
            result.clone(),
        )?;

        if src_init_lamports < minimum_balance {
            assert_eq!(result, Err(ProgramError::Custom(0)));
            return result;
        } else if u64::MAX - src_init_lamports + minimum_balance < dst_init_lamports {
            assert_eq!(result, Err(ProgramError::Custom(0)));
            return result;
        }

        assert_eq!(accounts[0].lamports(), minimum_balance);
        assert_eq!(
            accounts[1].lamports(),
            dst_init_lamports + src_init_lamports - minimum_balance
        );
        assert!(result.is_ok())
    }

    result
}
