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

use crate::num::NonMaxUsize;

use super::{AllocPtr, AllocSlice, AllocationError, IAlloc};
use core::fmt::Debug;
use core::ptr::NonNull;

#[crate::stabby]
pub struct VecInner<T, Alloc: IAlloc> {
    pub(crate) start: AllocPtr<T, Alloc>,
    pub(crate) end: NonNull<T>,
    pub(crate) capacity: NonNull<T>,
    pub(crate) alloc: Alloc,
}

#[crate::stabby]
pub struct Vec<T, Alloc: IAlloc = super::DefaultAllocator>(pub(crate) VecInner<T, Alloc>);

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
    ///
    /// # Panics
    /// If the allocator failed to provide a large enough allocation.
    pub fn with_capacity_in(capacity: usize, alloc: Alloc) -> Self {
        let mut this = Self::new_in(alloc);
        this.reserve(capacity);
        this
    }
    /// Constructs a new vector, allocating sufficient space for `capacity` elements.
    ///
    /// # Panics
    /// If the allocator failed to provide a large enough allocation.
    pub fn with_capacity(capacity: usize) -> Self
    where
        Alloc: Default,
    {
        Self::with_capacity_in(capacity, Alloc::default())
    }
    /// Constructs a new vector in `alloc`, allocating sufficient space for `capacity` elements.
    /// # Errors
    /// Returns an [`AllocationError`] if the allocator couldn't provide a sufficient allocation.
    pub fn try_with_capacity_in(capacity: usize, alloc: Alloc) -> Result<Self, AllocationError> {
        let mut this = Self::new_in(alloc);
        this.try_reserve(capacity)?;
        Ok(this)
    }
    /// Constructs a new vector, allocating sufficient space for `capacity` elements.
    /// # Errors
    /// Returns an [`AllocationError`] if the allocator couldn't provide a sufficient allocation.
    pub fn try_with_capacity(capacity: usize) -> Result<Self, AllocationError>
    where
        Alloc: Default,
    {
        Self::try_with_capacity_in(capacity, Alloc::default())
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
    /// Adds `value` at the end of `self`.
    /// # Panics
    /// This function panics if the vector tried to grow due to
    /// being full, and the allocator failed to provide a new allocation.
    pub fn push(&mut self, value: T) {
        if self.0.end == self.0.capacity {
            self.grow();
        }
        unsafe { self.0.end.as_ptr().write(value) }
        self.0.end = ptr_add(self.0.end, 1)
    }
    /// Adds `value` at the end of `self`.
    ///
    /// # Errors
    /// This function gives back the `value` if the vector tried to grow due to
    /// being full, and the allocator failed to provide a new allocation.
    ///
    /// `self` is still valid should that happen.
    pub fn try_push(&mut self, value: T) -> Result<(), T> {
        if self.0.end == self.0.capacity && self.try_grow().is_err() {
            return Err(value);
        }
        unsafe { self.0.end.as_ptr().write(value) }
        self.0.end = ptr_add(self.0.end, 1);
        Ok(())
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
        self.try_grow().unwrap();
    }
    fn try_grow(&mut self) -> Result<NonMaxUsize, AllocationError> {
        if self.capacity() == 0 {
            let first_capacity = Self::FIRST_CAPACITY;
            self.try_reserve(first_capacity)
        } else {
            self.try_reserve((self.capacity() >> 1).max(1))
        }
    }
    /// Ensures that `additional` more elements can be pushed on `self` without reallocating.
    ///
    /// This may reallocate once to provide this guarantee.
    ///
    /// # Panics
    /// This function panics if the allocator failed to provide an appropriate allocation.
    pub fn reserve(&mut self, additional: usize) {
        self.try_reserve(additional).unwrap();
    }
    /// Ensures that `additional` more elements can be pushed on `self` without reallocating.
    ///
    /// This may reallocate once to provide this guarantee.
    ///
    /// # Errors
    /// Returns Ok(new_capacity) if succesful (including if no reallocation was needed),
    /// otherwise returns Err(AllocationError)
    pub fn try_reserve(&mut self, additional: usize) -> Result<NonMaxUsize, AllocationError> {
        if self.remaining_capacity() < additional {
            let new_capacity = self.len() + additional;
            let start = if self.capacity() != 0 {
                unsafe { self.0.start.realloc(&mut self.0.alloc, new_capacity) }
            } else {
                AllocPtr::alloc_array(&mut self.0.alloc, new_capacity)
            };
            let Some(start) = start else {
                return Err(AllocationError());
            };
            let end = ptr_add(*start, self.len());
            let capacity = ptr_add(*start, new_capacity);
            self.0.start = start;
            self.0.end = end;
            self.0.capacity = capacity;
            Ok(unsafe { NonMaxUsize::new_unchecked(new_capacity) })
        } else {
            let mut capacity = self.capacity();
            if capacity == usize::MAX {
                capacity -= 1;
            }
            Ok(unsafe { NonMaxUsize::new_unchecked(capacity) })
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
    /// Extends `self` using a `memcpy`.
    /// This may be faster than extending through an iterator.
    /// # Panics
    /// If extending required an allocation that failed.
    pub fn copy_extend(&mut self, slice: &[T])
    where
        T: Copy,
    {
        self.try_copy_extend(slice).unwrap();
    }
    /// Extends `self` using a `memcpy`.
    /// This may be faster than extending through an iterator.
    /// # Errors
    /// If extending required an allocation that failed.
    pub fn try_copy_extend(&mut self, slice: &[T]) -> Result<(), AllocationError>
    where
        T: Copy,
    {
        if slice.is_empty() {
            return Ok(());
        }
        self.try_reserve(slice.len())?;
        unsafe {
            core::ptr::copy_nonoverlapping(slice.as_ptr(), self.0.end.as_ptr(), slice.len());
            self.set_len(self.len() + slice.len());
        }
        Ok(())
    }
    pub fn iter(&self) -> core::slice::Iter<'_, T> {
        self.into_iter()
    }
    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, T> {
        self.into_iter()
    }
    /// Removes the specified range from the vector in bulk,
    /// returning all removed elements as an iterator.
    /// If the iterator is dropped before being fully consumed,
    /// it drops the remaining removed elements.
    ///
    /// If the drain is leaked, then the vector may lose and leak elements,
    /// even if they weren't in the specified `range`
    ///
    /// # Panics
    /// This function immediately panics if the range has a negative size, or if the range exceeeds `self.len()`
    pub fn drain<R: core::ops::RangeBounds<usize>>(&mut self, range: R) -> Drain<'_, T, Alloc> {
        let original_len = self.len();
        let from = match range.start_bound() {
            core::ops::Bound::Included(i) => *i,
            core::ops::Bound::Excluded(i) => *i + 1,
            core::ops::Bound::Unbounded => 0,
        };
        let to = match range.end_bound() {
            core::ops::Bound::Included(i) => *i + 1,
            core::ops::Bound::Excluded(i) => *i,
            core::ops::Bound::Unbounded => original_len,
        };
        assert!(to >= from);
        assert!(to <= original_len);
        unsafe { self.set_len(from) };
        Drain {
            vec: self,
            from,
            to,
            index: from,
            original_len,
        }
    }
    /// Removes the specified range from the vector in bulk,
    /// returning all removed elements as an iterator.
    /// If the iterator is dropped before being fully consumed,
    /// it drops the remaining removed elements.
    ///
    /// If the drain is leaked, then the vector may lose and leak elements,
    /// even if they weren't in the specified `range`
    pub fn try_drain<R: core::ops::RangeBounds<usize>>(
        &mut self,
        range: R,
    ) -> Option<Drain<'_, T, Alloc>> {
        let original_len = self.len();
        let from = match range.start_bound() {
            core::ops::Bound::Included(i) => *i,
            core::ops::Bound::Excluded(i) => *i + 1,
            core::ops::Bound::Unbounded => 0,
        };
        let to = match range.end_bound() {
            core::ops::Bound::Included(i) => *i + 1,
            core::ops::Bound::Excluded(i) => *i,
            core::ops::Bound::Unbounded => original_len,
        };
        if to >= from || to <= original_len {
            return None;
        }
        unsafe { self.set_len(from) };
        Some(Drain {
            vec: self,
            from,
            to,
            index: from,
            original_len,
        })
    }
    /// Removes the element at `index` without reordering.
    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index < self.len() {
            unsafe {
                let value = self.0.start.as_ptr().add(index).read();
                core::ptr::copy(
                    self.0.start.as_ptr().add(index + 1),
                    self.0.start.as_ptr().add(index),
                    self.len() - (index + 1),
                );
                self.set_len(self.len() - 1);
                Some(value)
            }
        } else {
            None
        }
    }
    /// Swaps the elements at positions `a` and `b`
    ///
    /// # Panics
    /// Panics if either index is out of bound.
    pub fn swap(&mut self, a: usize, b: usize) {
        assert!(a < self.len());
        assert!(b < self.len());
        unsafe { core::ptr::swap(self.0.start.as_ptr().add(a), self.0.start.as_ptr().add(b)) };
    }
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            unsafe {
                let value = self.0.end.as_ptr().sub(1).read();
                self.set_len(self.len() - 1);
                Some(value)
            }
        }
    }
    /// Removes the element at `index`, moving the last element in its place.
    ///
    /// This is more efficient than [`Self::remove`], but causes reordering.
    pub fn swap_remove(&mut self, index: usize) -> Option<T> {
        if index >= self.len() {
            return None;
        }
        self.swap(index, self.len() - 1);
        self.pop()
    }
}

