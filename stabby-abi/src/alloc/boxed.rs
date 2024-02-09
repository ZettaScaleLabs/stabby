//
// Copyright (c) 2023 ZettaScale Technology
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.inner which is available at
// http://www.eclipse.org/legal/epl-2.inner, or the Apache License, Version 2.inner
// which is available at https://www.apache.org/licenses/LICENSE-2.inner.
//
// SPDX-License-Identifier: EPL-2.inner OR Apache-2.inner
//
// Contributors:
//   Pierre Avital, <pierre.avital@me.com>
//

use crate::IntoDyn;

use super::{vec::*, AllocPtr, AllocSlice, IAlloc};
use core::fmt::Debug;

/// An ABI-stable Box, provided `Alloc` is ABI-stable.
#[crate::stabby]
pub struct Box<T, Alloc: IAlloc = super::DefaultAllocator> {
    ptr: AllocPtr<T, Alloc>,
}
unsafe impl<T: Send, Alloc: IAlloc + Send> Send for Box<T, Alloc> {}
unsafe impl<T: Sync, Alloc: IAlloc> Sync for Box<T, Alloc> {}
#[cfg(feature = "libc")]
impl<T> Box<T> {
    /// Attempts to allocate [`Self`], initializing it with `constructor`.
    ///
    /// Note that the allocation may or may not be zeroed.
    ///
    /// # Panics
    /// If the allocator fails to provide an appropriate allocation.
    pub fn make<F: FnOnce(&mut core::mem::MaybeUninit<T>)>(constructor: F) -> Self {
        Self::make_in(constructor, super::DefaultAllocator::new())
    }
    /// Attempts to allocate [`Self`] and store `value` in it.
    ///
    /// # Panics
    /// If the allocator fails to provide an appropriate allocation.
    pub fn new(value: T) -> Self {
        Self::new_in(value, super::DefaultAllocator::new())
    }
}
impl<T, Alloc: IAlloc> Box<T, Alloc> {
    /// Attempts to allocate [`Self`], initializing it with `constructor`.
    ///
    /// # Errors
    /// Returns the constructor and the allocator in case of failure.
    ///
    /// # Notes
    /// Note that the allocation may or may not be zeroed.
    pub fn try_make_in<F: FnOnce(&mut core::mem::MaybeUninit<T>)>(
        constructor: F,
        mut alloc: Alloc,
    ) -> Result<Self, (F, Alloc)> {
        let mut ptr = match AllocPtr::alloc(&mut alloc) {
            Some(mut ptr) => {
                unsafe { core::ptr::write(&mut ptr.prefix_mut().alloc, alloc) };
                ptr
            }
            None => return Err((constructor, alloc)),
        };
        unsafe {
            constructor(core::mem::transmute::<&mut T, _>(ptr.as_mut()));
        }
        Ok(Self { ptr })
    }
    /// Attempts to allocate a [`Self`] and store `value` in it
    /// # Errors
    /// Returns `value` and the allocator in case of failure.
    pub fn try_new_in(value: T, alloc: Alloc) -> Result<Self, (T, Alloc)> {
        let this = Self::try_make_in(
            |slot| unsafe {
                slot.write(core::ptr::read(&value));
            },
            alloc,
        );
        match this {
            Ok(this) => Ok(this),
            Err((_, a)) => Err((value, a)),
        }
    }
    /// Attempts to allocate [`Self`], initializing it with `constructor`.
    ///
    /// Note that the allocation may or may not be zeroed.
    ///
    /// # Panics
    /// If the allocator fails to provide an appropriate allocation.
    pub fn make_in<F: FnOnce(&mut core::mem::MaybeUninit<T>)>(
        constructor: F,
        mut alloc: Alloc,
    ) -> Self {
        let mut ptr = match AllocPtr::alloc(&mut alloc) {
            Some(mut ptr) => {
                unsafe { core::ptr::write(&mut ptr.prefix_mut().alloc, alloc) };
                ptr
            }
            None => panic!("Allocation failed"),
        };
        unsafe {
            constructor(core::mem::transmute::<&mut T, _>(ptr.as_mut()));
        }
        Self { ptr }
    }
    /// Attempts to allocate [`Self`] and store `value` in it.
    ///
    /// # Panics
    /// If the allocator fails to provide an appropriate allocation.
    pub fn new_in(value: T, alloc: Alloc) -> Self {
        Self::make_in(
            move |slot| {
                slot.write(value);
            },
            alloc,
        )
    }
    /// Extracts the value from the allocation, freeing said allocation.
    pub fn into_inner(mut this: Self) -> T {
        let ret = unsafe { core::ptr::read(&*this) };
        this.free();
        core::mem::forget(this);
        ret
    }
    /// Returns the pointer to the inner raw allocation, leaking `this`.
    ///
    /// Note that the pointer may be dangling if `T` is zero-sized.
    pub fn into_raw(this: Self) -> AllocPtr<T, Alloc> {
        let inner = this.ptr;
        core::mem::forget(this);
        inner
    }
    /// Constructs `Self` from a raw allocation.
    /// # Safety
    /// No other container must own (even partially) `this`.
    pub const unsafe fn from_raw(this: AllocPtr<T, Alloc>) -> Self {
        Self { ptr: this }
    }
}

