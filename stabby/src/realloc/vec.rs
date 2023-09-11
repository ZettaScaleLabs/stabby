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

use core::ptr::NonNull;

use super::{AllocPtr, AllocSlice, IAlloc};

#[crate::stabby]
pub struct VecInner<T, Alloc: IAlloc> {
    pub(crate) start: AllocPtr<T, Alloc>,
    pub(crate) end: NonNull<T>,
    pub(crate) capacity: NonNull<T>,
    pub(crate) alloc: Alloc,
}

#[cfg(feature = "libc")]
#[crate::stabby]
pub struct Vec<T, Alloc: IAlloc = crate::realloc::libc_alloc::LibcAlloc>(
    pub(crate) VecInner<T, Alloc>,
);
#[cfg(not(feature = "libc"))]
#[crate::stabby]
pub struct Vec<T, Alloc: IAlloc>(pub(crate) VecInner<T, Alloc>);

pub(crate) const fn ptr_diff<T>(lhs: NonNull<T>, rhs: NonNull<T>) -> usize {
    let diff = if core::mem::size_of::<T>() == 0 {
        unsafe { lhs.as_ptr().cast::<u8>().offset_from(rhs.as_ptr().cast()) }
    } else {
        unsafe { lhs.as_ptr().offset_from(rhs.as_ptr()) }
    };
    debug_assert!(diff >= 0);
    diff as usize
}
pub(crate) const fn ptr_add<T>(lhs: NonNull<T>, rhs: usize) -> NonNull<T> {
    if core::mem::size_of::<T>() == 0 {
        unsafe { NonNull::new_unchecked(lhs.as_ptr().cast::<u8>().add(rhs)).cast() }
    } else {
        unsafe { NonNull::new_unchecked(lhs.as_ptr().add(rhs)) }
    }
}

