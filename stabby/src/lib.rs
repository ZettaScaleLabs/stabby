#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(feature = "alloc")]
extern crate alloc;

pub use stabby_macros::{stabby, vtable};

pub use stabby_abi as abi;

#[cfg(feature = "alloc")]
mod allocs;
#[cfg(feature = "alloc")]
pub use allocs::*;

pub mod slice;
pub mod str;
pub mod tuple;

pub use crate::abi::future;
pub use crate::abi::option;
pub use crate::abi::result;

#[cfg(test)]
mod tests;