impl<T, Alloc: IAlloc> Box<T, Alloc> {
    fn free(&mut self) {
        let mut alloc = unsafe { core::ptr::read(&self.ptr.prefix().alloc) };
        unsafe { self.ptr.free(&mut alloc) }
    }
}

impl<T: Clone, Alloc: IAlloc + Clone> Clone for Box<T, Alloc> {
    fn clone(&self) -> Self {
        Box::new_in(T::clone(self), unsafe { self.ptr.prefix() }.alloc.clone())
    }
}
impl<T, Alloc: IAlloc> core::ops::Deref for Box<T, Alloc> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T, Alloc: IAlloc> core::ops::DerefMut for Box<T, Alloc> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.as_mut() }
    }
}
impl<T, Alloc: IAlloc> crate::IPtr for Box<T, Alloc> {
    unsafe fn as_ref<U: Sized>(&self) -> &U {
        self.ptr.cast().as_ref()
    }
}
impl<T, Alloc: IAlloc> crate::IPtrMut for Box<T, Alloc> {
    unsafe fn as_mut<U: Sized>(&mut self) -> &mut U {
        self.ptr.cast().as_mut()
    }
}
impl<T, Alloc: IAlloc> crate::IPtrOwned for Box<T, Alloc> {
    fn drop(this: &mut core::mem::ManuallyDrop<Self>, drop: unsafe extern "C" fn(&mut ())) {
        let rthis = &mut ***this;
        unsafe {
            drop(core::mem::transmute(rthis));
        }
        this.free();
    }
}
impl<T, Alloc: IAlloc> Drop for Box<T, Alloc> {
    fn drop(&mut self) {
        unsafe {
            core::ptr::drop_in_place(self.ptr.as_mut());
        }
        self.free()
    }
}
impl<T, Alloc: IAlloc> IntoDyn for Box<T, Alloc> {
    type Anonymized = Box<(), Alloc>;
    type Target = T;
    fn anonimize(self) -> Self::Anonymized {
        let original_prefix = self.ptr.prefix_ptr();
        let anonymized = unsafe { core::mem::transmute::<_, Self::Anonymized>(self) };
        let anonymized_prefix = anonymized.ptr.prefix_ptr();
        assert_eq!(anonymized_prefix, original_prefix);
        anonymized
    }
}

