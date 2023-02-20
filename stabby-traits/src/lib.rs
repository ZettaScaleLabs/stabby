#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
pub(crate) use std as alloc;
#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

pub use stabby_macros::stabby;

pub use type_layouts::*;
pub use typenum::*;
pub mod type_layouts;

pub mod holes {
    include!(concat!(env!("OUT_DIR"), "/holes.rs"));
}
pub(crate) mod stabby_traits {
    pub use super::holes;
}

// mod stable_impls;
