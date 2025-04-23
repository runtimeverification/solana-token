#![feature(prelude_import)]
#![feature(fmt_helpers_for_derive)]
//#[prelude_import]
//use std::prelude::rust_2021::*;
//#[macro_use]
//extern crate std;
pub mod error {
    //! Error types
    use crate::pinocchio::program_error::ProgramError;
    /// Errors that may be returned by the Token program.
    pub enum TokenError {
        /// Lamport balance below rent-exempt threshold.
        NotRentExempt,
        /// Insufficient funds for the operation requested.
        InsufficientFunds,
        /// Invalid Mint.
        InvalidMint,
        /// Account not associated with this Mint.
        MintMismatch,
        /// Owner does not match.
        OwnerMismatch,
        /// This token's supply is fixed and new tokens cannot be minted.
        FixedSupply,
        /// The account cannot be initialized because it is already being used.
        AlreadyInUse,
        /// Invalid number of provided signers.
        InvalidNumberOfProvidedSigners,
        /// Invalid number of required signers.
        InvalidNumberOfRequiredSigners,
        /// State is uninitialized.
        UninitializedState,
        /// Instruction does not support native tokens
        NativeNotSupported,
        /// Non-native account can only be closed if its balance is zero
        NonNativeHasBalance,
        /// Invalid instruction
        InvalidInstruction,
        /// State is invalid for requested operation.
        InvalidState,
        /// Operation overflowed
        Overflow,
        /// Account does not support specified authority type.
        AuthorityTypeNotSupported,
        /// This token mint cannot freeze accounts.
        MintCannotFreeze,
        /// Account is frozen; all account operations will fail
        AccountFrozen,
        /// Mint decimals mismatch between the client and mint
        MintDecimalsMismatch,
        /// Instruction does not support non-native tokens
        NonNativeNotSupported,
    }
    #[automatically_derived]
    impl ::core::clone::Clone for TokenError {
        #[inline]
        fn clone(&self) -> TokenError {
            match self {
                TokenError::NotRentExempt => TokenError::NotRentExempt,
                TokenError::InsufficientFunds => TokenError::InsufficientFunds,
                TokenError::InvalidMint => TokenError::InvalidMint,
                TokenError::MintMismatch => TokenError::MintMismatch,
                TokenError::OwnerMismatch => TokenError::OwnerMismatch,
                TokenError::FixedSupply => TokenError::FixedSupply,
                TokenError::AlreadyInUse => TokenError::AlreadyInUse,
                TokenError::InvalidNumberOfProvidedSigners => {
                    TokenError::InvalidNumberOfProvidedSigners
                }
                TokenError::InvalidNumberOfRequiredSigners => {
                    TokenError::InvalidNumberOfRequiredSigners
                }
                TokenError::UninitializedState => TokenError::UninitializedState,
                TokenError::NativeNotSupported => TokenError::NativeNotSupported,
                TokenError::NonNativeHasBalance => TokenError::NonNativeHasBalance,
                TokenError::InvalidInstruction => TokenError::InvalidInstruction,
                TokenError::InvalidState => TokenError::InvalidState,
                TokenError::Overflow => TokenError::Overflow,
                TokenError::AuthorityTypeNotSupported => {
                    TokenError::AuthorityTypeNotSupported
                }
                TokenError::MintCannotFreeze => TokenError::MintCannotFreeze,
                TokenError::AccountFrozen => TokenError::AccountFrozen,
                TokenError::MintDecimalsMismatch => TokenError::MintDecimalsMismatch,
                TokenError::NonNativeNotSupported => TokenError::NonNativeNotSupported,
            }
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for TokenError {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    TokenError::NotRentExempt => "NotRentExempt",
                    TokenError::InsufficientFunds => "InsufficientFunds",
                    TokenError::InvalidMint => "InvalidMint",
                    TokenError::MintMismatch => "MintMismatch",
                    TokenError::OwnerMismatch => "OwnerMismatch",
                    TokenError::FixedSupply => "FixedSupply",
                    TokenError::AlreadyInUse => "AlreadyInUse",
                    TokenError::InvalidNumberOfProvidedSigners => {
                        "InvalidNumberOfProvidedSigners"
                    }
                    TokenError::InvalidNumberOfRequiredSigners => {
                        "InvalidNumberOfRequiredSigners"
                    }
                    TokenError::UninitializedState => "UninitializedState",
                    TokenError::NativeNotSupported => "NativeNotSupported",
                    TokenError::NonNativeHasBalance => "NonNativeHasBalance",
                    TokenError::InvalidInstruction => "InvalidInstruction",
                    TokenError::InvalidState => "InvalidState",
                    TokenError::Overflow => "Overflow",
                    TokenError::AuthorityTypeNotSupported => "AuthorityTypeNotSupported",
                    TokenError::MintCannotFreeze => "MintCannotFreeze",
                    TokenError::AccountFrozen => "AccountFrozen",
                    TokenError::MintDecimalsMismatch => "MintDecimalsMismatch",
                    TokenError::NonNativeNotSupported => "NonNativeNotSupported",
                },
            )
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for TokenError {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {}
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for TokenError {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for TokenError {
        #[inline]
        fn eq(&self, other: &TokenError) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    impl From<TokenError> for ProgramError {
        fn from(e: TokenError) -> Self {
            ProgramError::Custom(e as u32)
        }
    }
}
pub mod instruction {
    //! Instruction types.
    use crate::pinocchio::{program_error::ProgramError, pubkey::Pubkey};
    use super::error::TokenError;
    /// Instructions supported by the token program.
    #[repr(C)]
    pub enum TokenInstruction<'a> {
        /// Initializes a new mint and optionally deposits all the newly minted
        /// tokens in an account.
        ///
        /// The `InitializeMint` instruction requires no signers and MUST be
        /// included within the same Transaction as the system program's
        /// `CreateAccount` instruction that creates the account being initialized.
        /// Otherwise another party can acquire ownership of the uninitialized
        /// account.
        ///
        /// Accounts expected by this instruction:
        ///
        ///   0. `[writable]` The mint to initialize.
        ///   1. `[]` Rent sysvar
        InitializeMint {
            /// Number of base 10 digits to the right of the decimal place.
            decimals: u8,
            /// The authority/multisignature to mint tokens.
            mint_authority: Pubkey,
            /// The freeze authority/multisignature of the mint.
            freeze_authority: Option<Pubkey>,
        },
        /// Initializes a new account to hold tokens.  If this account is associated
        /// with the native mint then the token balance of the initialized account
        /// will be equal to the amount of SOL in the account. If this account is
        /// associated with another mint, that mint must be initialized before this
        /// command can succeed.
        ///
        /// The [`InitializeAccount`] instruction requires no signers and MUST be
        /// included within the same Transaction as the system program's
        /// `CreateAccount` instruction that creates the account being initialized.
        /// Otherwise another party can acquire ownership of the uninitialized
        /// account.
        ///
        /// Accounts expected by this instruction:
        ///
        ///   0. `[writable]`  The account to initialize.
        ///   1. `[]` The mint this account will be associated with.
        ///   2. `[]` The new account's owner/multisignature.
        ///   3. `[]` Rent sysvar
        InitializeAccount,
        /// Initializes a multisignature account with N provided signers.
        ///
        /// Multisignature accounts can used in place of any single owner/delegate
        /// accounts in any token instruction that require an owner/delegate to be
        /// present.  The variant field represents the number of signers (M)
        /// required to validate this multisignature account.
        ///
        /// The [`InitializeMultisig`] instruction requires no signers and MUST be
        /// included within the same Transaction as the system program's
        /// `CreateAccount` instruction that creates the account being initialized.
        /// Otherwise another party can acquire ownership of the uninitialized
        /// account.
        ///
        /// Accounts expected by this instruction:
        ///
        ///   0. `[writable]` The multisignature account to initialize.
        ///   1. `[]` Rent sysvar
        ///   2. `..+N` `[signer]` The signer accounts, must equal to N where `1 <= N <= 11`.
        InitializeMultisig {
            /// The number of signers (M) required to validate this multisignature
            /// account.
            m: u8,
        },
        /// Transfers tokens from one account to another either directly or via a
        /// delegate.  If this account is associated with the native mint then equal
        /// amounts of SOL and Tokens will be transferred to the destination
        /// account.
        ///
        /// Accounts expected by this instruction:
        ///
        ///   * Single owner/delegate
        ///   0. `[writable]` The source account.
        ///   1. `[writable]` The destination account.
        ///   2. `[signer]` The source account's owner/delegate.
        ///
        ///   * Multisignature owner/delegate
        ///   0. `[writable]` The source account.
        ///   1. `[writable]` The destination account.
        ///   2. `[]` The source account's multisignature owner/delegate.
        ///   3. `..+M` `[signer]` M signer accounts.
        Transfer {
            /// The amount of tokens to transfer.
            amount: u64,
        },
        /// Approves a delegate.  A delegate is given the authority over tokens on
        /// behalf of the source account's owner.
        ///
        /// Accounts expected by this instruction:
        ///
        ///   * Single owner
        ///   0. `[writable]` The source account.
        ///   1. `[]` The delegate.
        ///   2. `[signer]` The source account owner.
        ///
        ///   * Multisignature owner
        ///   0. `[writable]` The source account.
        ///   1. `[]` The delegate.
        ///   2. `[]` The source account's multisignature owner.
        ///   3. `..+M` `[signer]` M signer accounts
        Approve {
            /// The amount of tokens the delegate is approved for.
            amount: u64,
        },
        /// Revokes the delegate's authority.
        ///
        /// Accounts expected by this instruction:
        ///
        ///   * Single owner
        ///   0. `[writable]` The source account.
        ///   1. `[signer]` The source account owner.
        ///
        ///   * Multisignature owner
        ///   0. `[writable]` The source account.
        ///   1. `[]` The source account's multisignature owner.
        ///   2. `..+M` `[signer]` M signer accounts
        Revoke,
        /// Sets a new authority of a mint or account.
        ///
        /// Accounts expected by this instruction:
        ///
        ///   * Single authority
        ///   0. `[writable]` The mint or account to change the authority of.
        ///   1. `[signer]` The current authority of the mint or account.
        ///
        ///   * Multisignature authority
        ///   0. `[writable]` The mint or account to change the authority of.
        ///   1. `[]` The mint's or account's current multisignature authority.
        ///   2. `..+M` `[signer]` M signer accounts
        SetAuthority {
            /// The type of authority to update.
            authority_type: AuthorityType,
            /// The new authority
            new_authority: Option<Pubkey>,
        },
        /// Mints new tokens to an account.  The native mint does not support
        /// minting.
        ///
        /// Accounts expected by this instruction:
        ///
        ///   * Single authority
        ///   0. `[writable]` The mint.
        ///   1. `[writable]` The account to mint tokens to.
        ///   2. `[signer]` The mint's minting authority.
        ///
        ///   * Multisignature authority
        ///   0. `[writable]` The mint.
        ///   1. `[writable]` The account to mint tokens to.
        ///   2. `[]` The mint's multisignature mint-tokens authority.
        ///   3. `..+M` `[signer]` M signer accounts.
        MintTo {
            /// The amount of new tokens to mint.
            amount: u64,
        },
        /// Burns tokens by removing them from an account.  `Burn` does not support
        /// accounts associated with the native mint, use `CloseAccount` instead.
        ///
        /// Accounts expected by this instruction:
        ///
        ///   * Single owner/delegate
        ///   0. `[writable]` The account to burn from.
        ///   1. `[writable]` The token mint.
        ///   2. `[signer]` The account's owner/delegate.
        ///
        ///   * Multisignature owner/delegate
        ///   0. `[writable]` The account to burn from.
        ///   1. `[writable]` The token mint.
        ///   2. `[]` The account's multisignature owner/delegate.
        ///   3. `..+M` `[signer]` M signer accounts.
        Burn {
            /// The amount of tokens to burn.
            amount: u64,
        },
        /// Close an account by transferring all its SOL to the destination account.
        /// Non-native accounts may only be closed if its token amount is zero.
        ///
        /// Accounts expected by this instruction:
        ///
        ///   * Single owner
        ///   0. `[writable]` The account to close.
        ///   1. `[writable]` The destination account.
        ///   2. `[signer]` The account's owner.
        ///
        ///   * Multisignature owner
        ///   0. `[writable]` The account to close.
        ///   1. `[writable]` The destination account.
        ///   2. `[]` The account's multisignature owner.
        ///   3. `..+M` `[signer]` M signer accounts.
        CloseAccount,
        /// Freeze an Initialized account using the Mint's [`freeze_authority`] (if
        /// set).
        ///
        /// Accounts expected by this instruction:
        ///
        ///   * Single owner
        ///   0. `[writable]` The account to freeze.
        ///   1. `[]` The token mint.
        ///   2. `[signer]` The mint freeze authority.
        ///
        ///   * Multisignature owner
        ///   0. `[writable]` The account to freeze.
        ///   1. `[]` The token mint.
        ///   2. `[]` The mint's multisignature freeze authority.
        ///   3. `..+M` `[signer]` M signer accounts.
        FreezeAccount,
        /// Thaw a Frozen account using the Mint's [`freeze_authority`] (if set).
        ///
        /// Accounts expected by this instruction:
        ///
        ///   * Single owner
        ///   0. `[writable]` The account to freeze.
        ///   1. `[]` The token mint.
        ///   2. `[signer]` The mint freeze authority.
        ///
        ///   * Multisignature owner
        ///   0. `[writable]` The account to freeze.
        ///   1. `[]` The token mint.
        ///   2. `[]` The mint's multisignature freeze authority.
        ///   3. `..+M` `[signer]` M signer accounts.
        ThawAccount,
        /// Transfers tokens from one account to another either directly or via a
        /// delegate.  If this account is associated with the native mint then equal
        /// amounts of SOL and Tokens will be transferred to the destination
        /// account.
        ///
        /// This instruction differs from Transfer in that the token mint and
        /// decimals value is checked by the caller.  This may be useful when
        /// creating transactions offline or within a hardware wallet.
        ///
        /// Accounts expected by this instruction:
        ///
        ///   * Single owner/delegate
        ///   0. `[writable]` The source account.
        ///   1. `[]` The token mint.
        ///   2. `[writable]` The destination account.
        ///   3. `[signer]` The source account's owner/delegate.
        ///
        ///   * Multisignature owner/delegate
        ///   0. `[writable]` The source account.
        ///   1. `[]` The token mint.
        ///   2. `[writable]` The destination account.
        ///   3. `[]` The source account's multisignature owner/delegate.
        ///   4. `..+M` `[signer]` M signer accounts.
        TransferChecked {
            /// The amount of tokens to transfer.
            amount: u64,
            /// Expected number of base 10 digits to the right of the decimal place.
            decimals: u8,
        },
        /// Approves a delegate.  A delegate is given the authority over tokens on
        /// behalf of the source account's owner.
        ///
        /// This instruction differs from Approve in that the token mint and
        /// decimals value is checked by the caller.  This may be useful when
        /// creating transactions offline or within a hardware wallet.
        ///
        /// Accounts expected by this instruction:
        ///
        ///   * Single owner
        ///   0. `[writable]` The source account.
        ///   1. `[]` The token mint.
        ///   2. `[]` The delegate.
        ///   3. `[signer]` The source account owner.
        ///
        ///   * Multisignature owner
        ///   0. `[writable]` The source account.
        ///   1. `[]` The token mint.
        ///   2. `[]` The delegate.
        ///   3. `[]` The source account's multisignature owner.
        ///   4. `..+M` `[signer]` M signer accounts
        ApproveChecked {
            /// The amount of tokens the delegate is approved for.
            amount: u64,
            /// Expected number of base 10 digits to the right of the decimal place.
            decimals: u8,
        },
        /// Mints new tokens to an account.  The native mint does not support
        /// minting.
        ///
        /// This instruction differs from [`MintTo`] in that the decimals value is
        /// checked by the caller.  This may be useful when creating transactions
        /// offline or within a hardware wallet.
        ///
        /// Accounts expected by this instruction:
        ///
        ///   * Single authority
        ///   0. `[writable]` The mint.
        ///   1. `[writable]` The account to mint tokens to.
        ///   2. `[signer]` The mint's minting authority.
        ///
        ///   * Multisignature authority
        ///   0. `[writable]` The mint.
        ///   1. `[writable]` The account to mint tokens to.
        ///   2. `[]` The mint's multisignature mint-tokens authority.
        ///   3. `..+M` `[signer]` M signer accounts.
        MintToChecked {
            /// The amount of new tokens to mint.
            amount: u64,
            /// Expected number of base 10 digits to the right of the decimal place.
            decimals: u8,
        },
        /// Burns tokens by removing them from an account.  [`BurnChecked`] does not
        /// support accounts associated with the native mint, use `CloseAccount`
        /// instead.
        ///
        /// This instruction differs from Burn in that the decimals value is checked
        /// by the caller. This may be useful when creating transactions offline or
        /// within a hardware wallet.
        ///
        /// Accounts expected by this instruction:
        ///
        ///   * Single owner/delegate
        ///   0. `[writable]` The account to burn from.
        ///   1. `[writable]` The token mint.
        ///   2. `[signer]` The account's owner/delegate.
        ///
        ///   * Multisignature owner/delegate
        ///   0. `[writable]` The account to burn from.
        ///   1. `[writable]` The token mint.
        ///   2. `[]` The account's multisignature owner/delegate.
        ///   3. `..+M` `[signer]` M signer accounts.
        BurnChecked {
            /// The amount of tokens to burn.
            amount: u64,
            /// Expected number of base 10 digits to the right of the decimal place.
            decimals: u8,
        },
        /// Like [`InitializeAccount`], but the owner pubkey is passed via instruction
        /// data rather than the accounts list. This variant may be preferable
        /// when using Cross Program Invocation from an instruction that does
        /// not need the owner's `AccountInfo` otherwise.
        ///
        /// Accounts expected by this instruction:
        ///
        ///   0. `[writable]`  The account to initialize.
        ///   1. `[]` The mint this account will be associated with.
        ///   3. `[]` Rent sysvar
        InitializeAccount2 {
            /// The new account's owner/multisignature.
            owner: Pubkey,
        },
        /// Given a wrapped / native token account (a token account containing SOL)
        /// updates its amount field based on the account's underlying `lamports`.
        /// This is useful if a non-wrapped SOL account uses
        /// `system_instruction::transfer` to move lamports to a wrapped token
        /// account, and needs to have its token `amount` field updated.
        ///
        /// Accounts expected by this instruction:
        ///
        ///   0. `[writable]`  The native token account to sync with its underlying
        ///      lamports.
        SyncNative,
        /// Like [`InitializeAccount2`], but does not require the Rent sysvar to be
        /// provided
        ///
        /// Accounts expected by this instruction:
        ///
        ///   0. `[writable]`  The account to initialize.
        ///   1. `[]` The mint this account will be associated with.
        InitializeAccount3 {
            /// The new account's owner/multisignature.
            owner: Pubkey,
        },
        /// Like [`InitializeMultisig`], but does not require the Rent sysvar to be
        /// provided
        ///
        /// Accounts expected by this instruction:
        ///
        ///   0. `[writable]` The multisignature account to initialize.
        ///   1. `..+N` `[signer]` The signer accounts, must equal to N where `1 <= N <= 11`.
        InitializeMultisig2 {
            /// The number of signers (M) required to validate this multisignature
            /// account.
            m: u8,
        },
        /// Like [`InitializeMint`], but does not require the Rent sysvar to be
        /// provided
        ///
        /// Accounts expected by this instruction:
        ///
        ///   0. `[writable]` The mint to initialize.
        InitializeMint2 {
            /// Number of base 10 digits to the right of the decimal place.
            decimals: u8,
            /// The authority/multisignature to mint tokens.
            mint_authority: Pubkey,
            /// The freeze authority/multisignature of the mint.
            freeze_authority: Option<Pubkey>,
        },
        /// Gets the required size of an account for the given mint as a
        /// little-endian `u64`.
        ///
        /// Return data can be fetched using `sol_get_return_data` and deserializing
        /// the return data as a little-endian `u64`.
        ///
        /// Accounts expected by this instruction:
        ///
        ///   0. `[]` The mint to calculate for
        GetAccountDataSize,
        /// Initialize the Immutable Owner extension for the given token account
        ///
        /// Fails if the account has already been initialized, so must be called
        /// before [`InitializeAccount`].
        ///
        /// No-ops in this version of the program, but is included for compatibility
        /// with the Associated Token Account program.
        ///
        /// Accounts expected by this instruction:
        ///
        ///   0. `[writable]`  The account to initialize.
        ///
        /// Data expected by this instruction:
        ///   None
        InitializeImmutableOwner,
        /// Convert an Amount of tokens to a `UiAmount` `string`, using the given
        /// mint. In this version of the program, the mint can only specify the
        /// number of decimals.
        ///
        /// Fails on an invalid mint.
        ///
        /// Return data can be fetched using `sol_get_return_data` and deserialized
        /// with `String::from_utf8`.
        ///
        /// Accounts expected by this instruction:
        ///
        ///   0. `[]` The mint to calculate for
        AmountToUiAmount {
            /// The amount of tokens to reformat.
            amount: u64,
        },
        /// Convert a `UiAmount` of tokens to a little-endian `u64` raw Amount, using
        /// the given mint. In this version of the program, the mint can only
        /// specify the number of decimals.
        ///
        /// Return data can be fetched using `sol_get_return_data` and deserializing
        /// the return data as a little-endian `u64`.
        ///
        /// Accounts expected by this instruction:
        ///
        ///   0. `[]` The mint to calculate for
        UiAmountToAmount {
            /// The `ui_amount` of tokens to reformat.
            ui_amount: &'a str,
        },
    }
    #[automatically_derived]
    impl<'a> ::core::clone::Clone for TokenInstruction<'a> {
        #[inline]
        fn clone(&self) -> TokenInstruction<'a> {
            match self {
                TokenInstruction::InitializeMint {
                    decimals: __self_0,
                    mint_authority: __self_1,
                    freeze_authority: __self_2,
                } => {
                    TokenInstruction::InitializeMint {
                        decimals: ::core::clone::Clone::clone(__self_0),
                        mint_authority: ::core::clone::Clone::clone(__self_1),
                        freeze_authority: ::core::clone::Clone::clone(__self_2),
                    }
                }
                TokenInstruction::InitializeAccount => {
                    TokenInstruction::InitializeAccount
                }
                TokenInstruction::InitializeMultisig { m: __self_0 } => {
                    TokenInstruction::InitializeMultisig {
                        m: ::core::clone::Clone::clone(__self_0),
                    }
                }
                TokenInstruction::Transfer { amount: __self_0 } => {
                    TokenInstruction::Transfer {
                        amount: ::core::clone::Clone::clone(__self_0),
                    }
                }
                TokenInstruction::Approve { amount: __self_0 } => {
                    TokenInstruction::Approve {
                        amount: ::core::clone::Clone::clone(__self_0),
                    }
                }
                TokenInstruction::Revoke => TokenInstruction::Revoke,
                TokenInstruction::SetAuthority {
                    authority_type: __self_0,
                    new_authority: __self_1,
                } => {
                    TokenInstruction::SetAuthority {
                        authority_type: ::core::clone::Clone::clone(__self_0),
                        new_authority: ::core::clone::Clone::clone(__self_1),
                    }
                }
                TokenInstruction::MintTo { amount: __self_0 } => {
                    TokenInstruction::MintTo {
                        amount: ::core::clone::Clone::clone(__self_0),
                    }
                }
                TokenInstruction::Burn { amount: __self_0 } => {
                    TokenInstruction::Burn {
                        amount: ::core::clone::Clone::clone(__self_0),
                    }
                }
                TokenInstruction::CloseAccount => TokenInstruction::CloseAccount,
                TokenInstruction::FreezeAccount => TokenInstruction::FreezeAccount,
                TokenInstruction::ThawAccount => TokenInstruction::ThawAccount,
                TokenInstruction::TransferChecked {
                    amount: __self_0,
                    decimals: __self_1,
                } => {
                    TokenInstruction::TransferChecked {
                        amount: ::core::clone::Clone::clone(__self_0),
                        decimals: ::core::clone::Clone::clone(__self_1),
                    }
                }
                TokenInstruction::ApproveChecked {
                    amount: __self_0,
                    decimals: __self_1,
                } => {
                    TokenInstruction::ApproveChecked {
                        amount: ::core::clone::Clone::clone(__self_0),
                        decimals: ::core::clone::Clone::clone(__self_1),
                    }
                }
                TokenInstruction::MintToChecked {
                    amount: __self_0,
                    decimals: __self_1,
                } => {
                    TokenInstruction::MintToChecked {
                        amount: ::core::clone::Clone::clone(__self_0),
                        decimals: ::core::clone::Clone::clone(__self_1),
                    }
                }
                TokenInstruction::BurnChecked {
                    amount: __self_0,
                    decimals: __self_1,
                } => {
                    TokenInstruction::BurnChecked {
                        amount: ::core::clone::Clone::clone(__self_0),
                        decimals: ::core::clone::Clone::clone(__self_1),
                    }
                }
                TokenInstruction::InitializeAccount2 { owner: __self_0 } => {
                    TokenInstruction::InitializeAccount2 {
                        owner: ::core::clone::Clone::clone(__self_0),
                    }
                }
                TokenInstruction::SyncNative => TokenInstruction::SyncNative,
                TokenInstruction::InitializeAccount3 { owner: __self_0 } => {
                    TokenInstruction::InitializeAccount3 {
                        owner: ::core::clone::Clone::clone(__self_0),
                    }
                }
                TokenInstruction::InitializeMultisig2 { m: __self_0 } => {
                    TokenInstruction::InitializeMultisig2 {
                        m: ::core::clone::Clone::clone(__self_0),
                    }
                }
                TokenInstruction::InitializeMint2 {
                    decimals: __self_0,
                    mint_authority: __self_1,
                    freeze_authority: __self_2,
                } => {
                    TokenInstruction::InitializeMint2 {
                        decimals: ::core::clone::Clone::clone(__self_0),
                        mint_authority: ::core::clone::Clone::clone(__self_1),
                        freeze_authority: ::core::clone::Clone::clone(__self_2),
                    }
                }
                TokenInstruction::GetAccountDataSize => {
                    TokenInstruction::GetAccountDataSize
                }
                TokenInstruction::InitializeImmutableOwner => {
                    TokenInstruction::InitializeImmutableOwner
                }
                TokenInstruction::AmountToUiAmount { amount: __self_0 } => {
                    TokenInstruction::AmountToUiAmount {
                        amount: ::core::clone::Clone::clone(__self_0),
                    }
                }
                TokenInstruction::UiAmountToAmount { ui_amount: __self_0 } => {
                    TokenInstruction::UiAmountToAmount {
                        ui_amount: ::core::clone::Clone::clone(__self_0),
                    }
                }
            }
        }
    }
    #[automatically_derived]
    impl<'a> ::core::fmt::Debug for TokenInstruction<'a> {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                TokenInstruction::InitializeMint {
                    decimals: __self_0,
                    mint_authority: __self_1,
                    freeze_authority: __self_2,
                } => {
                    ::core::fmt::Formatter::debug_struct_field3_finish(
                        f,
                        "InitializeMint",
                        "decimals",
                        __self_0,
                        "mint_authority",
                        __self_1,
                        "freeze_authority",
                        &__self_2,
                    )
                }
                TokenInstruction::InitializeAccount => {
                    ::core::fmt::Formatter::write_str(f, "InitializeAccount")
                }
                TokenInstruction::InitializeMultisig { m: __self_0 } => {
                    ::core::fmt::Formatter::debug_struct_field1_finish(
                        f,
                        "InitializeMultisig",
                        "m",
                        &__self_0,
                    )
                }
                TokenInstruction::Transfer { amount: __self_0 } => {
                    ::core::fmt::Formatter::debug_struct_field1_finish(
                        f,
                        "Transfer",
                        "amount",
                        &__self_0,
                    )
                }
                TokenInstruction::Approve { amount: __self_0 } => {
                    ::core::fmt::Formatter::debug_struct_field1_finish(
                        f,
                        "Approve",
                        "amount",
                        &__self_0,
                    )
                }
                TokenInstruction::Revoke => {
                    ::core::fmt::Formatter::write_str(f, "Revoke")
                }
                TokenInstruction::SetAuthority {
                    authority_type: __self_0,
                    new_authority: __self_1,
                } => {
                    ::core::fmt::Formatter::debug_struct_field2_finish(
                        f,
                        "SetAuthority",
                        "authority_type",
                        __self_0,
                        "new_authority",
                        &__self_1,
                    )
                }
                TokenInstruction::MintTo { amount: __self_0 } => {
                    ::core::fmt::Formatter::debug_struct_field1_finish(
                        f,
                        "MintTo",
                        "amount",
                        &__self_0,
                    )
                }
                TokenInstruction::Burn { amount: __self_0 } => {
                    ::core::fmt::Formatter::debug_struct_field1_finish(
                        f,
                        "Burn",
                        "amount",
                        &__self_0,
                    )
                }
                TokenInstruction::CloseAccount => {
                    ::core::fmt::Formatter::write_str(f, "CloseAccount")
                }
                TokenInstruction::FreezeAccount => {
                    ::core::fmt::Formatter::write_str(f, "FreezeAccount")
                }
                TokenInstruction::ThawAccount => {
                    ::core::fmt::Formatter::write_str(f, "ThawAccount")
                }
                TokenInstruction::TransferChecked {
                    amount: __self_0,
                    decimals: __self_1,
                } => {
                    ::core::fmt::Formatter::debug_struct_field2_finish(
                        f,
                        "TransferChecked",
                        "amount",
                        __self_0,
                        "decimals",
                        &__self_1,
                    )
                }
                TokenInstruction::ApproveChecked {
                    amount: __self_0,
                    decimals: __self_1,
                } => {
                    ::core::fmt::Formatter::debug_struct_field2_finish(
                        f,
                        "ApproveChecked",
                        "amount",
                        __self_0,
                        "decimals",
                        &__self_1,
                    )
                }
                TokenInstruction::MintToChecked {
                    amount: __self_0,
                    decimals: __self_1,
                } => {
                    ::core::fmt::Formatter::debug_struct_field2_finish(
                        f,
                        "MintToChecked",
                        "amount",
                        __self_0,
                        "decimals",
                        &__self_1,
                    )
                }
                TokenInstruction::BurnChecked {
                    amount: __self_0,
                    decimals: __self_1,
                } => {
                    ::core::fmt::Formatter::debug_struct_field2_finish(
                        f,
                        "BurnChecked",
                        "amount",
                        __self_0,
                        "decimals",
                        &__self_1,
                    )
                }
                TokenInstruction::InitializeAccount2 { owner: __self_0 } => {
                    ::core::fmt::Formatter::debug_struct_field1_finish(
                        f,
                        "InitializeAccount2",
                        "owner",
                        &__self_0,
                    )
                }
                TokenInstruction::SyncNative => {
                    ::core::fmt::Formatter::write_str(f, "SyncNative")
                }
                TokenInstruction::InitializeAccount3 { owner: __self_0 } => {
                    ::core::fmt::Formatter::debug_struct_field1_finish(
                        f,
                        "InitializeAccount3",
                        "owner",
                        &__self_0,
                    )
                }
                TokenInstruction::InitializeMultisig2 { m: __self_0 } => {
                    ::core::fmt::Formatter::debug_struct_field1_finish(
                        f,
                        "InitializeMultisig2",
                        "m",
                        &__self_0,
                    )
                }
                TokenInstruction::InitializeMint2 {
                    decimals: __self_0,
                    mint_authority: __self_1,
                    freeze_authority: __self_2,
                } => {
                    ::core::fmt::Formatter::debug_struct_field3_finish(
                        f,
                        "InitializeMint2",
                        "decimals",
                        __self_0,
                        "mint_authority",
                        __self_1,
                        "freeze_authority",
                        &__self_2,
                    )
                }
                TokenInstruction::GetAccountDataSize => {
                    ::core::fmt::Formatter::write_str(f, "GetAccountDataSize")
                }
                TokenInstruction::InitializeImmutableOwner => {
                    ::core::fmt::Formatter::write_str(f, "InitializeImmutableOwner")
                }
                TokenInstruction::AmountToUiAmount { amount: __self_0 } => {
                    ::core::fmt::Formatter::debug_struct_field1_finish(
                        f,
                        "AmountToUiAmount",
                        "amount",
                        &__self_0,
                    )
                }
                TokenInstruction::UiAmountToAmount { ui_amount: __self_0 } => {
                    ::core::fmt::Formatter::debug_struct_field1_finish(
                        f,
                        "UiAmountToAmount",
                        "ui_amount",
                        &__self_0,
                    )
                }
            }
        }
    }
    #[automatically_derived]
    impl<'a> ::core::marker::StructuralPartialEq for TokenInstruction<'a> {}
    #[automatically_derived]
    impl<'a> ::core::cmp::PartialEq for TokenInstruction<'a> {
        #[inline]
        fn eq(&self, other: &TokenInstruction<'a>) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
                && match (self, other) {
                    (
                        TokenInstruction::InitializeMint {
                            decimals: __self_0,
                            mint_authority: __self_1,
                            freeze_authority: __self_2,
                        },
                        TokenInstruction::InitializeMint {
                            decimals: __arg1_0,
                            mint_authority: __arg1_1,
                            freeze_authority: __arg1_2,
                        },
                    ) => {
                        __self_0 == __arg1_0 && __self_1 == __arg1_1
                            && __self_2 == __arg1_2
                    }
                    (
                        TokenInstruction::InitializeMultisig { m: __self_0 },
                        TokenInstruction::InitializeMultisig { m: __arg1_0 },
                    ) => __self_0 == __arg1_0,
                    (
                        TokenInstruction::Transfer { amount: __self_0 },
                        TokenInstruction::Transfer { amount: __arg1_0 },
                    ) => __self_0 == __arg1_0,
                    (
                        TokenInstruction::Approve { amount: __self_0 },
                        TokenInstruction::Approve { amount: __arg1_0 },
                    ) => __self_0 == __arg1_0,
                    (
                        TokenInstruction::SetAuthority {
                            authority_type: __self_0,
                            new_authority: __self_1,
                        },
                        TokenInstruction::SetAuthority {
                            authority_type: __arg1_0,
                            new_authority: __arg1_1,
                        },
                    ) => __self_0 == __arg1_0 && __self_1 == __arg1_1,
                    (
                        TokenInstruction::MintTo { amount: __self_0 },
                        TokenInstruction::MintTo { amount: __arg1_0 },
                    ) => __self_0 == __arg1_0,
                    (
                        TokenInstruction::Burn { amount: __self_0 },
                        TokenInstruction::Burn { amount: __arg1_0 },
                    ) => __self_0 == __arg1_0,
                    (
                        TokenInstruction::TransferChecked {
                            amount: __self_0,
                            decimals: __self_1,
                        },
                        TokenInstruction::TransferChecked {
                            amount: __arg1_0,
                            decimals: __arg1_1,
                        },
                    ) => __self_0 == __arg1_0 && __self_1 == __arg1_1,
                    (
                        TokenInstruction::ApproveChecked {
                            amount: __self_0,
                            decimals: __self_1,
                        },
                        TokenInstruction::ApproveChecked {
                            amount: __arg1_0,
                            decimals: __arg1_1,
                        },
                    ) => __self_0 == __arg1_0 && __self_1 == __arg1_1,
                    (
                        TokenInstruction::MintToChecked {
                            amount: __self_0,
                            decimals: __self_1,
                        },
                        TokenInstruction::MintToChecked {
                            amount: __arg1_0,
                            decimals: __arg1_1,
                        },
                    ) => __self_0 == __arg1_0 && __self_1 == __arg1_1,
                    (
                        TokenInstruction::BurnChecked {
                            amount: __self_0,
                            decimals: __self_1,
                        },
                        TokenInstruction::BurnChecked {
                            amount: __arg1_0,
                            decimals: __arg1_1,
                        },
                    ) => __self_0 == __arg1_0 && __self_1 == __arg1_1,
                    (
                        TokenInstruction::InitializeAccount2 { owner: __self_0 },
                        TokenInstruction::InitializeAccount2 { owner: __arg1_0 },
                    ) => __self_0 == __arg1_0,
                    (
                        TokenInstruction::InitializeAccount3 { owner: __self_0 },
                        TokenInstruction::InitializeAccount3 { owner: __arg1_0 },
                    ) => __self_0 == __arg1_0,
                    (
                        TokenInstruction::InitializeMultisig2 { m: __self_0 },
                        TokenInstruction::InitializeMultisig2 { m: __arg1_0 },
                    ) => __self_0 == __arg1_0,
                    (
                        TokenInstruction::InitializeMint2 {
                            decimals: __self_0,
                            mint_authority: __self_1,
                            freeze_authority: __self_2,
                        },
                        TokenInstruction::InitializeMint2 {
                            decimals: __arg1_0,
                            mint_authority: __arg1_1,
                            freeze_authority: __arg1_2,
                        },
                    ) => {
                        __self_0 == __arg1_0 && __self_1 == __arg1_1
                            && __self_2 == __arg1_2
                    }
                    (
                        TokenInstruction::AmountToUiAmount { amount: __self_0 },
                        TokenInstruction::AmountToUiAmount { amount: __arg1_0 },
                    ) => __self_0 == __arg1_0,
                    (
                        TokenInstruction::UiAmountToAmount { ui_amount: __self_0 },
                        TokenInstruction::UiAmountToAmount { ui_amount: __arg1_0 },
                    ) => __self_0 == __arg1_0,
                    _ => true,
                }
        }
    }
    /// Specifies the authority type for `SetAuthority` instructions
    #[repr(u8)]
    pub enum AuthorityType {
        /// Authority to mint new tokens
        MintTokens,
        /// Authority to freeze any account associated with the Mint
        FreezeAccount,
        /// Owner of a given token account
        AccountOwner,
        /// Authority to close a token account
        CloseAccount,
    }
    #[automatically_derived]
    impl ::core::clone::Clone for AuthorityType {
        #[inline]
        fn clone(&self) -> AuthorityType {
            match self {
                AuthorityType::MintTokens => AuthorityType::MintTokens,
                AuthorityType::FreezeAccount => AuthorityType::FreezeAccount,
                AuthorityType::AccountOwner => AuthorityType::AccountOwner,
                AuthorityType::CloseAccount => AuthorityType::CloseAccount,
            }
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for AuthorityType {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    AuthorityType::MintTokens => "MintTokens",
                    AuthorityType::FreezeAccount => "FreezeAccount",
                    AuthorityType::AccountOwner => "AccountOwner",
                    AuthorityType::CloseAccount => "CloseAccount",
                },
            )
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for AuthorityType {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for AuthorityType {
        #[inline]
        fn eq(&self, other: &AuthorityType) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    impl AuthorityType {
        pub fn into(&self) -> u8 {
            match self {
                AuthorityType::MintTokens => 0,
                AuthorityType::FreezeAccount => 1,
                AuthorityType::AccountOwner => 2,
                AuthorityType::CloseAccount => 3,
            }
        }
        pub fn from(index: u8) -> Result<Self, ProgramError> {
            match index {
                0 => Ok(AuthorityType::MintTokens),
                1 => Ok(AuthorityType::FreezeAccount),
                2 => Ok(AuthorityType::AccountOwner),
                3 => Ok(AuthorityType::CloseAccount),
                _ => Err(TokenError::InvalidInstruction.into()),
            }
        }
    }
}
pub mod native_mint {
    //! The Mint that represents the native token.
    use crate::pinocchio::pubkey::Pubkey;
    /// There are `10^9` lamports in one SOL
    pub const DECIMALS: u8 = 9;
    pub const ID: Pubkey = crate::pinocchio_pubkey::from_str(
        "So11111111111111111111111111111111111111112",
    );
    #[inline(always)]
    pub fn is_native_mint(mint: &Pubkey) -> bool {
        mint == &ID
    }
}
pub mod state {
    use crate::pinocchio::program_error::ProgramError;
    pub mod account {
        use crate::pinocchio::pubkey::Pubkey;
        use super::{account_state::AccountState, COption, Initializable, RawType};
        /// Incinerator address.
        const INCINERATOR_ID: Pubkey = crate::pinocchio_pubkey::from_str(
            "1nc1nerator11111111111111111111111111111111",
        );
        /// System program id.
        const SYSTEM_PROGRAM_ID: Pubkey = crate::pinocchio_pubkey::from_str(
            "11111111111111111111111111111111",
        );
        /// Internal representation of a token account data.
        #[repr(C)]
        pub struct Account {
            /// The mint associated with this account
            pub mint: Pubkey,
            /// The owner of this account.
            pub owner: Pubkey,
            /// The amount of tokens this account holds.
            amount: [u8; 8],
            /// If `delegate` is `Some` then `delegated_amount` represents
            /// the amount authorized by the delegate.
            delegate: COption<Pubkey>,
            /// The account's state.
            pub state: AccountState,
            /// Indicates whether this account represents a native token or not.
            is_native: [u8; 4],
            /// If `is_native.is_some`, this is a native token, and the value logs the
            /// rent-exempt reserve. An Account is required to be rent-exempt, so
            /// the value is used by the Processor to ensure that wrapped SOL
            /// accounts do not drop below this threshold.
            native_amount: [u8; 8],
            /// The amount delegated.
            delegated_amount: [u8; 8],
            /// Optional authority to close the account.
            close_authority: COption<Pubkey>,
        }
        impl Account {
            #[inline(always)]
            pub fn set_amount(&mut self, amount: u64) {
                self.amount = amount.to_le_bytes();
            }
            #[inline(always)]
            pub fn amount(&self) -> u64 {
                u64::from_le_bytes(self.amount)
            }
            #[inline(always)]
            pub fn clear_delegate(&mut self) {
                self.delegate.0[0] = 0;
            }
            #[inline(always)]
            pub fn set_delegate(&mut self, delegate: &Pubkey) {
                self.delegate.0[0] = 1;
                self.delegate.1 = *delegate;
            }
            #[inline(always)]
            pub fn delegate(&self) -> Option<&Pubkey> {
                if self.delegate.0[0] == 1 { Some(&self.delegate.1) } else { None }
            }
            #[inline(always)]
            pub fn set_native(&mut self, value: bool) {
                self.is_native[0] = value as u8;
            }
            #[inline(always)]
            pub fn is_native(&self) -> bool {
                self.is_native[0] == 1
            }
            #[inline(always)]
            pub fn set_native_amount(&mut self, amount: u64) {
                self.native_amount = amount.to_le_bytes();
            }
            #[inline(always)]
            pub fn native_amount(&self) -> Option<u64> {
                if self.is_native() {
                    Some(u64::from_le_bytes(self.native_amount))
                } else {
                    None
                }
            }
            #[inline(always)]
            pub fn set_delegated_amount(&mut self, amount: u64) {
                self.delegated_amount = amount.to_le_bytes();
            }
            #[inline(always)]
            pub fn delegated_amount(&self) -> u64 {
                u64::from_le_bytes(self.delegated_amount)
            }
            #[inline(always)]
            pub fn clear_close_authority(&mut self) {
                self.close_authority.0[0] = 0;
            }
            #[inline(always)]
            pub fn set_close_authority(&mut self, value: &Pubkey) {
                self.close_authority.0[0] = 1;
                self.close_authority.1 = *value;
            }
            #[inline(always)]
            pub fn close_authority(&self) -> Option<&Pubkey> {
                if self.close_authority.0[0] == 1 {
                    Some(&self.close_authority.1)
                } else {
                    None
                }
            }
            #[inline(always)]
            pub fn is_frozen(&self) -> bool {
                self.state == AccountState::Frozen
            }
            #[inline(always)]
            pub fn is_owned_by_system_program_or_incinerator(&self) -> bool {
                SYSTEM_PROGRAM_ID == self.owner || INCINERATOR_ID == self.owner
            }
        }
        impl RawType for Account {
            const LEN: usize = core::mem::size_of::<Account>();
        }
        impl Initializable for Account {
            #[inline(always)]
            fn is_initialized(&self) -> bool {
                self.state != AccountState::Uninitialized
            }
        }
    }
    pub mod account_state {
        #[repr(u8)]
        pub enum AccountState {
            /// Account is not yet initialized
            Uninitialized,
            /// Account is initialized; the account owner and/or delegate may perform
            /// permitted operations on this account
            Initialized,
            /// Account has been frozen by the mint freeze authority. Neither the
            /// account owner nor the delegate are able to perform operations on
            /// this account.
            Frozen,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for AccountState {
            #[inline]
            fn clone(&self) -> AccountState {
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for AccountState {}
        #[automatically_derived]
        impl ::core::fmt::Debug for AccountState {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(
                    f,
                    match self {
                        AccountState::Uninitialized => "Uninitialized",
                        AccountState::Initialized => "Initialized",
                        AccountState::Frozen => "Frozen",
                    },
                )
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for AccountState {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for AccountState {
            #[inline]
            fn eq(&self, other: &AccountState) -> bool {
                let __self_discr = ::core::intrinsics::discriminant_value(self);
                let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                __self_discr == __arg1_discr
            }
        }
    }
    pub mod mint {
        use crate::pinocchio::pubkey::Pubkey;
        use super::{COption, Initializable, RawType};
        /// Internal representation of a mint data.
        #[repr(C)]
        pub struct Mint {
            /// Optional authority used to mint new tokens. The mint authority may only
            /// be provided during mint creation. If no mint authority is present
            /// then the mint has a fixed supply and no further tokens may be
            /// minted.
            pub mint_authority: COption<Pubkey>,
            /// Total supply of tokens.
            supply: [u8; 8],
            /// Number of base 10 digits to the right of the decimal place.
            pub decimals: u8,
            /// Is `true` if this structure has been initialized.
            is_initialized: u8,
            /// Optional authority to freeze token accounts.
            pub freeze_authority: COption<Pubkey>,
        }
        impl Mint {
            #[inline(always)]
            pub fn set_supply(&mut self, supply: u64) {
                self.supply = supply.to_le_bytes();
            }
            #[inline(always)]
            pub fn supply(&self) -> u64 {
                u64::from_le_bytes(self.supply)
            }
            #[inline(always)]
            pub fn set_initialized(&mut self, value: bool) {
                self.is_initialized = value as u8;
            }
            #[inline(always)]
            pub fn clear_mint_authority(&mut self) {
                self.mint_authority.0[0] = 0;
            }
            #[inline(always)]
            pub fn set_mint_authority(&mut self, mint_authority: &Pubkey) {
                self.mint_authority.0[0] = 1;
                self.mint_authority.1 = *mint_authority;
            }
            #[inline(always)]
            pub fn mint_authority(&self) -> Option<&Pubkey> {
                if self.mint_authority.0[0] == 1 {
                    Some(&self.mint_authority.1)
                } else {
                    None
                }
            }
            #[inline(always)]
            pub fn clear_freeze_authority(&mut self) {
                self.freeze_authority.0[0] = 0;
            }
            #[inline(always)]
            pub fn set_freeze_authority(&mut self, freeze_authority: &Pubkey) {
                self.freeze_authority.0[0] = 1;
                self.freeze_authority.1 = *freeze_authority;
            }
            #[inline(always)]
            pub fn freeze_authority(&self) -> Option<&Pubkey> {
                if self.freeze_authority.0[0] == 1 {
                    Some(&self.freeze_authority.1)
                } else {
                    None
                }
            }
        }
        impl RawType for Mint {
            /// The length of the `Mint` account data.
            const LEN: usize = core::mem::size_of::<Mint>();
        }
        impl Initializable for Mint {
            #[inline(always)]
            fn is_initialized(&self) -> bool {
                self.is_initialized == 1
            }
        }
    }
    pub mod multisig {
        use crate::pinocchio::pubkey::Pubkey;
        use super::{Initializable, RawType};
        /// Minimum number of multisignature signers (min N)
        pub const MIN_SIGNERS: usize = 1;
        /// Maximum number of multisignature signers (max N)
        pub const MAX_SIGNERS: usize = 11;
        /// Multisignature data.
        #[repr(C)]
        pub struct Multisig {
            /// Number of signers required.
            pub m: u8,
            /// Number of valid signers.
            pub n: u8,
            /// Is `true` if this structure has been initialized
            is_initialized: u8,
            /// Signer public keys
            pub signers: [Pubkey; MAX_SIGNERS],
        }
        impl Multisig {
            /// Utility function that checks index is between [`MIN_SIGNERS`] and [`MAX_SIGNERS`].
            pub fn is_valid_signer_index(index: usize) -> bool {
                (MIN_SIGNERS..=MAX_SIGNERS).contains(&index)
            }
            #[inline]
            pub fn set_initialized(&mut self, value: bool) {
                self.is_initialized = value as u8;
            }
        }
        impl RawType for Multisig {
            /// The length of the `Mint` account data.
            const LEN: usize = core::mem::size_of::<Multisig>();
        }
        impl Initializable for Multisig {
            #[inline(always)]
            fn is_initialized(&self) -> bool {
                self.is_initialized == 1
            }
        }
    }
    /// Type alias for fields represented as `COption`.
    pub type COption<T> = ([u8; 4], T);
    /// Marker trait for types that can cast from a raw pointer.
    ///
    /// It is up to the type implementing this trait to guarantee that the cast is safe,
    /// i.e., that the fields of the type are well aligned and there are no padding bytes.
    pub trait RawType {
        /// The length of the type.
        ///
        /// This must be equal to the size of each individual field in the type.
        const LEN: usize;
    }
    /// Trait to represent a type that can be initialized.
    pub trait Initializable {
        /// Return `true` if the object is initialized.
        fn is_initialized(&self) -> bool;
    }
    /// Return a reference for an initialized `T` from the given bytes.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `bytes` contains a valid representation of `T`.
    #[inline(always)]
    pub unsafe fn load<T: Initializable + RawType>(
        bytes: &[u8],
    ) -> Result<&T, ProgramError> {
        load_unchecked(bytes)
            .and_then(|t: &T| {
                if t.is_initialized() {
                    Ok(t)
                } else {
                    Err(ProgramError::UninitializedAccount)
                }
            })
    }
    /// Return a `T` reference from the given bytes.
    ///
    /// This function does not check if the data is initialized.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `bytes` contains a valid representation of `T`.
    #[inline(always)]
    pub unsafe fn load_unchecked<T: RawType>(bytes: &[u8]) -> Result<&T, ProgramError> {
        if bytes.len() != T::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(&*(bytes.as_ptr() as *const T))
    }
    /// Return a mutable reference for an initialized `T` from the given bytes.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `bytes` contains a valid representation of `T`.
    #[inline(always)]
    pub unsafe fn load_mut<T: Initializable + RawType>(
        bytes: &mut [u8],
    ) -> Result<&mut T, ProgramError> {
        load_mut_unchecked(bytes)
            .and_then(|t: &mut T| {
                if t.is_initialized() {
                    Ok(t)
                } else {
                    Err(ProgramError::UninitializedAccount)
                }
            })
    }
    /// Return a mutable `T` reference from the given bytes.
    ///
    /// This function does not check if the data is initialized.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `bytes` contains a valid representation of `T`.
    #[inline(always)]
    pub unsafe fn load_mut_unchecked<T: RawType>(
        bytes: &mut [u8],
    ) -> Result<&mut T, ProgramError> {
        if bytes.len() != T::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(&mut *(bytes.as_mut_ptr() as *mut T))
    }
}
pub mod program {
    ///The const program ID.
    pub const ID: crate::pinocchio_pubkey::pinocchio::pubkey::Pubkey = crate::pinocchio_pubkey::from_str(
        "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
    );
    ///Returns `true` if given pubkey is the program ID.
    #[inline]
    pub fn check_id(id: &crate::pinocchio_pubkey::pinocchio::pubkey::Pubkey) -> bool {
        id == &ID
    }
    ///Returns the program ID.
    #[inline]
    pub const fn id() -> crate::pinocchio_pubkey::pinocchio::pubkey::Pubkey {
        ID
    }
}