/// An ABI-stable boxed slice.
///
/// Note that unlike `std`'s [`Box<[T}>`], this carries the capacity around in the allocation prefix,
/// allowing the reconversion into a [`super::vec::Vec<T, Alloc>`] to keep track
/// of the capacity.
///
/// The inner pointer may be dangling if the slice's length is 0 or `T` is a ZST.
#[crate::stabby]
pub struct BoxedSlice<T, Alloc: IAlloc = super::DefaultAllocator> {
    pub(crate) slice: AllocSlice<T, Alloc>,
    pub(crate) alloc: Alloc,
}
impl<T, Alloc: IAlloc> BoxedSlice<T, Alloc> {
    /// The number of elements in the boxed slice.
    pub const fn len(&self) -> usize {
        ptr_diff(self.slice.end, self.slice.start.ptr)
    }
    /// Returns `true` if the slice is empty.
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Cast into a standard slice.
    pub fn as_slice(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.slice.start.as_ptr(), self.len()) }
    }
    /// Cast into a standard mutable slice.
    pub fn as_slice_mut(&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.slice.start.as_ptr(), self.len()) }
    }
    pub(crate) fn into_raw_components(self) -> (AllocSlice<T, Alloc>, usize, Alloc) {
        let slice = self.slice;
        let alloc = unsafe { core::ptr::read(&self.alloc) };
        core::mem::forget(self);
        let capacity = if core::mem::size_of::<T>() == 0 || slice.is_empty() {
            0
        } else {
            unsafe {
                slice
                    .start
                    .prefix()
                    .capacity
                    .load(core::sync::atomic::Ordering::Relaxed)
            }
        };
        (slice, capacity, alloc)
    }
}
impl<T: Eq, Alloc: IAlloc> Eq for BoxedSlice<T, Alloc> {}
impl<T: PartialEq, Alloc: IAlloc> PartialEq for BoxedSlice<T, Alloc> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}
impl<T: Ord, Alloc: IAlloc> Ord for BoxedSlice<T, Alloc> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_slice().cmp(other.as_slice())
    }
}
impl<T: PartialOrd, Alloc: IAlloc> PartialOrd for BoxedSlice<T, Alloc> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.as_slice().partial_cmp(other.as_slice())
    }
}
impl<T: core::hash::Hash, Alloc: IAlloc> core::hash::Hash for BoxedSlice<T, Alloc> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state)
    }
}
impl<T, Alloc: IAlloc> From<Vec<T, Alloc>> for BoxedSlice<T, Alloc> {
    fn from(value: Vec<T, Alloc>) -> Self {
        let (mut slice, capacity, alloc) = value.into_raw_components();
        if capacity != 0 {
            unsafe {
                slice.start.prefix_mut().capacity = core::sync::atomic::AtomicUsize::new(capacity);
            }
            Self {
                slice: AllocSlice {
                    start: slice.start,
                    end: slice.end,
                },
                alloc,
            }
        } else {
            Self { slice, alloc }
        }
    }
}
impl<T, Alloc: IAlloc> From<BoxedSlice<T, Alloc>> for Vec<T, Alloc> {
    fn from(value: BoxedSlice<T, Alloc>) -> Self {
        let (slice, capacity, alloc) = value.into_raw_components();
        if capacity != 0 {
            Vec {
                inner: VecInner {
                    start: slice.start,
                    end: slice.end,
                    capacity: ptr_add(slice.start.ptr, capacity),
                    alloc,
                },
            }
        } else {
            Vec {
                inner: VecInner {
                    start: slice.start,
                    end: slice.end,
                    capacity: if core::mem::size_of::<T>() == 0 {
                        unsafe { core::mem::transmute(usize::MAX) }
                    } else {
                        slice.start.ptr
                    },
                    alloc,
                },
            }
        }
    }
}

impl<T, Alloc: IAlloc> Drop for BoxedSlice<T, Alloc> {
    fn drop(&mut self) {
        unsafe { core::ptr::drop_in_place(self.as_slice_mut()) }
        if core::mem::size_of::<T>() != 0 && !self.is_empty() {
            unsafe { self.slice.start.free(&mut self.alloc) }
        }
    }
}

impl<T: Debug, Alloc: IAlloc> Debug for BoxedSlice<T, Alloc> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.as_slice().fmt(f)
    }
}
impl<T: core::fmt::LowerHex, Alloc: IAlloc> core::fmt::LowerHex for BoxedSlice<T, Alloc> {
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
impl<T: core::fmt::UpperHex, Alloc: IAlloc> core::fmt::UpperHex for BoxedSlice<T, Alloc> {
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
impl<'a, T, Alloc: IAlloc> IntoIterator for &'a BoxedSlice<T, Alloc> {
    type Item = &'a T;
    type IntoIter = core::slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}
impl<'a, T, Alloc: IAlloc> IntoIterator for &'a mut BoxedSlice<T, Alloc> {
    type Item = &'a mut T;
    type IntoIter = core::slice::IterMut<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice_mut().iter_mut()
    }
}
impl<T, Alloc: IAlloc> IntoIterator for BoxedSlice<T, Alloc> {
    type Item = T;
    type IntoIter = super::vec::IntoIter<T, Alloc>;
    fn into_iter(self) -> Self::IntoIter {
        let this: super::vec::Vec<T, Alloc> = self.into();
        this.into_iter()
    }
}
pub use super::string::BoxedStr;
