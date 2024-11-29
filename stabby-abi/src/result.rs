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

use stabby_macros::tyeval;

pub use crate::enums::IDeterminant;
use crate::enums::IDeterminantProvider;
use crate::istable::IBitMask;
use crate::report::FieldReport;
use crate::str::Str;
use crate::unsigned::IUnsignedBase;
use crate::{self as stabby, unreachable_unchecked, Bit, IStable, B0};
use crate::{Alignment, Tuple, Unsigned};

#[repr(transparent)]
/// An ABI-stable, niche optimizing equivalent of [`core::result::Result`]
pub struct Result<Ok, Err>
where
    Ok: IDeterminantProvider<Err>,
    Err: IStable,
{
    storage: Storage<<Self as IStable>::Size, <Self as IStable>::Align>,
}
impl<Ok: Unpin, Err: Unpin> Unpin for Result<Ok, Err>
where
    Ok: IDeterminantProvider<Err>,
    Err: IStable,
{
}
type Determinant<Ok, Err> = <Ok as IDeterminantProvider<Err>>::Determinant;
// SAFETY: See fields
unsafe impl<Ok, Err> IStable for Result<Ok, Err>
where
    Ok: IDeterminantProvider<Err>,
    Err: IStable,
{
    /// The size is the max of the variants' sizes, plus the size of the determinant upgraded to a multiple of the alignment.
    type Size = tyeval!(<<Determinant<Ok, Err> as IStable>::Size as Unsigned>::NextMultipleOf<Self::Align> + <Ok::Size as Unsigned>::Max<Err::Size>);
    /// The alignment is the max of the variants' alignments.
    type Align = <Ok::Align as Alignment>::Max<Err::Align>;
    // If either variant may contain an indirection, so may their sum-type.
    type ContainsIndirections = <Ok::ContainsIndirections as Bit>::Or<Err::ContainsIndirections>;
    // We trust the DeterminantProvider with this computation, it usually just discards the values for safety.
    type ForbiddenValues =
        <<Ok as IDeterminantProvider<Err>>::NicheExporter as IStable>::ForbiddenValues;
    // OH NO! But we have lots of testing.
    type UnusedBits = <<Tuple<Determinant<Ok, Err>, <Self::Align as Alignment>::AsUint> as IStable>::UnusedBits as IBitMask>::BitOr<<<<Ok as IDeterminantProvider<Err>>::NicheExporter as IStable>::UnusedBits as IBitMask>::Shift<<<Determinant<Ok, Err> as IStable>::Size as Unsigned>::NextMultipleOf<Self::Align>>>;
    // Rust doesn't know `stabby` Results may have niches.
    type HasExactlyOneNiche = B0;
    #[cfg(feature = "experimental-ctypes")]
    type CType = <Storage<<Self as IStable>::Size, <Self as IStable>::Align> as IStable>::CType;
    const REPORT: &'static crate::report::TypeReport = &crate::report::TypeReport {
        name: Str::new("Result"),
        module: Str::new("stabby_abi::result"),
        tyty: crate::report::TyTy::Enum(Str::new("stabby")),
        version: 1,
        fields: crate::StableLike::new(Some(&FieldReport {
            name: Str::new("Ok"),
            ty: Ok::REPORT,
            next_field: crate::StableLike::new(Some(&FieldReport {
                name: Str::new("Err"),
                ty: Err::REPORT,
                next_field: crate::StableLike::new(None),
            })),
        })),
    };
    const ID: u64 = crate::report::gen_id(Self::REPORT);
}
use seal::Storage;
mod seal {
    use super::*;
    #[stabby::stabby]
    pub struct Storage<Size: Unsigned, Align: Alignment + Alignment> {
        inner: <Align::Divide<Size> as IUnsignedBase>::Array<Align::AsUint>,
    }
}

