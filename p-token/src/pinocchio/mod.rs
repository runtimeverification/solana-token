#![feature(prelude_import)]
//! A library to build a Solana program in Rust.
//!
//! This library is intended to be used by on-chain programs only. It provides
//! a zero-dependency library to minimise dependencies conflits. For off-chain
//! programs, use instead the [`solana-sdk`] crate, which re-exports all modules
//! from [`solana-program`].
//!
//! [`solana-sdk`]: https://docs.rs/solana-sdk/latest/solana_sdk/
//! [`solana-program`]: https://docs.rs/solana-program/latest/solana_program/
#![no_std]
//#[prelude_import]
//use core::prelude::rust_2021::*;
//#[macro_use]
//extern crate core;
//extern crate compiler_builtins as _;
pub mod account_info {
    //! Data structures to represent account information.
    use core::{
        marker::PhantomData, mem::ManuallyDrop, ptr::NonNull, slice::from_raw_parts_mut,
    };
    use crate::pinocchio::{program_error::ProgramError, pubkey::Pubkey, ProgramResult};
    /// Maximum number of bytes a program may add to an account during a
    /// single realloc.
    pub const MAX_PERMITTED_DATA_INCREASE: usize = 1_024 * 10;
    /// Raw account data.
    ///
    /// This data is wrapped in an `AccountInfo` struct, which provides safe access
    /// to the data.
    #[repr(C)]
    pub(crate) struct Account {
        /// Borrow state of the account data.
        ///
        /// 0) We reuse the duplicate flag for this. We set it to 0b0000_0000.
        /// 1) We use the first four bits to track state of lamport borrow
        /// 2) We use the second four bits to track state of data borrow
        ///
        /// 4 bit state: [1 bit mutable borrow flag | u3 immmutable borrow flag]
        /// This gives us up to 7 immutable borrows. Note that does not mean 7
        /// duplicate account infos, but rather 7 calls to borrow lamports or
        /// borrow data across all duplicate account infos.
        pub(crate) borrow_state: u8,
        /// Indicates whether the transaction was signed by this account.
        is_signer: u8,
        /// Indicates whether the account is writable.
        is_writable: u8,
        /// Indicates whether this account represents a program.
        executable: u8,
        /// Account's original data length when it was serialized for the
        /// current program invocation.
        ///
        /// The value of this field is lazily initialized to the current data length
        /// and the [`SET_LEN_MASK`] flag on first access. When reading this field,
        /// the flag is cleared to retrieve the original data length by using the
        /// [`GET_LEN_MASK`] mask.
        ///
        /// Currently, this value is only used for `realloc` to determine if the
        /// account data length has changed from the original serialized length beyond
        /// the maximum permitted data increase.
        original_data_len: u32,
        /// Public key of the account
        key: Pubkey,
        /// Program that owns this account
        owner: Pubkey,
        /// The lamports in the account.  Modifiable by programs.
        lamports: u64,
        /// Length of the data.
        pub(crate) data_len: u64,
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Account {
        #[inline]
        fn clone(&self) -> Account {
            let _: ::core::clone::AssertParamIsClone<u8>;
            let _: ::core::clone::AssertParamIsClone<u32>;
            let _: ::core::clone::AssertParamIsClone<Pubkey>;
            let _: ::core::clone::AssertParamIsClone<u64>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for Account {}
    #[automatically_derived]
    impl ::core::default::Default for Account {
        #[inline]
        fn default() -> Account {
            Account {
                borrow_state: ::core::default::Default::default(),
                is_signer: ::core::default::Default::default(),
                is_writable: ::core::default::Default::default(),
                executable: ::core::default::Default::default(),
                original_data_len: ::core::default::Default::default(),
                key: ::core::default::Default::default(),
                owner: ::core::default::Default::default(),
                lamports: ::core::default::Default::default(),
                data_len: ::core::default::Default::default(),
            }
        }
    }
    /// Mask to indicate the original data length has been set.
    ///
    /// This takes advantage of the fact that the original data length will not
    /// be greater than 10_000_000 bytes, so we can use the most significant bit
    /// as a flag to indicate that the original data length has been set and lazily
    /// initialize its value.
    const SET_LEN_MASK: u32 = 1 << 31;
    /// Mask to retrieve the original data length.
    ///
    /// This mask is used to retrieve the original data length from the `original_data_len`
    /// by clearing the flag that indicates the original data length has been set.
    const GET_LEN_MASK: u32 = !SET_LEN_MASK;
    /// Wrapper struct for an `Account`.
    ///
    /// This struct provides safe access to the data in an `Account`. It is also
    /// used to track borrows of the account data and lamports, given that an
    /// account can be "shared" across multiple `AccountInfo` instances.
    #[repr(C)]
    pub struct AccountInfo {
        /// Raw (pointer to) account data.
        ///
        /// Note that this is a pointer can be shared across multiple `AccountInfo`.
        pub(crate) raw: *mut Account,
    }
    #[automatically_derived]
    impl ::core::clone::Clone for AccountInfo {
        #[inline]
        fn clone(&self) -> AccountInfo {
            AccountInfo {
                raw: ::core::clone::Clone::clone(&self.raw),
            }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for AccountInfo {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for AccountInfo {
        #[inline]
        fn eq(&self, other: &AccountInfo) -> bool {
            self.raw == other.raw
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for AccountInfo {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<*mut Account>;
        }
    }
    impl AccountInfo {
        /// Public key of the account.
        #[inline(always)]
        pub fn key(&self) -> &Pubkey {
            unsafe { &(*self.raw).key }
        }
        /// Program that owns this account.
        #[inline(always)]
        pub fn owner(&self) -> &Pubkey {
            unsafe { &(*self.raw).owner }
        }
        /// Indicates whether the transaction was signed by this account.
        #[inline(always)]
        pub fn is_signer(&self) -> bool {
            unsafe { (*self.raw).is_signer != 0 }
        }
        /// Indicates whether the account is writable.
        #[inline(always)]
        pub fn is_writable(&self) -> bool {
            unsafe { (*self.raw).is_writable != 0 }
        }
        /// Indicates whether this account represents a program.
        ///
        /// Program accounts are always read-only.
        #[inline(always)]
        pub fn executable(&self) -> bool {
            unsafe { (*self.raw).executable != 0 }
        }
        /// Returns the size of the data in the account.
        #[inline(always)]
        pub fn data_len(&self) -> usize {
            unsafe { (*self.raw).data_len as usize }
        }
        /// Returns the lamports in the account.
        #[inline(always)]
        pub fn lamports(&self) -> u64 {
            unsafe { (*self.raw).lamports }
        }
        /// Indicates whether the account data is empty.
        ///
        /// An account is considered empty if the data length is zero.
        #[inline(always)]
        pub fn data_is_empty(&self) -> bool {
            self.data_len() == 0
        }
        /// Changes the owner of the account.
        #[allow(invalid_reference_casting)]
        #[inline]
        pub fn assign(&self, new_owner: &Pubkey) {
            unsafe {
                core::ptr::write_volatile(
                    &(*self.raw).owner as *const _ as *mut Pubkey,
                    *new_owner,
                );
            }
        }
        /// Returns a read-only reference to the lamports in the account.
        ///
        /// # Safety
        ///
        /// This method is unsafe because it does not return a `Ref`, thus leaving the borrow
        /// flag untouched. Useful when an instruction has verified non-duplicate accounts.
        #[inline]
        pub unsafe fn borrow_lamports_unchecked(&self) -> &u64 {
            &(*self.raw).lamports
        }
        /// Returns a mutable reference to the lamports in the account.
        ///
        /// # Safety
        ///
        /// This method is unsafe because it does not return a `Ref`, thus leaving the borrow
        /// flag untouched. Useful when an instruction has verified non-duplicate accounts.
        #[allow(clippy::mut_from_ref)]
        #[inline]
        pub unsafe fn borrow_mut_lamports_unchecked(&self) -> &mut u64 {
            &mut (*self.raw).lamports
        }
        /// Returns a read-only reference to the data in the account.
        ///
        /// # Safety
        ///
        /// This method is unsafe because it does not return a `Ref`, thus leaving the borrow
        /// flag untouched. Useful when an instruction has verified non-duplicate accounts.
        #[inline]
        pub unsafe fn borrow_data_unchecked(&self) -> &[u8] {
            core::slice::from_raw_parts(self.data_ptr(), self.data_len())
        }
        /// Returns a mutable reference to the data in the account.
        ///
        /// # Safety
        ///
        /// This method is unsafe because it does not return a `Ref`, thus leaving the borrow
        /// flag untouched. Useful when an instruction has verified non-duplicate accounts.
        #[allow(clippy::mut_from_ref)]
        #[inline]
        pub unsafe fn borrow_mut_data_unchecked(&self) -> &mut [u8] {
            core::slice::from_raw_parts_mut(self.data_ptr(), self.data_len())
        }
        /// Tries to get a read-only reference to the lamport field, failing if the
        /// field is already mutable borrowed or if 7 borrows already exist.
        pub fn try_borrow_lamports(&self) -> Result<Ref<u64>, ProgramError> {
            let borrow_state = unsafe { &mut (*self.raw).borrow_state };
            if *borrow_state & 0b_1000_0000 != 0 {
                return Err(ProgramError::AccountBorrowFailed);
            }
            if *borrow_state & 0b_0111_0000 == 0b_0111_0000 {
                return Err(ProgramError::AccountBorrowFailed);
            }
            *borrow_state += 1 << LAMPORTS_SHIFT;
            Ok(Ref {
                value: unsafe { NonNull::from(&(*self.raw).lamports) },
                state: unsafe { NonNull::new_unchecked(borrow_state) },
                borrow_shift: LAMPORTS_SHIFT,
                marker: PhantomData,
            })
        }
        /// Tries to get a read only reference to the lamport field, failing if the field
        /// is already borrowed in any form.
        pub fn try_borrow_mut_lamports(&self) -> Result<RefMut<u64>, ProgramError> {
            let borrow_state = unsafe { &mut (*self.raw).borrow_state };
            if *borrow_state & 0b_1111_0000 != 0 {
                return Err(ProgramError::AccountBorrowFailed);
            }
            *borrow_state |= 0b_1000_0000;
            Ok(RefMut {
                value: unsafe { NonNull::from(&mut (*self.raw).lamports) },
                state: unsafe { NonNull::new_unchecked(borrow_state) },
                borrow_mask: LAMPORTS_MASK,
                marker: PhantomData,
            })
        }
        /// Tries to get a read only reference to the data field, failing if the field
        /// is already mutable borrowed or if 7 borrows already exist.
        pub fn try_borrow_data(&self) -> Result<Ref<[u8]>, ProgramError> {
            let borrow_state = unsafe { &mut (*self.raw).borrow_state };
            if *borrow_state & 0b_0000_1000 != 0 {
                return Err(ProgramError::AccountBorrowFailed);
            }
            if *borrow_state & 0b_0111 == 0b0111 {
                return Err(ProgramError::AccountBorrowFailed);
            }
            *borrow_state += 1;
            Ok(Ref {
                value: unsafe {
                    NonNull::from(
                        core::slice::from_raw_parts(self.data_ptr(), self.data_len()),
                    )
                },
                state: unsafe { NonNull::new_unchecked(borrow_state) },
                borrow_shift: DATA_SHIFT,
                marker: PhantomData,
            })
        }
        /// Tries to get a read only reference to the data field, failing if the field
        /// is already borrowed in any form.
        pub fn try_borrow_mut_data(&self) -> Result<RefMut<[u8]>, ProgramError> {
            let borrow_state = unsafe { &mut (*self.raw).borrow_state };
            if *borrow_state & 0b_0000_1111 != 0 {
                return Err(ProgramError::AccountBorrowFailed);
            }
            *borrow_state |= 0b_0000_1000;
            Ok(RefMut {
                value: unsafe {
                    NonNull::from(from_raw_parts_mut(self.data_ptr(), self.data_len()))
                },
                state: unsafe { NonNull::new_unchecked(borrow_state) },
                borrow_mask: DATA_MASK,
                marker: PhantomData,
            })
        }
        /// Realloc the account's data and optionally zero-initialize the new
        /// memory.
        ///
        /// Note:  Account data can be increased within a single call by up to
        /// [`MAX_PERMITTED_DATA_INCREASE`] bytes.
        ///
        /// Note: Memory used to grow is already zero-initialized upon program
        /// entrypoint and re-zeroing it wastes compute units.  If within the same
        /// call a program reallocs from larger to smaller and back to larger again
        /// the new space could contain stale data.  Pass `true` for `zero_init` in
        /// this case, otherwise compute units will be wasted re-zero-initializing.
        ///
        /// # Safety
        ///
        /// This method makes assumptions about the layout and location of memory
        /// referenced by `AccountInfo` fields. It should only be called for
        /// instances of `AccountInfo` that were created by the runtime and received
        /// in the `process_instruction` entrypoint of a program.
        pub fn realloc(
            &self,
            new_len: usize,
            zero_init: bool,
        ) -> Result<(), ProgramError> {
            let mut data = self.try_borrow_mut_data()?;
            let current_len = data.len();
            if new_len == current_len {
                return Ok(());
            }
            let original_len = {
                let length = unsafe { (*self.raw).original_data_len };
                if length & SET_LEN_MASK == SET_LEN_MASK {
                    (length & GET_LEN_MASK) as usize
                } else {
                    unsafe {
                        (*self.raw).original_data_len = (current_len as u32)
                            | SET_LEN_MASK;
                    }
                    current_len
                }
            };
            if new_len.saturating_sub(original_len) > MAX_PERMITTED_DATA_INCREASE {
                return Err(ProgramError::InvalidRealloc);
            }
            unsafe {
                let data_ptr = data.as_mut_ptr();
                *(data_ptr.offset(-8) as *mut u64) = new_len as u64;
                data.value = NonNull::from(from_raw_parts_mut(data_ptr, new_len));
            }
            if zero_init {
                let len_increase = new_len.saturating_sub(current_len);
                if len_increase > 0 {
                    unsafe {
                        #[cfg(not(target_os = "solana"))]
                        core::ptr::write_bytes(
                            data.as_mut_ptr().add(current_len),
                            0,
                            len_increase,
                        );
                    }
                }
            }
            Ok(())
        }
        /// Zero out the the account's data length, lamports and owner fields, effectively
        /// closing the account.
        ///
        /// This doesn't protect against future reinitialization of the account
        /// since the account data will need to be zeroed out as well; otherwise the lenght,
        /// lamports and owner can be set again before the data is wiped out from
        /// the ledger using the keypair of the account being close.
        #[inline]
        pub fn close(&self) -> ProgramResult {
            {
                let _ = self.try_borrow_mut_data()?;
            }
            unsafe {
                self.close_unchecked();
            }
            Ok(())
        }
        /// Zero out the the account's data length, lamports and owner fields, effectively
        /// closing the account.
        ///
        /// This doesn't protect against future reinitialization of the account
        /// since the account data will need to be zeroed out as well; otherwise the lenght,
        /// lamports and owner can be set again before the data is wiped out from
        /// the ledger using the keypair of the account being close.
        ///
        /// # Safety
        ///
        /// This method is unsafe because it does not check if the account data is already
        /// borrowed. It should only be called when the account is not being used.
        ///
        /// It also makes assumptions about the layout and location of memory
        /// referenced by `AccountInfo` fields. It should only be called for
        /// instances of `AccountInfo` that were created by the runtime and received
        /// in the `process_instruction` entrypoint of a program.
        #[inline(always)]
        pub unsafe fn close_unchecked(&self) {}
        /// Returns the memory address of the account data.
        fn data_ptr(&self) -> *mut u8 {
            unsafe {
                (self.raw as *const _ as *mut u8).add(core::mem::size_of::<Account>())
            }
        }
    }
    /// Bytes to shift to get to the borrow state of lamports.
    const LAMPORTS_SHIFT: u8 = 4;
    /// Bytes to shift to get to the borrow state of data.
    const DATA_SHIFT: u8 = 0;
    /// Reference to account data or lamports with checked borrow rules.
    pub struct Ref<'a, T: ?Sized> {
        value: NonNull<T>,
        state: NonNull<u8>,
        /// Indicates the type of borrow (lamports or data) by representing the
        /// shift amount.
        borrow_shift: u8,
        /// The `value` raw pointer is only valid while the `&'a T` lives so we claim
        /// to hold a reference to it.
        marker: PhantomData<&'a T>,
    }
    impl<'a, T: ?Sized> Ref<'a, T> {
        #[inline]
        pub fn map<U: ?Sized, F>(orig: Ref<'a, T>, f: F) -> Ref<'a, U>
        where
            F: FnOnce(&T) -> &U,
        {
            let orig = ManuallyDrop::new(orig);
            Ref {
                value: NonNull::from(f(&*orig)),
                state: orig.state,
                borrow_shift: orig.borrow_shift,
                marker: PhantomData,
            }
        }
        #[inline]
        pub fn filter_map<U: ?Sized, F>(
            orig: Ref<'a, T>,
            f: F,
        ) -> Result<Ref<'a, U>, Self>
        where
            F: FnOnce(&T) -> Option<&U>,
        {
            let orig = ManuallyDrop::new(orig);
            match f(&*orig) {
                Some(value) => {
                    Ok(Ref {
                        value: NonNull::from(value),
                        state: orig.state,
                        borrow_shift: orig.borrow_shift,
                        marker: PhantomData,
                    })
                }
                None => Err(ManuallyDrop::into_inner(orig)),
            }
        }
    }
    impl<'a, T: ?Sized> core::ops::Deref for Ref<'a, T> {
        type Target = T;
        fn deref(&self) -> &Self::Target {
            unsafe { self.value.as_ref() }
        }
    }
    impl<'a, T: ?Sized> Drop for Ref<'a, T> {
        fn drop(&mut self) {
            unsafe { *self.state.as_mut() -= 1 << self.borrow_shift };
        }
    }
    /// Mask representing the mutable borrow flag for lamports.
    const LAMPORTS_MASK: u8 = 0b_0111_1111;
    /// Mask representing the mutable borrow flag for data.
    const DATA_MASK: u8 = 0b_1111_0111;
    /// Mutable reference to account data or lamports with checked borrow rules.
    pub struct RefMut<'a, T: ?Sized> {
        value: NonNull<T>,
        state: NonNull<u8>,
        /// Indicates the type of borrow (lamports or data) by representing the
        /// mutable borrow mask.
        borrow_mask: u8,
        /// The `value` raw pointer is only valid while the `&'a T` lives so we claim
        /// to hold a reference to it.
        marker: PhantomData<&'a mut T>,
    }
    impl<'a, T: ?Sized> RefMut<'a, T> {
        #[inline]
        pub fn map<U: ?Sized, F>(orig: RefMut<'a, T>, f: F) -> RefMut<'a, U>
        where
            F: FnOnce(&mut T) -> &mut U,
        {
            let mut orig = ManuallyDrop::new(orig);
            RefMut {
                value: NonNull::from(f(&mut *orig)),
                state: orig.state,
                borrow_mask: orig.borrow_mask,
                marker: PhantomData,
            }
        }
        #[inline]
        pub fn filter_map<U: ?Sized, F>(
            orig: RefMut<'a, T>,
            f: F,
        ) -> Result<RefMut<'a, U>, Self>
        where
            F: FnOnce(&mut T) -> Option<&mut U>,
        {
            let mut orig = ManuallyDrop::new(orig);
            match f(&mut *orig) {
                Some(value) => {
                    let value = NonNull::from(value);
                    Ok(RefMut {
                        value,
                        state: orig.state,
                        borrow_mask: orig.borrow_mask,
                        marker: PhantomData,
                    })
                }
                None => Err(ManuallyDrop::into_inner(orig)),
            }
        }
    }
    impl<'a, T: ?Sized> core::ops::Deref for RefMut<'a, T> {
        type Target = T;
        fn deref(&self) -> &Self::Target {
            unsafe { self.value.as_ref() }
        }
    }
    impl<'a, T: ?Sized> core::ops::DerefMut for RefMut<'a, T> {
        fn deref_mut(&mut self) -> &mut <Self as core::ops::Deref>::Target {
            unsafe { self.value.as_mut() }
        }
    }
    impl<'a, T: ?Sized> Drop for RefMut<'a, T> {
        fn drop(&mut self) {
            unsafe { *self.state.as_mut() &= self.borrow_mask };
        }
    }
}
pub mod entrypoint {
    //! Macros and functions for defining the program entrypoint and setting up
    //! global handlers.
    pub mod lazy {
        use crate::pinocchio::{
            account_info::{Account, AccountInfo, MAX_PERMITTED_DATA_INCREASE},
            program_error::ProgramError, pubkey::Pubkey, BPF_ALIGN_OF_U128,
            NON_DUP_MARKER,
        };
        /// Context to access data from the input buffer for the instruction.
        ///
        /// This is a wrapper around the input buffer that provides methods to read the accounts
        /// and instruction data. It is used by the lazy entrypoint to access the input data on demand.
        pub struct InstructionContext {
            /// Pointer to the runtime input buffer for the instruction.
            input: *mut u8,
            /// Number of remaining accounts.
            ///
            /// This value is decremented each time [`next_account`] is called.
            remaining: u64,
            /// Current memory offset on the input buffer.
            offset: usize,
        }
        impl InstructionContext {
            /// Creates a new [`InstructionContext`] for the input buffer.
            #[inline(always)]
            pub fn new(input: *mut u8) -> Self {
                Self {
                    input,
                    remaining: unsafe { *(input as *const u64) },
                    offset: core::mem::size_of::<u64>(),
                }
            }
            /// Reads the next account for the instruction.
            ///
            /// The account is represented as a [`MaybeAccount`], since it can either
            /// represent and [`AccountInfo`] or the index of a duplicated account. It is up to the
            /// caller to handle the mapping back to the source account.
            ///
            /// # Error
            ///
            /// Returns a [`ProgramError::NotEnoughAccountKeys`] error if there are
            /// no remaining accounts.
            #[inline(always)]
            pub fn next_account(&mut self) -> Result<MaybeAccount, ProgramError> {
                self.remaining = self
                    .remaining
                    .checked_sub(1)
                    .ok_or(ProgramError::NotEnoughAccountKeys)?;
                Ok(unsafe { read_account(self.input, &mut self.offset) })
            }
            /// Returns the next account for the instruction.
            ///
            /// Note that this method does *not* decrement the number of remaining accounts, but moves
            /// the offset forward. It is intended for use when the caller is certain on the number of
            /// remaining accounts.
            ///
            /// # Safety
            ///
            /// It is up to the caller to guarantee that there are remaining accounts; calling this when
            /// there are no more remaining accounts results in undefined behavior.
            #[inline(always)]
            pub unsafe fn next_account_unchecked(&mut self) -> MaybeAccount {
                read_account(self.input, &mut self.offset)
            }
            /// Returns the number of available accounts.
            #[inline(always)]
            pub fn available(&self) -> u64 {
                unsafe { *(self.input as *const u64) }
            }
            /// Returns the number of remaining accounts.
            ///
            /// This value is decremented each time [`next_account`] is called.
            #[inline(always)]
            pub fn remaining(&self) -> u64 {
                self.remaining
            }
            /// Returns the instruction data for the instruction.
            ///
            /// This method can only be used after all accounts have been read; otherwise, it will
            /// return a [`ProgramError::InvalidInstructionData`] error.
            #[inline(always)]
            pub fn instruction_data(&mut self) -> Result<&[u8], ProgramError> {
                if self.remaining > 0 {
                    return Err(ProgramError::InvalidInstructionData);
                }
                Ok(unsafe { self.instruction_data_unchecked() })
            }
            /// Returns the instruction data for the instruction.
            ///
            /// # Safety
            ///
            /// It is up to the caller to guarantee that all accounts have been read; calling this method
            /// before reading all accounts will result in undefined behavior.
            #[inline(always)]
            pub unsafe fn instruction_data_unchecked(&mut self) -> &[u8] {
                let data_len = *(self.input.add(self.offset) as *const usize);
                let offset = self.offset + core::mem::size_of::<u64>();
                core::slice::from_raw_parts(self.input.add(offset), data_len)
            }
            /// Returns the program id for the instruction.
            ///
            /// This method can only be used after all accounts have been read; otherwise, it will
            /// return a [`ProgramError::InvalidInstructionData`] error.
            #[inline(always)]
            pub fn program_id(&mut self) -> Result<&Pubkey, ProgramError> {
                if self.remaining > 0 {
                    return Err(ProgramError::InvalidInstructionData);
                }
                Ok(unsafe { self.program_id_unchecked() })
            }
            /// Returns the program id for the instruction.
            ///
            /// # Safety
            ///
            /// It is up to the caller to guarantee that all accounts have been read; calling this method
            /// before reading all accounts will result in undefined behavior.
            #[inline(always)]
            pub unsafe fn program_id_unchecked(&mut self) -> &Pubkey {
                let data_len = *(self.input.add(self.offset) as *const usize);
                &*(self.input.add(self.offset + core::mem::size_of::<u64>() + data_len)
                    as *const Pubkey)
            }
        }
        /// Wrapper type around an [`AccountInfo`] that may be a duplicate.
        pub enum MaybeAccount {
            /// An [`AccountInfo`] that is not a duplicate.
            Account(AccountInfo),
            /// The index of the original account that was duplicated.
            Duplicated(u8),
        }
        impl MaybeAccount {
            /// Extracts the wrapped [`AccountInfo`].
            ///
            /// It is up to the caller to guarantee that the [`MaybeAccount`] really is in an
            /// [`MaybeAccount::Account`]. Calling this method when the variant is a
            /// [`MaybeAccount::Duplicated`] will result in a panic.
            #[inline(always)]
            pub fn assume_account(self) -> AccountInfo {
                let MaybeAccount::Account(account) = self else {
                    {
                        ::core::panicking::panic_fmt(format_args!("Duplicated account"));
                    }
                };
                account
            }
        }
        /// Read an account from the input buffer.
        ///
        /// This can only be called with a buffer that was serialized by the runtime as
        /// it assumes a specific memory layout.
        #[allow(clippy::cast_ptr_alignment, clippy::missing_safety_doc)]
        #[inline(always)]
        unsafe fn read_account(input: *mut u8, offset: &mut usize) -> MaybeAccount {
            let account: *mut Account = input.add(*offset) as *mut _;
            if (*account).borrow_state == NON_DUP_MARKER {
                (*account).borrow_state = 0b_0000_0000;
                *offset += core::mem::size_of::<Account>();
                *offset += (*account).data_len as usize;
                *offset += MAX_PERMITTED_DATA_INCREASE;
                *offset += (*offset as *const u8).align_offset(BPF_ALIGN_OF_U128);
                *offset += core::mem::size_of::<u64>();
                MaybeAccount::Account(AccountInfo { raw: account })
            } else {
                *offset += core::mem::size_of::<u64>();
                MaybeAccount::Duplicated((*account).borrow_state)
            }
        }
    }
    pub use lazy::{InstructionContext, MaybeAccount};
    use crate::pinocchio::{
        account_info::{Account, AccountInfo, MAX_PERMITTED_DATA_INCREASE},
        pubkey::Pubkey, BPF_ALIGN_OF_U128, NON_DUP_MARKER,
    };
    /// Start address of the memory region used for program heap.
    pub const HEAP_START_ADDRESS: u64 = 0x300000000;
    /// Length of the heap memory region used for program heap.
    pub const HEAP_LENGTH: usize = 32 * 1024;
    #[deprecated(
        since = "0.6.0",
        note = "Use `ProgramResult` from the crate root instead"
    )]
    /// The result of a program execution.
    pub type ProgramResult = super::ProgramResult;
    #[deprecated(since = "0.6.0", note = "Use `SUCCESS` from the crate root instead")]
    /// Return value for a successful program execution.
    pub const SUCCESS: u64 = super::SUCCESS;
    /// Deserialize the input arguments.
    ///
    /// This can only be called from the entrypoint function of a Solana program and with
    /// a buffer that was serialized by the runtime.
    #[allow(clippy::cast_ptr_alignment, clippy::missing_safety_doc)]
    #[inline(always)]
    pub unsafe fn deserialize<'a, const MAX_ACCOUNTS: usize>(
        input: *mut u8,
        accounts: &mut [core::mem::MaybeUninit<AccountInfo>],
    ) -> (&'a Pubkey, usize, &'a [u8]) {
        let mut offset: usize = 0;
        let total_accounts = *(input.add(offset) as *const u64) as usize;
        offset += core::mem::size_of::<u64>();
        let processed = if total_accounts > 0 {
            let processed = core::cmp::min(total_accounts, MAX_ACCOUNTS);
            for i in 0..processed {
                let account_info: *mut Account = input.add(offset) as *mut _;
                if (*account_info).borrow_state == NON_DUP_MARKER {
                    (*account_info).borrow_state = 0b_0000_0000;
                    offset += core::mem::size_of::<Account>();
                    offset += (*account_info).data_len as usize;
                    offset += MAX_PERMITTED_DATA_INCREASE;
                    offset += (offset as *const u8).align_offset(BPF_ALIGN_OF_U128);
                    offset += core::mem::size_of::<u64>();
                    accounts[i].write(AccountInfo { raw: account_info });
                } else {
                    offset += core::mem::size_of::<u64>();
                    accounts[i]
                        .write(
                            accounts[(*account_info).borrow_state as usize]
                                .assume_init_ref()
                                .clone(),
                        );
                }
            }
            for _ in processed..total_accounts {
                let account_info: *mut Account = input.add(offset) as *mut _;
                if (*account_info).borrow_state == NON_DUP_MARKER {
                    offset += core::mem::size_of::<Account>();
                    offset += (*account_info).data_len as usize;
                    offset += MAX_PERMITTED_DATA_INCREASE;
                    offset += (offset as *const u8).align_offset(BPF_ALIGN_OF_U128);
                    offset += core::mem::size_of::<u64>();
                } else {
                    offset += core::mem::size_of::<u64>();
                }
            }
            processed
        } else {
            0
        };
        let instruction_data_len = *(input.add(offset) as *const u64) as usize;
        offset += core::mem::size_of::<u64>();
        let instruction_data = {
            core::slice::from_raw_parts(input.add(offset), instruction_data_len)
        };
        offset += instruction_data_len;
        let program_id: &Pubkey = &*(input.add(offset) as *const Pubkey);
        (program_id, processed, instruction_data)
    }
    #[cfg(not(feature = "std"))]
    /// Zero global allocator.
    pub struct NoAllocator;
    #[cfg(not(feature = "std"))]
    unsafe impl core::alloc::GlobalAlloc for NoAllocator {
        #[inline]
        unsafe fn alloc(&self, _: core::alloc::Layout) -> *mut u8 {
            {
                ::core::panicking::panic_fmt(format_args!("** NO ALLOCATOR **"));
            };
        }
        #[inline]
        unsafe fn dealloc(&self, _: *mut u8, _: core::alloc::Layout) {}
    }
}
pub mod instruction {
    //! Instruction types.
    use core::{marker::PhantomData, ops::Deref};
    use crate::pinocchio::{account_info::AccountInfo, pubkey::Pubkey};
    /// Information about a CPI instruction.
    #[repr(C)]
    pub struct Instruction<'a, 'b, 'c, 'd>
    where
        'a: 'b,
    {
        /// Public key of the program.
        pub program_id: &'c Pubkey,
        /// Data expected by the program instruction.
        pub data: &'d [u8],
        /// Metadata describing accounts that should be passed to the program.
        pub accounts: &'b [AccountMeta<'a>],
    }
    #[automatically_derived]
    impl<'a, 'b, 'c, 'd> ::core::fmt::Debug for Instruction<'a, 'b, 'c, 'd>
    where
        'a: 'b,
    {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field3_finish(
                f,
                "Instruction",
                "program_id",
                &self.program_id,
                "data",
                &self.data,
                "accounts",
                &&self.accounts,
            )
        }
    }
    #[automatically_derived]
    impl<'a, 'b, 'c, 'd> ::core::clone::Clone for Instruction<'a, 'b, 'c, 'd>
    where
        'a: 'b,
    {
        #[inline]
        fn clone(&self) -> Instruction<'a, 'b, 'c, 'd> {
            Instruction {
                program_id: ::core::clone::Clone::clone(&self.program_id),
                data: ::core::clone::Clone::clone(&self.data),
                accounts: ::core::clone::Clone::clone(&self.accounts),
            }
        }
    }
    /// Use to query and convey information about the sibling instruction components
    /// when calling the `sol_get_processed_sibling_instruction` syscall.
    #[repr(C)]
    pub struct ProcessedSiblingInstruction {
        /// Length of the instruction data
        pub data_len: u64,
        /// Number of AccountMeta structures
        pub accounts_len: u64,
    }
    #[automatically_derived]
    impl ::core::default::Default for ProcessedSiblingInstruction {
        #[inline]
        fn default() -> ProcessedSiblingInstruction {
            ProcessedSiblingInstruction {
                data_len: ::core::default::Default::default(),
                accounts_len: ::core::default::Default::default(),
            }
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for ProcessedSiblingInstruction {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "ProcessedSiblingInstruction",
                "data_len",
                &self.data_len,
                "accounts_len",
                &&self.accounts_len,
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for ProcessedSiblingInstruction {
        #[inline]
        fn clone(&self) -> ProcessedSiblingInstruction {
            let _: ::core::clone::AssertParamIsClone<u64>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for ProcessedSiblingInstruction {}
    #[automatically_derived]
    impl ::core::cmp::Eq for ProcessedSiblingInstruction {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<u64>;
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for ProcessedSiblingInstruction {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for ProcessedSiblingInstruction {
        #[inline]
        fn eq(&self, other: &ProcessedSiblingInstruction) -> bool {
            self.data_len == other.data_len && self.accounts_len == other.accounts_len
        }
    }
    /// An `Account` for CPI invocations.
    ///
    /// This struct contains the same information as an [`AccountInfo`], but has
    /// the memory layout as expected by `sol_invoke_signed_c` syscall.
    #[repr(C)]
    pub struct Account<'a> {
        key: *const Pubkey,
        lamports: *const u64,
        data_len: u64,
        data: *const u8,
        owner: *const Pubkey,
        rent_epoch: u64,
        is_signer: bool,
        is_writable: bool,
        executable: bool,
        /// The pointers to the `AccountInfo` data are only valid for as long as the
        /// `&'a AccountInfo` lives. Instead of holding a reference to the actual `AccountInfo`,
        /// which would increase the size of the type, we claim to hold a reference without
        /// actually holding one using a `PhantomData<&'a AccountInfo>`.
        _account_info: PhantomData<&'a AccountInfo>,
    }
    #[automatically_derived]
    impl<'a> ::core::clone::Clone for Account<'a> {
        #[inline]
        fn clone(&self) -> Account<'a> {
            Account {
                key: ::core::clone::Clone::clone(&self.key),
                lamports: ::core::clone::Clone::clone(&self.lamports),
                data_len: ::core::clone::Clone::clone(&self.data_len),
                data: ::core::clone::Clone::clone(&self.data),
                owner: ::core::clone::Clone::clone(&self.owner),
                rent_epoch: ::core::clone::Clone::clone(&self.rent_epoch),
                is_signer: ::core::clone::Clone::clone(&self.is_signer),
                is_writable: ::core::clone::Clone::clone(&self.is_writable),
                executable: ::core::clone::Clone::clone(&self.executable),
                _account_info: ::core::clone::Clone::clone(&self._account_info),
            }
        }
    }
    #[inline(always)]
    const fn offset<T, U>(ptr: *const T, offset: usize) -> *const U {
        unsafe { (ptr as *const u8).add(offset) as *const U }
    }
    impl<'a> From<&'a AccountInfo> for Account<'a> {
        fn from(account: &'a AccountInfo) -> Self {
            Account {
                key: offset(account.raw, 8),
                lamports: offset(account.raw, 72),
                data_len: account.data_len() as u64,
                data: offset(account.raw, 88),
                owner: offset(account.raw, 40),
                rent_epoch: 0,
                is_signer: account.is_signer(),
                is_writable: account.is_writable(),
                executable: account.executable(),
                _account_info: PhantomData::<&'a AccountInfo>,
            }
        }
    }
    /// Describes a single account read or written by a program during instruction
    /// execution.
    ///
    /// When constructing an [`Instruction`], a list of all accounts that may be
    /// read or written during the execution of that instruction must be supplied.
    /// Any account that may be mutated by the program during execution, either its
    /// data or metadata such as held lamports, must be writable.
    ///
    /// Note that because the Solana runtime schedules parallel transaction
    /// execution around which accounts are writable, care should be taken that only
    /// accounts which actually may be mutated are specified as writable.
    #[repr(C)]
    pub struct AccountMeta<'a> {
        pub pubkey: &'a Pubkey,
        pub is_writable: bool,
        pub is_signer: bool,
    }
    #[automatically_derived]
    impl<'a> ::core::fmt::Debug for AccountMeta<'a> {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field3_finish(
                f,
                "AccountMeta",
                "pubkey",
                &self.pubkey,
                "is_writable",
                &self.is_writable,
                "is_signer",
                &&self.is_signer,
            )
        }
    }
    #[automatically_derived]
    impl<'a> ::core::clone::Clone for AccountMeta<'a> {
        #[inline]
        fn clone(&self) -> AccountMeta<'a> {
            AccountMeta {
                pubkey: ::core::clone::Clone::clone(&self.pubkey),
                is_writable: ::core::clone::Clone::clone(&self.is_writable),
                is_signer: ::core::clone::Clone::clone(&self.is_signer),
            }
        }
    }
    impl<'a> AccountMeta<'a> {
        #[inline(always)]
        pub fn new(pubkey: &'a Pubkey, is_writable: bool, is_signer: bool) -> Self {
            Self {
                pubkey,
                is_writable,
                is_signer,
            }
        }
        #[inline(always)]
        pub fn readonly(pubkey: &'a Pubkey) -> Self {
            Self::new(pubkey, false, false)
        }
        #[inline(always)]
        pub fn writable(pubkey: &'a Pubkey) -> Self {
            Self::new(pubkey, true, false)
        }
        #[inline(always)]
        pub fn readonly_signer(pubkey: &'a Pubkey) -> Self {
            Self::new(pubkey, false, true)
        }
        #[inline(always)]
        pub fn writable_signer(pubkey: &'a Pubkey) -> Self {
            Self::new(pubkey, true, true)
        }
    }
    impl<'a> From<&'a AccountInfo> for AccountMeta<'a> {
        fn from(account: &'a crate::pinocchio::account_info::AccountInfo) -> Self {
            AccountMeta::new(account.key(), account.is_writable(), account.is_signer())
        }
    }
    #[repr(C)]
    pub struct Seed<'a> {
        /// Seed bytes.
        pub(crate) seed: *const u8,
        /// Length of the seed bytes.
        pub(crate) len: u64,
        /// The pointer to the seed bytes is only valid while the `&'a [u8]` lives. Instead
        /// of holding a reference to the actual `[u8]`, which would increase the size of the
        /// type, we claim to hold a reference without actually holding one using a
        /// `PhantomData<&'a [u8]>`.
        _bytes: PhantomData<&'a [u8]>,
    }
    #[automatically_derived]
    impl<'a> ::core::fmt::Debug for Seed<'a> {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field3_finish(
                f,
                "Seed",
                "seed",
                &self.seed,
                "len",
                &self.len,
                "_bytes",
                &&self._bytes,
            )
        }
    }
    #[automatically_derived]
    impl<'a> ::core::clone::Clone for Seed<'a> {
        #[inline]
        fn clone(&self) -> Seed<'a> {
            Seed {
                seed: ::core::clone::Clone::clone(&self.seed),
                len: ::core::clone::Clone::clone(&self.len),
                _bytes: ::core::clone::Clone::clone(&self._bytes),
            }
        }
    }
    impl<'a> From<&'a [u8]> for Seed<'a> {
        fn from(value: &'a [u8]) -> Self {
            Self {
                seed: value.as_ptr(),
                len: value.len() as u64,
                _bytes: PhantomData::<&[u8]>,
            }
        }
    }
    impl<'a, const SIZE: usize> From<&'a [u8; SIZE]> for Seed<'a> {
        fn from(value: &'a [u8; SIZE]) -> Self {
            Self {
                seed: value.as_ptr(),
                len: value.len() as u64,
                _bytes: PhantomData::<&[u8]>,
            }
        }
    }
    impl Deref for Seed<'_> {
        type Target = [u8];
        fn deref(&self) -> &Self::Target {
            unsafe { core::slice::from_raw_parts(self.seed, self.len as usize) }
        }
    }
    /// Represents a [program derived address][pda] (PDA) signer controlled by the
    /// calling program.
    ///
    /// [pda]: https://solana.com/docs/core/cpi#program-derived-addresses
    #[repr(C)]
    pub struct Signer<'a, 'b> {
        /// Signer seeds.
        pub(crate) seeds: *const Seed<'a>,
        /// Number of seeds.
        pub(crate) len: u64,
        /// The pointer to the seeds is only valid while the `&'b [Seed<'a>]` lives. Instead
        /// of holding a reference to the actual `[Seed<'a>]`, which would increase the size
        /// of the type, we claim to hold a reference without actually holding one using a
        /// `PhantomData<&'b [Seed<'a>]>`.
        _seeds: PhantomData<&'b [Seed<'a>]>,
    }
    #[automatically_derived]
    impl<'a, 'b> ::core::fmt::Debug for Signer<'a, 'b> {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field3_finish(
                f,
                "Signer",
                "seeds",
                &self.seeds,
                "len",
                &self.len,
                "_seeds",
                &&self._seeds,
            )
        }
    }
    #[automatically_derived]
    impl<'a, 'b> ::core::clone::Clone for Signer<'a, 'b> {
        #[inline]
        fn clone(&self) -> Signer<'a, 'b> {
            Signer {
                seeds: ::core::clone::Clone::clone(&self.seeds),
                len: ::core::clone::Clone::clone(&self.len),
                _seeds: ::core::clone::Clone::clone(&self._seeds),
            }
        }
    }
    impl<'a, 'b> From<&'b [Seed<'a>]> for Signer<'a, 'b> {
        fn from(value: &'b [Seed<'a>]) -> Self {
            Self {
                seeds: value.as_ptr(),
                len: value.len() as u64,
                _seeds: PhantomData::<&'b [Seed<'a>]>,
            }
        }
    }
    impl<'a, 'b, const SIZE: usize> From<&'b [Seed<'a>; SIZE]> for Signer<'a, 'b> {
        fn from(value: &'b [Seed<'a>; SIZE]) -> Self {
            Self {
                seeds: value.as_ptr(),
                len: value.len() as u64,
                _seeds: PhantomData::<&'b [Seed<'a>]>,
            }
        }
    }
}
pub mod log {
    //! Logging utilities for Rust-based Solana programs.
    //!
    //! Logging is the main mechanism for getting debugging information out of
    //! running Solana programs, and there are several functions available for doing
    //! so efficiently, depending on the type of data being logged.
    //!
    //! The most common way to emit logs is through the [`msg!`] macro, which logs
    //! simple strings, as well as [formatted strings][fs].
    //!
    //! [`msg!`]: crate::msg!
    //! [fs]: https://doc.rust-lang.org/std/fmt/
    //!
    //! Logs can be viewed in multiple ways:
    //!
    //! - The `solana logs` command displays logs for all transactions executed on a
    //!   network. Note though that transactions that fail during pre-flight
    //!   simulation are not displayed here.
    //! - When submitting transactions via [`RpcClient`], if Rust's own logging is
    //!   active then the `solana_rpc_client` crate logs at the "debug" level any logs
    //!   for transactions that failed during simulation. If using [`env_logger`]
    //!   these logs can be activated by setting `RUST_LOG=solana_rpc_client=debug`.
    //! - Logs can be retrieved from a finalized transaction by calling
    //!   [`RpcClient::get_transaction`].
    //! - Block explorers may display logs.
    //!
    //! [`RpcClient`]: https://docs.rs/solana-rpc-client/latest/solana_rpc_client/rpc_client/struct.RpcClient.html
    //! [`env_logger`]: https://docs.rs/env_logger
    //! [`RpcClient::get_transaction`]: https://docs.rs/solana-rpc-client/latest/solana_rpc_client/rpc_client/struct.RpcClient.html#method.get_transaction
    //!
    //! While most logging functions are defined in this module, [`Pubkey`]s can
    //! also be efficiently logged with the [`Pubkey::log`] function.
    //!
    //! [`Pubkey`]: crate::pubkey::Pubkey
    //! [`Pubkey::log`]: crate::pubkey::Pubkey::log
    use crate::pinocchio::{account_info::AccountInfo, pubkey::log};
    /// Print a string to the log.
    #[inline(always)]
    pub fn sol_log(message: &str) {
        #[cfg(not(target_os = "solana"))] core::hint::black_box(message);
    }
    /// Print 64-bit values represented as hexadecimal to the log.
    #[inline]
    pub fn sol_log_64(arg1: u64, arg2: u64, arg3: u64, arg4: u64, arg5: u64) {
        #[cfg(not(target_os = "solana"))]
        core::hint::black_box((arg1, arg2, arg3, arg4, arg5));
    }
    /// Print some slices as base64.
    pub fn sol_log_data(data: &[&[u8]]) {
        #[cfg(not(target_os = "solana"))] core::hint::black_box(data);
    }
    /// Print the hexadecimal representation of a slice.
    pub fn sol_log_slice(slice: &[u8]) {
        for (i, s) in slice.iter().enumerate() {
            sol_log_64(0, 0, 0, i as u64, *s as u64);
        }
    }
    /// Print the hexadecimal representation of the program's input parameters.
    ///
    /// - `accounts` - A slice of [`AccountInfo`].
    /// - `data` - The instruction data.
    pub fn sol_log_params(accounts: &[AccountInfo], data: &[u8]) {
        for (i, account) in accounts.iter().enumerate() {
            crate::pinocchio::log::sol_log("AccountInfo");
            sol_log_64(0, 0, 0, 0, i as u64);
            crate::pinocchio::log::sol_log("- Is signer");
            sol_log_64(0, 0, 0, 0, account.is_signer() as u64);
            crate::pinocchio::log::sol_log("- Key");
            log(account.key());
            crate::pinocchio::log::sol_log("- Lamports");
            sol_log_64(0, 0, 0, 0, account.lamports());
            crate::pinocchio::log::sol_log("- Account data length");
            sol_log_64(0, 0, 0, 0, account.data_len() as u64);
            crate::pinocchio::log::sol_log("- Owner");
            log(account.owner());
        }
        crate::pinocchio::log::sol_log("Instruction data");
        sol_log_slice(data);
    }
    /// Print the remaining compute units available to the program.
    #[inline]
    pub fn sol_log_compute_units() {}
}
pub mod memory {
    //! Basic low-level memory operations.
    //!
    //! Within the SBF environment, these are implemented as syscalls and executed by
    //! the runtime in native code.
    /// Like C `memcpy`.
    ///
    /// # Arguments
    ///
    /// - `dst` - Destination
    /// - `src` - Source
    /// - `n` - Number of bytes to copy
    ///
    /// # Errors
    ///
    /// When executed within a SBF program, the memory regions spanning `n` bytes
    /// from from the start of `dst` and `src` must be mapped program memory. If not,
    /// the program will abort.
    ///
    /// The memory regions spanning `n` bytes from `dst` and `src` from the start
    /// of `dst` and `src` must not overlap. If they do, then the program will abort
    /// or, if run outside of the SBF VM, will panic.
    ///
    /// # Safety
    ///
    /// This function does not verify that `n` is less than or equal to the
    /// lengths of the `dst` and `src` slices passed to it &mdash; it will copy
    /// bytes to and from beyond the slices.
    ///
    /// Specifying an `n` greater than either the length of `dst` or `src` will
    /// likely introduce undefined behavior.
    #[inline]
    pub unsafe fn sol_memcpy(dst: &mut [u8], src: &[u8], n: usize) {
        #[cfg(not(target_os = "solana"))] core::hint::black_box((dst, src, n));
    }
    /// Like C `memmove`.
    ///
    /// # Arguments
    ///
    /// - `dst` - Destination
    /// - `src` - Source
    /// - `n` - Number of bytes to copy
    ///
    /// # Errors
    ///
    /// When executed within a SBF program, the memory regions spanning `n` bytes
    /// from from `dst` and `src` must be mapped program memory. If not, the program
    /// will abort.
    ///
    /// # Safety
    ///
    /// The same safety rules apply as in [`ptr::copy`].
    ///
    /// [`ptr::copy`]: https://doc.rust-lang.org/std/ptr/fn.copy.html
    #[inline]
    pub unsafe fn sol_memmove(dst: *mut u8, src: *mut u8, n: usize) {
        #[cfg(not(target_os = "solana"))] core::hint::black_box((dst, src, n));
    }
    /// Like C `memcmp`.
    ///
    /// # Arguments
    ///
    /// - `s1` - Slice to be compared
    /// - `s2` - Slice to be compared
    /// - `n` - Number of bytes to compare
    ///
    /// # Errors
    ///
    /// When executed within a SBF program, the memory regions spanning `n` bytes
    /// from from the start of `dst` and `src` must be mapped program memory. If not,
    /// the program will abort.
    ///
    /// # Safety
    ///
    /// It does not verify that `n` is less than or equal to the lengths of the
    /// `dst` and `src` slices passed to it &mdash; it will read bytes beyond the
    /// slices.
    ///
    /// Specifying an `n` greater than either the length of `dst` or `src` will
    /// likely introduce undefined behavior.
    #[inline]
    pub unsafe fn sol_memcmp(s1: &[u8], s2: &[u8], n: usize) -> i32 {
        #[allow(unused_mut)]
        let mut result = 0;
        #[cfg(not(target_os = "solana"))] core::hint::black_box((s1, s2, n, result));
        result
    }
    /// Like C `memset`.
    ///
    /// # Arguments
    ///
    /// - `s` - Slice to be set
    /// - `c` - Repeated byte to set
    /// - `n` - Number of bytes to set
    ///
    /// # Errors
    ///
    /// When executed within a SBF program, the memory region spanning `n` bytes
    /// from from the start of `s` must be mapped program memory. If not, the program
    /// will abort.
    ///
    /// # Safety
    ///
    /// This function does not verify that `n` is less than or equal to the length
    /// of the `s` slice passed to it &mdash; it will write bytes beyond the
    /// slice.
    ///
    /// Specifying an `n` greater than the length of `s` will likely introduce
    /// undefined behavior.
    #[inline]
    pub unsafe fn sol_memset(s: &mut [u8], c: u8, n: usize) {
        #[cfg(not(target_os = "solana"))] core::hint::black_box((s, c, n));
    }
}
pub mod program {
    //! Cross-program invocation helpers.
    use core::{mem::MaybeUninit, ops::Deref};
    use crate::pinocchio::{
        account_info::AccountInfo,
        instruction::{Account, AccountMeta, Instruction, Signer},
        program_error::ProgramError, pubkey::Pubkey, ProgramResult,
    };
    /// An `Instruction` as expected by `sol_invoke_signed_c`.
    ///
    /// DO NOT EXPOSE THIS STRUCT:
    ///
    /// To ensure pointers are valid upon use, the scope of this struct should
    /// only be limited to the stack where sol_invoke_signed_c happens and then
    /// discarded immediately after.
    #[repr(C)]
    struct CInstruction<'a> {
        /// Public key of the program.
        program_id: *const Pubkey,
        /// Accounts expected by the program instruction.
        accounts: *const AccountMeta<'a>,
        /// Number of accounts expected by the program instruction.
        accounts_len: u64,
        /// Data expected by the program instruction.
        data: *const u8,
        /// Length of the data expected by the program instruction.
        data_len: u64,
    }
    #[automatically_derived]
    impl<'a> ::core::fmt::Debug for CInstruction<'a> {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field5_finish(
                f,
                "CInstruction",
                "program_id",
                &self.program_id,
                "accounts",
                &self.accounts,
                "accounts_len",
                &self.accounts_len,
                "data",
                &self.data,
                "data_len",
                &&self.data_len,
            )
        }
    }
    #[automatically_derived]
    impl<'a> ::core::marker::StructuralPartialEq for CInstruction<'a> {}
    #[automatically_derived]
    impl<'a> ::core::cmp::PartialEq for CInstruction<'a> {
        #[inline]
        fn eq(&self, other: &CInstruction<'a>) -> bool {
            self.program_id == other.program_id && self.accounts == other.accounts
                && self.accounts_len == other.accounts_len && self.data == other.data
                && self.data_len == other.data_len
        }
    }
    #[automatically_derived]
    impl<'a> ::core::clone::Clone for CInstruction<'a> {
        #[inline]
        fn clone(&self) -> CInstruction<'a> {
            CInstruction {
                program_id: ::core::clone::Clone::clone(&self.program_id),
                accounts: ::core::clone::Clone::clone(&self.accounts),
                accounts_len: ::core::clone::Clone::clone(&self.accounts_len),
                data: ::core::clone::Clone::clone(&self.data),
                data_len: ::core::clone::Clone::clone(&self.data_len),
            }
        }
    }
    impl<'a> From<&Instruction<'a, '_, '_, '_>> for CInstruction<'a> {
        fn from(instruction: &Instruction<'a, '_, '_, '_>) -> Self {
            CInstruction {
                program_id: instruction.program_id,
                accounts: instruction.accounts.as_ptr(),
                accounts_len: instruction.accounts.len() as u64,
                data: instruction.data.as_ptr(),
                data_len: instruction.data.len() as u64,
            }
        }
    }
    /// Invoke a cross-program instruction.
    ///
    /// # Important
    ///
    /// The accounts on the `account_infos` slice must be in the same order as the
    /// `accounts` field of the `instruction`.
    #[inline(always)]
    pub fn invoke<const ACCOUNTS: usize>(
        instruction: &Instruction,
        account_infos: &[&AccountInfo; ACCOUNTS],
    ) -> ProgramResult {
        invoke_signed(instruction, account_infos, &[])
    }
    /// Invoke a cross-program instruction with signatures.
    ///
    /// # Important
    ///
    /// The accounts on the `account_infos` slice must be in the same order as the
    /// `accounts` field of the `instruction`.
    pub fn invoke_signed<const ACCOUNTS: usize>(
        instruction: &Instruction,
        account_infos: &[&AccountInfo; ACCOUNTS],
        signers_seeds: &[Signer],
    ) -> ProgramResult {
        if instruction.accounts.len() < ACCOUNTS {
            return Err(ProgramError::NotEnoughAccountKeys);
        }
        const UNINIT: MaybeUninit<Account> = MaybeUninit::<Account>::uninit();
        let mut accounts = [UNINIT; ACCOUNTS];
        for index in 0..ACCOUNTS {
            let account_info = account_infos[index];
            let account_meta = &instruction.accounts[index];
            if account_info.key() != account_meta.pubkey {
                return Err(ProgramError::InvalidArgument);
            }
            if account_meta.is_writable {
                let _ = account_info.try_borrow_mut_data()?;
                let _ = account_info.try_borrow_mut_lamports()?;
            } else {
                let _ = account_info.try_borrow_data()?;
                let _ = account_info.try_borrow_lamports()?;
            }
            accounts[index].write(Account::from(account_infos[index]));
        }
        unsafe {
            invoke_signed_unchecked(
                instruction,
                core::slice::from_raw_parts(accounts.as_ptr() as _, ACCOUNTS),
                signers_seeds,
            );
        }
        Ok(())
    }
    /// Invoke a cross-program instruction but don't enforce Rust's aliasing rules.
    ///
    /// This function does not check that [`Ref`]s within [`Account`]s are properly
    /// borrowable as described in the documentation for that function. Those checks
    /// consume CPU cycles that this function avoids.
    ///
    /// # Safety
    ///
    /// If any of the writable accounts passed to the callee contain data that is
    /// borrowed within the calling program, and that data is written to by the
    /// callee, then Rust's aliasing rules will be violated and cause undefined
    /// behavior.
    ///
    /// # Important
    ///
    /// The accounts on the `account_infos` slice must be in the same order as the
    /// `accounts` field of the `instruction`.
    #[inline(always)]
    pub unsafe fn invoke_unchecked(instruction: &Instruction, accounts: &[Account]) {
        invoke_signed_unchecked(instruction, accounts, &[])
    }
    /// Invoke a cross-program instruction with signatures but don't enforce Rust's
    /// aliasing rules.
    ///
    /// This function does not check that [`Ref`]s within [`Account`]s are properly
    /// borrowable as described in the documentation for that function. Those checks
    /// consume CPU cycles that this function avoids.
    ///
    /// # Safety
    ///
    /// If any of the writable accounts passed to the callee contain data that is
    /// borrowed within the calling program, and that data is written to by the
    /// callee, then Rust's aliasing rules will be violated and cause undefined
    /// behavior.
    ///
    /// # Important
    ///
    /// The accounts on the `account_infos` slice must be in the same order as the
    /// `accounts` field of the `instruction`.
    pub unsafe fn invoke_signed_unchecked(
        instruction: &Instruction,
        accounts: &[Account],
        signers_seeds: &[Signer],
    ) {
        #[cfg(not(target_os = "solana"))]
        core::hint::black_box((instruction, accounts, signers_seeds));
    }
    /// Maximum size that can be set using [`set_return_data`].
    pub const MAX_RETURN_DATA: usize = 1024;
    /// Set the running program's return data.
    ///
    /// Return data is a dedicated per-transaction buffer for data passed
    /// from cross-program invoked programs back to their caller.
    ///
    /// The maximum size of return data is [`MAX_RETURN_DATA`]. Return data is
    /// retrieved by the caller with [`get_return_data`].
    pub fn set_return_data(data: &[u8]) {
        #[cfg(not(target_os = "solana"))] core::hint::black_box(data);
    }
    /// Get the return data from an invoked program.
    ///
    /// For every transaction there is a single buffer with maximum length
    /// [`MAX_RETURN_DATA`], paired with a [`Pubkey`] representing the program ID of
    /// the program that most recently set the return data. Thus the return data is
    /// a global resource and care must be taken to ensure that it represents what
    /// is expected: called programs are free to set or not set the return data; and
    /// the return data may represent values set by programs multiple calls down the
    /// call stack, depending on the circumstances of transaction execution.
    ///
    /// Return data is set by the callee with [`set_return_data`].
    ///
    /// Return data is cleared before every CPI invocation &mdash; a program that
    /// has invoked no other programs can expect the return data to be `None`; if no
    /// return data was set by the previous CPI invocation, then this function
    /// returns `None`.
    ///
    /// Return data is not cleared after returning from CPI invocations &mdash; a
    /// program that has called another program may retrieve return data that was
    /// not set by the called program, but instead set by a program further down the
    /// call stack; or, if a program calls itself recursively, it is possible that
    /// the return data was not set by the immediate call to that program, but by a
    /// subsequent recursive call to that program. Likewise, an external RPC caller
    /// may see return data that was not set by the program it is directly calling,
    /// but by a program that program called.
    ///
    /// For more about return data see the [documentation for the return data proposal][rdp].
    ///
    /// [rdp]: https://docs.solanalabs.com/proposals/return-data
    pub fn get_return_data() -> Option<ReturnData> {
        #[cfg(not(target_os = "solana"))] core::hint::black_box(None)
    }
    /// Struct to hold the return data from an invoked program.
    pub struct ReturnData {
        /// Program that most recently set the return data.
        program_id: Pubkey,
        /// Return data set by the program.
        data: [core::mem::MaybeUninit<u8>; MAX_RETURN_DATA],
        /// Length of the return data.
        size: usize,
    }
    impl ReturnData {
        /// Returns the program that most recently set the return data.
        pub fn program_id(&self) -> &Pubkey {
            &self.program_id
        }
        /// Return the data set by the program.
        pub fn as_slice(&self) -> &[u8] {
            unsafe { core::slice::from_raw_parts(self.data.as_ptr() as _, self.size) }
        }
    }
    impl Deref for ReturnData {
        type Target = [u8];
        fn deref(&self) -> &Self::Target {
            self.as_slice()
        }
    }
}
pub mod program_error {
    //! Errors generated by programs.
    //!
    //! Current implementation is based on the `ProgramError` enum from
    //! the Solana SDK:
    //!
    //! https://github.com/anza-xyz/agave/blob/master/sdk/program/src/program_error.rs
    //!
    //! Considerations:
    //!
    //! - Not deriving `thiserror::Error` for now, as it's not clear if it's needed.
    /// Reasons the program may fail.
    pub enum ProgramError {
        /// Allows on-chain programs to implement program-specific error types and see them returned
        /// by the Solana runtime. A program-specific error may be any type that is represented as
        /// or serialized to a u32 integer.
        ///
        /// Custom program error: `{0:#x}`
        Custom(u32),
        /// The arguments provided to a program instruction were invalid
        InvalidArgument,
        /// An instruction's data contents was invalid
        InvalidInstructionData,
        /// An account's data contents was invalid
        InvalidAccountData,
        /// An account's data was too small
        AccountDataTooSmall,
        /// An account's balance was too small to complete the instruction
        InsufficientFunds,
        /// The account did not have the expected program id
        IncorrectProgramId,
        /// A signature was required but not found
        MissingRequiredSignature,
        /// An initialize instruction was sent to an account that has already been initialized
        AccountAlreadyInitialized,
        /// An attempt to operate on an account that hasn't been initialized
        UninitializedAccount,
        /// The instruction expected additional account keys
        NotEnoughAccountKeys,
        /// Failed to borrow a reference to account data, already borrowed
        AccountBorrowFailed,
        /// Length of the seed is too long for address generation
        MaxSeedLengthExceeded,
        /// Provided seeds do not result in a valid address
        InvalidSeeds,
        /// IO Error
        BorshIoError,
        /// An account does not have enough lamports to be rent-exempt
        AccountNotRentExempt,
        /// Unsupported sysvar
        UnsupportedSysvar,
        /// Provided owner is not allowed
        IllegalOwner,
        /// Accounts data allocations exceeded the maximum allowed per transaction
        MaxAccountsDataAllocationsExceeded,
        /// Account data reallocation was invalid
        InvalidRealloc,
        /// Instruction trace length exceeded the maximum allowed per transaction
        MaxInstructionTraceLengthExceeded,
        /// Builtin programs must consume compute units
        BuiltinProgramsMustConsumeComputeUnits,
        /// Invalid account owner
        InvalidAccountOwner,
        /// Program arithmetic overflowed
        ArithmeticOverflow,
        /// Account is immutable
        Immutable,
        /// Incorrect authority provided
        IncorrectAuthority,
    }
    #[automatically_derived]
    impl ::core::clone::Clone for ProgramError {
        #[inline]
        fn clone(&self) -> ProgramError {
            match self {
                ProgramError::Custom(__self_0) => {
                    ProgramError::Custom(::core::clone::Clone::clone(__self_0))
                }
                ProgramError::InvalidArgument => ProgramError::InvalidArgument,
                ProgramError::InvalidInstructionData => {
                    ProgramError::InvalidInstructionData
                }
                ProgramError::InvalidAccountData => ProgramError::InvalidAccountData,
                ProgramError::AccountDataTooSmall => ProgramError::AccountDataTooSmall,
                ProgramError::InsufficientFunds => ProgramError::InsufficientFunds,
                ProgramError::IncorrectProgramId => ProgramError::IncorrectProgramId,
                ProgramError::MissingRequiredSignature => {
                    ProgramError::MissingRequiredSignature
                }
                ProgramError::AccountAlreadyInitialized => {
                    ProgramError::AccountAlreadyInitialized
                }
                ProgramError::UninitializedAccount => ProgramError::UninitializedAccount,
                ProgramError::NotEnoughAccountKeys => ProgramError::NotEnoughAccountKeys,
                ProgramError::AccountBorrowFailed => ProgramError::AccountBorrowFailed,
                ProgramError::MaxSeedLengthExceeded => {
                    ProgramError::MaxSeedLengthExceeded
                }
                ProgramError::InvalidSeeds => ProgramError::InvalidSeeds,
                ProgramError::BorshIoError => ProgramError::BorshIoError,
                ProgramError::AccountNotRentExempt => ProgramError::AccountNotRentExempt,
                ProgramError::UnsupportedSysvar => ProgramError::UnsupportedSysvar,
                ProgramError::IllegalOwner => ProgramError::IllegalOwner,
                ProgramError::MaxAccountsDataAllocationsExceeded => {
                    ProgramError::MaxAccountsDataAllocationsExceeded
                }
                ProgramError::InvalidRealloc => ProgramError::InvalidRealloc,
                ProgramError::MaxInstructionTraceLengthExceeded => {
                    ProgramError::MaxInstructionTraceLengthExceeded
                }
                ProgramError::BuiltinProgramsMustConsumeComputeUnits => {
                    ProgramError::BuiltinProgramsMustConsumeComputeUnits
                }
                ProgramError::InvalidAccountOwner => ProgramError::InvalidAccountOwner,
                ProgramError::ArithmeticOverflow => ProgramError::ArithmeticOverflow,
                ProgramError::Immutable => ProgramError::Immutable,
                ProgramError::IncorrectAuthority => ProgramError::IncorrectAuthority,
            }
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for ProgramError {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                ProgramError::Custom(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Custom",
                        &__self_0,
                    )
                }
                ProgramError::InvalidArgument => {
                    ::core::fmt::Formatter::write_str(f, "InvalidArgument")
                }
                ProgramError::InvalidInstructionData => {
                    ::core::fmt::Formatter::write_str(f, "InvalidInstructionData")
                }
                ProgramError::InvalidAccountData => {
                    ::core::fmt::Formatter::write_str(f, "InvalidAccountData")
                }
                ProgramError::AccountDataTooSmall => {
                    ::core::fmt::Formatter::write_str(f, "AccountDataTooSmall")
                }
                ProgramError::InsufficientFunds => {
                    ::core::fmt::Formatter::write_str(f, "InsufficientFunds")
                }
                ProgramError::IncorrectProgramId => {
                    ::core::fmt::Formatter::write_str(f, "IncorrectProgramId")
                }
                ProgramError::MissingRequiredSignature => {
                    ::core::fmt::Formatter::write_str(f, "MissingRequiredSignature")
                }
                ProgramError::AccountAlreadyInitialized => {
                    ::core::fmt::Formatter::write_str(f, "AccountAlreadyInitialized")
                }
                ProgramError::UninitializedAccount => {
                    ::core::fmt::Formatter::write_str(f, "UninitializedAccount")
                }
                ProgramError::NotEnoughAccountKeys => {
                    ::core::fmt::Formatter::write_str(f, "NotEnoughAccountKeys")
                }
                ProgramError::AccountBorrowFailed => {
                    ::core::fmt::Formatter::write_str(f, "AccountBorrowFailed")
                }
                ProgramError::MaxSeedLengthExceeded => {
                    ::core::fmt::Formatter::write_str(f, "MaxSeedLengthExceeded")
                }
                ProgramError::InvalidSeeds => {
                    ::core::fmt::Formatter::write_str(f, "InvalidSeeds")
                }
                ProgramError::BorshIoError => {
                    ::core::fmt::Formatter::write_str(f, "BorshIoError")
                }
                ProgramError::AccountNotRentExempt => {
                    ::core::fmt::Formatter::write_str(f, "AccountNotRentExempt")
                }
                ProgramError::UnsupportedSysvar => {
                    ::core::fmt::Formatter::write_str(f, "UnsupportedSysvar")
                }
                ProgramError::IllegalOwner => {
                    ::core::fmt::Formatter::write_str(f, "IllegalOwner")
                }
                ProgramError::MaxAccountsDataAllocationsExceeded => {
                    ::core::fmt::Formatter::write_str(
                        f,
                        "MaxAccountsDataAllocationsExceeded",
                    )
                }
                ProgramError::InvalidRealloc => {
                    ::core::fmt::Formatter::write_str(f, "InvalidRealloc")
                }
                ProgramError::MaxInstructionTraceLengthExceeded => {
                    ::core::fmt::Formatter::write_str(
                        f,
                        "MaxInstructionTraceLengthExceeded",
                    )
                }
                ProgramError::BuiltinProgramsMustConsumeComputeUnits => {
                    ::core::fmt::Formatter::write_str(
                        f,
                        "BuiltinProgramsMustConsumeComputeUnits",
                    )
                }
                ProgramError::InvalidAccountOwner => {
                    ::core::fmt::Formatter::write_str(f, "InvalidAccountOwner")
                }
                ProgramError::ArithmeticOverflow => {
                    ::core::fmt::Formatter::write_str(f, "ArithmeticOverflow")
                }
                ProgramError::Immutable => {
                    ::core::fmt::Formatter::write_str(f, "Immutable")
                }
                ProgramError::IncorrectAuthority => {
                    ::core::fmt::Formatter::write_str(f, "IncorrectAuthority")
                }
            }
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for ProgramError {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<u32>;
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for ProgramError {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for ProgramError {
        #[inline]
        fn eq(&self, other: &ProgramError) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
                && match (self, other) {
                    (ProgramError::Custom(__self_0), ProgramError::Custom(__arg1_0)) => {
                        __self_0 == __arg1_0
                    }
                    _ => true,
                }
        }
    }
    /// Builtin return values occupy the upper 32 bits
    const BUILTIN_BIT_SHIFT: usize = 32;
    pub const CUSTOM_ZERO: u64 = (1 as u64) << BUILTIN_BIT_SHIFT;
    pub const INVALID_ARGUMENT: u64 = (2 as u64) << BUILTIN_BIT_SHIFT;
    pub const INVALID_INSTRUCTION_DATA: u64 = (3 as u64) << BUILTIN_BIT_SHIFT;
    pub const INVALID_ACCOUNT_DATA: u64 = (4 as u64) << BUILTIN_BIT_SHIFT;
    pub const ACCOUNT_DATA_TOO_SMALL: u64 = (5 as u64) << BUILTIN_BIT_SHIFT;
    pub const INSUFFICIENT_FUNDS: u64 = (6 as u64) << BUILTIN_BIT_SHIFT;
    pub const INCORRECT_PROGRAM_ID: u64 = (7 as u64) << BUILTIN_BIT_SHIFT;
    pub const MISSING_REQUIRED_SIGNATURES: u64 = (8 as u64) << BUILTIN_BIT_SHIFT;
    pub const ACCOUNT_ALREADY_INITIALIZED: u64 = (9 as u64) << BUILTIN_BIT_SHIFT;
    pub const UNINITIALIZED_ACCOUNT: u64 = (10 as u64) << BUILTIN_BIT_SHIFT;
    pub const NOT_ENOUGH_ACCOUNT_KEYS: u64 = (11 as u64) << BUILTIN_BIT_SHIFT;
    pub const ACCOUNT_BORROW_FAILED: u64 = (12 as u64) << BUILTIN_BIT_SHIFT;
    pub const MAX_SEED_LENGTH_EXCEEDED: u64 = (13 as u64) << BUILTIN_BIT_SHIFT;
    pub const INVALID_SEEDS: u64 = (14 as u64) << BUILTIN_BIT_SHIFT;
    pub const BORSH_IO_ERROR: u64 = (15 as u64) << BUILTIN_BIT_SHIFT;
    pub const ACCOUNT_NOT_RENT_EXEMPT: u64 = (16 as u64) << BUILTIN_BIT_SHIFT;
    pub const UNSUPPORTED_SYSVAR: u64 = (17 as u64) << BUILTIN_BIT_SHIFT;
    pub const ILLEGAL_OWNER: u64 = (18 as u64) << BUILTIN_BIT_SHIFT;
    pub const MAX_ACCOUNTS_DATA_ALLOCATIONS_EXCEEDED: u64 = (19 as u64)
        << BUILTIN_BIT_SHIFT;
    pub const INVALID_ACCOUNT_DATA_REALLOC: u64 = (20 as u64) << BUILTIN_BIT_SHIFT;
    pub const MAX_INSTRUCTION_TRACE_LENGTH_EXCEEDED: u64 = (21 as u64)
        << BUILTIN_BIT_SHIFT;
    pub const BUILTIN_PROGRAMS_MUST_CONSUME_COMPUTE_UNITS: u64 = (22 as u64)
        << BUILTIN_BIT_SHIFT;
    pub const INVALID_ACCOUNT_OWNER: u64 = (23 as u64) << BUILTIN_BIT_SHIFT;
    pub const ARITHMETIC_OVERFLOW: u64 = (24 as u64) << BUILTIN_BIT_SHIFT;
    pub const IMMUTABLE: u64 = (25 as u64) << BUILTIN_BIT_SHIFT;
    pub const INCORRECT_AUTHORITY: u64 = (26 as u64) << BUILTIN_BIT_SHIFT;
    impl From<u64> for ProgramError {
        fn from(error: u64) -> Self {
            match error {
                CUSTOM_ZERO => Self::Custom(0),
                INVALID_ARGUMENT => Self::InvalidArgument,
                INVALID_INSTRUCTION_DATA => Self::InvalidInstructionData,
                INVALID_ACCOUNT_DATA => Self::InvalidAccountData,
                ACCOUNT_DATA_TOO_SMALL => Self::AccountDataTooSmall,
                INSUFFICIENT_FUNDS => Self::InsufficientFunds,
                INCORRECT_PROGRAM_ID => Self::IncorrectProgramId,
                MISSING_REQUIRED_SIGNATURES => Self::MissingRequiredSignature,
                ACCOUNT_ALREADY_INITIALIZED => Self::AccountAlreadyInitialized,
                UNINITIALIZED_ACCOUNT => Self::UninitializedAccount,
                NOT_ENOUGH_ACCOUNT_KEYS => Self::NotEnoughAccountKeys,
                ACCOUNT_BORROW_FAILED => Self::AccountBorrowFailed,
                MAX_SEED_LENGTH_EXCEEDED => Self::MaxSeedLengthExceeded,
                INVALID_SEEDS => Self::InvalidSeeds,
                BORSH_IO_ERROR => Self::BorshIoError,
                ACCOUNT_NOT_RENT_EXEMPT => Self::AccountNotRentExempt,
                UNSUPPORTED_SYSVAR => Self::UnsupportedSysvar,
                ILLEGAL_OWNER => Self::IllegalOwner,
                MAX_ACCOUNTS_DATA_ALLOCATIONS_EXCEEDED => {
                    Self::MaxAccountsDataAllocationsExceeded
                }
                INVALID_ACCOUNT_DATA_REALLOC => Self::InvalidRealloc,
                MAX_INSTRUCTION_TRACE_LENGTH_EXCEEDED => {
                    Self::MaxInstructionTraceLengthExceeded
                }
                BUILTIN_PROGRAMS_MUST_CONSUME_COMPUTE_UNITS => {
                    Self::BuiltinProgramsMustConsumeComputeUnits
                }
                INVALID_ACCOUNT_OWNER => Self::InvalidAccountOwner,
                ARITHMETIC_OVERFLOW => Self::ArithmeticOverflow,
                IMMUTABLE => Self::Immutable,
                INCORRECT_AUTHORITY => Self::IncorrectAuthority,
                _ => Self::Custom(error as u32),
            }
        }
    }
    impl From<ProgramError> for u64 {
        fn from(error: ProgramError) -> Self {
            match error {
                ProgramError::InvalidArgument => INVALID_ARGUMENT,
                ProgramError::InvalidInstructionData => INVALID_INSTRUCTION_DATA,
                ProgramError::InvalidAccountData => INVALID_ACCOUNT_DATA,
                ProgramError::AccountDataTooSmall => ACCOUNT_DATA_TOO_SMALL,
                ProgramError::InsufficientFunds => INSUFFICIENT_FUNDS,
                ProgramError::IncorrectProgramId => INCORRECT_PROGRAM_ID,
                ProgramError::MissingRequiredSignature => MISSING_REQUIRED_SIGNATURES,
                ProgramError::AccountAlreadyInitialized => ACCOUNT_ALREADY_INITIALIZED,
                ProgramError::UninitializedAccount => UNINITIALIZED_ACCOUNT,
                ProgramError::NotEnoughAccountKeys => NOT_ENOUGH_ACCOUNT_KEYS,
                ProgramError::AccountBorrowFailed => ACCOUNT_BORROW_FAILED,
                ProgramError::MaxSeedLengthExceeded => MAX_SEED_LENGTH_EXCEEDED,
                ProgramError::InvalidSeeds => INVALID_SEEDS,
                ProgramError::BorshIoError => BORSH_IO_ERROR,
                ProgramError::AccountNotRentExempt => ACCOUNT_NOT_RENT_EXEMPT,
                ProgramError::UnsupportedSysvar => UNSUPPORTED_SYSVAR,
                ProgramError::IllegalOwner => ILLEGAL_OWNER,
                ProgramError::MaxAccountsDataAllocationsExceeded => {
                    MAX_ACCOUNTS_DATA_ALLOCATIONS_EXCEEDED
                }
                ProgramError::InvalidRealloc => INVALID_ACCOUNT_DATA_REALLOC,
                ProgramError::MaxInstructionTraceLengthExceeded => {
                    MAX_INSTRUCTION_TRACE_LENGTH_EXCEEDED
                }
                ProgramError::BuiltinProgramsMustConsumeComputeUnits => {
                    BUILTIN_PROGRAMS_MUST_CONSUME_COMPUTE_UNITS
                }
                ProgramError::InvalidAccountOwner => INVALID_ACCOUNT_OWNER,
                ProgramError::ArithmeticOverflow => ARITHMETIC_OVERFLOW,
                ProgramError::Immutable => IMMUTABLE,
                ProgramError::IncorrectAuthority => INCORRECT_AUTHORITY,
                ProgramError::Custom(error) => {
                    if error == 0 { CUSTOM_ZERO } else { error as u64 }
                }
            }
        }
    }
}
pub mod pubkey {
    //! Public key type and functions.
    use crate::pinocchio::program_error::ProgramError;
    /// Number of bytes in a pubkey
    pub const PUBKEY_BYTES: usize = 32;
    /// maximum length of derived `Pubkey` seed
    pub const MAX_SEED_LEN: usize = 32;
    /// Maximum number of seeds
    pub const MAX_SEEDS: usize = 16;
    /// The address of a [Solana account][account].
    ///
    /// [account]: https://solana.com/docs/core/accounts
    pub type Pubkey = [u8; PUBKEY_BYTES];
    /// Log a `Pubkey` from a program
    #[inline(always)]
    pub fn log(pubkey: &Pubkey) {
        #[cfg(not(target_os = "solana"))] core::hint::black_box(pubkey);
    }
    /// Find a valid [program derived address][pda] and its corresponding bump seed.
    ///
    /// [pda]: https://solana.com/docs/core/cpi#program-derived-addresses
    ///
    /// Program derived addresses (PDAs) are account keys that only the program,
    /// `program_id`, has the authority to sign. The address is of the same form
    /// as a Solana `Pubkey`, except they are ensured to not be on the ed25519
    /// curve and thus have no associated private key. When performing
    /// cross-program invocations the program can "sign" for the key by calling
    /// [`invoke_signed`] and passing the same seeds used to generate the
    /// address, along with the calculated _bump seed_, which this function
    /// returns as the second tuple element. The runtime will verify that the
    /// program associated with this address is the caller and thus authorized
    /// to be the signer.
    ///
    /// [`invoke_signed`]: crate::program::invoke_signed
    ///
    /// The `seeds` are application-specific, and must be carefully selected to
    /// uniquely derive accounts per application requirements. It is common to
    /// use static strings and other pubkeys as seeds.
    ///
    /// Because the program address must not lie on the ed25519 curve, there may
    /// be seed and program id combinations that are invalid. For this reason,
    /// an extra seed (the bump seed) is calculated that results in a
    /// point off the curve. The bump seed must be passed as an additional seed
    /// when calling `invoke_signed`.
    ///
    /// The processes of finding a valid program address is by trial and error,
    /// and even though it is deterministic given a set of inputs it can take a
    /// variable amount of time to succeed across different inputs.  This means
    /// that when called from an on-chain program it may incur a variable amount
    /// of the program's compute budget.  Programs that are meant to be very
    /// performant may not want to use this function because it could take a
    /// considerable amount of time. Programs that are already at risk
    /// of exceeding their compute budget should call this with care since
    /// there is a chance that the program's budget may be occasionally
    /// and unpredictably exceeded.
    ///
    /// As all account addresses accessed by an on-chain Solana program must be
    /// explicitly passed to the program, it is typical for the PDAs to be
    /// derived in off-chain client programs, avoiding the compute cost of
    /// generating the address on-chain. The address may or may not then be
    /// verified by re-deriving it on-chain, depending on the requirements of
    /// the program. This verification may be performed without the overhead of
    /// re-searching for the bump key by using the [`create_program_address`]
    /// function.
    ///
    /// [`create_program_address`]: Pubkey::create_program_address
    ///
    /// **Warning**: Because of the way the seeds are hashed there is a potential
    /// for program address collisions for the same program id.  The seeds are
    /// hashed sequentially which means that seeds {"abcdef"}, {"abc", "def"},
    /// and {"ab", "cd", "ef"} will all result in the same program address given
    /// the same program id. Since the chance of collision is local to a given
    /// program id, the developer of that program must take care to choose seeds
    /// that do not collide with each other. For seed schemes that are susceptible
    /// to this type of hash collision, a common remedy is to insert separators
    /// between seeds, e.g. transforming {"abc", "def"} into {"abc", "-", "def"}.
    ///
    /// # Panics
    ///
    /// Panics in the statistically improbable event that a bump seed could not be
    /// found. Use [`try_find_program_address`] to handle this case.
    ///
    /// [`try_find_program_address`]: #try_find_program_address
    ///
    /// Panics if any of the following are true:
    ///
    /// - the number of provided seeds is greater than, _or equal to_, [`MAX_SEEDS`],
    /// - any individual seed's length is greater than [`MAX_SEED_LEN`].
    #[inline(always)]
    pub fn find_program_address(seeds: &[&[u8]], program_id: &Pubkey) -> (Pubkey, u8) {
        try_find_program_address(seeds, program_id)
            .unwrap_or_else(|| {
                ::core::panicking::panic_fmt(
                    format_args!("Unable to find a viable program address bump seed"),
                );
            })
    }
    /// Find a valid [program derived address][pda] and its corresponding bump seed.
    ///
    /// [pda]: https://solana.com/docs/core/cpi#program-derived-addresses
    ///
    /// The only difference between this method and [`find_program_address`]
    /// is that this one returns `None` in the statistically improbable event
    /// that a bump seed cannot be found; or if any of `find_program_address`'s
    /// preconditions are violated.
    ///
    /// See the documentation for [`find_program_address`] for a full description.
    ///
    /// [`find_program_address`]: #find_program_address
    #[inline]
    pub fn try_find_program_address(
        seeds: &[&[u8]],
        program_id: &Pubkey,
    ) -> Option<(Pubkey, u8)> {
        #[cfg(not(target_os = "solana"))]
        {
            core::hint::black_box((seeds, program_id));
            None
        }
    }
    /// Create a valid [program derived address][pda] without searching for a bump seed.
    ///
    /// [pda]: https://solana.com/docs/core/cpi#program-derived-addresses
    ///
    /// Because this function does not create a bump seed, it may unpredictably
    /// return an error for any given set of seeds and is not generally suitable
    /// for creating program derived addresses.
    ///
    /// However, it can be used for efficiently verifying that a set of seeds plus
    /// bump seed generated by [`find_program_address`] derives a particular
    /// address as expected. See the example for details.
    ///
    /// See the documentation for [`find_program_address`] for a full description
    /// of program derived addresses and bump seeds.
    ///
    /// Note that this function does *not* validate whether the given `seeds` are within
    /// the valid length or not. It will return an error in case of invalid seeds length,
    /// incurring the cost of the syscall.
    ///
    /// [`find_program_address`]: #find_program_address
    #[inline]
    pub fn create_program_address(
        seeds: &[&[u8]],
        program_id: &Pubkey,
    ) -> Result<Pubkey, ProgramError> {
        #[cfg(not(target_os = "solana"))]
        {
            core::hint::black_box((seeds, program_id));
            {
                ::core::panicking::panic_fmt(
                    format_args!(
                        "create_program_address is only available on target `solana`",
                    ),
                );
            }
        }
    }
    /// Create a valid [program derived address][pda] without searching for a bump seed.
    ///
    /// [pda]: https://solana.com/docs/core/cpi#program-derived-addresses
    ///
    /// Because this function does not create a bump seed, it may unpredictably
    /// return an error for any given set of seeds and is not generally suitable
    /// for creating program derived addresses.
    ///
    /// However, it can be used for efficiently verifying that a set of seeds plus
    /// bump seed generated by [`find_program_address`] derives a particular
    /// address as expected. See the example for details.
    ///
    /// See the documentation for [`find_program_address`] for a full description
    /// of program derived addresses and bump seeds.
    ///
    /// Note that this function validates whether the given `seeds` are within the valid
    /// length or not, returning an error without incurring the cost of the syscall.
    ///
    /// [`find_program_address`]: #find_program_address
    #[inline(always)]
    pub fn checked_create_program_address(
        seeds: &[&[u8]],
        program_id: &Pubkey,
    ) -> Result<Pubkey, ProgramError> {
        if seeds.len() > MAX_SEEDS {
            return Err(ProgramError::MaxSeedLengthExceeded);
        }
        if seeds.iter().any(|seed| seed.len() > MAX_SEED_LEN) {
            return Err(ProgramError::MaxSeedLengthExceeded);
        }
        create_program_address(seeds, program_id)
    }
}
pub mod syscalls {
    //! Syscall functions.
    use crate::pinocchio::{
        instruction::{AccountMeta, ProcessedSiblingInstruction},
        pubkey::Pubkey,
    };
    extern "C" {
        pub fn sol_log_(message: *const u8, len: u64) -> ();
    }
    extern "C" {
        pub fn sol_log_64_(arg1: u64, arg2: u64, arg3: u64, arg4: u64, arg5: u64) -> ();
    }
    extern "C" {
        pub fn sol_log_compute_units_() -> ();
    }
    extern "C" {
        pub fn sol_log_pubkey(pubkey_addr: *const u8) -> ();
    }
    extern "C" {
        pub fn sol_create_program_address(
            seeds_addr: *const u8,
            seeds_len: u64,
            program_id_addr: *const u8,
            address_bytes_addr: *const u8,
        ) -> u64;
    }
    extern "C" {
        pub fn sol_try_find_program_address(
            seeds_addr: *const u8,
            seeds_len: u64,
            program_id_addr: *const u8,
            address_bytes_addr: *const u8,
            bump_seed_addr: *const u8,
        ) -> u64;
    }
    extern "C" {
        pub fn sol_sha256(vals: *const u8, val_len: u64, hash_result: *mut u8) -> u64;
    }
    extern "C" {
        pub fn sol_keccak256(vals: *const u8, val_len: u64, hash_result: *mut u8) -> u64;
    }
    extern "C" {
        pub fn sol_secp256k1_recover(
            hash: *const u8,
            recovery_id: u64,
            signature: *const u8,
            result: *mut u8,
        ) -> u64;
    }
    extern "C" {
        pub fn sol_blake3(vals: *const u8, val_len: u64, hash_result: *mut u8) -> u64;
    }
    extern "C" {
        pub fn sol_get_clock_sysvar(addr: *mut u8) -> u64;
    }
    extern "C" {
        pub fn sol_get_epoch_schedule_sysvar(addr: *mut u8) -> u64;
    }
    extern "C" {
        pub fn sol_get_fees_sysvar(addr: *mut u8) -> u64;
    }
    extern "C" {
        pub fn sol_get_rent_sysvar(addr: *mut u8) -> u64;
    }
    extern "C" {
        pub fn sol_get_last_restart_slot(addr: *mut u8) -> u64;
    }
    extern "C" {
        pub fn sol_memcpy_(dst: *mut u8, src: *const u8, n: u64) -> ();
    }
    extern "C" {
        pub fn sol_memmove_(dst: *mut u8, src: *const u8, n: u64) -> ();
    }
    extern "C" {
        pub fn sol_memcmp_(s1: *const u8, s2: *const u8, n: u64, result: *mut i32) -> ();
    }
    extern "C" {
        pub fn sol_memset_(s: *mut u8, c: u8, n: u64) -> ();
    }
    extern "C" {
        pub fn sol_invoke_signed_c(
            instruction_addr: *const u8,
            account_infos_addr: *const u8,
            account_infos_len: u64,
            signers_seeds_addr: *const u8,
            signers_seeds_len: u64,
        ) -> u64;
    }
    extern "C" {
        pub fn sol_invoke_signed_rust(
            instruction_addr: *const u8,
            account_infos_addr: *const u8,
            account_infos_len: u64,
            signers_seeds_addr: *const u8,
            signers_seeds_len: u64,
        ) -> u64;
    }
    extern "C" {
        pub fn sol_set_return_data(data: *const u8, length: u64) -> ();
    }
    extern "C" {
        pub fn sol_get_return_data(
            data: *mut u8,
            length: u64,
            program_id: *mut Pubkey,
        ) -> u64;
    }
    extern "C" {
        pub fn sol_log_data(data: *const u8, data_len: u64) -> ();
    }
    extern "C" {
        pub fn sol_get_processed_sibling_instruction(
            index: u64,
            meta: *mut ProcessedSiblingInstruction,
            program_id: *mut Pubkey,
            data: *mut u8,
            accounts: *mut AccountMeta,
        ) -> u64;
    }
    extern "C" {
        pub fn sol_get_stack_height() -> u64;
    }
    extern "C" {
        pub fn sol_curve_validate_point(
            curve_id: u64,
            point_addr: *const u8,
            result: *mut u8,
        ) -> u64;
    }
    extern "C" {
        pub fn sol_curve_group_op(
            curve_id: u64,
            group_op: u64,
            left_input_addr: *const u8,
            right_input_addr: *const u8,
            result_point_addr: *mut u8,
        ) -> u64;
    }
    extern "C" {
        pub fn sol_curve_multiscalar_mul(
            curve_id: u64,
            scalars_addr: *const u8,
            points_addr: *const u8,
            points_len: u64,
            result_point_addr: *mut u8,
        ) -> u64;
    }
    extern "C" {
        pub fn sol_curve_pairing_map(
            curve_id: u64,
            point: *const u8,
            result: *mut u8,
        ) -> u64;
    }
    extern "C" {
        pub fn sol_alt_bn128_group_op(
            group_op: u64,
            input: *const u8,
            input_size: u64,
            result: *mut u8,
        ) -> u64;
    }
    extern "C" {
        pub fn sol_big_mod_exp(params: *const u8, result: *mut u8) -> u64;
    }
    extern "C" {
        pub fn sol_get_epoch_rewards_sysvar(addr: *mut u8) -> u64;
    }
    extern "C" {
        pub fn sol_poseidon(
            parameters: u64,
            endianness: u64,
            vals: *const u8,
            val_len: u64,
            hash_result: *mut u8,
        ) -> u64;
    }
    extern "C" {
        pub fn sol_remaining_compute_units() -> u64;
    }
    extern "C" {
        pub fn sol_alt_bn128_compression(
            op: u64,
            input: *const u8,
            input_size: u64,
            result: *mut u8,
        ) -> u64;
    }
}
pub mod sysvars {
    //! Provides access to cluster system accounts.
    use crate::pinocchio::program_error::ProgramError;
    pub mod clock {
        //! Information about the network's clock, ticks, slots, etc.
        use super::Sysvar;
        //use crate::pinocchio::impl_sysvar_get;
        /// The unit of time given to a leader for encoding a block.
        ///
        /// It is some some number of _ticks_ long.
        pub type Slot = u64;
        /// The unit of time a given leader schedule is honored.
        ///
        /// It lasts for some number of [`Slot`]s.
        pub type Epoch = u64;
        /// An approximate measure of real-world time.
        ///
        /// Expressed as Unix time (i.e. seconds since the Unix epoch).
        pub type UnixTimestamp = i64;
        /// A representation of network time.
        ///
        /// All members of `Clock` start from 0 upon network boot.
        #[repr(C)]
        pub struct Clock {
            /// The current `Slot`.
            pub slot: Slot,
            /// The timestamp of the first `Slot` in this `Epoch`.
            pub epoch_start_timestamp: UnixTimestamp,
            /// The current `Epoch`.
            pub epoch: Epoch,
            /// The future `Epoch` for which the leader schedule has
            /// most recently been calculated.
            pub leader_schedule_epoch: Epoch,
            /// The approximate real world time of the current slot.
            ///
            /// This value was originally computed from genesis creation time and
            /// network time in slots, incurring a lot of drift. Following activation of
            /// the [`timestamp_correction` and `timestamp_bounding`][tsc] features it
            /// is calculated using a [validator timestamp oracle][oracle].
            ///
            /// [tsc]: https://docs.solanalabs.com/implemented-proposals/bank-timestamp-correction
            /// [oracle]: https://docs.solanalabs.com/implemented-proposals/validator-timestamp-oracle
            pub unix_timestamp: UnixTimestamp,
        }
        #[automatically_derived]
        impl ::core::marker::Copy for Clock {}
        #[automatically_derived]
        impl ::core::clone::Clone for Clock {
            #[inline]
            fn clone(&self) -> Clock {
                let _: ::core::clone::AssertParamIsClone<Slot>;
                let _: ::core::clone::AssertParamIsClone<UnixTimestamp>;
                let _: ::core::clone::AssertParamIsClone<Epoch>;
                *self
            }
        }
        #[automatically_derived]
        impl ::core::default::Default for Clock {
            #[inline]
            fn default() -> Clock {
                Clock {
                    slot: ::core::default::Default::default(),
                    epoch_start_timestamp: ::core::default::Default::default(),
                    epoch: ::core::default::Default::default(),
                    leader_schedule_epoch: ::core::default::Default::default(),
                    unix_timestamp: ::core::default::Default::default(),
                }
            }
        }
        pub const DEFAULT_TICKS_PER_SLOT: u64 = 64;
        /// The default tick rate that the cluster attempts to achieve (160 per second).
        ///
        /// Note that the actual tick rate at any given time should be expected to drift.
        pub const DEFAULT_TICKS_PER_SECOND: u64 = 160;
        /// The expected duration of a slot (400 milliseconds).
        pub const DEFAULT_MS_PER_SLOT: u64 = 1_000 * DEFAULT_TICKS_PER_SLOT
            / DEFAULT_TICKS_PER_SECOND;
        impl Sysvar for Clock {
            fn get() -> Result<Self, crate::pinocchio::program_error::ProgramError> {
                let mut var = Self::default();
                let var_addr = &mut var as *mut _ as *mut u8;
                #[cfg(not(target_os = "solana"))]
                let result = core::hint::black_box(var_addr as *const _ as u64);
                match result {
                    crate::pinocchio::SUCCESS => Ok(var),
                    e => Err(e.into()),
                }
            }
        }
    }
    pub mod fees {
        //! Calculation of transaction fees.
        use super::{clock::DEFAULT_MS_PER_SLOT, Sysvar};
        //use crate::pinocchio::impl_sysvar_get;
        /// Fee calculator for processing transactions
        pub struct FeeCalculator {
            /// The current cost of a signature in lamports.
            /// This amount may increase/decrease over time based on cluster processing
            /// load.
            pub lamports_per_signature: u64,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for FeeCalculator {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "FeeCalculator",
                    "lamports_per_signature",
                    &&self.lamports_per_signature,
                )
            }
        }
        #[automatically_derived]
        impl ::core::default::Default for FeeCalculator {
            #[inline]
            fn default() -> FeeCalculator {
                FeeCalculator {
                    lamports_per_signature: ::core::default::Default::default(),
                }
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for FeeCalculator {
            #[inline]
            fn clone(&self) -> FeeCalculator {
                let _: ::core::clone::AssertParamIsClone<u64>;
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for FeeCalculator {}
        impl FeeCalculator {
            /// Create a new instance of the FeeCalculator
            pub fn new(lamports_per_signature: u64) -> Self {
                Self { lamports_per_signature }
            }
        }
        /// Governs the fee rate for the cluster
        pub struct FeeRateGovernor {
            /// The current cost of a signature
            pub lamports_per_signature: u64,
            /// The target cost of a signature
            pub target_lamports_per_signature: u64,
            /// The target number of signatures per slot
            pub target_signatures_per_slot: u64,
            /// Minimum lamports per signature
            pub min_lamports_per_signature: u64,
            /// Maximum lamports per signature
            pub max_lamports_per_signature: u64,
            /// Percentage of fees to burn (0-100)
            pub burn_percent: u8,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for FeeRateGovernor {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                let names: &'static _ = &[
                    "lamports_per_signature",
                    "target_lamports_per_signature",
                    "target_signatures_per_slot",
                    "min_lamports_per_signature",
                    "max_lamports_per_signature",
                    "burn_percent",
                ];
                let values: &[&dyn ::core::fmt::Debug] = &[
                    &self.lamports_per_signature,
                    &self.target_lamports_per_signature,
                    &self.target_signatures_per_slot,
                    &self.min_lamports_per_signature,
                    &self.max_lamports_per_signature,
                    &&self.burn_percent,
                ];
                ::core::fmt::Formatter::debug_struct_fields_finish(
                    f,
                    "FeeRateGovernor",
                    names,
                    values,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for FeeRateGovernor {
            #[inline]
            fn clone(&self) -> FeeRateGovernor {
                FeeRateGovernor {
                    lamports_per_signature: ::core::clone::Clone::clone(
                        &self.lamports_per_signature,
                    ),
                    target_lamports_per_signature: ::core::clone::Clone::clone(
                        &self.target_lamports_per_signature,
                    ),
                    target_signatures_per_slot: ::core::clone::Clone::clone(
                        &self.target_signatures_per_slot,
                    ),
                    min_lamports_per_signature: ::core::clone::Clone::clone(
                        &self.min_lamports_per_signature,
                    ),
                    max_lamports_per_signature: ::core::clone::Clone::clone(
                        &self.max_lamports_per_signature,
                    ),
                    burn_percent: ::core::clone::Clone::clone(&self.burn_percent),
                }
            }
        }
        pub const DEFAULT_TARGET_LAMPORTS_PER_SIGNATURE: u64 = 10_000;
        pub const DEFAULT_TARGET_SIGNATURES_PER_SLOT: u64 = 50 * DEFAULT_MS_PER_SLOT;
        /// 100% of fees now goes to validators...I'll confirm...
        pub const DEFAULT_BURN_PERCENT: u8 = 50;
        impl Default for FeeRateGovernor {
            fn default() -> Self {
                Self {
                    lamports_per_signature: 0,
                    target_lamports_per_signature: DEFAULT_TARGET_LAMPORTS_PER_SIGNATURE,
                    target_signatures_per_slot: DEFAULT_TARGET_SIGNATURES_PER_SLOT,
                    min_lamports_per_signature: 0,
                    max_lamports_per_signature: 0,
                    burn_percent: DEFAULT_BURN_PERCENT,
                }
            }
        }
        impl FeeRateGovernor {
            /// Create a new FeeCalculator based on current cluster signature throughput
            pub fn create_fee_calculator(&self) -> FeeCalculator {
                FeeCalculator::new(self.lamports_per_signature)
            }
            /// Calculate unburned fee from a fee total, returns (unburned, burned)
            pub fn burn(&self, fees: u64) -> (u64, u64) {
                let burned = fees * u64::from(self.burn_percent) / 100;
                (fees - burned, burned)
            }
        }
        /// Fees sysvar
        pub struct Fees {
            /// Fee calculator for processing transactions
            pub fee_calculator: FeeCalculator,
            /// Fee rate governor
            pub fee_rate_governor: FeeRateGovernor,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for Fees {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field2_finish(
                    f,
                    "Fees",
                    "fee_calculator",
                    &self.fee_calculator,
                    "fee_rate_governor",
                    &&self.fee_rate_governor,
                )
            }
        }
        #[automatically_derived]
        impl ::core::default::Default for Fees {
            #[inline]
            fn default() -> Fees {
                Fees {
                    fee_calculator: ::core::default::Default::default(),
                    fee_rate_governor: ::core::default::Default::default(),
                }
            }
        }
        impl Fees {
            /// Create a new instance of the Fees sysvar
            pub fn new(
                fee_calculator: FeeCalculator,
                fee_rate_governor: FeeRateGovernor,
            ) -> Self {
                Self {
                    fee_calculator,
                    fee_rate_governor,
                }
            }
        }
        impl Sysvar for Fees {
            fn get() -> Result<Self, crate::pinocchio::program_error::ProgramError> {
                let mut var = Self::default();
                let var_addr = &mut var as *mut _ as *mut u8;
                #[cfg(not(target_os = "solana"))]
                let result = core::hint::black_box(var_addr as *const _ as u64);
                match result {
                    crate::pinocchio::SUCCESS => Ok(var),
                    e => Err(e.into()),
                }
            }
        }
    }
    pub mod rent {
        //! This account contains the current cluster rent.
        //!
        //! This is required for the rent sysvar implementation.
        use super::Sysvar;
        use crate::pinocchio::{
            account_info::{AccountInfo, Ref},
            /*impl_sysvar_get,*/ program_error::ProgramError, pubkey::Pubkey,
        };
        /// The ID of the rent sysvar.
        pub const RENT_ID: Pubkey = [
            6, 167, 213, 23, 25, 44, 92, 81, 33, 140, 201, 76, 61, 74, 241, 127, 88, 218,
            238, 8, 155, 161, 253, 68, 227, 219, 217, 138, 0, 0, 0, 0,
        ];
        /// Default rental rate in lamports/byte-year.
        ///
        /// This calculation is based on:
        /// - 10^9 lamports per SOL
        /// - $1 per SOL
        /// - $0.01 per megabyte day
        /// - $3.65 per megabyte year
        pub const DEFAULT_LAMPORTS_PER_BYTE_YEAR: u64 = 1_000_000_000 / 100 * 365
            / (1024 * 1024);
        /// Default amount of time (in years) the balance has to include rent for the
        /// account to be rent exempt.
        pub const DEFAULT_EXEMPTION_THRESHOLD: f64 = 2.0;
        /// Default amount of time (in years) the balance has to include rent for the
        /// account to be rent exempt as a `u64`.
        const DEFAULT_EXEMPTION_THRESHOLD_AS_U64: u64 = 2;
        /// The `u64` representation of the default exemption threshold.
        ///
        /// This is used to check whether the `f64` value can be safely cast to a `u64`.
        const F64_EXEMPTION_THRESHOLD_AS_U64: u64 = 4611686018427387904;
        /// Default percentage of collected rent that is burned.
        ///
        /// Valid values are in the range [0, 100]. The remaining percentage is
        /// distributed to validators.
        pub const DEFAULT_BURN_PERCENT: u8 = 50;
        /// Account storage overhead for calculation of base rent.
        ///
        /// This is the number of bytes required to store an account with no data. It is
        /// added to an accounts data length when calculating [`Rent::minimum_balance`].
        pub const ACCOUNT_STORAGE_OVERHEAD: u64 = 128;
        /// Rent sysvar data
        #[repr(C)]
        pub struct Rent {
            /// Rental rate in lamports per byte-year
            pub lamports_per_byte_year: u64,
            /// Exemption threshold in years
            pub exemption_threshold: f64,
            /// Burn percentage
            pub burn_percent: u8,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Rent {
            #[inline]
            fn clone(&self) -> Rent {
                Rent {
                    lamports_per_byte_year: ::core::clone::Clone::clone(
                        &self.lamports_per_byte_year,
                    ),
                    exemption_threshold: ::core::clone::Clone::clone(
                        &self.exemption_threshold,
                    ),
                    burn_percent: ::core::clone::Clone::clone(&self.burn_percent),
                }
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for Rent {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field3_finish(
                    f,
                    "Rent",
                    "lamports_per_byte_year",
                    &self.lamports_per_byte_year,
                    "exemption_threshold",
                    &self.exemption_threshold,
                    "burn_percent",
                    &&self.burn_percent,
                )
            }
        }
        #[automatically_derived]
        impl ::core::default::Default for Rent {
            #[inline]
            fn default() -> Rent {
                Rent {
                    lamports_per_byte_year: ::core::default::Default::default(),
                    exemption_threshold: ::core::default::Default::default(),
                    burn_percent: ::core::default::Default::default(),
                }
            }
        }
        impl Rent {
            /// The length of the `Rent` sysvar account data.
            pub const LEN: usize = 8 + 8 + 1;
            /// Return a `Rent` from the given account info.
            ///
            /// This method performs a check on the account info key.
            #[inline]
            pub fn from_account_info(
                account_info: &AccountInfo,
            ) -> Result<Ref<Rent>, ProgramError> {
                if account_info.key() != &RENT_ID {
                    return Err(ProgramError::InvalidArgument);
                }
                Ok(
                    Ref::map(
                        account_info.try_borrow_data()?,
                        |data| unsafe { Self::from_bytes_unchecked(data) },
                    ),
                )
            }
            /// Return a `Mint` from the given account info.
            ///
            /// This method performs a check on the account info key, but does not
            /// perform the borrow check.
            ///
            /// # Safety
            ///
            /// The caller must ensure that it is safe to borrow the account data – e.g., there are
            /// no mutable borrows of the account data.
            #[inline]
            pub unsafe fn from_account_info_unchecked(
                account_info: &AccountInfo,
            ) -> Result<&Self, ProgramError> {
                if account_info.key() != &RENT_ID {
                    return Err(ProgramError::InvalidArgument);
                }
                Ok(Self::from_bytes_unchecked(account_info.borrow_data_unchecked()))
            }
            /// Return a `Rent` from the given bytes.
            ///
            /// This method performs a length validation. The caller must ensure that `bytes` contains
            /// a valid representation of `Rent`.
            #[inline]
            pub fn from_bytes(bytes: &[u8]) -> Result<&Self, ProgramError> {
                if bytes.len() != Self::LEN {
                    return Err(ProgramError::InvalidArgument);
                }
                Ok(unsafe { Self::from_bytes_unchecked(bytes) })
            }
            /// Return a `Rent` from the given bytes.
            ///
            /// # Safety
            ///
            /// The caller must ensure that `bytes` contains a valid representation of `Rent` and
            /// that is has the expected length.
            #[inline]
            pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
                &*(bytes.as_ptr() as *const Rent)
            }
            /// Calculate how much rent to burn from the collected rent.
            ///
            /// The first value returned is the amount burned. The second is the amount
            /// to distribute to validators.
            #[inline]
            pub fn calculate_burn(&self, rent_collected: u64) -> (u64, u64) {
                let burned_portion = (rent_collected * u64::from(self.burn_percent))
                    / 100;
                (burned_portion, rent_collected - burned_portion)
            }
            /// Rent due on account's data length with balance.
            #[inline]
            pub fn due(
                &self,
                balance: u64,
                data_len: usize,
                years_elapsed: f64,
            ) -> RentDue {
                if self.is_exempt(balance, data_len) {
                    RentDue::Exempt
                } else {
                    RentDue::Paying(self.due_amount(data_len, years_elapsed))
                }
            }
            /// Rent due for account that is known to be not exempt.
            #[inline]
            pub fn due_amount(&self, data_len: usize, years_elapsed: f64) -> u64 {
                let actual_data_len = data_len as u64 + ACCOUNT_STORAGE_OVERHEAD;
                let lamports_per_year = self.lamports_per_byte_year * actual_data_len;
                (lamports_per_year as f64 * years_elapsed) as u64
            }
            /// Calculates the minimum balance for rent exemption.
            ///
            /// This method avoids floating-point operations when the `exemption_threshold`
            /// is the default value.
            ///
            /// # Arguments
            ///
            /// * `data_len` - The number of bytes in the account
            ///
            /// # Returns
            ///
            /// The minimum balance in lamports for rent exemption.
            #[inline]
            pub fn minimum_balance(&self, data_len: usize) -> u64 {
                let bytes = data_len as u64;
                if self.is_default_rent_threshold() {
                    ((ACCOUNT_STORAGE_OVERHEAD + bytes) * self.lamports_per_byte_year)
                        * DEFAULT_EXEMPTION_THRESHOLD_AS_U64
                } else {
                    (((ACCOUNT_STORAGE_OVERHEAD + bytes) * self.lamports_per_byte_year)
                        as f64 * self.exemption_threshold) as u64
                }
            }
            /// Determines if an account can be considered rent exempt.
            ///
            /// # Arguments
            ///
            /// * `lamports` - The balance of the account in lamports
            /// * `data_len` - The size of the account in bytes
            ///
            /// # Returns
            ///
            /// `true`` if the account is rent exempt, `false`` otherwise.
            #[inline]
            pub fn is_exempt(&self, lamports: u64, data_len: usize) -> bool {
                lamports >= self.minimum_balance(data_len)
            }
            /// Determines if the `exemption_threshold` is the default value.
            ///
            /// This is used to check whether the `f64` value can be safely cast to a `u64`
            /// to avoid floating-point operations.
            #[inline]
            fn is_default_rent_threshold(&self) -> bool {
                u64::from_le_bytes(self.exemption_threshold.to_le_bytes())
                    == F64_EXEMPTION_THRESHOLD_AS_U64
            }
        }
        impl Sysvar for Rent {
            fn get() -> Result<Self, crate::pinocchio::program_error::ProgramError> {
                let mut var = Self::default();
                let var_addr = &mut var as *mut _ as *mut u8;
                #[cfg(not(target_os = "solana"))]
                let result = core::hint::black_box(var_addr as *const _ as u64);
                match result {
                    crate::pinocchio::SUCCESS => Ok(var),
                    e => Err(e.into()),
                }
            }
        }
        /// The return value of [`Rent::due`].
        pub enum RentDue {
            /// Used to indicate the account is rent exempt.
            Exempt,
            /// The account owes this much rent.
            Paying(u64),
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for RentDue {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match self {
                    RentDue::Exempt => ::core::fmt::Formatter::write_str(f, "Exempt"),
                    RentDue::Paying(__self_0) => {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "Paying",
                            &__self_0,
                        )
                    }
                }
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for RentDue {}
        #[automatically_derived]
        impl ::core::clone::Clone for RentDue {
            #[inline]
            fn clone(&self) -> RentDue {
                let _: ::core::clone::AssertParamIsClone<u64>;
                *self
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Eq for RentDue {
            #[inline]
            #[doc(hidden)]
            #[coverage(off)]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<u64>;
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for RentDue {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for RentDue {
            #[inline]
            fn eq(&self, other: &RentDue) -> bool {
                let __self_discr = ::core::intrinsics::discriminant_value(self);
                let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                __self_discr == __arg1_discr
                    && match (self, other) {
                        (RentDue::Paying(__self_0), RentDue::Paying(__arg1_0)) => {
                            __self_0 == __arg1_0
                        }
                        _ => true,
                    }
            }
        }
        impl RentDue {
            /// Return the lamports due for rent.
            pub fn lamports(&self) -> u64 {
                match self {
                    RentDue::Exempt => 0,
                    RentDue::Paying(x) => *x,
                }
            }
            /// Return 'true' if rent exempt.
            pub fn is_exempt(&self) -> bool {
                match self {
                    RentDue::Exempt => true,
                    RentDue::Paying(_) => false,
                }
            }
        }
    }
    /// A type that holds sysvar data.
    pub trait Sysvar: Default + Sized {
        /// Load the sysvar directly from the runtime.
        ///
        /// This is the preferred way to load a sysvar. Calling this method does not
        /// incur any deserialization overhead, and does not require the sysvar
        /// account to be passed to the program.
        ///
        /// Not all sysvars support this method. If not, it returns
        /// [`ProgramError::UnsupportedSysvar`].
        fn get() -> Result<Self, ProgramError> {
            Err(ProgramError::UnsupportedSysvar)
        }
    }
}
/// Default panic handler.
///
/// This macro sets up a default panic handler that logs the panic message and the file where the
/// panic occurred.
///
/// Note that this requires the `"std"` feature to be enabled.
#[cfg(feature = "std")]
#[macro_export]
macro_rules! default_panic_handler {
    () => {
        /// Default panic handler.
        #[cfg(all(not(feature = "custom-panic"), target_os = "solana"))]
        #[no_mangle]
        fn custom_panic(info: &core::panic::PanicInfo<'_>) {
            // Panic reporting.
            $crate::msg!("{}", info);
        }
    };
}

/// Default panic handler.
///
/// This macro sets up a default panic handler that logs the file where the panic occurred.
///
/// This is used when the `"std"` feature is disabled.
#[cfg(not(feature = "std"))]
#[macro_export]
macro_rules! default_panic_handler {
    () => {
        /// Default panic handler.
        #[cfg(all(not(feature = "custom-panic"), target_os = "solana"))]
        #[no_mangle]
        fn custom_panic(info: &core::panic::PanicInfo<'_>) {
            if let Some(location) = info.location() {
                $crate::log::sol_log(location.file());
            }
            // Panic reporting.
            $crate::log::sol_log("** PANICKED **");
        }
    };
}
/// Declare the program entrypoint.
///
/// This macro is similar to the `entrypoint!` macro, but it does not set up a global allocator
/// nor a panic handler. This is useful when the program will set up its own allocator and panic
/// handler.
#[macro_export]
macro_rules! program_entrypoint {
    ( $process_instruction:ident ) => {
        program_entrypoint!($process_instruction, { $crate::pinocchio::MAX_TX_ACCOUNTS });
    };
    ( $process_instruction:ident, $maximum:expr ) => {
        /// Program entrypoint.
        #[no_mangle]
        pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
            const UNINIT: core::mem::MaybeUninit<$crate::pinocchio::account_info::AccountInfo> =
                core::mem::MaybeUninit::<$crate::pinocchio::account_info::AccountInfo>::uninit();
            // Create an array of uninitialized account infos.
            let mut accounts = [UNINIT; $maximum];

            let (program_id, count, instruction_data) =
                $crate::pinocchio::entrypoint::deserialize::<$maximum>(input, &mut accounts);

            // Call the program's entrypoint passing `count` account infos; we know that
            // they are initialized so we cast the pointer to a slice of `[AccountInfo]`.
            match $process_instruction(
                &program_id,
                core::slice::from_raw_parts(accounts.as_ptr() as _, count),
                &instruction_data,
            ) {
                Ok(()) => $crate::pinocchio::SUCCESS,
                Err(error) => error.into(),
            }
        }
    };
}
#[macro_export]
#[cfg(not(feature = "std"))]
macro_rules! msg {
    ( $msg:expr ) => {
        $crate::pinocchio::log::sol_log($msg)
    };
}

#[cfg(feature = "std")]
#[macro_export]
macro_rules! msg {
    ( $msg:expr ) => {
        $crate::pinocchio::log::sol_log($msg)
    };
    ( $( $arg:tt )* ) => ($crate::pinocchio::log::sol_log(&format!($($arg)*)));
}
#[deprecated(since = "0.7.0", note = "Use the `entrypoint` module instead")]
pub use entrypoint::lazy as lazy_entrypoint;
/// Maximum number of accounts that a transaction may process.
///
/// This value is used to set the maximum number of accounts that a program
/// is expecting and statically initialize the array of `AccountInfo`.
///
/// This is based on the current [maximum number of accounts] that a transaction
/// may lock in a block.
///
/// [maximum number of accounts]: https://github.com/anza-xyz/agave/blob/2e6ca8c1f62db62c1db7f19c9962d4db43d0d550/runtime/src/bank.rs#L3209-L3221
pub const MAX_TX_ACCOUNTS: usize = 128;
/// `assert_eq(core::mem::align_of::<u128>(), 8)` is true for BPF but not
/// for some host machines.
const BPF_ALIGN_OF_U128: usize = 8;
/// Value used to indicate that a serialized account is not a duplicate.
const NON_DUP_MARKER: u8 = u8::MAX;
/// Return value for a successful program execution.
pub const SUCCESS: u64 = 0;
/// The result of a program execution.
pub type ProgramResult = Result<(), program_error::ProgramError>;