impl<T: Clone, Alloc: IAlloc + Clone> Clone for Vec<T, Alloc> {
    fn clone(&self) -> Self {
        let mut ret = Self::with_capacity_in(self.len(), self.0.alloc.clone());
        for (i, item) in self.iter().enumerate() {
            unsafe { ret.0.start.ptr.as_ptr().add(i).write(item.clone()) }
        }
        unsafe { ret.set_len(self.len()) };
        ret
    }
}
impl<T: PartialEq, Alloc: IAlloc, Rhs: AsRef<[T]>> PartialEq<Rhs> for Vec<T, Alloc> {
    fn eq(&self, other: &Rhs) -> bool {
        self.as_slice() == other.as_ref()
    }
}
impl<T: Eq, Alloc: IAlloc> Eq for Vec<T, Alloc> {}
impl<T: PartialOrd, Alloc: IAlloc, Rhs: AsRef<[T]>> PartialOrd<Rhs> for Vec<T, Alloc> {
    fn partial_cmp(&self, other: &Rhs) -> Option<core::cmp::Ordering> {
        self.as_slice().partial_cmp(other.as_ref())
    }
}
impl<T: Ord, Alloc: IAlloc> Ord for Vec<T, Alloc> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_slice().cmp(other.as_slice())
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
impl<T, Alloc: IAlloc> core::convert::AsRef<[T]> for Vec<T, Alloc> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}
impl<T, Alloc: IAlloc> core::ops::DerefMut for Vec<T, Alloc> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_slice_mut()
    }
}
impl<T, Alloc: IAlloc> core::convert::AsMut<[T]> for Vec<T, Alloc> {
    fn as_mut(&mut self) -> &mut [T] {
        self.as_slice_mut()
    }
}
impl<T, Alloc: IAlloc + Default> Default for Vec<T, Alloc> {
    fn default() -> Self {
        Self::new()
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
impl<T, Alloc: IAlloc> core::iter::Extend<T> for Vec<T, Alloc> {
    fn extend<Iter: IntoIterator<Item = T>>(&mut self, iter: Iter) {
        let iter = iter.into_iter();
        let min = iter.size_hint().0;
        self.reserve(min);
        for item in iter {
            self.push(item);
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

impl<T: Debug, Alloc: IAlloc> Debug for Vec<T, Alloc> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.as_slice().fmt(f)
    }
}
impl<T: core::fmt::LowerHex, Alloc: IAlloc> core::fmt::LowerHex for Vec<T, Alloc> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut first = true;
        for item in self {
            if !first {
                f.write_str(":")?;
            }
            first = false;
            core::fmt::LowerHex::fmt(item, f)?;
        }
        Ok(())
    }
}
impl<T: core::fmt::UpperHex, Alloc: IAlloc> core::fmt::UpperHex for Vec<T, Alloc> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut first = true;
        for item in self {
            if !first {
                f.write_str(":")?;
            }
            first = false;
            core::fmt::UpperHex::fmt(item, f)?;
        }
        Ok(())
    }
}
impl<'a, T, Alloc: IAlloc> IntoIterator for &'a Vec<T, Alloc> {
    type Item = &'a T;
    type IntoIter = core::slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}
