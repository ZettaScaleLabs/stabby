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

//! Stable results!

use core::ops::DerefMut;

pub use crate::enums::IDeterminant;
use crate::enums::IDeterminantProvider;
use crate::padding::Padded;
use crate::Union;
use crate::{self as stabby, unreachable_unchecked, IStable};

/// An ABI-stable, niche optimizing equivalent of [`core::result::Result`]
#[stabby::stabby]
pub struct Result<Ok, Err>
where
    Ok: IDeterminantProvider<Err>,
    Err: IStable,
{
    niche_exporter: <Ok as IDeterminantProvider<Err>>::NicheExporter,
    discriminant: <Ok as IDeterminantProvider<Err>>::Determinant,
    #[allow(clippy::type_complexity)]
    union: Union<
        Padded<<Ok as IDeterminantProvider<Err>>::OkShift, Ok>,
        Padded<<Ok as IDeterminantProvider<Err>>::ErrShift, Err>,
    >,
}

impl<Ok: Clone, Err: Clone> Clone for Result<Ok, Err>
where
    Ok: IDeterminantProvider<Err>,
    Err: IStable,
{
    fn clone(&self) -> Self {
        self.match_ref(|ok| Self::Ok(ok.clone()), |err| Self::Err(err.clone()))
    }
}
impl<Ok, Err> core::fmt::Debug for Result<Ok, Err>
where
    Ok: IDeterminantProvider<Err>,
    Err: IStable,
    Ok: core::fmt::Debug,
    Err: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.as_ref().fmt(f)
    }
}
impl<Ok, Err> core::hash::Hash for Result<Ok, Err>
where
    Ok: IDeterminantProvider<Err>,
    Err: IStable,
    Ok: core::hash::Hash,
    Err: core::hash::Hash,
{
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        if self.is_ok() {
            true.hash(state);
            unsafe { &self.union.ok }.hash(state);
        } else {
            false.hash(state);
            unsafe { &self.union.err }.hash(state);
        }
    }
}
impl<Ok, Err> core::cmp::PartialEq for Result<Ok, Err>
where
    Ok: IDeterminantProvider<Err>,
    Err: IStable,
    Ok: core::cmp::PartialEq,
    Err: core::cmp::PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self.is_ok(), other.is_ok()) {
            (true, true) => unsafe { self.union.ok.eq(&other.union.ok) },
            (false, false) => unsafe { self.union.err.eq(&other.union.err) },
            _ => false,
        }
    }
}
impl<Ok, Err> core::cmp::Eq for Result<Ok, Err>
where
    Ok: IDeterminantProvider<Err>,
    Err: IStable,
    Ok: core::cmp::Eq,
    Err: core::cmp::Eq,
{
}
impl<Ok, Err> From<core::result::Result<Ok, Err>> for Result<Ok, Err>
where
    Ok: IDeterminantProvider<Err>,
    Err: IStable,
{
    fn from(value: core::result::Result<Ok, Err>) -> Self {
        match value {
            Ok(value) => Self::Ok(value),
            Err(value) => Self::Err(value),
        }
    }
}
impl<Ok, Err> From<Result<Ok, Err>> for core::result::Result<Ok, Err>
where
    Ok: IDeterminantProvider<Err>,
    Err: IStable,
{
    fn from(value: Result<Ok, Err>) -> Self {
        value.match_owned(Ok, Err)
    }
}
impl<Ok, Err> Drop for Result<Ok, Err>
where
    Ok: IDeterminantProvider<Err>,
    Err: IStable,
{
    fn drop(&mut self) {
        self.match_mut(
            |ok| unsafe { core::ptr::drop_in_place(ok) },
            |err| unsafe { core::ptr::drop_in_place(err) },
        )
    }
}
impl<Ok, Err> Result<Ok, Err>
where
    Ok: IDeterminantProvider<Err>,
    Err: IStable,
{
    /// Construct the `Ok` variant.
    #[allow(non_snake_case)]
    pub fn Ok(value: Ok) -> Self {
        let mut union = Union {
            ok: core::mem::ManuallyDrop::new(Padded {
                lpad: Default::default(),
                value,
            }),
        };
        Self {
            niche_exporter: Default::default(),
            discriminant: unsafe {
                <Ok as IDeterminantProvider<Err>>::Determinant::ok(&mut union as *mut _ as *mut _)
            },
            union,
        }
    }
    /// Construct the `Err` variant.
    #[allow(non_snake_case)]
    pub fn Err(value: Err) -> Self {
        let mut union = Union {
            err: core::mem::ManuallyDrop::new(Padded {
                lpad: Default::default(),
                value,
            }),
        };
        Self {
            niche_exporter: Default::default(),
            discriminant: unsafe {
                <Ok as IDeterminantProvider<Err>>::Determinant::err(&mut union as *mut _ as *mut _)
            },
            union,
        }
    }
    /// Converts to a standard [`Result`](core::result::Result) of immutable references to the variants.
    #[allow(clippy::missing_errors_doc)]
    pub fn as_ref(&self) -> core::result::Result<&Ok, &Err> {
        self.match_ref(Ok, Err)
    }

    /// Equivalent to `match &self`. If you need multiple branches to obtain mutable access or ownership
    /// of a local, use [`Self::match_ref_ctx`] instead.
    pub fn match_ref<'a, U, FnOk: FnOnce(&'a Ok) -> U, FnErr: FnOnce(&'a Err) -> U>(
        &'a self,
        ok: FnOk,
        err: FnErr,
    ) -> U {
        if self.is_ok() {
            unsafe { ok(&self.union.ok.value) }
        } else {
            unsafe { err(&self.union.err.value) }
        }
    }
    /// Equivalent to `match &self`.
    pub fn match_ref_ctx<'a, T, U, FnOk: FnOnce(T, &'a Ok) -> U, FnErr: FnOnce(T, &'a Err) -> U>(
        &'a self,
        ctx: T,
        ok: FnOk,
        err: FnErr,
    ) -> U {
        if self.is_ok() {
            unsafe { ok(ctx, &self.union.ok.value) }
        } else {
            unsafe { err(ctx, &self.union.err.value) }
        }
    }
    /// Equivalent to `match &mut self`. If you need multiple branches to obtain mutable access or ownership
    /// of a local, use [`Self::match_mut_ctx`] instead.
    pub fn match_mut<'a, U, FnOk: FnOnce(&'a mut Ok) -> U, FnErr: FnOnce(&'a mut Err) -> U>(
        &'a mut self,
        ok: FnOk,
        err: FnErr,
    ) -> U {
        let r;
        let union = &mut self.union as *mut _ as *mut u8;
        if Self::is_ok(self) {
            unsafe {
                r = ok(&mut self.union.ok.deref_mut().value);
                self.discriminant = <Ok as IDeterminantProvider<Err>>::Determinant::ok(union);
            }
        } else {
            unsafe {
                r = err(&mut self.union.err.deref_mut().value);
                self.discriminant = <Ok as IDeterminantProvider<Err>>::Determinant::err(union);
            }
        }
        r
    }
    /// Equivalent to `match &mut self`.
    pub fn match_mut_ctx<
        'a,
        T,
        U,
        FnOk: FnOnce(T, &'a mut Ok) -> U,
        FnErr: FnOnce(T, &'a mut Err) -> U,
    >(
        &'a mut self,
        ctx: T,
        ok: FnOk,
        err: FnErr,
    ) -> U {
        let r;
        let union = &mut self.union as *mut _ as *mut u8;
        if Self::is_ok(self) {
            unsafe {
                r = ok(ctx, &mut self.union.ok.deref_mut().value);
                self.discriminant = <Ok as IDeterminantProvider<Err>>::Determinant::ok(union);
            }
        } else {
            unsafe {
                r = err(ctx, &mut self.union.err.deref_mut().value);
                self.discriminant = <Ok as IDeterminantProvider<Err>>::Determinant::err(union);
            }
        }
        r
    }
    /// Equivalent to `match self`. If you need multiple branches to obtain mutable access or ownership
    /// of a local, use [`Self::match_owned_ctx`] instead.
    pub fn match_owned<U, FnOk: FnOnce(Ok) -> U, FnErr: FnOnce(Err) -> U>(
        self,
        ok: FnOk,
        err: FnErr,
    ) -> U {
        let is_ok = self.is_ok();
        let union = self.union.clone();
        core::mem::forget(self);
        if is_ok {
            ok(core::mem::ManuallyDrop::into_inner(unsafe { union.ok }).value)
        } else {
            err(core::mem::ManuallyDrop::into_inner(unsafe { union.err }).value)
        }
    }
    /// Equivalent to `match self`.
    pub fn match_owned_ctx<U, T, FnOk: FnOnce(T, Ok) -> U, FnErr: FnOnce(T, Err) -> U>(
        self,
        ctx: T,
        ok: FnOk,
        err: FnErr,
    ) -> U {
        let is_ok = self.is_ok();
        let union = self.union.clone();
        core::mem::forget(self);
        if is_ok {
            ok(
                ctx,
                core::mem::ManuallyDrop::into_inner(unsafe { union.ok }).value,
            )
        } else {
            err(
                ctx,
                core::mem::ManuallyDrop::into_inner(unsafe { union.err }).value,
            )
        }
    }
    /// Returns `true` if in the `Ok` variant.
    pub fn is_ok(&self) -> bool {
        self.discriminant.is_ok(&self.union as *const _ as *const _)
    }
    /// Returns `true` if in the `Err` variant.
    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }
    /// Returns the `Ok` variant if it exists, `None` otherwise.
    pub fn ok(self) -> Option<Ok> {
        self.match_owned(Some, |_| None)
    }
    /// Returns the `Err` variant if it exists, `None` otherwise.
    pub fn err(self) -> Option<Err> {
        self.match_owned(|_| None, Some)
    }
    /// Returns the `Ok` variant by reference if it exists, `None` otherwise.
    pub fn ok_ref(&self) -> Option<&Ok> {
        self.match_ref(Some, |_| None)
    }
    /// Returns the `Err` variant by reference if it exists, `None` otherwise.
    pub fn err_ref(&self) -> Option<&Err> {
        self.match_ref(|_| None, Some)
    }
    /// Returns the `Ok` variant by mutable reference if it exists, `None` otherwise.
    pub fn ok_mut(&mut self) -> Option<&mut Ok> {
        self.match_mut(Some, |_| None)
    }
    /// Returns the `Err` variant by mutable reference if it exists, `None` otherwise.
    pub fn err_mut(&mut self) -> Option<&mut Err> {
        self.match_mut(|_| None, Some)
    }
    /// Applies a computation to the `Ok` variant.
    pub fn map<F: FnOnce(Ok) -> U, U>(self, f: F) -> Result<U, Err>
    where
        U: IDeterminantProvider<Err>,
    {
        self.match_owned(move |x| Result::Ok(f(x)), |x| Result::Err(x))
    }
    /// Applies a fallible computation to the `Err` variant.
    pub fn and_then<F: FnOnce(Ok) -> Result<U, Err>, U>(self, f: F) -> Result<U, Err>
    where
        U: IDeterminantProvider<Err>,
    {
        self.match_owned(f, |x| Result::Err(x))
    }
    /// Returns the `Ok` variant if applicable, calling `f` on the `Err` otherwise.
    pub fn unwrap_or_else<F: FnOnce(Err) -> Ok>(self, f: F) -> Ok {
        self.match_owned(|x| x, f)
    }
    /// # Safety
    /// Called on an `Err`, this triggers Undefined Behaviour.
    pub unsafe fn unwrap_unchecked(self) -> Ok {
        self.unwrap_or_else(|_| unsafe { unreachable_unchecked!() })
    }
    /// # Panics
    /// If `!self.is_ok()`
    pub fn unwrap(self) -> Ok
    where
        Err: core::fmt::Debug,
    {
        self.unwrap_or_else(|e| panic!("Result::unwrap called on Err variant: {e:?}"))
    }
    /// Returns the `Err` variant if applicable, calling `f` on the `Ok` otherwise.
    pub fn unwrap_err_or_else<F: FnOnce(Ok) -> Err>(self, f: F) -> Err {
        self.match_owned(f, |x| x)
    }
    /// # Safety
    /// Called on an `Ok`, this triggers Undefined Behaviour.
    pub unsafe fn unwrap_err_unchecked(self) -> Err {
        self.unwrap_err_or_else(|_| unsafe { unreachable_unchecked!() })
    }
    /// # Panics
    /// If `!self.is_err()`
    pub fn unwrap_err(self) -> Err
    where
        Ok: core::fmt::Debug,
    {
        self.unwrap_err_or_else(|e| panic!("Result::unwrap_err called on Ok variant: {e:?}"))
    }
}
