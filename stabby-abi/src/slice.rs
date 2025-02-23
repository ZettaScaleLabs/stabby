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

//! Stable slices!

use crate as stabby;
use core::ops::{Deref, DerefMut};

/// An ABI stable equivalent of `&'a [T]`
#[stabby::stabby]
pub struct Slice<'a, T: 'a> {
    /// The start of the slice.
    pub start: core::ptr::NonNull<T>,
    /// The length of the slice.
    pub len: usize,
    /// Ensures the slice has correct lifetime and variance.
    pub marker: core::marker::PhantomData<&'a ()>,
}
// SAFETY: Slices are analogous to references.
unsafe impl<'a, T: 'a> Send for Slice<'a, T> where &'a T: Send {}
// SAFETY: Slices are analogous to references.
unsafe impl<'a, T: 'a> Sync for Slice<'a, T> where &'a T: Sync {}
impl<'a, T: 'a> Clone for Slice<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<'a, T: 'a> Copy for Slice<'a, T> {}

impl<'a, T: 'a> Slice<'a, T> {
    /// Convert `&[T]` to its ABI-stable equivalent.
    pub const fn new(value: &'a [T]) -> Self {
        Self {
            start: unsafe { core::ptr::NonNull::new_unchecked(value.as_ptr() as *mut T) },
            len: value.len(),
            marker: core::marker::PhantomData,
        }
    }
    /// Obtain `&[T]` from its ABI-stable equivalent.
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
impl<T> Deref for Slice<'_, T> {
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
    /// The start of the slice.
    pub start: core::ptr::NonNull<T>,
    /// The length of the slice.
    pub len: usize,
    /// Ensures the slice has correct lifetime and variance.
    pub marker: core::marker::PhantomData<&'a mut ()>,
}
// SAFETY: SliceMut is analogous to a mutable reference
unsafe impl<'a, T: 'a> Send for SliceMut<'a, T> where &'a mut T: Send {}
// SAFETY: SliceMut is analogous to a mutable reference
unsafe impl<'a, T: 'a> Sync for SliceMut<'a, T> where &'a mut T: Sync {}
impl<T> Deref for SliceMut<'_, T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        unsafe { core::slice::from_raw_parts(self.start.as_ref(), self.len) }
    }
}
impl<T> DerefMut for SliceMut<'_, T> {
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

#[cfg(feature = "serde")]
mod serde_impl {
    use super::*;
    use serde::{de::Visitor, Deserialize, Serialize};
    impl<T: Serialize> Serialize for Slice<'_, T> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let slice: &[T] = self;
            slice.serialize(serializer)
        }
    }
    impl<T: Serialize> Serialize for SliceMut<'_, T> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let slice: &[T] = self;
            slice.serialize(serializer)
        }
    }
    impl<'a> Deserialize<'a> for Slice<'a, u8> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'a>,
        {
            deserializer.deserialize_bytes(BytesVisitor(core::marker::PhantomData))
        }
    }
    struct BytesVisitor<'a>(core::marker::PhantomData<Slice<'a, u8>>);
    impl<'a> Visitor<'a> for BytesVisitor<'a> {
        type Value = Slice<'a, u8>;
        fn visit_borrowed_bytes<E>(self, v: &'a [u8]) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v.into())
        }
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "A borrowed_str")
        }
    }
}
