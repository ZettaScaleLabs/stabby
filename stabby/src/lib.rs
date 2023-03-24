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

#[cfg(feature = "alloc")]
extern crate alloc;

pub use stabby_macros::{dynptr, stabby, vtable};

pub use stabby_abi as abi;

#[cfg(feature = "alloc")]
mod allocs;
#[cfg(feature = "alloc")]
pub use allocs::*;

pub use stabby_abi::{Dyn, DynRef};
pub mod tuple;

pub mod future {
    pub use crate::abi::future::*;
    #[cfg(feature = "alloc")]
    pub type DynFuture<'a, Output> =
        crate::dynptr!(Box<dyn Future<Output = Output> + Send + Sync + 'a>);
    #[cfg(feature = "alloc")]
    pub type DynFutureUnsync<'a, Output> =
        crate::dynptr!(Box<dyn Future<Output = Output> + Send + 'a>);
    #[cfg(feature = "alloc")]
    pub type DynFutureUnsend<'a, Output> = crate::dynptr!(Box<dyn Future<Output = Output> + 'a>);
}
pub use crate::abi::{closure, option, result, slice, str};

#[cfg(test)]
mod tests;
