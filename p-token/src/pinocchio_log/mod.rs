#![feature(prelude_import)]
//! Lightweight log utility for Solana programs.
//!
//! This crate provides a `Logger` struct that can be used to efficiently log messages
//! in a Solana program. The `Logger` struct is a wrapper around a fixed-size buffer,
//! where types that implement the `Log` trait can be appended to the buffer.
//!
//! The `Logger` struct is generic over the size of the buffer, and the buffer size
//! should be chosen based on the expected size of the log messages. When the buffer is
//! full, the log message will be truncated. This is represented by the `@` character
//! at the end of the log message.
//!
//! # Example
//!
//! Creating a `Logger` with a buffer size of `100` bytes, and appending a string and an
//! `u64` value:
//!
//! ```
//! use pinocchio_log::logger::Logger;
//!
//! let mut logger = Logger::<100>::default();
//! logger.append("balance=");
//! logger.append(1_000_000_000);
//! logger.log();
//!
//! // Clear the logger buffer.
//! logger.clear();
//!
//! logger.append(&["Hello ", "world!"]);
//! logger.log();
//! ```
//!
//! It also support adding precision to numeric types:
//!
//! ```
//! use pinocchio_log::logger::{Argument, Logger};
//!
//! let mut logger = Logger::<100>::default();
//!
//! let lamports = 1_000_000_000u64;
//!
//! logger.append("balance (SOL)=");
//! logger.append_with_args(lamports, &[Argument::Precision(9)]);
//! logger.log();
//! ```
#![no_std]
extern crate core;
extern crate std;
//extern crate compiler_builtins as _;
pub mod logger {
    use core::{mem::MaybeUninit, ops::Deref, slice::from_raw_parts};
    #[cfg(not(target_os = "solana"))]
    extern crate std;
    /// Byte representation of the digits [0, 9].
    const DIGITS: [u8; 10] = [
        b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9',
    ];
    /// Bytes for a truncated `str` log message.
    const TRUNCATED_SLICE: [u8; 3] = [b'.', b'.', b'.'];
    /// Byte representing a truncated log.
    const TRUCATED: u8 = b'@';
    /// An uninitialized byte.
    const UNINIT_BYTE: MaybeUninit<u8> = MaybeUninit::uninit();
    /// Logger to efficiently format log messages.
    ///
    /// The logger is a fixed size buffer that can be used to format log messages
    /// before sending them to the log output. Any type that implements the `Log`
    /// trait can be appended to the logger.
    pub struct Logger<const BUFFER: usize> {
        buffer: [MaybeUninit<u8>; BUFFER],
        offset: usize,
    }
    impl<const BUFFER: usize> Default for Logger<BUFFER> {
        #[inline]
        fn default() -> Self {
            Self {
                buffer: [UNINIT_BYTE; BUFFER],
                offset: 0,
            }
        }
    }
    impl<const BUFFER: usize> Deref for Logger<BUFFER> {
        type Target = [u8];
        fn deref(&self) -> &Self::Target {
            unsafe { from_raw_parts(self.buffer.as_ptr() as *const _, self.offset) }
        }
    }
    impl<const BUFFER: usize> Logger<BUFFER> {
        /// Append a value to the logger.
        #[inline(always)]
        pub fn append<T: Log>(&mut self, value: T) -> &mut Self {
            self.append_with_args(value, &[]);
            self
        }
        /// Append a value to the logger with formatting arguments.
        #[inline]
        pub fn append_with_args<T: Log>(
            &mut self,
            value: T,
            args: &[Argument],
        ) -> &mut Self {
            if self.is_full() {
                if BUFFER > 0 {
                    unsafe {
                        let last = self.buffer.get_unchecked_mut(BUFFER - 1);
                        last.write(TRUCATED);
                    }
                }
            } else {
                self.offset
                    += value.write_with_args(&mut self.buffer[self.offset..], args);
            }
            self
        }
        /// Log the message in the buffer.
        #[inline(always)]
        pub fn log(&self) {
            log_message(self);
        }
        /// Clear the buffer.
        #[inline(always)]
        pub fn clear(&mut self) {
            self.offset = 0;
        }
        /// Check if the buffer is empty.
        #[inline(always)]
        pub fn is_empty(&self) -> bool {
            self.offset == 0
        }
        /// Check if the buffer is full.
        #[inline(always)]
        pub fn is_full(&self) -> bool {
            self.offset == BUFFER
        }
        /// Get the length of the buffer.
        #[inline(always)]
        pub fn len(&self) -> usize {
            self.offset
        }
        /// Get the remaining space in the buffer.
        #[inline(always)]
        pub fn remaining(&self) -> usize {
            BUFFER - self.offset
        }
    }
    /// Log a message.
    #[inline(always)]
    pub fn log_message(message: &[u8]) {
        ()
    }
    /// Formatting arguments.
    ///
    /// Arguments can be used to specify additional formatting options for the log message.
    /// Note that types might not support all arguments.
    #[non_exhaustive]
    pub enum Argument {
        /// Number of decimal places to display for numbers.
        ///
        /// This is only applicable for numeric types.
        Precision(u8),
        /// Truncate the output at the end when the specified maximum number of characters
        /// is exceeded.
        ///
        /// This is only applicable for `str` types.
        TruncateEnd(usize),
        /// Truncate the output at the start when the specified maximum number of characters
        /// is exceeded.
        ///
        /// This is only applicable for `str` types.
        TruncateStart(usize),
    }
    /// Trait to specify the log behavior for a type.
    pub trait Log {
        #[inline(always)]
        fn debug(&self, buffer: &mut [MaybeUninit<u8>]) -> usize {
            self.debug_with_args(buffer, &[])
        }
        #[inline(always)]
        fn debug_with_args(
            &self,
            buffer: &mut [MaybeUninit<u8>],
            args: &[Argument],
        ) -> usize {
            self.write_with_args(buffer, args)
        }
        #[inline(always)]
        fn write(&self, buffer: &mut [MaybeUninit<u8>]) -> usize {
            self.write_with_args(buffer, &[])
        }
        fn write_with_args(
            &self,
            buffer: &mut [MaybeUninit<u8>],
            parameters: &[Argument],
        ) -> usize;
    }
    impl Log for u8 {
        #[inline]
        fn write_with_args(
            &self,
            buffer: &mut [MaybeUninit<u8>],
            args: &[Argument],
        ) -> usize {
            if buffer.is_empty() {
                return 0;
            }
            match *self {
                0 => {
                    unsafe {
                        buffer.get_unchecked_mut(0).write(*DIGITS.get_unchecked(0));
                    }
                    1
                }
                mut value => {
                    let mut digits = [UNINIT_BYTE; 3];
                    let mut offset = 3;
                    while value > 0 {
                        let remainder = value % 10;
                        value /= 10;
                        offset -= 1;
                        unsafe {
                            digits
                                .get_unchecked_mut(offset)
                                .write(*DIGITS.get_unchecked(remainder as usize));
                        }
                    }
                    let precision = if let Some(Argument::Precision(p)) = args
                        .iter()
                        .find(|arg| match arg {
                            Argument::Precision(_) => true,
                            _ => false,
                        })
                    {
                        *p as usize
                    } else {
                        0
                    };
                    let mut available = 3 - offset;
                    if precision > 0 {
                        while precision >= available {
                            available += 1;
                            offset -= 1;
                            unsafe {
                                digits
                                    .get_unchecked_mut(offset)
                                    .write(*DIGITS.get_unchecked(0));
                            }
                        }
                        available += 1;
                    }
                    let length = buffer.len();
                    let (overflow, written, fraction) = if available <= length {
                        (false, available, precision)
                    } else {
                        (true, length, precision.saturating_sub(available - length))
                    };
                    unsafe {
                        let source = digits.as_ptr().add(offset);
                        let ptr = buffer.as_mut_ptr();
                        #[cfg(not(target_os = "solana"))]
                        {
                            if precision == 0 {
                                core::ptr::copy_nonoverlapping(source, ptr, written);
                            } else {
                                let integer_part = written - (fraction + 1);
                                core::ptr::copy_nonoverlapping(source, ptr, integer_part);
                                (ptr.add(integer_part) as *mut u8).write(b'.');
                                core::ptr::copy_nonoverlapping(
                                    source.add(integer_part),
                                    ptr.add(integer_part + 1),
                                    fraction,
                                );
                            }
                        }
                    }
                    if overflow {
                        unsafe {
                            let last = buffer.get_unchecked_mut(written - 1);
                            last.write(TRUCATED);
                        }
                    }
                    written
                }
            }
        }
    }
    impl Log for u16 {
        #[inline]
        fn write_with_args(
            &self,
            buffer: &mut [MaybeUninit<u8>],
            args: &[Argument],
        ) -> usize {
            if buffer.is_empty() {
                return 0;
            }
            match *self {
                0 => {
                    unsafe {
                        buffer.get_unchecked_mut(0).write(*DIGITS.get_unchecked(0));
                    }
                    1
                }
                mut value => {
                    let mut digits = [UNINIT_BYTE; 5];
                    let mut offset = 5;
                    while value > 0 {
                        let remainder = value % 10;
                        value /= 10;
                        offset -= 1;
                        unsafe {
                            digits
                                .get_unchecked_mut(offset)
                                .write(*DIGITS.get_unchecked(remainder as usize));
                        }
                    }
                    let precision = if let Some(Argument::Precision(p)) = args
                        .iter()
                        .find(|arg| match arg {
                            Argument::Precision(_) => true,
                            _ => false,
                        })
                    {
                        *p as usize
                    } else {
                        0
                    };
                    let mut available = 5 - offset;
                    if precision > 0 {
                        while precision >= available {
                            available += 1;
                            offset -= 1;
                            unsafe {
                                digits
                                    .get_unchecked_mut(offset)
                                    .write(*DIGITS.get_unchecked(0));
                            }
                        }
                        available += 1;
                    }
                    let length = buffer.len();
                    let (overflow, written, fraction) = if available <= length {
                        (false, available, precision)
                    } else {
                        (true, length, precision.saturating_sub(available - length))
                    };
                    unsafe {
                        let source = digits.as_ptr().add(offset);
                        let ptr = buffer.as_mut_ptr();
                        #[cfg(not(target_os = "solana"))]
                        {
                            if precision == 0 {
                                core::ptr::copy_nonoverlapping(source, ptr, written);
                            } else {
                                let integer_part = written - (fraction + 1);
                                core::ptr::copy_nonoverlapping(source, ptr, integer_part);
                                (ptr.add(integer_part) as *mut u8).write(b'.');
                                core::ptr::copy_nonoverlapping(
                                    source.add(integer_part),
                                    ptr.add(integer_part + 1),
                                    fraction,
                                );
                            }
                        }
                    }
                    if overflow {
                        unsafe {
                            let last = buffer.get_unchecked_mut(written - 1);
                            last.write(TRUCATED);
                        }
                    }
                    written
                }
            }
        }
    }
    impl Log for u32 {
        #[inline]
        fn write_with_args(
            &self,
            buffer: &mut [MaybeUninit<u8>],
            args: &[Argument],
        ) -> usize {
            if buffer.is_empty() {
                return 0;
            }
            match *self {
                0 => {
                    unsafe {
                        buffer.get_unchecked_mut(0).write(*DIGITS.get_unchecked(0));
                    }
                    1
                }
                mut value => {
                    let mut digits = [UNINIT_BYTE; 10];
                    let mut offset = 10;
                    while value > 0 {
                        let remainder = value % 10;
                        value /= 10;
                        offset -= 1;
                        unsafe {
                            digits
                                .get_unchecked_mut(offset)
                                .write(*DIGITS.get_unchecked(remainder as usize));
                        }
                    }
                    let precision = if let Some(Argument::Precision(p)) = args
                        .iter()
                        .find(|arg| match arg {
                            Argument::Precision(_) => true,
                            _ => false,
                        })
                    {
                        *p as usize
                    } else {
                        0
                    };
                    let mut available = 10 - offset;
                    if precision > 0 {
                        while precision >= available {
                            available += 1;
                            offset -= 1;
                            unsafe {
                                digits
                                    .get_unchecked_mut(offset)
                                    .write(*DIGITS.get_unchecked(0));
                            }
                        }
                        available += 1;
                    }
                    let length = buffer.len();
                    let (overflow, written, fraction) = if available <= length {
                        (false, available, precision)
                    } else {
                        (true, length, precision.saturating_sub(available - length))
                    };
                    unsafe {
                        let source = digits.as_ptr().add(offset);
                        let ptr = buffer.as_mut_ptr();
                        #[cfg(not(target_os = "solana"))]
                        {
                            if precision == 0 {
                                core::ptr::copy_nonoverlapping(source, ptr, written);
                            } else {
                                let integer_part = written - (fraction + 1);
                                core::ptr::copy_nonoverlapping(source, ptr, integer_part);
                                (ptr.add(integer_part) as *mut u8).write(b'.');
                                core::ptr::copy_nonoverlapping(
                                    source.add(integer_part),
                                    ptr.add(integer_part + 1),
                                    fraction,
                                );
                            }
                        }
                    }
                    if overflow {
                        unsafe {
                            let last = buffer.get_unchecked_mut(written - 1);
                            last.write(TRUCATED);
                        }
                    }
                    written
                }
            }
        }
    }
    impl Log for u64 {
        #[inline]
        fn write_with_args(
            &self,
            buffer: &mut [MaybeUninit<u8>],
            args: &[Argument],
        ) -> usize {
            if buffer.is_empty() {
                return 0;
            }
            match *self {
                0 => {
                    unsafe {
                        buffer.get_unchecked_mut(0).write(*DIGITS.get_unchecked(0));
                    }
                    1
                }
                mut value => {
                    let mut digits = [UNINIT_BYTE; 20];
                    let mut offset = 20;
                    while value > 0 {
                        let remainder = value % 10;
                        value /= 10;
                        offset -= 1;
                        unsafe {
                            digits
                                .get_unchecked_mut(offset)
                                .write(*DIGITS.get_unchecked(remainder as usize));
                        }
                    }
                    let precision = if let Some(Argument::Precision(p)) = args
                        .iter()
                        .find(|arg| match arg {
                            Argument::Precision(_) => true,
                            _ => false,
                        })
                    {
                        *p as usize
                    } else {
                        0
                    };
                    let mut available = 20 - offset;
                    if precision > 0 {
                        while precision >= available {
                            available += 1;
                            offset -= 1;
                            unsafe {
                                digits
                                    .get_unchecked_mut(offset)
                                    .write(*DIGITS.get_unchecked(0));
                            }
                        }
                        available += 1;
                    }
                    let length = buffer.len();
                    let (overflow, written, fraction) = if available <= length {
                        (false, available, precision)
                    } else {
                        (true, length, precision.saturating_sub(available - length))
                    };
                    unsafe {
                        let source = digits.as_ptr().add(offset);
                        let ptr = buffer.as_mut_ptr();
                        #[cfg(not(target_os = "solana"))]
                        {
                            if precision == 0 {
                                core::ptr::copy_nonoverlapping(source, ptr, written);
                            } else {
                                let integer_part = written - (fraction + 1);
                                core::ptr::copy_nonoverlapping(source, ptr, integer_part);
                                (ptr.add(integer_part) as *mut u8).write(b'.');
                                core::ptr::copy_nonoverlapping(
                                    source.add(integer_part),
                                    ptr.add(integer_part + 1),
                                    fraction,
                                );
                            }
                        }
                    }
                    if overflow {
                        unsafe {
                            let last = buffer.get_unchecked_mut(written - 1);
                            last.write(TRUCATED);
                        }
                    }
                    written
                }
            }
        }
    }
    impl Log for u128 {
        #[inline]
        fn write_with_args(
            &self,
            buffer: &mut [MaybeUninit<u8>],
            args: &[Argument],
        ) -> usize {
            if buffer.is_empty() {
                return 0;
            }
            match *self {
                0 => {
                    unsafe {
                        buffer.get_unchecked_mut(0).write(*DIGITS.get_unchecked(0));
                    }
                    1
                }
                mut value => {
                    let mut digits = [UNINIT_BYTE; 39];
                    let mut offset = 39;
                    while value > 0 {
                        let remainder = value % 10;
                        value /= 10;
                        offset -= 1;
                        unsafe {
                            digits
                                .get_unchecked_mut(offset)
                                .write(*DIGITS.get_unchecked(remainder as usize));
                        }
                    }
                    let precision = if let Some(Argument::Precision(p)) = args
                        .iter()
                        .find(|arg| match arg {
                            Argument::Precision(_) => true,
                            _ => false,
                        })
                    {
                        *p as usize
                    } else {
                        0
                    };
                    let mut available = 39 - offset;
                    if precision > 0 {
                        while precision >= available {
                            available += 1;
                            offset -= 1;
                            unsafe {
                                digits
                                    .get_unchecked_mut(offset)
                                    .write(*DIGITS.get_unchecked(0));
                            }
                        }
                        available += 1;
                    }
                    let length = buffer.len();
                    let (overflow, written, fraction) = if available <= length {
                        (false, available, precision)
                    } else {
                        (true, length, precision.saturating_sub(available - length))
                    };
                    unsafe {
                        let source = digits.as_ptr().add(offset);
                        let ptr = buffer.as_mut_ptr();
                        #[cfg(not(target_os = "solana"))]
                        {
                            if precision == 0 {
                                core::ptr::copy_nonoverlapping(source, ptr, written);
                            } else {
                                let integer_part = written - (fraction + 1);
                                core::ptr::copy_nonoverlapping(source, ptr, integer_part);
                                (ptr.add(integer_part) as *mut u8).write(b'.');
                                core::ptr::copy_nonoverlapping(
                                    source.add(integer_part),
                                    ptr.add(integer_part + 1),
                                    fraction,
                                );
                            }
                        }
                    }
                    if overflow {
                        unsafe {
                            let last = buffer.get_unchecked_mut(written - 1);
                            last.write(TRUCATED);
                        }
                    }
                    written
                }
            }
        }
    }
    impl Log for i8 {
        #[inline]
        fn write_with_args(
            &self,
            buffer: &mut [MaybeUninit<u8>],
            args: &[Argument],
        ) -> usize {
            if buffer.is_empty() {
                return 0;
            }
            match *self {
                0 => {
                    unsafe {
                        buffer.get_unchecked_mut(0).write(*DIGITS.get_unchecked(0));
                    }
                    1
                }
                mut value => {
                    let mut prefix = 0;
                    if *self < 0 {
                        unsafe {
                            buffer.get_unchecked_mut(0).write(b'-');
                        }
                        prefix += 1;
                        value = -value;
                    }
                    prefix + (value as u8).write_with_args(&mut buffer[prefix..], args)
                }
            }
        }
    }
    impl Log for i16 {
        #[inline]
        fn write_with_args(
            &self,
            buffer: &mut [MaybeUninit<u8>],
            args: &[Argument],
        ) -> usize {
            if buffer.is_empty() {
                return 0;
            }
            match *self {
                0 => {
                    unsafe {
                        buffer.get_unchecked_mut(0).write(*DIGITS.get_unchecked(0));
                    }
                    1
                }
                mut value => {
                    let mut prefix = 0;
                    if *self < 0 {
                        unsafe {
                            buffer.get_unchecked_mut(0).write(b'-');
                        }
                        prefix += 1;
                        value = -value;
                    }
                    prefix + (value as u16).write_with_args(&mut buffer[prefix..], args)
                }
            }
        }
    }
    impl Log for i32 {
        #[inline]
        fn write_with_args(
            &self,
            buffer: &mut [MaybeUninit<u8>],
            args: &[Argument],
        ) -> usize {
            if buffer.is_empty() {
                return 0;
            }
            match *self {
                0 => {
                    unsafe {
                        buffer.get_unchecked_mut(0).write(*DIGITS.get_unchecked(0));
                    }
                    1
                }
                mut value => {
                    let mut prefix = 0;
                    if *self < 0 {
                        unsafe {
                            buffer.get_unchecked_mut(0).write(b'-');
                        }
                        prefix += 1;
                        value = -value;
                    }
                    prefix + (value as u32).write_with_args(&mut buffer[prefix..], args)
                }
            }
        }
    }
    impl Log for i64 {
        #[inline]
        fn write_with_args(
            &self,
            buffer: &mut [MaybeUninit<u8>],
            args: &[Argument],
        ) -> usize {
            if buffer.is_empty() {
                return 0;
            }
            match *self {
                0 => {
                    unsafe {
                        buffer.get_unchecked_mut(0).write(*DIGITS.get_unchecked(0));
                    }
                    1
                }
                mut value => {
                    let mut prefix = 0;
                    if *self < 0 {
                        unsafe {
                            buffer.get_unchecked_mut(0).write(b'-');
                        }
                        prefix += 1;
                        value = -value;
                    }
                    prefix + (value as u64).write_with_args(&mut buffer[prefix..], args)
                }
            }
        }
    }
    impl Log for i128 {
        #[inline]
        fn write_with_args(
            &self,
            buffer: &mut [MaybeUninit<u8>],
            args: &[Argument],
        ) -> usize {
            if buffer.is_empty() {
                return 0;
            }
            match *self {
                0 => {
                    unsafe {
                        buffer.get_unchecked_mut(0).write(*DIGITS.get_unchecked(0));
                    }
                    1
                }
                mut value => {
                    let mut prefix = 0;
                    if *self < 0 {
                        unsafe {
                            buffer.get_unchecked_mut(0).write(b'-');
                        }
                        prefix += 1;
                        value = -value;
                    }
                    prefix + (value as u128).write_with_args(&mut buffer[prefix..], args)
                }
            }
        }
    }
    /// Implement the log trait for the &str type.
    impl Log for &str {
        #[inline]
        fn debug_with_args(
            &self,
            buffer: &mut [MaybeUninit<u8>],
            _args: &[Argument],
        ) -> usize {
            if buffer.is_empty() {
                return 0;
            }
            unsafe {
                buffer.get_unchecked_mut(0).write(b'"');
            }
            let mut offset = 1;
            offset += self.write(&mut buffer[offset..]);
            match buffer.len() - offset {
                0 => {
                    unsafe {
                        buffer.get_unchecked_mut(offset - 1).write(TRUCATED);
                    }
                }
                _ => {
                    unsafe {
                        buffer.get_unchecked_mut(offset).write(b'"');
                    }
                    offset += 1;
                }
            }
            offset
        }
        #[inline]
        fn write_with_args(
            &self,
            buffer: &mut [MaybeUninit<u8>],
            args: &[Argument],
        ) -> usize {
            let (size, truncate_end) = match args
                .iter()
                .find(|arg| match arg {
                    Argument::TruncateEnd(_) | Argument::TruncateStart(_) => true,
                    _ => false,
                })
            {
                Some(Argument::TruncateEnd(size)) => (*size, Some(true)),
                Some(Argument::TruncateStart(size)) => (*size, Some(false)),
                _ => (buffer.len(), None),
            };
            let (offset, source, length, prefix, truncated) = if truncate_end.is_none() {
                let length = core::cmp::min(size, self.len());
                (buffer.as_mut_ptr(), self.as_ptr(), length, 0, length != self.len())
            } else {
                let length = core::cmp::min(size, buffer.len());
                let ptr = buffer.as_mut_ptr();
                if length >= self.len() {
                    (ptr, self.as_ptr(), self.len(), 0, false)
                } else if length > TRUNCATED_SLICE.len() {
                    let length = length - TRUNCATED_SLICE.len();
                    unsafe {
                        let (offset, source, destination) = if truncate_end == Some(true)
                        {
                            (length, self.as_ptr(), ptr)
                        } else {
                            (
                                0,
                                self.as_ptr().add(self.len() - length),
                                ptr.add(TRUNCATED_SLICE.len()),
                            )
                        };
                        core::ptr::copy_nonoverlapping(
                            TRUNCATED_SLICE.as_ptr(),
                            ptr.add(offset) as *mut _,
                            TRUNCATED_SLICE.len(),
                        );
                        (destination, source, length, TRUNCATED_SLICE.len(), false)
                    }
                } else {
                    (ptr, TRUNCATED_SLICE.as_ptr(), length, 0, true)
                }
            };
            unsafe {
                core::ptr::copy_nonoverlapping(source, offset as *mut _, length);
            }
            if truncated {
                unsafe {
                    let last = buffer.get_unchecked_mut(length - 1);
                    last.write(TRUCATED);
                }
            }
            prefix + length
        }
    }
    impl<T> Log for &[T]
    where
        T: Log,
    {
        #[inline]
        fn write_with_args(
            &self,
            buffer: &mut [MaybeUninit<u8>],
            _args: &[Argument],
        ) -> usize {
            if buffer.is_empty() {
                return 0;
            }
            let length = buffer.len();
            unsafe {
                buffer.get_unchecked_mut(0).write(b'[');
            }
            let mut offset = 1;
            for value in self.iter() {
                if offset >= length {
                    unsafe {
                        buffer.get_unchecked_mut(length - 1).write(TRUCATED);
                    }
                    offset = length;
                    break;
                }
                if offset > 1 {
                    if offset + 2 >= length {
                        unsafe {
                            buffer.get_unchecked_mut(length - 1).write(TRUCATED);
                        }
                        offset = length;
                        break;
                    } else {
                        unsafe {
                            buffer.get_unchecked_mut(offset).write(b',');
                            buffer.get_unchecked_mut(offset + 1).write(b' ');
                        }
                        offset += 2;
                    }
                }
                offset += value.debug(&mut buffer[offset..]);
            }
            if offset < length {
                unsafe {
                    buffer.get_unchecked_mut(offset).write(b']');
                }
                offset += 1;
            }
            offset
        }
    }
    impl<T, const N: usize> Log for &[T; N]
    where
        T: Log,
    {
        #[inline]
        fn write_with_args(
            &self,
            buffer: &mut [MaybeUninit<u8>],
            _args: &[Argument],
        ) -> usize {
            if buffer.is_empty() {
                return 0;
            }
            let length = buffer.len();
            unsafe {
                buffer.get_unchecked_mut(0).write(b'[');
            }
            let mut offset = 1;
            for value in self.iter() {
                if offset >= length {
                    unsafe {
                        buffer.get_unchecked_mut(length - 1).write(TRUCATED);
                    }
                    offset = length;
                    break;
                }
                if offset > 1 {
                    if offset + 2 >= length {
                        unsafe {
                            buffer.get_unchecked_mut(length - 1).write(TRUCATED);
                        }
                        offset = length;
                        break;
                    } else {
                        unsafe {
                            buffer.get_unchecked_mut(offset).write(b',');
                            buffer.get_unchecked_mut(offset + 1).write(b' ');
                        }
                        offset += 2;
                    }
                }
                offset += value.debug(&mut buffer[offset..]);
            }
            if offset < length {
                unsafe {
                    buffer.get_unchecked_mut(offset).write(b']');
                }
                offset += 1;
            }
            offset
        }
    }
}
#[cfg(feature = "macro")]
pub use pinocchio_log_macro::*;
