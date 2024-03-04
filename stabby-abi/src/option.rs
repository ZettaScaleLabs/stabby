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

//! A stable option for when rust's `Option<T>` isn't!

use crate::enums::IDeterminantProvider;
use crate::result::OkGuard;
use crate::{unreachable_unchecked, IStable};

/// A niche optimizing equivalent of [`core::option::Option`] that's ABI-stable regardless of the inner type's niches.
#[crate::stabby]
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Option<T: IStable + IDeterminantProvider<()>> {
    inner: crate::result::Result<T, ()>,
}
impl<T: IStable> core::fmt::Debug for Option<T>
where
    T: IDeterminantProvider<()>,
    T: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.as_ref().fmt(f)
    }
}
impl<T: IStable> From<core::option::Option<T>> for Option<T>
where
    T: IDeterminantProvider<()>,
{
    fn from(value: core::option::Option<T>) -> Self {
        match value {
            Some(value) => Self {
                inner: crate::result::Result::Ok(value),
            },
            None => Self {
                inner: crate::result::Result::Err(()),
            },
        }
    }
}
impl<T: IStable> From<Option<T>> for core::option::Option<T>
where
    T: IDeterminantProvider<()>,
{
    fn from(value: Option<T>) -> Self {
        value.inner.ok()
    }
}
impl<T: IStable> Default for Option<T>
where
    T: IDeterminantProvider<()>,
{
    fn default() -> Self {
        Self::None()
    }
}
/// A guard that ensures that niche determinants are reinserted if the `Some` variant of an [`Option`] is re-established after it may have been mutated.
///
/// When dropped, this guard ensures that the result's determinant is properly set.
/// Failing to drop this guard may result in undefined behaviour.
pub type SomeGuard<'a, T> = OkGuard<'a, T, ()>;
impl<T: IStable> Option<T>
where
    T: IDeterminantProvider<()>,
{
    /// Construct the `Some` variant.
    #[allow(non_snake_case)]
    pub fn Some(value: T) -> Self {
        Self {
            inner: crate::result::Result::Ok(value),
        }
    }
    /// Construct the `None` variant.
    #[allow(non_snake_case)]
    pub fn None() -> Self {
        Self {
            inner: crate::result::Result::Err(()),
        }
    }
    /// Returns a reference to the option's contents if they exist.
    pub fn as_ref(&self) -> core::option::Option<&T> {
        self.match_ref(Some, || None)
    }
    /// Returns a mutable reference to the option's contents if they exist.
    pub fn as_mut(&mut self) -> core::option::Option<SomeGuard<T>> {
        self.match_mut(Some, || None)
    }
    /// Equivalent to `match &self`. If you need multiple branches to obtain mutable access or ownership
    /// of a local, use [`Self::match_ref_ctx`] instead.
    pub fn match_ref<'a, U, FnSome: FnOnce(&'a T) -> U, FnNone: FnOnce() -> U>(
        &'a self,
        some: FnSome,
        none: FnNone,
    ) -> U {
        self.inner.match_ref(some, |_| none())
    }
    /// Equivalent to `match &self`.
    pub fn match_ref_ctx<'a, I, U, FnSome: FnOnce(I, &'a T) -> U, FnNone: FnOnce(I) -> U>(
        &'a self,
        ctx: I,
        some: FnSome,
        none: FnNone,
    ) -> U {
        self.inner.match_ref_ctx(ctx, some, move |ctx, _| none(ctx))
    }
    /// Equivalent to `match &mut self`. If you need multiple branches to obtain mutable access or ownership
    /// of a local, use [`Self::match_mut_ctx`] instead.
    pub fn match_mut<'a, U, FnSome: FnOnce(SomeGuard<'a, T>) -> U, FnNone: FnOnce() -> U>(
        &'a mut self,
        some: FnSome,
        none: FnNone,
    ) -> U {
        self.inner.match_mut(some, |_| none())
    }
    /// Equivalent to `match &mut self`.
    pub fn match_mut_ctx<
        'a,
        I,
        U,
        FnSome: FnOnce(I, SomeGuard<'a, T>) -> U,
        FnNone: FnOnce(I) -> U,
    >(
        &'a mut self,
        ctx: I,
        some: FnSome,
        none: FnNone,
    ) -> U {
        self.inner.match_mut_ctx(ctx, some, move |ctx, _| none(ctx))
    }
    /// Equivalent to `match self`. If you need multiple branches to obtain mutable access or ownership
    /// of a local, use [`Self::match_owned_ctx`] instead.
    pub fn match_owned<U, FnSome: FnOnce(T) -> U, FnNone: FnOnce() -> U>(
        self,
        some: FnSome,
        none: FnNone,
    ) -> U {
        self.inner.match_owned(some, |_| none())
    }
    /// Equivalent to `match self`.
    pub fn match_owned_ctx<I, U, FnSome: FnOnce(I, T) -> U, FnNone: FnOnce(I) -> U>(
        self,
        ctx: I,
        some: FnSome,
        none: FnNone,
    ) -> U {
        self.inner
            .match_owned_ctx(ctx, some, move |ctx, _| none(ctx))
    }
    /// Returns `true` if `self` contains a value.
    pub fn is_some(&self) -> bool {
        self.inner.is_ok()
    }
    /// Returns `true` if `self` doesn't contain a value.
    pub fn is_none(&self) -> bool {
        !self.is_some()
    }
    /// Unwraps the option, or runs `f` if no value was in it.
    pub fn unwrap_or_else<F: FnOnce() -> T>(self, f: F) -> T {
        self.match_owned(|x| x, f)
    }
    /// # Safety
    /// Calling this on `Self::None()` is UB.
    pub unsafe fn unwrap_unchecked(self) -> T {
        self.unwrap_or_else(|| unsafe { unreachable_unchecked!() })
    }
    /// # Panics
    /// If `!self.is_some`
    pub fn unwrap(self) -> T {
        self.unwrap_or_else(|| panic!("Option::unwrap called on None"))
    }
}
