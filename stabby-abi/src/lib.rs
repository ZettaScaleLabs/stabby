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

//! The core of the [`stabby`](https://crates.io/crates/stabby) ABI.
//!
//! This crate is generally not meant to be used directly, but through the `stabby` crate.

#![deny(
    missing_docs,
    clippy::missing_panics_doc,
    clippy::missing_const_for_fn,
    clippy::missing_safety_doc,
    clippy::missing_errors_doc,
    // clippy::undocumented_unsafe_blocks
)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(stabby_nightly, feature(freeze))]

#[cfg(feature = "alloc-rs")]
extern crate alloc as alloc_rs;

/// ABI-stable smart pointers and allocated data structures, with support for custom allocators.
pub mod alloc;
/// Extending [Non-Zero Types](core::num) to enable niches for other values than 0.
pub mod num;

pub use stabby_macros::{canary_suffixes, dynptr, export, import, stabby, vtable as vtmacro};
use typenum2::unsigned::Alignment;

use core::fmt::{Debug, Display};

/// A no-op that fails to compile if `T` isn't proven ABI-stable by stabby.
pub const fn assert_stable<T: IStable>() {}

/// An ABI-stable tuple.
pub use tuple::Tuple2 as Tuple;

/// Generate the [`IStable::REPORT`] and [`IStable::ID`] fields for an implementation of [`IStable`].
#[macro_export]
macro_rules! primitive_report {
    ($name: expr, $ty: ty) => {
        const REPORT: &'static $crate::report::TypeReport = &$crate::report::TypeReport {
            name: $crate::str::Str::new($name),
            module: $crate::str::Str::new(core::module_path!()),
            fields: $crate::StableLike::new(Some(&$crate::report::FieldReport {
                name: $crate::str::Str::new("inner"),
                ty: <$ty as $crate::IStable>::REPORT,
                next_field: $crate::StableLike::new(None),
            })),
            version: 0,
            tyty: $crate::report::TyTy::Struct,
        };
        const ID: u64 = $crate::report::gen_id(Self::REPORT);
    };
    ($name: expr) => {
        const REPORT: &'static $crate::report::TypeReport = &$crate::report::TypeReport {
            name: $crate::str::Str::new($name),
            module: $crate::str::Str::new(core::module_path!()),
            fields: $crate::StableLike::new(None),
            version: 0,
            tyty: $crate::report::TyTy::Struct,
        };
        const ID: u64 = $crate::report::gen_id(Self::REPORT);
    };
}

/// A support module for stabby's dark magic.
///
/// It implements basic arithmetics in the type system, and needs to be included in stabby for the ternaries
/// to keep trait bounds that are needed for proofs to work out.
pub mod typenum2;
use istable::{ISaturatingAdd, Saturator};
#[doc(hidden)]
pub use typenum2::*;

/// A re-export of `rustversion` used in macros for dark magic.
///
/// Its API is subject to un-anounced changes.
pub use rustversion as __rustversion;

/// A support macro for stabby's dark magic.
///
/// Its API is subject to un-anounced changes.
#[macro_export]
macro_rules! impl_vtable_constructor {
    ($pre178: item => $post178: item) => {
        #[$crate::__rustversion::before(1.78.0)]
        $pre178
        #[$crate::__rustversion::since(1.78.0)]
        $post178
    };
}

/// Fires a compile error if the layout of a type is deemed sub-optimal.
#[macro_export]
macro_rules! assert_optimal_layout {
    ($t: ty) => {
        const _: () = {
            assert!(<$t>::has_optimal_layout());
        };
    };
}
pub use crate::enums::IDeterminantProvider;
/// Helpers to treat ABI-stable types as if they were their unstable equivalents.
pub mod as_mut;
/// ABI-stable equivalents of iterators.
pub mod iter;

