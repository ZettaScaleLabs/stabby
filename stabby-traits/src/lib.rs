#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
pub(crate) use std as alloc;
#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

pub use stabby_macros::stabby;

pub use type_layouts::{AssertStable, IStable as Stable};
pub mod type_layouts;

pub mod holes {
    include!(concat!(env!("OUT_DIR"), "/holes.rs"));
}

mod stable_impls;
