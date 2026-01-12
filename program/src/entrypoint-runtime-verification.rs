//! Program entrypoint for runtime verification proofs of original spl token implmentation

use {
    crate::{
        instruction::TokenInstruction,
        processor::Processor,
        state::{Account, AccountState, Mint, Multisig},
        ID as PROGRAM_ID,
    },
    solana_account_info::AccountInfo,
    solana_program_error::{ProgramError, ProgramResult},
    solana_program_pack::Pack,
    solana_pubkey::{self as pubkey, Pubkey},
    solana_sysvar::Sysvar,
    spl_token_interface::{error::TokenError, native_mint},
    std::intrinsics::assume,
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

struct MintWrapper(Result<Mint, ProgramError>);

impl MintWrapper {
    fn is_initialized(&self) -> Result<bool, ProgramError> {
        match &self.0 {
            Ok(m) => Ok(m.is_initialized),
            Err(e) => Err(e.clone()),
        }
    }

    fn mint_authority(&self) -> Option<&Pubkey> {
        match &self.0 {
            Ok(m) => match m.mint_authority.as_ref() {
                solana_program_option::COption::Some(pk) => Some(pk),
                solana_program_option::COption::None => None,
            },
            Err(_) => None,
        }
    }

    fn freeze_authority(&self) -> Option<&Pubkey> {
        match &self.0 {
            Ok(m) => match m.freeze_authority.as_ref() {
                solana_program_option::COption::Some(pk) => Some(pk),
                solana_program_option::COption::None => None,
            },
            Err(_) => None,
        }
    }

    fn supply(&self) -> u64 {
        match &self.0 {
            Ok(m) => m.supply,
            Err(_) => 0,
        }
    }

    fn decimals(&self) -> u8 {
        match &self.0 {
            Ok(m) => m.decimals,
            Err(_) => 0,
        }
    }
}

fn get_mint(account_info: &AccountInfo) -> MintWrapper {
    MintWrapper(Mint::unpack_unchecked(&account_info.data.borrow()))
}

macro_rules! assert_pubkey_from_slice {
    ($actual:expr, $slice:expr) => {{
        let expected_pubkey = Pubkey::new_from_array($slice.try_into().unwrap());
        assert_eq!($actual, expected_pubkey);
    }};
}

/// Macros to abstract API differences between spl-token and p-token.
/// spl-token AccountInfo has fields (.key, .owner), p-token has methods
/// (.key(), .owner()). spl-token wrappers have methods (.mint(), .decimals()),
/// p-token has fields (.mint, .decimals).
macro_rules! key {
    ($acc:expr) => {
        $acc.key
    };
}
macro_rules! owner {
    ($acc:expr) => {
        $acc.owner
    };
}
macro_rules! mint {
    ($acc:expr) => {
        $acc.mint()
    };
}
macro_rules! decimals {
    ($m:expr) => {
        $m.decimals()
    };
}
macro_rules! account_owner {
    ($acc:expr) => {
        $acc.owner()
    };
}
/// Cheatcode macros to abstract naming differences.
macro_rules! cheatcode_mint {
    ($acc:expr) => {
        cheatcode_is_spl_mint($acc)
    };
}
macro_rules! cheatcode_account {
    ($acc:expr) => {
        cheatcode_is_spl_account($acc)
    };
}
macro_rules! cheatcode_multisig {
    ($acc:expr) => {
        cheatcode_is_spl_multisig($acc)
    };
}
/// Process call macro to abstract the processor call difference.
/// spl-token prepends discriminator and uses TokenInstruction::unpack.
macro_rules! call_process_mint_to_checked {
    ($accounts:expr, $instruction_data:expr) => {{
        let mut data_with_disc = [0u8; 10];
        data_with_disc[0] = 14; // MintToChecked discriminator
        data_with_disc[1..].copy_from_slice($instruction_data);
        let TokenInstruction::MintToChecked { amount, decimals } =
            TokenInstruction::unpack(&data_with_disc)?
        else {
            unreachable!()
        };
        Processor::process_mint_to(&PROGRAM_ID, $accounts, amount, Some(decimals))
    }};
}
macro_rules! call_process_approve {
    ($accounts:expr, $instruction_data:expr) => {{
        let mut data_with_disc = [0u8; 9];
        data_with_disc[0] = 4; // Approve discriminator
        data_with_disc[1..].copy_from_slice($instruction_data);
        let TokenInstruction::Approve { amount } = TokenInstruction::unpack(&data_with_disc)?
        else {
            unreachable!()
        };
        Processor::process_approve(&PROGRAM_ID, $accounts, amount, None)
    }};
}
macro_rules! call_process_revoke {
    ($accounts:expr) => {
        Processor::process_revoke(&PROGRAM_ID, $accounts)
    };
}
macro_rules! call_process_transfer {
    ($accounts:expr, $instruction_data:expr) => {{
        let mut data_with_disc = [0u8; 9];
        data_with_disc[0] = 3; // Transfer discriminator
        data_with_disc[1..].copy_from_slice($instruction_data);
        let TokenInstruction::Transfer { amount } = TokenInstruction::unpack(&data_with_disc)?
        else {
            unreachable!()
        };
        Processor::process_transfer(&PROGRAM_ID, $accounts, amount, None)
    }};
}
macro_rules! call_process_mint_to {
    ($accounts:expr, $instruction_data:expr) => {{
        let mut data_with_disc = [0u8; 9];
        data_with_disc[0] = 7; // MintTo discriminator
        data_with_disc[1..].copy_from_slice($instruction_data);
        Processor::process(&crate::id(), $accounts, &data_with_disc)
    }};
}
macro_rules! call_process_burn {
    ($accounts:expr, $instruction_data:expr) => {{
        let mut data_with_disc = [0u8; 9];
        data_with_disc[0] = 8; // Burn discriminator
        data_with_disc[1..].copy_from_slice($instruction_data);
        Processor::process(&crate::id(), $accounts, &data_with_disc)
    }};
}
/// Macros for inner_test_validate_owner
macro_rules! is_signer {
    ($acc:expr) => {
        $acc.is_signer
    };
}
macro_rules! is_owned_by_program {
    ($acc:expr) => {
        $acc.owner == &crate::id()
    };
}
macro_rules! multisig_signers {
    ($multisig:expr) => {
        $multisig.signers()
    };
}
macro_rules! multisig_m {
    ($multisig:expr) => {
        $multisig.m()
    };
}
macro_rules! multisig_n {
    ($multisig:expr) => {
        $multisig.n()
    };
}
macro_rules! is_valid_signer_index {
    ($n:expr) => {
        (($n) >= 1) && (($n) <= 11)
    };
}
macro_rules! call_process_initialize_multisig {
    ($accounts:expr, $instruction_data:expr) => {{
        let mut data_with_disc = [0u8; 2];
        data_with_disc[0] = 2;
        data_with_disc[1..].copy_from_slice($instruction_data);
        Processor::process(&crate::id(), $accounts, &data_with_disc)
    }};
}
macro_rules! call_process_initialize_multisig2 {
    ($accounts:expr, $instruction_data:expr) => {{
        let mut data_with_disc = [0u8; 2];
        data_with_disc[0] = 19;
        data_with_disc[1..].copy_from_slice($instruction_data);
        Processor::process(&crate::id(), $accounts, &data_with_disc)
    }};
}
macro_rules! get_rent_sysvar {
    () => {
        solana_rent::Rent::get().unwrap()
    };
}
macro_rules! call_process_set_authority {
    ($accounts:expr, $instruction_data:expr) => {{
        let mut data_with_disc = [0u8; 35];
        data_with_disc[0] = 6;
        data_with_disc[1..].copy_from_slice($instruction_data);
        Processor::process(&crate::id(), $accounts, &data_with_disc)
    }};
}
/// Macros for test_process_initialize_mint_freeze
macro_rules! cheatcode_rent {
    ($acc:expr) => {
        cheatcode_is_spl_rent($acc)
    };
}
macro_rules! rent_id {
    () => {
        &solana_sysvar::rent::ID
    };
}
macro_rules! mint_decimals {
    ($mint:expr) => {
        $mint.decimals()
    };
}
macro_rules! assert_mint_authority {
    ($mint:expr, $slice:expr) => {{
        let expected_pubkey = Pubkey::new_from_array($slice.try_into().unwrap());
        assert_eq!(*$mint.mint_authority().unwrap(), expected_pubkey);
    }};
}
macro_rules! assert_freeze_authority {
    ($mint:expr, $slice:expr) => {{
        let expected_pubkey = Pubkey::new_from_array($slice.try_into().unwrap());
        assert_eq!(*$mint.freeze_authority().unwrap(), expected_pubkey);
    }};
}
macro_rules! call_process_initialize_mint {
    ($accounts:expr, $instruction_data:expr) => {{
        let mut data_with_disc = [0u8; 67];
        data_with_disc[0] = 0; // InitializeMint discriminator
        data_with_disc[1..].copy_from_slice($instruction_data);
        Processor::process(&crate::id(), $accounts, &data_with_disc)
    }};
}
macro_rules! call_process_initialize_mint_no_freeze {
    ($accounts:expr, $instruction_data:expr) => {{
        let mut data_with_disc = [0u8; 35];
        data_with_disc[0] = 0; // InitializeMint discriminator
        data_with_disc[1..].copy_from_slice($instruction_data);
        Processor::process(&crate::id(), $accounts, &data_with_disc)
    }};
}
macro_rules! call_process_initialize_mint2 {
    ($accounts:expr, $instruction_data:expr) => {{
        let mut data_with_disc = [0u8; 67];
        data_with_disc[0] = 20; // InitializeMint2 discriminator
        data_with_disc[1..].copy_from_slice($instruction_data);
        Processor::process(&crate::id(), $accounts, &data_with_disc)
    }};
}
macro_rules! call_process_initialize_mint2_no_freeze {
    ($accounts:expr, $instruction_data:expr) => {{
        let mut data_with_disc = [0u8; 35];
        data_with_disc[0] = 20; // InitializeMint2 discriminator
        data_with_disc[1..].copy_from_slice($instruction_data);
        Processor::process(&crate::id(), $accounts, &data_with_disc)
    }};
}
/// Macros for test_process_initialize_account
macro_rules! native_mint_id {
    () => {
        &native_mint::ID
    };
}
macro_rules! program_id {
    () => {
        &crate::id()
    };
}
macro_rules! account_info_owner {
    ($account:expr) => {
        $account.owner
    };
}
macro_rules! call_process_amount_to_ui_amount {
    ($accounts:expr, $instruction_data:expr) => {{
        let mut data_with_disc = [0u8; 9];
        data_with_disc[0] = 23;
        data_with_disc[1..].copy_from_slice($instruction_data);
        Processor::process(&crate::id(), $accounts, &data_with_disc)
    }};
}
macro_rules! call_process_ui_amount_to_amount {
    ($accounts:expr, $instruction_data:expr) => {{
        let mut data_with_disc = vec![24u8];
        data_with_disc.extend_from_slice($instruction_data);
        Processor::process(&crate::id(), $accounts, &data_with_disc)
    }};
}
macro_rules! call_process_transfer_checked {
    ($accounts:expr, $instruction_data:expr) => {{
        let mut data_with_disc = [0u8; 10];
        data_with_disc[0] = 12;
        data_with_disc[1..].copy_from_slice($instruction_data);
        Processor::process(&crate::id(), $accounts, &data_with_disc)
    }};
}
macro_rules! call_process_burn_checked {
    ($accounts:expr, $instruction_data:expr) => {{
        let mut data_with_disc = [0u8; 10];
        data_with_disc[0] = 15;
        data_with_disc[1..].copy_from_slice($instruction_data);
        Processor::process(&crate::id(), $accounts, &data_with_disc)
    }};
}
macro_rules! accounts_equal {
    ($a:expr, $b:expr) => {
        $a.key == $b.key
    };
}
macro_rules! account_state_uninitialized {
    () => {
        spl_token_interface::state::AccountState::Uninitialized
    };
}
macro_rules! account_state_initialized {
    () => {
        spl_token_interface::state::AccountState::Initialized
    };
}
macro_rules! call_process_initialize_account {
    ($accounts:expr) => {{
        Processor::process(&crate::id(), $accounts, &[1u8])
    }};
}
/// Macros for test_process_initialize_account2/3
macro_rules! call_process_initialize_account2 {
    ($accounts:expr, $instruction_data:expr) => {{
        let mut data_with_disc = [0u8; 33];
        data_with_disc[0] = 16; // InitializeAccount2 discriminator
        data_with_disc[1..].copy_from_slice($instruction_data);
        Processor::process(&crate::id(), $accounts, &data_with_disc)
    }};
}
macro_rules! call_process_initialize_account3 {
    ($accounts:expr, $instruction_data:expr) => {{
        let mut data_with_disc = [0u8; 33];
        data_with_disc[0] = 18; // InitializeAccount3 discriminator
        data_with_disc[1..].copy_from_slice($instruction_data);
        Processor::process(&crate::id(), $accounts, &data_with_disc)
    }};
}
macro_rules! pubkey_bytes {
    () => {
        pubkey::PUBKEY_BYTES
    };
}
macro_rules! assert_account_owner_eq_bytes {
    ($acc:expr, $bytes:expr) => {
        assert_eq!($acc.owner(), (*$bytes).into());
    };
}
macro_rules! assert_account_mint_eq_key {
    ($acc:expr, $key:expr) => {
        assert_eq!($acc.mint(), *$key);
    };
}
/// Macros for test_process_transfer
macro_rules! account_state_frozen {
    () => {
        spl_token_interface::state::AccountState::Frozen
    };
}
macro_rules! accounts_eq {
    ($a:expr, $b:expr) => {
        $a.key == $b.key
    };
}
macro_rules! call_process_transfer_inner {
    ($accounts:expr, $instruction_data:expr) => {{
        let mut data_with_disc = [0u8; 9];
        data_with_disc[0] = 3; // Transfer discriminator
        data_with_disc[1..].copy_from_slice($instruction_data);
        Processor::process(&crate::id(), $accounts, &data_with_disc)
    }};
}
macro_rules! account_mint {
    ($acc:expr) => {
        $acc.mint()
    };
}
macro_rules! owner_is_program {
    ($acc:expr) => {
        $acc.owner == &crate::id()
    };
}
macro_rules! call_process_close_account {
    ($accounts:expr) => {{
        let mut data_with_disc = [0u8; 1];
        data_with_disc[0] = 9;
        Processor::process(&crate::id(), $accounts, &data_with_disc)
    }};
}
macro_rules! incinerator_id {
    () => {
        solana_sdk_ids::incinerator::ID
    };
}
macro_rules! call_process_freeze_account {
    ($accounts:expr) => {{
        let mut data_with_disc = [0u8; 1];
        data_with_disc[0] = 10;
        Processor::process(&crate::id(), $accounts, &data_with_disc)
    }};
}
macro_rules! call_process_thaw_account {
    ($accounts:expr) => {{
        let mut data_with_disc = [0u8; 1];
        data_with_disc[0] = 11;
        Processor::process(&crate::id(), $accounts, &data_with_disc)
    }};
}
macro_rules! call_process_sync_native {
    ($accounts:expr) => {{
        let mut data_with_disc = [0u8; 1];
        data_with_disc[0] = 17;
        Processor::process(&crate::id(), $accounts, &data_with_disc)
    }};
}
macro_rules! call_process_get_account_data_size {
    ($accounts:expr) => {{
        let mut data_with_disc = [0u8; 1];
        data_with_disc[0] = 21;
        Processor::process(&crate::id(), $accounts, &data_with_disc)
    }};
}
macro_rules! call_process_initialize_immutable_owner {
    ($accounts:expr) => {{
        let mut data_with_disc = [0u8; 1];
        data_with_disc[0] = 22;
        Processor::process(&crate::id(), $accounts, &data_with_disc)
    }};
}
macro_rules! call_process_approve_checked {
    ($accounts:expr, $instruction_data:expr) => {{
        let mut data_with_disc = [0u8; 10];
        data_with_disc[0] = 13;
        data_with_disc[1..].copy_from_slice($instruction_data);
        Processor::process(&crate::id(), $accounts, &data_with_disc)
    }};
}

/// A wrapper struct as middleware so that the same functions called
/// on the p-token Account are called on the spl Account. However,
/// this means that fields have to be accessed through functions.
struct AccountWrapper(Result<Account, ProgramError>);

impl AccountWrapper {
    fn is_initialized(&self) -> Result<bool, ProgramError> {
        match &self.0 {
            Ok(a) => Ok(a.state != AccountState::Uninitialized),
            Err(e) => Err(e.clone()),
        }
    }

    fn amount(&self) -> u64 {
        match &self.0 {
            Ok(a) => a.amount,
            Err(_) => panic!("AccountWrapper amount: underlying account missing"),
        }
    }

    fn mint(&self) -> Pubkey {
        match &self.0 {
            Ok(a) => a.mint,
            Err(_) => panic!("AccountWrapper mint: underlying account missing"),
        }
    }

    fn owner(&self) -> Pubkey {
        match &self.0 {
            Ok(a) => a.owner,
            Err(_) => panic!("AccountWrapper owner: underlying account missing"),
        }
    }

    fn delegate(&self) -> Option<&Pubkey> {
        match &self.0 {
            Ok(a) => match a.delegate.as_ref() {
                solana_program_option::COption::None => None,
                solana_program_option::COption::Some(delegate) => Some(delegate),
            },
            Err(_) => None,
        }
    }

    fn delegated_amount(&self) -> u64 {
        match &self.0 {
            Ok(a) => a.delegated_amount,
            Err(_) => panic!("AccountWrapper delegated_amount: underlying account missing"),
        }
    }

    fn account_state(&self) -> Result<AccountState, ProgramError> {
        match &self.0 {
            Ok(a) => Ok(a.state),
            Err(e) => Err(e.clone()),
        }
    }

    fn is_native(&self) -> bool {
        match &self.0 {
            Ok(a) => a.is_native.is_some(),
            Err(_) => false,
        }
    }

    fn native_amount(&self) -> Option<u64> {
        match &self.0 {
            Ok(a) => match a.is_native {
                solana_program_option::COption::Some(amt) => Some(amt),
                solana_program_option::COption::None => None,
            },
            Err(_) => None,
        }
    }

    fn close_authority(&self) -> Option<&Pubkey> {
        match &self.0 {
            Ok(a) => match a.close_authority.as_ref() {
                solana_program_option::COption::Some(pk) => Some(pk),
                solana_program_option::COption::None => None,
            },
            Err(_) => None,
        }
    }

    fn is_owned_by_system_program_or_incinerator(&self) -> bool {
        match &self.0 {
            Ok(a) => a.owner == solana_sdk_ids::system_program::ID || a.owner == solana_sdk_ids::incinerator::ID,
            Err(_) => false,
        }
    }
}

/// So the AccountWrapper derefs the wrapped Account
impl core::ops::Deref for AccountWrapper {
    type Target = Result<Account, ProgramError>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Helper function from p-token must be implemented on AccountWrapper
fn get_account(account_info: &AccountInfo) -> AccountWrapper {
    AccountWrapper(Account::unpack_unchecked(&account_info.data.borrow()))
}

/// A wrapper struct as middleware so that the same functions called
/// on the p-token Multisig are called on the spl Multisig. However,
/// this means that fields have to be accessed through functions.
struct MultisigWrapper(Result<Multisig, ProgramError>);

impl MultisigWrapper {
    fn is_initialized(&self) -> Result<bool, ProgramError> {
        match &self.0 {
            Ok(m) => Ok(m.is_initialized),
            Err(e) => Err(e.clone()),
        }
    }

    fn signers(&self) -> &[Pubkey] {
        match &self.0 {
            Ok(m) => &m.signers[..],
            Err(_) => &[],
        }
    }

    fn m(&self) -> u8 {
        match &self.0 {
            Ok(m) => m.m,
            Err(_) => 0,
        }
    }

    fn n(&self) -> u8 {
        match &self.0 {
            Ok(m) => m.n,
            Err(_) => 0,
        }
    }
}

/// So the MultisigWrapper derefs the wrapped Multisig
impl core::ops::Deref for MultisigWrapper {
    type Target = Result<Multisig, ProgramError>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Helper function from p-token must be implemented on MultisigWrapper
fn get_multisig(account_info: &AccountInfo) -> MultisigWrapper {
    MultisigWrapper(Multisig::unpack_unchecked(&account_info.data.borrow()))
}

fn get_rent(account_info: &AccountInfo) -> solana_rent::Rent {
    // Directly deserialize from account data without key check
    bincode::deserialize(&account_info.data.borrow()).unwrap()
}

include!("../../shared/inner_test_validate_owner.rs");

// TODO: Not sure if these are needed since there is no UB like p-token
#[inline(never)]
fn cheatcode_is_spl_account(_: &AccountInfo) {}
#[inline(never)]
fn cheatcode_is_spl_mint(_: &AccountInfo) {}
#[inline(never)]
fn cheatcode_is_spl_multisig(_: &AccountInfo) {}
#[inline(never)]
fn cheatcode_is_spl_rent(_: &AccountInfo) {}

// special test for basic domain data access (SPL types)
#[inline(never)]
fn test_spltoken_domain_data(acc: &AccountInfo, mint: &AccountInfo, rent: &AccountInfo) {
    // Mutate mint via standard unpack/pack flow; use unwraps for brevity in tests
    cheatcode_is_spl_mint(mint);
    let mut m = Mint::unpack_unchecked(&mint.data.borrow()).unwrap();
    m.is_initialized = true;
    Mint::pack(m, &mut mint.data.borrow_mut()).unwrap();
    let m2 = Mint::unpack(&mint.data.borrow()).unwrap();
    assert!(m2.is_initialized);

    // Set Account.is_native in the simplest way (parity with p-token's boolean set_native(true))
    cheatcode_is_spl_account(acc);
    let mut a = Account::unpack_unchecked(&acc.data.borrow()).unwrap();
    a.is_native = solana_program_option::COption::Some(0);
    Account::pack(a, &mut acc.data.borrow_mut()).unwrap();
    // Verify via the same wrapper accessor used elsewhere
    let iacc = get_account(acc);
    assert!(iacc.is_native());

    // Basic owner self-check
    let owner = acc.owner;
    assert_eq!(acc.owner, owner);

    // Compare Rent behavior using the sysvar getter and the provided account
    let sysrent = solana_rent::Rent::get().unwrap();
    let rent_collected = 10;
    let (sys_burnt, sys_distributed) = sysrent.calculate_burn(rent_collected);
    assert!(sysrent.burn_percent > 100 || (sys_burnt <= rent_collected && sys_distributed <= rent_collected));

    cheatcode_is_spl_rent(rent);
    let prent = solana_rent::Rent::from_account_info(rent).unwrap_or(sysrent);
    let (acct_burnt, acct_distributed) = prent.calculate_burn(rent_collected);
    assert!(prent.burn_percent > 100 || (acct_burnt <= rent_collected && acct_distributed <= rent_collected));
}

// wrapper to ensure the test is retained in SMIR/IR outputs
#[no_mangle]
pub unsafe extern "C" fn use_tests(acc: &AccountInfo) {
    test_spltoken_domain_data(acc, acc, acc);
}

// Inline `assume` is used directly in test harnesses; no helper functions needed.

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
        // 0 - Initialize Mint Freeze
        0 => {
            // #[cfg(feature = "logging")]
            // msg!("Testing Instruction: Initialize Mint Freeze");
            let [_d, payload @ ..] = instruction_data else {
                return Err(TokenError::InvalidInstruction.into());
            };
            match payload.len() {
                x if 66 <= x => {
                    test_process_initialize_mint_freeze(
                        accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?,
                        payload.first_chunk().ok_or(TokenError::InvalidInstruction)?,
                    )
                }
                x if 34 <= x => {
                    test_process_initialize_mint_no_freeze(
                        accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?,
                        payload.first_chunk().ok_or(TokenError::InvalidInstruction)?,
                    )
                }
                _ => Err(TokenError::InvalidInstruction.into()),
            }
        }
        // 1 - Initialize Account
        1 => {
            test_process_initialize_account(
                accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?,
            )
        }
        // 2 - Initialize Multisig
        2 => {
            test_process_initialize_multisig(
                accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?,
                instruction_data[1..].first_chunk().ok_or(TokenError::InvalidInstruction)?,
            )
        }
        // 3 - Transfer
        3 => {
            if accounts.len() >= 4
                && accounts[2].data_len() == Multisig::LEN
                && accounts[2].owner == &crate::id()
            {
                test_process_transfer_multisig(
                    accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?, // CHANGE P-Token: accounts: &[AccountInfo; 4]
                    instruction_data.last_chunk().ok_or(TokenError::InvalidInstruction)?,
                )
            } else {
                test_process_transfer(
                    accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?, // CHANGE P-Token: accounts: &[AccountInfo; 3]
                    instruction_data.last_chunk().ok_or(TokenError::InvalidInstruction)?,
                )
            }
        }
        // 4 - Approve
        4 => {
            if accounts.len() >= 4
                && accounts[2].data_len() == Multisig::LEN
                && accounts[2].owner == &crate::id()
            {
                test_process_approve_multisig(
                    accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?, // CHANGE P-Token: accounts: &[AccountInfo; 4]
                    instruction_data.last_chunk().ok_or(TokenError::InvalidInstruction)?,
                )
            } else {
                test_process_approve(
                    accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?, // CHANGE P-Token: accounts: &[AccountInfo; 3]
                    instruction_data.last_chunk().ok_or(TokenError::InvalidInstruction)?,
                )
            }
        }
        // 5 - Revoke
        5 => {
            if accounts.len() >= 3
                && accounts[1].data_len() == Multisig::LEN
                && accounts[1].owner == &crate::id()
            {
                test_process_revoke_multisig(
                    accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?,
                )
            } else {
                test_process_revoke(
                    accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?,
                )
            }
        }
        // 6 - Set Authority Account
        6 => {
            // #[cfg(feature = "logging")]
            // msg!("Testing Instruction: Set Authority Account");
            if let Some(first_account) = accounts.first() {
                match first_account.data_len() {
                    Account::LEN => {
                        if accounts.len() >= 3
                            && accounts[1].data_len() == Multisig::LEN
                            && accounts[1].owner == &crate::id()
                        {
                            test_process_set_authority_account_multisig(
                                accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?,
                                instruction_data[1..].first_chunk().ok_or(TokenError::InvalidInstruction)?,
                            )
                        } else {
                            test_process_set_authority_account(
                                accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?,
                                instruction_data[1..].first_chunk().ok_or(TokenError::InvalidInstruction)?,
                            )
                        }
                    }
                    Mint::LEN => {
                        if accounts.len() >= 3
                            && accounts[1].data_len() == Multisig::LEN
                            && accounts[1].owner == &crate::id()
                        {
                            test_process_set_authority_mint_multisig(
                                accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?, // CHANGE P-Token: accounts: &[AccountInfo; 3]
                                instruction_data[1..].first_chunk().ok_or(TokenError::InvalidInstruction)?,
                            )
                        } else {
                            test_process_set_authority_mint(
                                accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?, // CHANGE P-Token: accounts: &[AccountInfo; 2]
                                instruction_data[1..].first_chunk().ok_or(TokenError::InvalidInstruction)?,
                            )
                        }
                    }
                    _ => Err(TokenError::InvalidInstruction.into()),
                }
            } else {
                Err(TokenError::InvalidInstruction.into())
            }
        }
        // 7 - Mint To
        7 => {
            if accounts.len() >= 4
                && accounts[2].data_len() == Multisig::LEN
                && accounts[2].owner == &crate::id()
            {
                test_process_mint_to_multisig(
                    accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?, // CHANGE P-Token: accounts: &[AccountInfo; 4]
                    instruction_data.last_chunk().ok_or(TokenError::InvalidInstruction)?,
                )
            } else {
                test_process_mint_to(
                    accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?, // CHANGE P-Token: accounts: &[AccountInfo; 3]
                    instruction_data.last_chunk().ok_or(TokenError::InvalidInstruction)?,
                )
            }
        }
        // 8 - Burn
        8 => {
            if accounts.len() >= 4
                && accounts[2].data_len() == Multisig::LEN
                && accounts[2].owner == &crate::id()
            {
                test_process_burn_multisig(
                    accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?, // CHANGE P-Token: accounts: &[AccountInfo; 4]
                    instruction_data.last_chunk().ok_or(TokenError::InvalidInstruction)?,
                )
            } else {
                test_process_burn(
                    accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?, // CHANGE P-Token: accounts: &[AccountInfo; 3]
                    instruction_data.last_chunk().ok_or(TokenError::InvalidInstruction)?,
                )
            }
        }
        // 9 - Close Account
        9 => {
            if accounts.len() >= 4
                && accounts[2].data_len() == Multisig::LEN
                && accounts[2].owner == &crate::id()
            {
                test_process_close_account_multisig(
                    accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?,
                )
            } else {
                test_process_close_account(
                    accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?,
                )
            }
        }
        // 10 - Freeze Account
        10 => {
            if accounts.len() >= 4
                && accounts[2].data_len() == Multisig::LEN
                && accounts[2].owner == &crate::id()
            {
                test_process_freeze_account_multisig(
                    accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?,
                )
            } else {
                test_process_freeze_account(
                    accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?,
                )
            }
        }
        // 11 - Thaw Account
        11 => {
            if accounts.len() >= 4
                && accounts[2].data_len() == Multisig::LEN
                && accounts[2].owner == &crate::id()
            {
                test_process_thaw_account_multisig(
                    accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?,
                )
            } else {
                test_process_thaw_account(
                    accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?,
                )
            }
        }
        // 12 - Transfer Checked
        12 => {
            if accounts.len() >= 5
                && accounts[3].data_len() == Multisig::LEN
                && accounts[3].owner == &crate::id()
            {
                test_process_transfer_checked_multisig(
                    accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?,
                    instruction_data[1..].first_chunk().ok_or(TokenError::InvalidInstruction)?,
                )
            } else {
                test_process_transfer_checked(
                    accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?,
                    instruction_data[1..].first_chunk().ok_or(TokenError::InvalidInstruction)?,
                )
            }
        }
        // 13 - Approve Checked
        13 => {
            if accounts.len() >= 5
                && accounts[3].data_len() == Multisig::LEN
                && accounts[3].owner == &crate::id()
            {
                test_process_approve_checked_multisig(
                    accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?,
                    instruction_data[1..].first_chunk().ok_or(TokenError::InvalidInstruction)?,
                )
            } else {
                test_process_approve_checked(
                    accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?,
                    instruction_data[1..].first_chunk().ok_or(TokenError::InvalidInstruction)?,
                )
            }
        }
        // 14 - Mint To Checked
        14 => {
            if accounts.len() >= 4
                && accounts[2].data_len() == Multisig::LEN
                && accounts[2].owner == &crate::id()
            {
                test_process_mint_to_checked_multisig(
                    accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?,
                    instruction_data[1..].first_chunk().ok_or(TokenError::InvalidInstruction)?,
                )
            } else {
                test_process_mint_to_checked(
                    accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?, // CHANGE P-Token: accounts: &[AccountInfo; 3]
                    instruction_data.last_chunk().ok_or(TokenError::InvalidInstruction)?,
                )
            }
        }
        // 15 - Burn Checked
        15 => {
            if accounts.len() >= 4
                && accounts[2].data_len() == Multisig::LEN
                && accounts[2].owner == &crate::id()
            {
                test_process_burn_checked_multisig(
                    accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?,
                    instruction_data[1..].first_chunk().ok_or(TokenError::InvalidInstruction)?,
                )
            } else {
                test_process_burn_checked(
                    accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?,
                    instruction_data[1..].first_chunk().ok_or(TokenError::InvalidInstruction)?,
                )
            }
        }
        // 16 - Initialize Account2
        16 => {
            test_process_initialize_account2(
                accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?, // CHANGE P-Token: accounts: &[AccountInfo; 3]
                instruction_data[1..].first_chunk().ok_or(TokenError::InvalidInstruction)?,
            )
        }
        // 17 - Sync Native
        17 => {
            test_process_sync_native(
                accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?,
            )
        }
        // 18 - Initialize Account3
        18 => {
            test_process_initialize_account3(
                accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?, // CHANGE P-Token: accounts: &[AccountInfo; 2]
                instruction_data[1..].first_chunk().ok_or(TokenError::InvalidInstruction)?,
            )
        }
        // 19 - Initialize Multisig2
        19 => {
            test_process_initialize_multisig2(
                accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?,
                instruction_data[1..].first_chunk().ok_or(TokenError::InvalidInstruction)?,
            )
        }
        // 20 - Initialize Mint2 Freeze
        20 => {
            // #[cfg(feature = "logging")]
            // msg!("Testing Instruction: Initialize Mint2 Freeze");
            let [_d, payload @ ..] = instruction_data else {
                return Err(TokenError::InvalidInstruction.into());
            };
            match payload.len() {
                x if 66 <= x => {
                    test_process_initialize_mint2_freeze(
                        accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?, // CHANGE P-Token: accounts: &[AccountInfo; 1]
                        instruction_data[1..].first_chunk().ok_or(TokenError::InvalidInstruction)?,
                    )
                }
                x if 34 <= x => {
                    test_process_initialize_mint2_no_freeze(
                        accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?, // CHANGE P-Token: accounts: &[AccountInfo; 1]
                        instruction_data[1..].first_chunk().ok_or(TokenError::InvalidInstruction)?,
                    )
                }
                _ => Err(TokenError::InvalidInstruction.into()),
            }
        }
        // 21 - Get Account Data Size
        21 => {
            test_process_get_account_data_size(
                accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?,
            )
        }
        // 22 - Initialize Immutable Owner
        22 => {
            test_process_initialize_immutable_owner(
                accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?,
            )
        }
        // 23 - Amount To Ui Amount
        23 => {
            test_process_amount_to_ui_amount(
                accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?,
                instruction_data[1..].first_chunk().ok_or(TokenError::InvalidInstruction)?,
            )
        }
        // 24 - Ui Amount To Amount
        24 => {
            test_process_ui_amount_to_amount(
                accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?,
                &instruction_data[1..],
            )
        }
        // 38 - Withdraw Excess Lamports
        38 => {
            // #[cfg(feature = "logging")]
            // msg!("Testing Instruction: Withdraw Excess Lamports");
            test_process_withdraw_excess_lamports(
                program_id,
                accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?,
                instruction_data.first_chunk().ok_or(TokenError::InvalidInstruction)?,
            )
        }
        // For all other instructions, just call the regular processor
        _ => {
            Processor::process(program_id, accounts, instruction_data)
        }
    }
}

/// program_id // Token Program ID
/// accounts[0] // Mint Info
/// accounts[1] // Rent Sysvar Info
/// instruction_data[0] // Discriminator 0 (Initialize Mint Freeze)
/// instruction_data[0]      // Decimals
/// instruction_data[1..33]  // Mint Authority Pubkey
/// instruction_data[33]     // Freeze Authority Exists? 1 for freeze
/// instruction_data[34..66] // instruction_data[33] == 1 ==> Freeze Authority Pubkey
include!("../../shared/test_process_initialize_mint_freeze.rs");

/// program_id // Token Program ID
/// accounts[0] // Mint Info
/// accounts[1] // Rent Sysvar Info
/// instruction_data[0] // Discriminator 0 (Initialize Mint No Freeze)
/// instruction_data[1]      // Decimals
/// instruction_data[2..34]  // Mint Authority Pubkey
/// instruction_data[33]     // Freeze Authority Exists? 0 for no freeze
include!("../../shared/test_process_initialize_mint_no_freeze.rs");

/// program_id // Token Program ID
/// accounts[0] // Source Info
/// accounts[1] // Mint Info
/// accounts[2] // Authority Info
/// accounts[3..14] // Signers
/// instruction_data[0..9] // Little Endian Bytes of u64 amount, and decimals
include!("../../shared/test_process_burn_checked_multisig.rs");

/// program_id // Token Program ID
/// accounts[0] // New Account Info
/// accounts[1] // Mint Info
/// accounts[2] // Owner Info
/// accounts[3] // Rent Sysvar Info
include!("../../shared/test_process_initialize_account.rs");

/// program_id // Token Program ID
/// accounts[0]   // Multisig Info
/// accounts[1]   // Rent Sysvar Info
/// accounts[2..] // Signers
/// accounts[2..].len() // n
/// instruction_data[1] // m
include!("../../shared/test_process_initialize_multisig.rs");

/// program_id // Token Program ID
/// accounts[0] // Mint Info
/// accounts[1] // Destination Info
/// accounts[2] // Owner Info
/// accounts[3..14] // Signers
/// instruction_data[0..9] // Little Endian Bytes of u64 amount, and decimals
include!("../../shared/test_process_mint_to_checked_multisig.rs");

/// accounts[0] // Source Info
/// accounts[1] // Mint Info
/// accounts[2] // Authority Info
/// accounts[3..14] // Signers
/// instruction_data[0..8] // Little Endian Bytes of u64 amount
include!("../../shared/test_process_burn_multisig.rs");

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

/// program_id // Token Program ID
/// accounts[0] // Source Info
/// accounts[1] // Mint Info
/// accounts[2] // Destination Info
/// accounts[3] // Authority Info
/// accounts[4..15] // Signers
/// instruction_data[0..9] // Little Endian Bytes of u64 amount, and decimals
include!("../../shared/test_process_transfer_checked_multisig.rs");

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

/// program_id // Token Program ID
/// accounts[0] // Account Info - Account Case
/// accounts[1] // Authority Info
/// accounts[2..13] // Signers
/// instruction_data[0] // Discriminator 6 (Set Authority Account)
/// instruction_data[1] // Authority Type (instruction)
include!("../../shared/test_process_set_authority_account_multisig.rs");

/// program_id // Token Program ID
include!("../../shared/test_process_set_authority_mint.rs");

include!("../../shared/test_process_set_authority_mint_multisig.rs");

/// accounts[0] // Source Account Info
/// accounts[1] // Expected Mint Info
/// accounts[2] // Delegate Info
/// accounts[3] // Owner Info
/// accounts[4..15] // Signers
/// instruction_data[0..8] // Little Endian Bytes of u64 amount
/// instruction_data[8] // Decimals
include!("../../shared/test_process_approve_checked_multisig.rs");

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
/// accounts[1] // Destination Info
/// accounts[2] // Authority Info
include!("../../shared/test_process_close_account.rs");

/// accounts[0] // Source Info
/// accounts[1] // Destination Info
/// accounts[2] // Authority Info
/// accounts[3..14] // Multisig Signers
include!("../../shared/test_process_close_account_multisig.rs");

/// accounts[0] // Source Account Info
/// accounts[1] // Mint Info
/// accounts[2] // Authority Info
/// accounts[3..13] // Signers
include!("../../shared/test_process_freeze_account_multisig.rs");

/// accounts[0] // Source Account Info
/// accounts[1] // Mint Info
/// accounts[2] // Authority Info
include!("../../shared/test_process_freeze_account.rs");

/// accounts[0] // Source Account Info
/// accounts[1] // Mint Info
/// accounts[2] // Authority Info
include!("../../shared/test_process_thaw_account.rs");

/// accounts[0] // Source Account Info
/// accounts[1] // Mint Info
/// accounts[2] // Authority Info
/// accounts[3..13] // Signers
include!("../../shared/test_process_thaw_account_multisig.rs");

/// accounts[0] // Source Info
/// accounts[1] // Mint Info
/// accounts[2] // Destination Info
/// accounts[3] // Authority Info
/// instruction_data[0..9] // Little Endian Bytes of u64 amount, and decimals
include!("../../shared/test_process_transfer_checked.rs");

/// accounts[0] // Source Account Info
/// accounts[1] // Expected Mint Info
/// accounts[2] // Delegate Info
/// accounts[3] // Owner Info
/// instruction_data[0..8] // Little Endian Bytes of u64 amount
/// instruction_data[8] // Decimals
include!("../../shared/test_process_approve_checked.rs");

include!("../../shared/test_process_mint_to_checked.rs");

include!("../../shared/test_process_burn_checked.rs");

/// Unified harness for WithdrawExcessLamports instruction (discriminator 38).
///
/// This harness handles all source account types (Account, Mint, Multisig) and
/// authority types (single signer or multisig) in one function.
///
/// program_id      // Token Program ID
/// accounts[0]     // Source Account Info (Account, Mint, or Multisig)
/// accounts[1]     // Destination Info
/// accounts[2]     // Authority Info
/// accounts[3..]   // Signers (for multisig authority)
/// instruction_data[0] // Discriminator 38 (Withdraw Excess Lamports)
#[inline(never)]
fn test_process_withdraw_excess_lamports(
    program_id: &Pubkey,
    accounts: &[AccountInfo; 4],
    instruction_data: &[u8; 1],
) -> ProgramResult {
    // Constrain discriminator and program id
    unsafe { assume(38 == instruction_data[0]); }
    unsafe { assume(program_id == &crate::id()); }

    let instruction_data_with_discriminator = instruction_data;

    //-Process Instruction-----------------------------------------------------
    let result = Processor::process(program_id, accounts, instruction_data_with_discriminator);

    //-Assert Postconditions---------------------------------------------------
    // NOTE: WithdrawExcessLamports (discriminator 38) is a token-2022 instruction that does not
    // exist in the original spl-token program. The original spl-token's TokenInstruction only
    // supports discriminators 0-24. When Processor::process receives discriminator 38, it fails
    // at TokenInstruction::unpack() with InvalidInstruction error.
    assert_eq!(result, Err(TokenError::InvalidInstruction.into()));
    result
}

include!("../../shared/test_process_initialize_account2.rs");

include!("../../shared/test_process_sync_native.rs");

include!("../../shared/test_process_initialize_account3.rs");

/// program_id // Token Program ID
/// accounts[0]   // Multisig Info
/// accounts[1..] // Signers
/// accounts[1..].len() // n
/// instruction_data[1] // m
include!("../../shared/test_process_initialize_multisig2.rs");

include!("../../shared/test_process_initialize_mint2_freeze.rs");

include!("../../shared/test_process_initialize_mint2_no_freeze.rs");

/// accounts[0] // Mint Info
include!("../../shared/test_process_get_account_data_size.rs");

include!("../../shared/test_process_initialize_immutable_owner.rs");

include!("../../shared/test_process_amount_to_ui_amount.rs");

include!("../../shared/test_process_ui_amount_to_amount.rs");

