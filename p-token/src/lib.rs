//! Another ERC20-like Token program for the Solana blockchain.

#![no_std]
#![feature(fmt_helpers_for_derive)]
#![feature(core_intrinsics)]
#![feature(structural_match)]
#![feature(coverage_attribute)]
#![feature(derive_clone_copy)]
#![feature(derive_eq)]
#![feature(panic_internals)]

mod entrypoint;
mod processor;
mod pinocchio;
mod pinocchio_pubkey;
mod pinocchio_log;
mod spl_token_interface;

#[inline(never)]
#[no_mangle]
fn main() {
    ()
}
