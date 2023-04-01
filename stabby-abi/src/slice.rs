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

//! Stable slices!

use crate as stabby;
use core::ops::{Deref, DerefMut};

/// An ABI stable equivalent of `&'a [T]`
#[stabby::stabby]
pub struct Slice<'a, T: 'a> {
    pub start: core::ptr::NonNull<T>,
    pub len: usize,
    pub marker: core::marker::PhantomData<&'a ()>,
}
unsafe impl<'a, T: 'a + Sync> Send for Slice<'a, T> {}
unsafe impl<'a, T: 'a + Sync> Sync for Slice<'a, T> {}
impl<'a, T: 'a> Clone for Slice<'a, T> {
    fn clone(&self) -> Self {
        unsafe { core::ptr::read(self) }
    }
}
impl<'a, T: 'a> Copy for Slice<'a, T> {}

impl<'a, T: 'a> Slice<'a, T> {
    pub const fn new(value: &'a [T]) -> Self {
        Self {
            start: unsafe { core::ptr::NonNull::new_unchecked(value.as_ptr() as *mut T) },
            len: value.len(),
            marker: core::marker::PhantomData,
        }
    }
    pub const fn as_slice(self) -> &'a [T] {
        unsafe { core::slice::from_raw_parts(self.start.as_ptr(), self.len) }
    }
}
impl<'a, T> From<&'a [T]> for Slice<'a, T> {
    fn from(value: &'a [T]) -> Self {
        Self::new(value)
    }
}
impl<'a, T> From<&'a mut [T]> for Slice<'a, T> {
    fn from(value: &'a mut [T]) -> Self {
        Self {
            start: unsafe { core::ptr::NonNull::new_unchecked(value.as_ptr() as *mut T) },
            len: value.len(),
            marker: core::marker::PhantomData,
        }
    }
}

impl<'a, T> From<Slice<'a, T>> for &'a [T] {
    fn from(value: Slice<'a, T>) -> Self {
        unsafe { core::slice::from_raw_parts(value.start.as_ref(), value.len) }
    }
}
impl<'a, T> Deref for Slice<'a, T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        unsafe { core::slice::from_raw_parts(self.start.as_ref(), self.len) }
    }
}
impl<'a, T: 'a> Eq for Slice<'a, T> where for<'b> &'b [T]: Eq {}
impl<'a, T: 'a> PartialEq for Slice<'a, T>
where
    for<'b> &'b [T]: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.deref() == other.deref()
    }
}
impl<'a, T: 'a> core::fmt::Debug for Slice<'a, T>
where
    for<'b> &'b [T]: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.deref().fmt(f)
    }
}
impl<'a, T: 'a> core::fmt::Display for Slice<'a, T>
where
    for<'b> &'b [T]: core::fmt::Display,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.deref().fmt(f)
    }
}
impl<'a, T: 'a> core::hash::Hash for Slice<'a, T>
where
    for<'b> &'b [T]: core::hash::Hash,
{
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.deref().hash(state);
    }
}

/// An ABI stable equivalent of `&'a mut T`
#[stabby::stabby]
pub struct SliceMut<'a, T: 'a> {
    pub start: core::ptr::NonNull<T>,
    pub len: usize,
    pub marker: core::marker::PhantomData<&'a mut ()>,
}
unsafe impl<'a, T: 'a + Sync> Send for SliceMut<'a, T> {}
unsafe impl<'a, T: 'a + Sync> Sync for SliceMut<'a, T> {}
impl<'a, T> Deref for SliceMut<'a, T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        unsafe { core::slice::from_raw_parts(self.start.as_ref(), self.len) }
    }
}
impl<'a, T> DerefMut for SliceMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { core::slice::from_raw_parts_mut(self.start.as_mut(), self.len) }
    }
}
impl<'a, T: 'a> Eq for SliceMut<'a, T> where for<'b> &'b [T]: Eq {}
impl<'a, T: 'a> PartialEq for SliceMut<'a, T>
where
    for<'b> &'b [T]: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.deref() == other.deref()
    }
}
impl<'a, T: 'a> core::fmt::Debug for SliceMut<'a, T>
where
    for<'b> &'b [T]: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.deref().fmt(f)
    }
}
impl<'a, T: 'a> core::fmt::Display for SliceMut<'a, T>
where
    for<'b> &'b [T]: core::fmt::Display,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.deref().fmt(f)
    }
}
impl<'a, T: 'a> core::hash::Hash for SliceMut<'a, T>
where
    for<'b> &'b [T]: core::hash::Hash,
{
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.deref().hash(state);
    }
}
impl<'a, T> From<&'a mut [T]> for SliceMut<'a, T> {
    fn from(value: &'a mut [T]) -> Self {
        Self {
            start: unsafe { core::ptr::NonNull::new_unchecked(value.as_mut_ptr()) },
            len: value.len(),
            marker: core::marker::PhantomData,
        }
    }
}
impl<'a, T> From<SliceMut<'a, T>> for Slice<'a, T> {
    fn from(value: SliceMut<'a, T>) -> Self {
        Self {
            start: value.start,
            len: value.len,
            marker: core::marker::PhantomData,
        }
    }
}
impl<'a, T> From<SliceMut<'a, T>> for &'a mut [T] {
    fn from(mut value: SliceMut<'a, T>) -> Self {
        unsafe { core::slice::from_raw_parts_mut(value.start.as_mut(), value.len) }
    }
}

impl<'a, T> From<SliceMut<'a, T>> for &'a [T] {
    fn from(mut value: SliceMut<'a, T>) -> Self {
        unsafe { core::slice::from_raw_parts(value.start.as_mut(), value.len) }
    }
}