impl<Size: Unsigned, Align: Alignment + Alignment> Storage<Size, Align> {
    const fn as_ptr(&self) -> *const u8 {
        self as *const Self as *const _
    }
    fn as_mut_ptr(&mut self) -> *mut u8 {
        self as *mut Self as *mut _
    }
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
            unsafe { self.ok_unchecked() }.hash(state);
        } else {
            false.hash(state);
            unsafe { self.err_unchecked() }.hash(state);
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
            (true, true) => unsafe { self.ok_unchecked().eq(other.ok_unchecked()) },
            (false, false) => unsafe { self.err_unchecked().eq(other.err_unchecked()) },
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
        unsafe {
            self.match_mut(
                |mut ok| core::ptr::drop_in_place::<Ok>(&mut *ok),
                |mut err| core::ptr::drop_in_place::<Err>(&mut *err),
            )
        }
    }
}
impl<Ok, Err> Result<Ok, Err>
where
    Ok: IDeterminantProvider<Err>,
    Err: IStable,
{
    const DET_SIZE: usize = <<<Determinant<Ok, Err> as IStable>::Size as Unsigned>::NextMultipleOf<
        <Self as IStable>::Align,
    > as Unsigned>::USIZE;
    const OK_OFFSET: usize =
        <<Ok as IDeterminantProvider<Err>>::OkShift as Unsigned>::USIZE + Self::DET_SIZE;
    const ERR_OFFSET: usize =
        <<Ok as IDeterminantProvider<Err>>::ErrShift as Unsigned>::USIZE + Self::DET_SIZE;
    const fn ok_ptr(
        storage: *const Storage<<Self as IStable>::Size, <Self as IStable>::Align>,
    ) -> *const Ok {
        unsafe { storage.cast::<u8>().add(Self::OK_OFFSET).cast() }
    }
    const fn ok_ptr_mut(
        storage: *mut Storage<<Self as IStable>::Size, <Self as IStable>::Align>,
    ) -> *mut Ok {
        unsafe { storage.cast::<u8>().add(Self::OK_OFFSET).cast() }
    }
    const fn err_ptr(
        storage: *const Storage<<Self as IStable>::Size, <Self as IStable>::Align>,
    ) -> *const Err {
        unsafe { storage.cast::<u8>().add(Self::ERR_OFFSET).cast() }
    }
    const fn err_ptr_mut(
        storage: *mut Storage<<Self as IStable>::Size, <Self as IStable>::Align>,
    ) -> *mut Err {
        unsafe { storage.cast::<u8>().add(Self::ERR_OFFSET).cast() }
    }
    const fn det_ptr(
        storage: *const Storage<<Self as IStable>::Size, <Self as IStable>::Align>,
    ) -> *const Determinant<Ok, Err> {
        storage.cast()
    }
    const fn det_ptr_mut(
        storage: *mut Storage<<Self as IStable>::Size, <Self as IStable>::Align>,
    ) -> *mut Determinant<Ok, Err> {
        storage.cast()
    }
    /// Construct the `Ok` variant.
    #[allow(non_snake_case)]
    pub fn Ok(value: Ok) -> Self {
        let mut storage = core::mem::MaybeUninit::zeroed();
        unsafe {
            let storage_ptr = storage.as_mut_ptr();
            Self::ok_ptr_mut(storage_ptr).write(value);
            Self::det_ptr_mut(storage_ptr).write(Determinant::<Ok, Err>::ok(storage_ptr.cast()));
            Self {
                storage: storage.assume_init(),
            }
        }
    }
    /// Construct the `Err` variant.
    #[allow(non_snake_case)]
    pub fn Err(value: Err) -> Self {
        let mut storage = core::mem::MaybeUninit::zeroed();
        unsafe {
            let storage_ptr = storage.as_mut_ptr();
            Self::err_ptr_mut(storage_ptr).write(value);
            Self::det_ptr_mut(storage_ptr).write(Determinant::<Ok, Err>::err(storage_ptr.cast()));
            Self {
                storage: storage.assume_init(),
            }
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
            unsafe { ok(self.ok_unchecked()) }
        } else {
            unsafe { err(self.err_unchecked()) }
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
            unsafe { ok(ctx, self.ok_unchecked()) }
        } else {
            unsafe { err(ctx, self.err_unchecked()) }
        }
    }
    /// Equivalent to `match &mut self`. If you need multiple branches to obtain mutable access or ownership
    /// of a local, use [`Self::match_mut_ctx`] instead.
    pub fn match_mut<
        'a,
        U,
        FnOk: FnOnce(OkGuard<'a, Ok, Err>) -> U,
        FnErr: FnOnce(ErrGuard<'a, Ok, Err>) -> U,
    >(
        &'a mut self,
        ok: FnOk,
        err: FnErr,
    ) -> U {
        let r;
        if Self::is_ok(self) {
            unsafe {
                r = ok(self.ok_mut_unchecked());
            }
        } else {
            unsafe {
                r = err(self.err_mut_unchecked());
            }
        }
        r
    }
    /// Equivalent to `match &mut self`.
    pub fn match_mut_ctx<
        'a,
        T,
        U,
        FnOk: FnOnce(T, OkGuard<'a, Ok, Err>) -> U,
        FnErr: FnOnce(T, ErrGuard<'_, Ok, Err>) -> U,
    >(
        &'a mut self,
        ctx: T,
        ok: FnOk,
        err: FnErr,
    ) -> U {
        let r;
        if Self::is_ok(self) {
            unsafe {
                r = ok(ctx, self.ok_mut_unchecked());
            }
        } else {
            unsafe {
                r = err(ctx, self.err_mut_unchecked());
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
        let storage = &self.storage;
        if is_ok {
            let t = unsafe { core::ptr::read(Self::ok_ptr(storage)) };
            core::mem::forget(self);
            ok(t)
        } else {
            let t = unsafe { core::ptr::read(Self::err_ptr(storage)) };
            core::mem::forget(self);
            err(t)
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
        let storage = &self.storage;
        if is_ok {
            let t = unsafe { core::ptr::read(Self::ok_ptr(storage)) };
            core::mem::forget(self);
            ok(ctx, t)
        } else {
            let t = unsafe { core::ptr::read(Self::err_ptr(storage)) };
            core::mem::forget(self);
            err(ctx, t)
        }
    }
    /// Returns `true` if in the `Ok` variant.
    pub fn is_ok(&self) -> bool {
        unsafe { &*Self::det_ptr(&self.storage) }.is_det_ok(self.storage.as_ptr())
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
    pub fn ok_mut(&mut self) -> Option<OkGuard<'_, Ok, Err>> {
        self.match_mut(Some, |_| None)
    }
    /// Returns the `Err` variant by mutable reference if it exists, `None` otherwise.
    pub fn err_mut(&mut self) -> Option<ErrGuard<'_, Ok, Err>> {
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
    const unsafe fn ok_unchecked(&self) -> &Ok {
        &*Self::ok_ptr(&self.storage)
    }
    const unsafe fn err_unchecked(&self) -> &Err {
        &*Self::err_ptr(&self.storage)
    }
    unsafe fn ok_mut_unchecked(&mut self) -> OkGuard<'_, Ok, Err> {
        OkGuard { inner: self }
    }
    unsafe fn err_mut_unchecked(&mut self) -> ErrGuard<Ok, Err> {
        ErrGuard { inner: self }
    }
}

/// A guard that ensures that niche determinants are reinserted if the `Ok` variant of an [`Result`] is re-established after it may have been mutated.
///
/// When dropped, this guard ensures that the result's determinant is properly set.
/// Failing to drop this guard may result in undefined behaviour.
pub struct OkGuard<'a, Ok, Err>
where
    Ok: IDeterminantProvider<Err>,
    Err: IStable,
{
    inner: &'a mut Result<Ok, Err>,
}
impl<Ok, Err> core::ops::Deref for OkGuard<'_, Ok, Err>
where
    Ok: IDeterminantProvider<Err>,
    Err: IStable,
{
    type Target = Ok;
    fn deref(&self) -> &Self::Target {
        unsafe { self.inner.ok_unchecked() }
    }
}
impl<Ok, Err> core::ops::DerefMut for OkGuard<'_, Ok, Err>
where
    Ok: IDeterminantProvider<Err>,
    Err: IStable,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *Result::<Ok, Err>::ok_ptr_mut(&mut self.inner.storage) }
    }
}
impl<Ok, Err> Drop for OkGuard<'_, Ok, Err>
where
    Ok: IDeterminantProvider<Err>,
    Err: IStable,
{
    fn drop(&mut self) {
        if <<Determinant<Ok, Err> as IDeterminant>::IsNicheTrick as Bit>::BOOL {
            unsafe { Determinant::<Ok, Err>::ok(self.inner.storage.as_mut_ptr()) };
        }
    }
}

/// A guard that ensures that niche determinants are reinserted if the `Err` variant of an [`Option`] is re-established after it may have been mutated.
///
/// When dropped, this guard ensures that the result's determinant is properly set.
/// Failing to drop this guard may result in undefined behaviour.
pub struct ErrGuard<'a, Ok, Err>
where
    Ok: IDeterminantProvider<Err>,
    Err: IStable,
{
    inner: &'a mut Result<Ok, Err>,
}

impl<Ok, Err> core::ops::Deref for ErrGuard<'_, Ok, Err>
where
    Ok: IDeterminantProvider<Err>,
    Err: IStable,
{
    type Target = Err;
    fn deref(&self) -> &Self::Target {
        unsafe { self.inner.err_unchecked() }
    }
}
impl<Ok, Err> core::ops::DerefMut for ErrGuard<'_, Ok, Err>
where
    Ok: IDeterminantProvider<Err>,
    Err: IStable,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *Result::<Ok, Err>::err_ptr_mut(&mut self.inner.storage) }
    }
}
impl<Ok, Err> Drop for ErrGuard<'_, Ok, Err>
where
    Ok: IDeterminantProvider<Err>,
    Err: IStable,
{
    fn drop(&mut self) {
        if <<Determinant<Ok, Err> as IDeterminant>::IsNicheTrick as Bit>::BOOL {
            unsafe { Determinant::<Ok, Err>::err(self.inner.storage.as_mut_ptr()) };
        }
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use super::*;
    use serde::{Deserialize, Serialize};
    impl<Ok: Serialize, Err: Serialize> Serialize for Result<Ok, Err>
    where
        Ok: IDeterminantProvider<Err>,
        Err: IStable,
    {
        fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let this: core::result::Result<_, _> = self.as_ref();
            this.serialize(serializer)
        }
    }
    impl<'a, Ok: IDeterminantProvider<Err>, Err: IStable> Deserialize<'a> for Result<Ok, Err>
    where
        core::result::Result<Ok, Err>: Deserialize<'a>,
    {
        fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
        where
            D: serde::Deserializer<'a>,
        {
            Ok(core::result::Result::<Ok, Err>::deserialize(deserializer)?.into())
        }
    }
}
