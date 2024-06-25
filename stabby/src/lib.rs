//
// Copyright (c) 2023 ZettaScale Technology
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   Pierre Avital, <pierre.avital@me.com>
//

#![deny(
    missing_docs,
    clippy::missing_panics_doc,
    clippy::missing_const_for_fn,
    clippy::missing_safety_doc,
    clippy::missing_errors_doc
)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(doctest), doc = include_str!("../README.md"))]

extern crate core;

pub use stabby_abi::{
    assert_unchecked, dynptr, export, import, stabby, unreachable_unchecked, vtmacro as vtable,
};

pub use stabby_abi as abi;

pub use stabby_abi::alloc::{self, boxed, collections, string, sync, vec};

pub use stabby_abi::{Dyn, DynRef};

pub mod compiler_version;

/// ABI-stable tuples
pub use crate::abi::tuples as tuple;

/// Futures can be ABI-stable if you wish hard enough
#[cfg_attr(
    stabby_unsafe_wakers = "true",
    deprecated = "Warning! you are using the `stabby/stabby_unsafe_wakers` feature. This could cause UB if you poll a future received from another shared library with mismatching ABI! (this API isn't actually deprecated)"
)]
pub mod future {
    pub use crate::abi::future::*;
    use crate::boxed::Box;
    /// A type alias for `dynptr!(Box<dyn Future<Output = Output> + Send + Sync + 'a>)`
    pub type DynFuture<'a, Output> =
        crate::dynptr!(Box<dyn Future<Output = Output> + Send + Sync + 'a>);
    /// A type alias for `dynptr!(Box<dyn Future<Output = Output> + Send + 'a>)`
    pub type DynFutureUnsync<'a, Output> =
        crate::dynptr!(Box<dyn Future<Output = Output> + Send + 'a>);
    /// A type alias for `dynptr!(Box<dyn Future<Output = Output> + 'a>)`
    pub type DynFutureUnsend<'a, Output> = crate::dynptr!(Box<dyn Future<Output = Output> + 'a>);
}

/// The collection of traits that make `dynptr!(Box<dyn Fn...>)` possible
pub use crate::abi::closure;
pub use crate::abi::{option, result, slice, str};

pub use crate::abi::{vtable::Any, AccessAs, IStable, IntoSuperTrait};

#[cfg(all(feature = "libloading", any(unix, windows, doc)))]
/// Integration with [`libloading`](::libloading), allowing symbol loads to be validated thanks to either reflection or canaries.
///
/// Requires the `libloading` feature to be enabled.
pub mod libloading;

/// ABI-stable representations of durations and instants.
pub mod time;

/// Like [`std::format`], but returning an ABI-stable [`String`](crate::string::String)
#[macro_export]
macro_rules! format {
    ($($t: tt)*) => {{
        use ::core::fmt::Write;
        let mut s = $crate::string::String::default();
        ::core::write!(s, $($t)*).map(move |_| s)
    }};
}

#[cfg(doc)]
#[doc = include_str!("../TUTORIAL.md")]
pub mod _tutorial_ {}
#[cfg(test)]
mod tests {
    mod enums;
    mod layouts;
    mod traits;
}
