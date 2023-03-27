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
//   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
//

#![cfg_attr(not(feature = "std"), no_std)]
#![doc = include_str!("../README.md")]

#[cfg(feature = "alloc")]
extern crate alloc;

pub use stabby_abi::{dynptr, stabby, vtmacro as vtable};

pub use stabby_abi as abi;

#[cfg(feature = "alloc")]
mod allocs;
#[cfg(feature = "alloc")]
pub use allocs::*;

pub use stabby_abi::{Dyn, DynRef};

pub mod compiler_version;

/// ABI-stable tuples
pub mod tuple;

/// Futures can be ABI-stable if you wish hard enough
pub mod future {
    pub use crate::abi::future::*;
    #[cfg(feature = "alloc")]
    /// A type alias for `dynptr!(Box<dyn Future<Output = Output> + Send + Sync + 'a>)`
    pub type DynFuture<'a, Output> =
        crate::dynptr!(Box<dyn Future<Output = Output> + Send + Sync + 'a>);
    #[cfg(feature = "alloc")]
    /// A type alias for `dynptr!(Box<dyn Future<Output = Output> + Send + 'a>)`
    pub type DynFutureUnsync<'a, Output> =
        crate::dynptr!(Box<dyn Future<Output = Output> + Send + 'a>);
    #[cfg(feature = "alloc")]
    /// A type alias for `dynptr!(Box<dyn Future<Output = Output> + 'a>)`
    pub type DynFutureUnsend<'a, Output> = crate::dynptr!(Box<dyn Future<Output = Output> + 'a>);
}

/// The collection of traits that make `dynptr!(Box<dyn Fn...>)` possible
pub use crate::abi::closure;
pub use crate::abi::{option, result, slice, str};

pub use crate::abi::{AccessAs, IStable};