/// Provides access to a value _as if_ it were of another type.
///
/// This is done by the following process:
/// - memcopy `self` into `copy`
/// - convert `copy` into `target: ManuallyDrop<Target>`
/// - provide a guard that can `Deref` or `DerefMut` into `target`
/// - upon dropping the mutable guard, convert `target` and assing `target` to `self`
///
/// This is always safe for non-self-referencial types.
pub trait AccessAs {
    /// Provides immutable access to a type as if it were its ABI-unstable equivalent.
    fn ref_as<T: ?Sized>(&self) -> <Self as as_mut::IGuardRef<T>>::Guard<'_>
    where
        Self: as_mut::IGuardRef<T>;
    /// Provides mutable access to a type as if it were its ABI-unstable equivalent.
    fn mut_as<T: ?Sized>(&mut self) -> <Self as as_mut::IGuardMut<T>>::GuardMut<'_>
    where
        Self: as_mut::IGuardMut<T>;
}

pub use fatptr::*;
/// How stabby does multi-trait objects.
mod fatptr;

/// Closures, but ABI-stable
pub mod closure;
/// Futures, but ABI-stable
pub mod future;
mod stable_impls;
/// Support for vtables for multi-trait objects
pub mod vtable;

// #[allow(type_alias_bounds)]
// pub type Stable<Source: IStabilize> = Source::Stable;

/// A ZST that's only allowed to exist if its generic parameter is ABI-stable.
pub struct AssertStable<T: IStable>(pub core::marker::PhantomData<T>);
impl<T: IStable> AssertStable<T> {
    /// Proves that `T` is ABI-stable.
    pub const fn assert() -> Self {
        Self(core::marker::PhantomData)
    }
}

/// Lets you tell `stabby` that `T` has the same stable layout as `As`.
///
/// Lying about this link between `T` and `As` will cause UB if a `#[repr(stabby)]` enum transitively contains
/// a value of this type.
///
/// If you want to be safe when using this, use [`NoNiches`] with the correct size and alignment for your
/// type.
#[repr(C)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StableLike<T, As> {
    value: T,
    marker: core::marker::PhantomData<As>,
}
impl<T: Debug, As> Debug for StableLike<T, As> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.value.fmt(f)
    }
}
impl<T: Display, As> Display for StableLike<T, As> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.value.fmt(f)
    }
}
impl<T: Clone, As> Clone for StableLike<T, As> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            marker: self.marker,
        }
    }
}
impl<T: Copy, As> Copy for StableLike<T, As> {}
trait ConstChecks {
    const CHECK: ();
}
impl<T, As: IStable> ConstChecks for StableLike<T, As> {
    const CHECK: () = {
        if core::mem::size_of::<T>() != <As::Size as Unsigned>::USIZE {
            panic!(
                "Attempted to construct `StableLike<T, As>` despite As::Size not matching T's size"
            )
        }
        if core::mem::align_of::<T>() != <As::Align as Unsigned>::USIZE {
            panic!(
                "Attempted to construct `StableLike<T, As>` despite As::Size not matching T's size"
            )
        }
    };
}
impl<T, As: IStable> StableLike<T, As> {
    /// Wraps a value in a type that provides information about its layout.
    ///
    /// Asserts that `T` and `As` have the same size and aligment at compile time,
    /// and relies on the user for the niche information to be correct.
    #[allow(clippy::let_unit_value)]
    pub const fn new(value: T) -> Self {
        _ = Self::CHECK;
        Self {
            value,
            marker: core::marker::PhantomData,
        }
    }
    /// Returns a reference to the underlying type
    /// # Safety
    /// This is only safe if `T` is FFI-safe, or if this `self` was constructed from a value
    /// of `T` that was instanciated within the same shared object.
    pub const unsafe fn as_ref_unchecked(&self) -> &T {
        &self.value
    }
    /// Returns a reference to the underlying type
    pub const fn as_ref(&self) -> &T
    where
        T: IStable,
    {
        &self.value
    }
    /// # Safety
    /// This is only safe if `T` is FFI-safe, or if this `self` was constructed from a value
    /// of `T` that was instanciated within the same shared object.
    #[rustversion::attr(since(1.86), const)]
    pub unsafe fn as_mut_unchecked(&mut self) -> &mut T {
        &mut self.value
    }
    /// # Safety
    /// This is only safe if `T` is FFI-safe, or if this `self` was constructed from a value
    /// of `T` that was instanciated within the same shared object.
    pub unsafe fn into_inner_unchecked(self) -> T {
        self.value
    }
    /// Extracts the inner value from `self`
    pub fn into_inner(self) -> T
    where
        T: IStable,
    {
        self.value
    }
}

