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

use super::{vec::*, AllocPtr, AllocSlice, IAlloc, Layout};
/// An ABI-stable Box, provided `Alloc` is ABI-stable.
#[cfg(not(feature = "libc"))]
#[crate::stabby]
pub struct Box<T, Alloc: IAlloc> {
    ptr: AllocPtr<T, Alloc>,
}
#[cfg(feature = "libc")]
#[crate::stabby]
pub struct Box<T, Alloc: IAlloc = crate::realloc::libc_alloc::LibcAlloc> {
    ptr: AllocPtr<T, Alloc>,
}
unsafe impl<T: Send, Alloc: IAlloc + Send> Send for Box<T, Alloc> {}
unsafe impl<T: Sync, Alloc: IAlloc> Sync for Box<T, Alloc> {}
impl<T, Alloc: IAlloc> Box<T, Alloc> {
    /// Attempts to allocate [`Self`], initializing it with `constructor`.
    ///
    /// Returns the constructor and the allocator in case of failure.
    ///
    /// Note that the allocation may or may not be zeroed.
    pub fn fallible_make_in<F: FnOnce(&mut core::mem::MaybeUninit<T>)>(
        constructor: F,
        mut alloc: Alloc,
    ) -> Result<Self, (F, Alloc)> {
        let layout = Layout::of::<T>();
        let mut ptr = if layout.size != 0 {
            match AllocPtr::alloc(&mut alloc) {
                Some(mut ptr) => {
                    unsafe { core::ptr::write(&mut ptr.prefix_mut().alloc, alloc) };
                    ptr
                }
                None => return Err((constructor, alloc)),
            }
        } else {
            AllocPtr::dangling()
        };
        unsafe {
            constructor(core::mem::transmute::<&mut T, _>(ptr.as_mut()));
        }
        Ok(Self { ptr })
    }
    /// Attempts to allocate a [`Self`] and store `value` in it, returning it and the allocator in case of failure.
    pub fn fallible_new_in(value: T, alloc: Alloc) -> Result<Self, (T, Alloc)> {
        match Self::fallible_make_in(
            |slot| unsafe {
                slot.write(core::ptr::read(&value));
            },
            alloc,
        ) {
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
        let layout = Layout::of::<T>();
        let mut ptr = if layout.size != 0 {
            match AllocPtr::alloc(&mut alloc) {
                Some(mut ptr) => {
                    unsafe { core::ptr::write(&mut ptr.prefix_mut().alloc, alloc) };
                    ptr
                }
                None => panic!("Allocation failed"),
            }
        } else {
            AllocPtr::dangling()
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

    /// Attempts to allocate [`Self`], initializing it with `constructor`.
    ///
    /// Note that the allocation may or may not be zeroed.
    ///
    /// # Panics
    /// If the allocator fails to provide an appropriate allocation.
    pub fn make<F: FnOnce(&mut core::mem::MaybeUninit<T>)>(constructor: F) -> Self
    where
        Alloc: Default,
    {
        Self::make_in(constructor, Alloc::default())
    }
    /// Attempts to allocate [`Self`] and store `value` in it.
    ///
    /// # Panics
    /// If the allocator fails to provide an appropriate allocation.
    pub fn new(value: T) -> Self
    where
        Alloc: Default,
    {
        Self::new_in(value, Alloc::default())
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
    pub unsafe fn from_raw(this: AllocPtr<T, Alloc>) -> Self {
        Self { ptr: this }
    }
}

impl<T, Alloc: IAlloc> Box<T, Alloc> {
    fn free(&mut self) {
        if Layout::of::<T>().size != 0 {
            let mut alloc = unsafe { core::ptr::read(&self.ptr.prefix().alloc) };
            unsafe { self.ptr.free(&mut alloc) }
        }
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
impl<T, Alloc: IAlloc> crate::abi::IPtr for Box<T, Alloc> {
    unsafe fn as_ref<U: Sized>(&self) -> &U {
        self.ptr.cast().as_ref()
    }
}
impl<T, Alloc: IAlloc> crate::abi::IPtrMut for Box<T, Alloc> {
    unsafe fn as_mut<U: Sized>(&mut self) -> &mut U {
        self.ptr.cast().as_mut()
    }
}
impl<T, Alloc: IAlloc> crate::abi::IPtrOwned for Box<T, Alloc> {
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

/// An ABI-stable boxed slice.
///
/// Note that unlike `std`'s [`Box<[T}>`], this carries the capacity around in the allocation prefix,
/// allowing the reconversion into a [`super::vec::Vec<T, Alloc>`] to keep track
/// of the capacity.
///
/// The inner pointer may be dangling if the slice's length is 0 or `T` is a ZST.
#[cfg(feature = "libc")]
#[crate::stabby]
pub struct BoxedSlice<T, Alloc: IAlloc = crate::realloc::libc_alloc::LibcAlloc> {
    pub(crate) slice: AllocSlice<T, Alloc>,
    pub(crate) alloc: Alloc,
}
#[cfg(not(feature = "libc"))]
pub struct BoxedSlice<T, Alloc: IAlloc> {
    pub(crate) slice: AllocSlice<T, Alloc>,
    pub(crate) alloc: Alloc,
}
impl<T, Alloc: IAlloc> BoxedSlice<T, Alloc> {
    pub const fn len(&self) -> usize {
        ptr_diff(self.slice.end, self.slice.start.ptr)
    }
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn as_slice(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.slice.start.as_ptr(), self.len()) }
    }
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
            Vec(VecInner {
                start: slice.start,
                end: slice.end,
                capacity: ptr_add(slice.start.ptr, capacity),
                alloc,
            })
        } else {
            Vec(VecInner {
                start: slice.start,
                end: slice.end,
                capacity: if core::mem::size_of::<T>() == 0 {
                    unsafe { core::mem::transmute(usize::MAX) }
                } else {
                    slice.start.ptr
                },
                alloc,
            })
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
