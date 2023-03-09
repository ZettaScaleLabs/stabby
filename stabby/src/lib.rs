#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(feature = "alloc")]
extern crate alloc;

pub use stabby_macros::{stabby, vtable};

pub mod abi;

#[cfg(feature = "alloc")]
mod allocs;
#[cfg(feature = "alloc")]
pub use allocs::*;
pub mod slice;
pub mod str;
pub mod tuple {
    pub use crate::abi::Tuple2;
}
pub mod option;
pub mod result;
// #[cfg(test)]
mod tests;