unsafe impl<T, As: IStable> IStable for StableLike<T, As> {
    type Size = As::Size;
    type Align = As::Align;
    type ForbiddenValues = As::ForbiddenValues;
    type UnusedBits = As::UnusedBits;
    type HasExactlyOneNiche = As::HasExactlyOneNiche;
    type ContainsIndirections = As::ContainsIndirections;
    #[cfg(feature = "experimental-ctypes")]
    type CType = As::CType;
    const ID: u64 = crate::report::gen_id(Self::REPORT);
    const REPORT: &'static report::TypeReport = As::REPORT;
}

/// Emulates a type of size `Size` and alignment `Align`.
///
/// Note that this is not a ZST, and that you may pass [`B0`] or [`B1`] as the this generic parameter if you
/// want to inform `stabby` that the type it emulates has exactly zero or one niche respectively that the
/// compiler knows about. This information can be used by `stabby` to determine that `core::option::Option`s
/// transitively containing the emulated type are indeed ABI-stable.
pub struct NoNiches<
    Size: Unsigned,
    Align: Alignment,
    HasExactlyOneNiche: ISaturatingAdd = Saturator,
    ContainsIndirections: Bit = B0,
>(
    Size::Padding,
    core::marker::PhantomData<(Size, Align, HasExactlyOneNiche, ContainsIndirections)>,
);
unsafe impl<
        Size: Unsigned,
        Align: Alignment,
        HasExactlyOneNiche: ISaturatingAdd,
        ContainsIndirections: Bit,
    > IStable for NoNiches<Size, Align, HasExactlyOneNiche, ContainsIndirections>
{
    type Size = Size;
    type Align = Align;
    type ForbiddenValues = End;
    type UnusedBits = End;
    type HasExactlyOneNiche = HasExactlyOneNiche;
    type ContainsIndirections = ContainsIndirections;
    #[cfg(feature = "experimental-ctypes")]
    type CType = ();
    primitive_report!("NoNiches");
}

/// Allows removing the [`IStable`] implementation from `T` if `Cond` is not also ABI-stable.
///
/// This is typically used in combination with [`StableLike`], for example in vtables to mark function
/// pointers as stable only if all of their arguments are stable.
#[repr(C)]
pub struct StableIf<T, Cond> {
    /// The actual value
    pub value: T,
    marker: core::marker::PhantomData<Cond>,
}
impl<T: Clone, Cond> Clone for StableIf<T, Cond> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            marker: self.marker,
        }
    }
}
impl<T: Copy, Cond> Copy for StableIf<T, Cond> {}
impl<T, Cond> StableIf<T, Cond> {
    /// # Safety
    /// Refer to type documentation
    pub const unsafe fn new(value: T) -> Self {
        Self {
            value,
            marker: core::marker::PhantomData,
        }
    }
}

impl<T, Cond> core::ops::Deref for StableIf<T, Cond> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl<T, Cond> core::ops::DerefMut for StableIf<T, Cond> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
unsafe impl<T: IStable, Cond: IStable> IStable for StableIf<T, Cond> {
    type Size = T::Size;
    type Align = T::Align;
    type ForbiddenValues = T::ForbiddenValues;
    type UnusedBits = T::UnusedBits;
    type HasExactlyOneNiche = T::HasExactlyOneNiche;
    type ContainsIndirections = T::ContainsIndirections;
    #[cfg(feature = "experimental-ctypes")]
    type CType = T::CType;
    const REPORT: &'static report::TypeReport = T::REPORT;
    const ID: u64 = crate::report::gen_id(Self::REPORT);
}

/// Used by proc-macros to concatenate fields before wrapping them in a [`Struct`] to compute their layout.
#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct FieldPair<A, B>(core::marker::PhantomData<(A, B)>);
/// Used by proc-macros to ensure a list of fields gets the proper end padding.
#[repr(transparent)]
pub struct Struct<T>(T);

/// Used by proc-macros to ensure a list of fields gets the proper end padding when specific alignments are requested.
pub struct AlignedStruct<T, Align>(core::marker::PhantomData<(T, Align)>);