impl<'a, T, Alloc: IAlloc> IntoIterator for &'a mut Vec<T, Alloc> {
    type Item = &'a mut T;
    type IntoIter = core::slice::IterMut<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice_mut().iter_mut()
    }
}
impl<T, Alloc: IAlloc> IntoIterator for Vec<T, Alloc> {
    type Item = T;
    type IntoIter = IntoIter<T, Alloc>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            vec: self,
            index: 0,
        }
    }
}
/// [`Vec`]'s iterator.
#[crate::stabby]
pub struct IntoIter<T, Alloc: IAlloc> {
    vec: Vec<T, Alloc>,
    index: usize,
}
impl<T, Alloc: IAlloc> Iterator for IntoIter<T, Alloc> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        (self.index < self.vec.len()).then(|| unsafe {
            let ret = self.vec.0.start.as_ptr().add(self.index).read();
            self.index += 1;
            ret
        })
    }
}
impl<T, Alloc: IAlloc> Drop for IntoIter<T, Alloc> {
    fn drop(&mut self) {
        unsafe {
            core::ptr::drop_in_place(&mut self.vec.as_slice_mut()[self.index..]);
            self.vec.set_len(0);
        }
    }
}
/// An iterator that removes elements from a [`Vec`].
///
/// Dropping the `Drain` will finish draining its specified range.
///
/// Note that leaking the `Drain` may cause its [`Vec`] to lose and leak elements,
/// even outside the specified range.
#[crate::stabby]
pub struct Drain<'a, T: 'a, Alloc: IAlloc + 'a> {
    vec: &'a mut Vec<T, Alloc>,
    from: usize,
    to: usize,
    index: usize,
    original_len: usize,
}
impl<'a, T: 'a, Alloc: IAlloc + 'a> Drain<'a, T, Alloc> {
    /// Prevents `self` from draining its vector any further, and applies the already
    /// commited drain.
    pub fn stop(mut self) {
        self.to = self.index
    }
    /// Turns the drain into a double ended drain, which impls [`DoubleEndedIterator`]
    pub fn double_ended(self) -> DoubleEndedDrain<'a, T, Alloc> {
        let ret = DoubleEndedDrain {
            vec: unsafe { core::ptr::read(&self.vec) },
            from: self.from,
            to: self.to,
            original_len: self.original_len,
            lindex: self.index,
            rindex: self.to,
        };
        core::mem::forget(self);
        ret
    }
}
impl<'a, T: 'a, Alloc: IAlloc + 'a> Iterator for Drain<'a, T, Alloc> {
    type Item = T;
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.to - self.index;
        (remaining, Some(remaining))
    }
    fn next(&mut self) -> Option<Self::Item> {
        (self.index < self.to).then(|| unsafe {
            let ret = self.vec.0.start.as_ptr().add(self.index).read();
            self.index += 1;
            ret
        })
    }
}
impl<'a, T: 'a, Alloc: IAlloc + 'a> ExactSizeIterator for Drain<'a, T, Alloc> {
    fn len(&self) -> usize {
        self.to - self.index
    }
}
impl<'a, T: 'a, Alloc: IAlloc + 'a> Drop for Drain<'a, T, Alloc> {
    fn drop(&mut self) {
        let tail_length = self.original_len - self.to;
        unsafe {
            core::ptr::drop_in_place(core::slice::from_raw_parts_mut(
                self.vec.0.start.as_ptr().add(self.index),
                self.to - self.index,
            ));
            core::ptr::copy(
                self.vec.0.start.as_ptr().add(self.to),
                self.vec.0.start.as_ptr().add(self.from),
                tail_length,
            );
            self.vec.set_len(tail_length + self.from);
        }
    }
}
#[crate::stabby]
pub struct DoubleEndedDrain<'a, T: 'a, Alloc: IAlloc + 'a> {
    vec: &'a mut Vec<T, Alloc>,
    from: usize,
    to: usize,
    original_len: usize,
    lindex: usize,
    rindex: usize,
}
impl<'a, T: 'a, Alloc: IAlloc + 'a> Iterator for DoubleEndedDrain<'a, T, Alloc> {
    type Item = T;
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.to - self.lindex;
        (remaining, Some(remaining))
    }
    fn next(&mut self) -> Option<Self::Item> {
        (self.lindex < self.rindex).then(|| unsafe {
            let ret = self.vec.0.start.as_ptr().add(self.lindex).read();
            self.lindex += 1;
            ret
        })
    }
}
impl<'a, T: 'a, Alloc: IAlloc + 'a> DoubleEndedIterator for DoubleEndedDrain<'a, T, Alloc> {
    fn next_back(&mut self) -> Option<Self::Item> {
        (self.lindex < self.rindex).then(|| unsafe {
            let ret = self.vec.0.start.as_ptr().add(self.rindex).read();
            self.rindex -= 1;
            ret
        })
    }
}
impl<'a, T: 'a, Alloc: IAlloc + 'a> ExactSizeIterator for DoubleEndedDrain<'a, T, Alloc> {
    fn len(&self) -> usize {
        self.rindex - self.lindex
    }
}
impl<'a, T: 'a, Alloc: IAlloc + 'a> Drop for DoubleEndedDrain<'a, T, Alloc> {
    fn drop(&mut self) {
        let tail_length = self.original_len - self.to;
        unsafe {
            core::ptr::drop_in_place(core::slice::from_raw_parts_mut(
                self.vec.0.start.as_ptr().add(self.lindex),
                self.rindex - self.lindex,
            ));
            core::ptr::copy(
                self.vec.0.start.as_ptr().add(self.to),
                self.vec.0.start.as_ptr().add(self.from),
                tail_length,
            );
            self.vec.set_len(tail_length + self.from);
        }
    }
}
#[cfg(feature = "std")]
impl<Alloc: IAlloc> std::io::Write for Vec<u8, Alloc> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self.try_copy_extend(buf) {
            Ok(()) => Ok(buf.len()),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::OutOfMemory, e)),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
