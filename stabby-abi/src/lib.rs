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

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
mod allocs;

pub use stabby_macros::{canary_suffixes, dynptr, export, import, stabby, vtable as vtmacro};

use core::fmt::{Debug, Display};

pub const fn assert_stable<T: IStable>() {}

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
            last_break: $crate::report::Version::NEVER,
            tyty: $crate::report::TyTy::Struct,
        };
    };
    ($name: expr) => {
        const REPORT: &'static $crate::report::TypeReport = &$crate::report::TypeReport {
            name: $crate::str::Str::new($name),
            module: $crate::str::Str::new(core::module_path!()),
            fields: $crate::StableLike::new(None),
            last_break: $crate::report::Version::NEVER,
            tyty: $crate::report::TyTy::Struct,
        };
    };
}

pub mod typenum2;
use istable::{ISaturatingAdd, Saturator};
#[doc(hidden)]
pub use typenum2::*;

#[macro_export]
macro_rules! assert_optimal_layout {
    ($t: ty) => {
        const _: () = {
            assert!(<$t>::has_optimal_layout());
        };
    };
}
pub use crate::enums::IDiscriminantProvider;
// pub use crate::Result;
pub mod as_mut;

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
    fn ref_as<T: ?Sized>(&self) -> <Self as as_mut::IGuardRef<T>>::Guard<'_>
    where
        Self: as_mut::IGuardRef<T>;
    fn mut_as<T: ?Sized>(&mut self) -> <Self as as_mut::IGuardMut<T>>::GuardMut<'_>
    where
        Self: as_mut::IGuardMut<T>;
}

pub use fatptr::*;
mod fatptr;
// pub use istabilize::IStabilize;
// mod istabilize;
pub mod closure;
pub mod future;
mod stable_impls;
pub mod vtable;

// #[allow(type_alias_bounds)]
// pub type Stable<Source: IStabilize> = Source::Stable;

pub struct AssertStable<T: IStable>(pub core::marker::PhantomData<T>);
impl<T: IStable> AssertStable<T> {
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
impl<T, As: IStable> StableLike<T, As> {
    pub const fn new(value: T) -> Self {
        if core::mem::size_of::<T>() != <As::Size as Unsigned>::USIZE {
            panic!(
                "Attempted to construct `StableLike<T, As>` despite As::Size not matching T's size"
            )
        }
        Self {
            value,
            marker: core::marker::PhantomData,
        }
    }
    /// # Safety
    /// This is only safe if `T` is FFI-safe, or if this `self` was constructed from a value
    /// of `T` that was instanciated within the same shared object.
    pub const unsafe fn as_ref_unchecked(&self) -> &T {
        &self.value
    }
    /// # Safety
    /// This is only safe if `T` is FFI-safe, or if this `self` was constructed from a value
    /// of `T` that was instanciated within the same shared object.
    pub unsafe fn as_mut_unchecked(&mut self) -> &mut T {
        &mut self.value
    }
    /// # Safety
    /// This is only safe if `T` is FFI-safe, or if this `self` was constructed from a value
    /// of `T` that was instanciated within the same shared object.
    pub unsafe fn into_inner_unchecked(self) -> T {
        self.value
    }
    pub fn into_inner(self) -> T
    where
        T: IStable,
    {
        self.value
    }
}

impl<T: IStable, As: IStable> core::ops::Deref for StableLike<T, As> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.as_ref_unchecked() }
    }
}
impl<T: IStable, As: IStable> core::ops::DerefMut for StableLike<T, As> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.as_mut_unchecked() }
    }
}
unsafe impl<T, As: IStable> IStable for StableLike<T, As> {
    type Size = As::Size;
    type Align = As::Align;
    type ForbiddenValues = As::ForbiddenValues;
    type UnusedBits = As::UnusedBits;
    type HasExactlyOneNiche = As::HasExactlyOneNiche;
    const REPORT: &'static report::TypeReport = As::REPORT;
}

/// Emulates a type of size `Size` and alignment `Align`.
///
/// Note that this is not a ZST, and that you may pass [`B0`] or [`B1`] as the this generic parameter if you
/// want to inform `stabby` that the type it emulates has exactly zero or one niche respectively that the
/// compiler knows about. This information can be used by `stabby` to determine that `core::option::Option`s
/// transitively containing the emulated type are indeed ABI-stable.
pub struct NoNiches<Size: Unsigned, Align: PowerOf2, HasExactlyOneNiche: ISaturatingAdd = Saturator>(
    Size::Padding,
    core::marker::PhantomData<(Size, Align, HasExactlyOneNiche)>,
);
unsafe impl<Size: Unsigned, Align: PowerOf2, HasExactlyOneNiche: ISaturatingAdd> IStable
    for NoNiches<Size, Align, HasExactlyOneNiche>
{
    type Size = Size;
    type Align = Align;
    type ForbiddenValues = End;
    type UnusedBits = End;
    type HasExactlyOneNiche = HasExactlyOneNiche;
    primitive_report!("NoNiches");
}

/// Allows removing the [`IStable`] implementation from `T` if `Cond` is not also ABI-stable.
///
/// This is typically used in combination with [`StableLike`], for example in vtables to mark function
/// pointers as stable only if all of their arguments are stable.
#[repr(C)]
pub struct StableIf<T, Cond> {
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
    const REPORT: &'static report::TypeReport = T::REPORT;
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct FieldPair<A, B>(core::marker::PhantomData<(A, B)>);
#[repr(transparent)]
pub struct Struct<T>(T);

#[repr(C)]
pub union Union<A, B> {
    pub ok: core::mem::ManuallyDrop<A>,
    pub err: core::mem::ManuallyDrop<B>,
}
impl<A, B> Clone for Union<A, B> {
    fn clone(&self) -> Self {
        unsafe { core::ptr::read(self) }
    }
}

pub mod checked_import;
pub mod enums;
pub mod padding;
pub mod result;
pub use result::Result;
pub mod option;
pub use option::Option;
pub mod report;
pub mod slice;
pub mod str;

pub use istable::{Array, End, IStable};

pub mod istable;
pub type NonZeroHole = U0;
