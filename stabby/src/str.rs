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

use crate as stabby;

use core::ops::{Deref, DerefMut};

/// An ABI stable equivalent of `&'a T`
#[stabby::stabby]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Str<'a> {
    pub(crate) inner: crate::slice::Slice<'a, u8>,
}
impl<'a> From<&'a str> for Str<'a> {
    fn from(value: &'a str) -> Self {
        Self {
            inner: value.as_bytes().into(),
        }
    }
}
impl<'a> From<&'a mut str> for Str<'a> {
    fn from(value: &'a mut str) -> Self {
        Self::from(&*value)
    }
}
impl<'a> From<Str<'a>> for &'a str {
    fn from(value: Str<'a>) -> Self {
        unsafe { core::str::from_utf8_unchecked(value.inner.into()) }
    }
}
impl<'a> Deref for Str<'a> {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        unsafe { core::str::from_utf8_unchecked(&self.inner) }
    }
}
impl core::fmt::Debug for Str<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.deref().fmt(f)
    }
}
impl core::fmt::Display for Str<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.deref().fmt(f)
    }
}
impl core::cmp::PartialOrd for Str<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.deref().partial_cmp(other.deref())
    }
}

/// An ABI stable equivalent of `&'a mut T`
#[stabby::stabby]
pub struct StrMut<'a> {
    pub(crate) inner: crate::slice::SliceMut<'a, u8>,
}
impl<'a> Deref for StrMut<'a> {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        unsafe { core::str::from_utf8_unchecked(&self.inner) }
    }
}
impl<'a> DerefMut for StrMut<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { core::str::from_utf8_unchecked_mut(&mut self.inner) }
    }
}
impl<'a> From<&'a mut str> for StrMut<'a> {
    fn from(value: &'a mut str) -> Self {
        Self {
            inner: unsafe { value.as_bytes_mut().into() },
        }
    }
}
impl<'a> From<StrMut<'a>> for Str<'a> {
    fn from(value: StrMut<'a>) -> Self {
        Self {
            inner: value.inner.into(),
        }
    }
}
impl<'a> From<StrMut<'a>> for &'a mut str {
    fn from(value: StrMut<'a>) -> Self {
        unsafe { core::str::from_utf8_unchecked_mut(value.inner.into()) }
    }
}

impl<'a> From<StrMut<'a>> for &'a str {
    fn from(value: StrMut<'a>) -> Self {
        unsafe { core::str::from_utf8_unchecked(value.inner.into()) }
    }
}
