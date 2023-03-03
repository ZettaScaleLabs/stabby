#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(feature = "alloc")]
extern crate alloc;

pub use stabby_macros::{stabby, vtable};

pub mod abi;
#[allow(type_alias_bounds)]
pub type Stable<Source: abi::IStabilize> = Source::Stable;

#[cfg(feature = "alloc")]
mod allocs;
#[cfg(feature = "alloc")]
pub use allocs::*;
pub mod slice;
pub mod str;
pub mod tuple {
    pub use crate::abi::Tuple2;
}
// #[cfg(test)]
mod tests;
