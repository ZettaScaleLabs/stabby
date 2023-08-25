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

//! Stable boxed slices and strings!

use crate::slice::SliceMut;
use crate::str::StrMut;
use crate::{self as stabby};
use core::fmt::{Debug, Display};
use core::hash::Hash;
use core::ops::{Deref, DerefMut};

#[stabby::stabby]
#[derive(Eq)]
pub struct BoxedSlice<T> {
    start: &'static mut (),
    len: usize,
    marker: core::marker::PhantomData<Box<T>>,
}
impl<T> BoxedSlice<T> {
    pub(crate) fn leak<'a>(self) -> SliceMut<'a, T> {
        let r = SliceMut {
            start: unsafe { core::mem::transmute_copy(&self.start) },
            len: self.len,
            marker: core::marker::PhantomData,
        };
        core::mem::forget(self);
        r
    }
}
impl<T: Debug> Debug for BoxedSlice<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.deref().fmt(f)
    }
}
impl<T: PartialEq> PartialEq for BoxedSlice<T> {
    fn eq(&self, other: &Self) -> bool {
        self.deref() == other.deref()
    }
}
impl<T: Hash> Hash for BoxedSlice<T> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.deref().hash(state);
    }
}
impl<T: Clone> Clone for BoxedSlice<T> {
    fn clone(&self) -> Self {
        Box::from_iter(self.iter().cloned()).into()
    }
}
impl<T> Deref for BoxedSlice<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        unsafe { core::slice::from_raw_parts(self.start as *const () as *const T, self.len) }
    }
}
impl<T> DerefMut for BoxedSlice<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { core::slice::from_raw_parts_mut(self.start as *mut () as *mut T, self.len) }
    }
}
impl<T> From<Box<[T]>> for BoxedSlice<T> {
    fn from(value: Box<[T]>) -> Self {
        let value = Box::leak(value);
        let len = value.len();
        let start = unsafe { &mut *(value.as_ptr() as *mut ()) };
        Self {
            start,
            len,
            marker: core::marker::PhantomData,
        }
    }
}
impl<T> From<BoxedSlice<T>> for Box<[T]> {
    fn from(value: BoxedSlice<T>) -> Self {
        unsafe { Box::from_raw(&mut *value.leak()) }
    }
}
impl<T> Drop for BoxedSlice<T> {
    fn drop(&mut self) {
        unsafe { _ = Box::from_raw(self.deref_mut()) };
    }
}

#[stabby::stabby]
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct BoxedStr {
    inner: BoxedSlice<u8>,
}
impl BoxedStr {
    pub(crate) fn leak(self) -> StrMut<'static> {
        let slice = unsafe { core::str::from_utf8_unchecked_mut(self.inner.leak().into()) };
        StrMut::from(slice)
    }
}
impl Deref for BoxedStr {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        unsafe { core::str::from_utf8_unchecked(&self.inner) }
    }
}
impl DerefMut for BoxedStr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { core::str::from_utf8_unchecked_mut(&mut self.inner) }
    }
}
impl From<Box<str>> for BoxedStr {
    fn from(value: Box<str>) -> Self {
        let value: Box<[u8]> = value.into();
        Self {
            inner: value.into(),
        }
    }
}
impl From<BoxedStr> for Box<str> {
    fn from(value: BoxedStr) -> Self {
        unsafe { Box::from_raw(&mut *value.leak()) }
    }
}
impl Debug for BoxedStr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(self.deref(), f)
    }
}
impl Display for BoxedStr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(self.deref(), f)
    }
}
