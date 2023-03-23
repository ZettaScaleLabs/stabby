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

pub mod future {
    pub use crate::abi::future::*;
    #[cfg(feature = "alloc")]
    pub type DynFuture<'a, Output> = crate::abi::Dyn<
        'a,
        Box<()>,
        crate::vtable!(crate::future::Future<Output = Output> + Send + Sync),
    >;
    #[cfg(feature = "alloc")]
    pub type DynFutureUnsync<'a, Output> =
        crate::abi::Dyn<'a, Box<()>, crate::vtable!(crate::future::Future<Output = Output> + Send)>;
    #[cfg(feature = "alloc")]
    pub type DynFutureUnsend<'a, Output> =
        crate::abi::Dyn<'a, Box<()>, crate::vtable!(crate::future::Future<Output = Output>)>;
}
pub use crate::abi::option;
pub use crate::abi::result;

#[cfg(test)]
mod tests;