impl<T, Alloc: IAlloc> Vec<T, Alloc> {
    /// Constructs a new vector in `alloc`. This doesn't actually allocate.
    pub const fn new_in(alloc: Alloc) -> Self {
        let start = AllocPtr::dangling();
        Self(VecInner {
            start,
            end: start.ptr,
            capacity: if Self::zst_mode() {
                unsafe { core::mem::transmute(usize::MAX) }
            } else {
                start.ptr
            },
            alloc,
        })
    }
    /// Constructs a new vector. This doesn't actually allocate.
    pub fn new() -> Self
    where
        Alloc: Default,
    {
        Self::new_in(Alloc::default())
    }
    /// Constructs a new vector in `alloc`, allocating sufficient space for `capacity` elements.
    pub fn with_capacity_in(capacity: usize, alloc: Alloc) -> Self {
        let mut this = Self::new_in(alloc);
        this.reserve(capacity);
        this
    }
    /// Constructs a new vector, allocating sufficient space for `capacity` elements.
    pub fn with_capacity(capacity: usize) -> Self
    where
        Alloc: Default,
    {
        Self::with_capacity_in(capacity, Alloc::default())
    }
    #[inline(always)]
    const fn zst_mode() -> bool {
        core::mem::size_of::<T>() == 0
    }
    pub const fn len(&self) -> usize {
        ptr_diff(self.0.end, self.0.start.ptr)
    }
    pub fn is_empty(&self) -> bool {
        self.0.end.as_ptr() == self.0.start.as_ptr()
    }
    /// Sets the length of the vector, not calling any destructors.
    /// # Safety
    /// This can lead to uninitialized memory being interpreted as an initialized value of `T`.
    pub unsafe fn set_len(&mut self, len: usize) {
        self.0.end = ptr_add(*self.0.start, len);
    }
    pub fn push(&mut self, value: T) {
        if self.0.end != self.0.capacity {
            unsafe { self.0.end.as_ptr().write(value) }
            self.0.end = ptr_add(self.0.end, 1)
        } else {
            self.grow();
            self.push(value);
        }
    }
    /// The total capacity of the vector.
    pub const fn capacity(&self) -> usize {
        ptr_diff(self.0.capacity, self.0.start.ptr)
    }
    /// The remaining number of elements that can be pushed before reallocating.
    pub const fn remaining_capacity(&self) -> usize {
        ptr_diff(self.0.capacity, self.0.end)
    }
    const FIRST_CAPACITY: usize = match 1024 / core::mem::size_of::<T>() {
        0 => 1,
        v @ 1..=8 => v,
        _ => 8,
    };
    fn grow(&mut self) {
        if self.capacity() == 0 {
            let first_capacity = Self::FIRST_CAPACITY;
            self.reserve(first_capacity)
        } else {
            self.reserve((self.capacity() >> 1).max(1))
        }
    }
    /// Ensures that `additional` more elements
    pub fn reserve(&mut self, additional: usize) {
        if self.remaining_capacity() < additional {
            let new_capacity = self.len() + additional;
            let start = if self.capacity() != 0 {
                unsafe { self.0.start.realloc(&mut self.0.alloc, new_capacity) }
            } else {
                AllocPtr::alloc_array(&mut self.0.alloc, new_capacity)
            };
            let Some(start) = start else {
                panic!("Allocation failed");
            };
            let end = ptr_add(*start, self.len());
            let capacity = ptr_add(*start, new_capacity);
            self.0.start = start;
            self.0.end = end;
            self.0.capacity = capacity;
        }
    }
    /// Removes all elements from `self` from the `len`th onward.
    ///
    /// Does nothing if `self.len() <= len`
    pub fn truncate(&mut self, len: usize) {
        if self.len() <= len {
            return;
        }
        unsafe {
            core::ptr::drop_in_place(&mut self[len..]);
            self.set_len(len)
        };
    }
    pub fn as_slice(&self) -> &[T] {
        let start = self.0.start;
        let end = self.0.end;
        unsafe { core::slice::from_raw_parts(start.as_ptr(), ptr_diff(end, *start)) }
    }
    pub fn as_slice_mut(&mut self) -> &mut [T] {
        let start = self.0.start;
        let end = self.0.end;
        unsafe { core::slice::from_raw_parts_mut(start.as_ptr(), ptr_diff(end, *start)) }
    }
    pub(crate) fn into_raw_components(self) -> (AllocSlice<T, Alloc>, usize, Alloc) {
        let VecInner {
            start,
            end,
            capacity: _,
            alloc,
        } = unsafe { core::ptr::read(&self.0) };
        let capacity = if core::mem::size_of::<T>() == 0 {
            0
        } else {
            self.capacity()
        };
        core::mem::forget(self);
        (AllocSlice { start, end }, capacity, alloc)
    }
}
macro_rules! impl_index {
    ($index: ty) => {
        impl<T, Alloc: IAlloc> core::ops::Index<$index> for Vec<T, Alloc> {
            type Output = <[T] as core::ops::Index<$index>>::Output;
            fn index(&self, index: $index) -> &Self::Output {
                &self.as_slice()[index]
            }
        }
        impl<T, Alloc: IAlloc> core::ops::IndexMut<$index> for Vec<T, Alloc> {
            fn index_mut(&mut self, index: $index) -> &mut Self::Output {
                &mut self.as_slice_mut()[index]
            }
        }
    };
}

impl<T, Alloc: IAlloc> core::ops::Deref for Vec<T, Alloc> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}
impl<T, Alloc: IAlloc> core::ops::DerefMut for Vec<T, Alloc> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_slice_mut()
    }
}
impl<T, Alloc: IAlloc> Drop for Vec<T, Alloc> {
    fn drop(&mut self) {
        unsafe { core::ptr::drop_in_place(self.as_slice_mut()) }
        if core::mem::size_of::<T>() != 0 && self.capacity() != 0 {
            unsafe { self.0.start.free(&mut self.0.alloc) }
        }
    }
}

impl_index!(usize);
impl_index!(core::ops::Range<usize>);
impl_index!(core::ops::RangeInclusive<usize>);
impl_index!(core::ops::RangeTo<usize>);
impl_index!(core::ops::RangeToInclusive<usize>);
impl_index!(core::ops::RangeFrom<usize>);
impl_index!(core::ops::RangeFull);
