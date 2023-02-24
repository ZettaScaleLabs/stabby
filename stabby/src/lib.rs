#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(feature = "alloc")]
extern crate alloc;

pub use stabby_macros::stabby;

pub mod type_layouts;
#[allow(type_alias_bounds)]
pub type Stable<Source: type_layouts::IStabilize> = Source::Stable;

#[cfg(feature = "alloc")]
mod allocs {
    pub mod boxed;
    pub mod vec {}
}
#[cfg(feature = "alloc")]
pub use allocs::*;
pub mod slice;
pub mod str;
pub mod tuple {
    pub use crate::type_layouts::Tuple2;
}
// #[cfg(test)]
mod tests;
