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

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(doctest), doc = include_str!("../README.md"))]

#[cfg(feature = "alloc")]
extern crate alloc;

pub use stabby_abi::{dynptr, export, import, stabby, vtmacro as vtable};

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
#[cfg_attr(
    feature = "unsafe_wakers",
    deprecated = "Warning! you are using the `stabby/unsafe_wakers` feature. This could cause UB if you poll a future received from another shared library with mismatching ABI! (this API isn't actually deprecated)"
)]
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

pub use crate::abi::{vtable::Any, AccessAs, IStable, IntoSuperTrait};

#[cfg(all(feature = "libloading", any(unix, windows)))]
pub mod libloading;

pub mod time {
    #[crate::stabby]
    pub struct Duration {
        pub secs: u64,
        pub nanos: u32,
    }
    impl From<core::time::Duration> for Duration {
        fn from(value: core::time::Duration) -> Self {
            Self {
                secs: value.as_secs(),
                nanos: value.subsec_nanos(),
            }
        }
    }
    impl From<Duration> for core::time::Duration {
        fn from(value: Duration) -> Self {
            Self::new(value.secs, value.nanos)
        }
    }
    #[crate::stabby]
    pub struct TimeSpec(pub Duration);
    #[crate::stabby]
    pub struct Instant(pub TimeSpec);
    #[crate::stabby]
    pub struct SystemTime(pub TimeSpec);
    #[cfg(feature = "std")]
    mod impls {
        use core::time::Duration;
        use std::time::UNIX_EPOCH;

        use super::TimeSpec;
        impl From<std::time::SystemTime> for TimeSpec {
            fn from(value: std::time::SystemTime) -> Self {
                Self(
                    value
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or(Duration::new(0, 0))
                        .into(),
                )
            }
        }
        impl From<TimeSpec> for std::time::SystemTime {
            fn from(value: TimeSpec) -> Self {
                UNIX_EPOCH + value.0.into()
            }
        }
        impl From<std::time::Instant> for TimeSpec {
            fn from(value: std::time::Instant) -> Self {
                unsafe { core::mem::transmute::<_, std::time::SystemTime>(value).into() }
            }
        }
        impl From<TimeSpec> for std::time::Instant {
            fn from(value: TimeSpec) -> Self {
                unsafe { core::mem::transmute::<std::time::SystemTime, _>(value.into()) }
            }
        }
    }
}
