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

//! Stable results!

use core::ops::DerefMut;

use crate::enums::IDiscriminantProvider;
pub use crate::enums::{IDiscriminant, IDiscriminantProviderInner};
use crate::padding::Padded;
use crate::Union;
use crate::{self as stabby, IStable};

#[stabby::stabby]
pub struct Result<Ok, Err>
where
    Ok: IDiscriminantProvider<Err>,
    Err: IStable,
{
    niche_exporter: <Ok as IDiscriminantProvider<Err>>::NicheExporter,
    discriminant: <Ok as IDiscriminantProvider<Err>>::Discriminant,
    #[allow(clippy::type_complexity)]
    union: Union<
        Padded<<Ok as IDiscriminantProvider<Err>>::OkShift, Ok>,
        Padded<<Ok as IDiscriminantProvider<Err>>::ErrShift, Err>,
    >,
}

impl<Ok: Clone, Err: Clone> Clone for Result<Ok, Err>
where
    Ok: IDiscriminantProvider<Err>,
    Err: IStable,
{
    fn clone(&self) -> Self {
        self.match_ref(|ok| Self::Ok(ok.clone()), |err| Self::Err(err.clone()))
    }
}
impl<Ok, Err> core::fmt::Debug for Result<Ok, Err>
where
    Ok: IDiscriminantProvider<Err>,
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
    Ok: IDiscriminantProvider<Err>,
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
    Ok: IDiscriminantProvider<Err>,
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
    Ok: IDiscriminantProvider<Err>,
    Err: IStable,
    Ok: core::cmp::Eq,
    Err: core::cmp::Eq,
{
}
impl<Ok, Err> From<core::result::Result<Ok, Err>> for Result<Ok, Err>
where
    Ok: IDiscriminantProvider<Err>,
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
    Ok: IDiscriminantProvider<Err>,
    Err: IStable,
{
    fn from(value: Result<Ok, Err>) -> Self {
        value.match_owned(Ok, Err)
    }
}
impl<Ok, Err> Drop for Result<Ok, Err>
where
    Ok: IDiscriminantProvider<Err>,
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
    Ok: IDiscriminantProvider<Err>,
    Err: IStable,
{
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
                <Ok as IDiscriminantProvider<Err>>::Discriminant::ok(&mut union as *mut _ as *mut _)
            },
            union,
        }
    }
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
                <Ok as IDiscriminantProvider<Err>>::Discriminant::err(
                    &mut union as *mut _ as *mut _,
                )
            },
            union,
        }
    }
    pub fn as_ref(&self) -> core::result::Result<&Ok, &Err> {
        self.match_ref(Ok, Err)
    }
    pub fn as_mut(&mut self) -> core::result::Result<&mut Ok, &mut Err> {
        self.match_mut(Ok, Err)
    }
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
    pub fn match_mut<'a, U, FnOk: FnOnce(&'a mut Ok) -> U, FnErr: FnOnce(&'a mut Err) -> U>(
        &'a mut self,
        ok: FnOk,
        err: FnErr,
    ) -> U {
        let r;
        let union = &mut self.union as *mut _ as *mut u8;
        if self.is_ok() {
            unsafe {
                r = ok(&mut self.union.ok.deref_mut().value);
                self.discriminant = <Ok as IDiscriminantProvider<Err>>::Discriminant::ok(union);
            }
        } else {
            unsafe {
                r = err(&mut self.union.err.deref_mut().value);
                self.discriminant = <Ok as IDiscriminantProvider<Err>>::Discriminant::err(union);
            }
        }
        r
    }
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
    pub fn is_ok(&self) -> bool {
        self.discriminant.is_ok(&self.union as *const _ as *const _)
    }
    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }
    pub fn ok(self) -> Option<Ok> {
        self.match_owned(Some, |_| None)
    }
    pub fn err(self) -> Option<Err> {
        self.match_owned(|_| None, Some)
    }
    pub fn ok_ref(&self) -> Option<&Ok> {
        self.match_ref(Some, |_| None)
    }
    pub fn err_ref(&self) -> Option<&Err> {
        self.match_ref(|_| None, Some)
    }
    pub fn ok_mut(&mut self) -> Option<&mut Ok> {
        self.match_mut(Some, |_| None)
    }
    pub fn err_mut(&mut self) -> Option<&mut Err> {
        self.match_mut(|_| None, Some)
    }
    pub fn map<F: FnOnce(Ok) -> U, U>(self, f: F) -> Result<U, Err>
    where
        U: IDiscriminantProvider<Err>,
    {
        self.match_owned(move |x| Result::Ok(f(x)), |x| Result::Err(x))
    }
    pub fn and_then<F: FnOnce(Ok) -> Result<U, Err>, U>(self, f: F) -> Result<U, Err>
    where
        U: IDiscriminantProvider<Err>,
    {
        self.match_owned(f, |x| Result::Err(x))
    }
    pub fn unwrap_or_else<F: FnOnce(Err) -> Ok>(self, f: F) -> Ok {
        self.match_owned(|x| x, f)
    }
    /// # Safety
    /// May trigger Undefined Behaviour if called on an Err variant.
    pub unsafe fn unwrap_unchecked(self) -> Ok {
        self.unwrap_or_else(|_| core::hint::unreachable_unchecked())
    }
    pub fn unwrap(self) -> Ok
    where
        Err: core::fmt::Debug,
    {
        self.unwrap_or_else(|e| panic!("Result::unwrap called on Err variant: {e:?}"))
    }
    pub fn unwrap_err_or_else<F: FnOnce(Ok) -> Err>(self, f: F) -> Err {
        self.match_owned(f, |x| x)
    }
    /// # Safety
    /// May trigger Undefined Behaviour if called on an Ok variant.
    pub unsafe fn unwrap_err_unchecked(self) -> Err {
        self.unwrap_err_or_else(|_| core::hint::unreachable_unchecked())
    }
    pub fn unwrap_err(self) -> Err
    where
        Ok: core::fmt::Debug,
    {
        self.unwrap_err_or_else(|e| panic!("Result::unwrap_err called on Ok variant: {e:?}"))
    }
}
