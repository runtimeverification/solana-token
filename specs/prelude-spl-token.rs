// =============================================================================
// Macros
// =============================================================================

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

// =============================================================================
// Cheatcodes
// =============================================================================

// TODO: Not sure if these are needed since there is no UB like p-token
#[inline(never)]
fn cheatcode_is_spl_account(_: &AccountInfo) {}
#[inline(never)]
fn cheatcode_is_spl_mint(_: &AccountInfo) {}
#[inline(never)]
fn cheatcode_is_spl_multisig(_: &AccountInfo) {}
#[inline(never)]
fn cheatcode_is_spl_rent(_: &AccountInfo) {}

// =============================================================================
// Wrappers
// =============================================================================

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
