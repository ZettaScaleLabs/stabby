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

//! Stable boxed slices and strings!

use alloc::sync::{Arc, Weak};
use stabby_abi::AccessAs;

use crate::{self as stabby};
use core::{
    fmt::{Debug, Display},
    hash::Hash,
    ops::Deref,
};

#[stabby::stabby]
#[derive(Eq)]
pub struct ArcSlice<T> {
    start: &'static (),
    len: usize,
    marker: core::marker::PhantomData<Box<T>>,
}
impl<T: Debug> Debug for ArcSlice<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.deref().fmt(f)
    }
}
impl<T: PartialEq> PartialEq for ArcSlice<T> {
    fn eq(&self, other: &Self) -> bool {
        self.deref() == other.deref()
    }
}
impl<T: Hash> Hash for ArcSlice<T> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.deref().hash(state);
    }
}
impl<T: Clone> Clone for ArcSlice<T> {
    fn clone(&self) -> Self {
        unsafe {
            Arc::increment_strong_count(self.start as *const () as *const T);
            core::ptr::read(self)
        }
    }
}
impl<T> Deref for ArcSlice<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        unsafe { core::slice::from_raw_parts(self.start as *const () as *const T, self.len) }
    }
}
impl<T> From<Box<[T]>> for ArcSlice<T> {
    fn from(value: Box<[T]>) -> Self {
        Self::from(<Arc<[T]>>::from(value))
    }
}
impl<T> From<Arc<[T]>> for ArcSlice<T> {
    fn from(value: Arc<[T]>) -> Self {
        let len = value.len();
        let value = Arc::into_raw(value);
        let start = unsafe { &*(value as *const T as *const ()) };
        Self {
            start,
            len,
            marker: core::marker::PhantomData,
        }
    }
}
impl<T> From<ArcSlice<T>> for Arc<[T]> {
    fn from(value: ArcSlice<T>) -> Self {
        let result = unsafe {
            Arc::from_raw(core::ptr::slice_from_raw_parts(
                value.start as *const () as *const T,
                value.len,
            ))
        };
        core::mem::forget(value);
        result
    }
}
impl<T> Drop for ArcSlice<T> {
    fn drop(&mut self) {
        unsafe { Arc::from_raw(core::ptr::slice_from_raw_parts(self.start, self.len)) };
    }
}

#[stabby::stabby]
pub struct WeakSlice<T> {
    start: &'static (),
    len: usize,
    marker: core::marker::PhantomData<Box<T>>,
}
impl<T: Clone> Clone for WeakSlice<T> {
    fn clone(&self) -> Self {
        self.ref_as::<Weak<[T]>>().clone().into()
    }
}
impl<T> Deref for WeakSlice<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        unsafe { core::slice::from_raw_parts(self.start as *const () as *const T, self.len) }
    }
}
impl<T> From<Weak<[T]>> for WeakSlice<T> {
    fn from(value: Weak<[T]>) -> Self {
        let len = match value.upgrade() {
            Some(value) => value.len(),
            None => 0,
        };
        let value = Weak::into_raw(value);
        let start = unsafe { &*(value as *const T as *const ()) };
        Self {
            start,
            len,
            marker: core::marker::PhantomData,
        }
    }
}
impl<T> From<WeakSlice<T>> for Weak<[T]> {
    fn from(value: WeakSlice<T>) -> Self {
        let result = unsafe {
            Weak::from_raw(core::ptr::slice_from_raw_parts(
                value.start as *const () as *const T,
                value.len,
            ))
        };
        core::mem::forget(value);
        result
    }
}
impl<T> Drop for WeakSlice<T> {
    fn drop(&mut self) {
        unsafe { Weak::from_raw(core::ptr::slice_from_raw_parts(self.start, self.len)) };
    }
}

#[stabby::stabby]
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ArcStr {
    inner: ArcSlice<u8>,
}
impl Debug for ArcStr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(self.deref(), f)
    }
}
impl Display for ArcStr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(self.deref(), f)
    }
}
impl Deref for ArcStr {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        unsafe { core::str::from_utf8_unchecked(&self.inner) }
    }
}
impl From<Arc<str>> for ArcStr {
    fn from(value: Arc<str>) -> Self {
        let value: Arc<[u8]> = value.into();
        Self {
            inner: value.into(),
        }
    }
}
impl From<Box<str>> for ArcStr {
    fn from(value: Box<str>) -> Self {
        let value: Arc<[u8]> = value.into_boxed_bytes().into();
        Self {
            inner: value.into(),
        }
    }
}
impl From<ArcStr> for Arc<str> {
    fn from(value: ArcStr) -> Self {
        unsafe { core::mem::transmute(Arc::<[u8]>::from(value.inner)) }
    }
}
