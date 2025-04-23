#![feature(prelude_import)]
//#[prelude_import]
//use std::prelude::rust_2021::*;
//#[macro_use]
//extern crate std;
pub use five8_const::decode_32_const;
pub mod pinocchio {
  pub use crate::pinocchio::*;
}
#[inline(always)]
pub const fn from_str(value: &str) -> pinocchio::pubkey::Pubkey {
    decode_32_const(value)
}