/// Used by [`crate::result::Result`]
#[repr(C)]
pub union Union<A, B> {
    /// The `ok` variant of the union.
    pub ok: core::mem::ManuallyDrop<A>,
    /// The `err` variant of the union.
    pub err: core::mem::ManuallyDrop<B>,
}
impl<A, B> Clone for Union<A, B> {
    fn clone(&self) -> Self {
        // SAFETY: `Union` is actually `Copy`
        unsafe { core::ptr::read(self) }
    }
}

/// How `stabby` exposes symbols that must be checked through canaries or reflection before being accessed to prevent UB after linking ABI-incompatible functions.
pub mod checked_import;
/// ABI-stable compact sum types!
pub mod enums;
/// Like [`core::result::Result`], but ABI-stable with niche optimizations!
pub mod result;
pub use result::Result;
/// Like [`core::option::Option`], but ABI-stable with niche optimizations!
pub mod option;
pub use option::Option;
/// A very simple ABI-stable reflection framework.
pub mod report;
/// ABI-stable slices.
pub mod slice;
/// ABI-stable strs.
pub mod str;
/// ABI-stable tuples.
pub mod tuple {
    include!(concat!(env!("OUT_DIR"), "/tuples.rs"));
}

pub use istable::{Array, End, IStable};

/// The heart of `stabby`: the [`IStable`] trait.
pub mod istable;

/// Expands to [`unreachable!()`](core::unreachable) in debug builds or if `--cfg stabby_check_unreachable=true` has been set in the `RUST_FLAGS`, and to [`core::hint::unreachable_unchecked`] otherwise.
///
/// This lets the compiler take advantage of the fact that the code is unreachable in release builds, and optimize accordingly, while giving you the opportunity to double check this at runtime in case of doubts.
///
/// # Panics
/// This macro panics if the code is actually reachable in debug mode.
/// This would mean that release code would be UB!
///
/// # Safety
/// This macro is inherently unsafe, as it can cause UB in release mode if the code is actually reachable.
#[macro_export]
macro_rules! unreachable_unchecked {
    () => {
        if cfg!(any(debug_assertions, stabby_check_unreachable = "true")) {
            ::core::unreachable!()
        } else {
            ::core::hint::unreachable_unchecked()
        }
    };
}

/// Expands to [`assert!(condition)`](core::assert) in debug builds or if `--cfg stabby_check_unreachable=true` has been set in the `RUST_FLAGS`, and to [`if condition {core::hint::unreachable_unchecked()}`](core::hint::unreachable_unchecked) otherwise.
///
/// This lets the compiler take advantage of the fact that the condition is always true in release builds, and optimize accordingly, while giving you the opportunity to double check this at runtime in case of doubts.
///
/// # Panics
/// This macro panics if the code is actually false in debug mode.
/// This would mean that release code would be UB!
///
/// # Safety
/// This macro is inherently unsafe, as it can cause UB in release mode if the assertion can actually be false.
#[macro_export]
macro_rules! assert_unchecked {
    ($e: expr, $($t: tt)*) => {
        if cfg!(any(debug_assertions, stabby_check_unreachable = "true")) {
            ::core::assert!($e, $($t)*);
        } else {
            if !$e {
                ::core::hint::unreachable_unchecked();
            }
        }
    };
}

/// Expands to [`assert_eq`](core::assert_eq) in debug builds or if `--cfg stabby_check_unreachable=true` has been set in the `RUST_FLAGS`, and to [`if a != b {core::hint::unreachable_unchecked()}`](core::hint::unreachable_unchecked) otherwise.
///
/// This lets the compiler take advantage of the fact that the condition is always true in release builds, and optimize accordingly, while giving you the opportunity to double check this at runtime in case of doubts.
///
/// # Panics
/// This macro panics if the code is actually false in debug mode.
/// This would mean that release code would be UB!
///
/// # Safety
/// This macro is inherently unsafe, as it can cause UB in release mode if the assertion can actually be false.
#[macro_export]
macro_rules! assert_eq_unchecked {
    ($a: expr, $b: expr, $($t: tt)*) => {
        if cfg!(any(debug_assertions, stabby_check_unreachable = "true")) {
            ::core::assert_eq!($a, $b, $($t)*);
        } else {
            if $a != $b {
                ::core::hint::unreachable_unchecked();
            }
        }
    };
}
