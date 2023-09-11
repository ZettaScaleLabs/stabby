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

use core::sync::atomic::{AtomicUsize, Ordering};

use super::{
    boxed::Box,
    vec::{ptr_add, ptr_diff, Vec, VecInner},
    AllocPtr, AllocSlice, IAlloc, Layout,
};

#[cfg(not(feature = "libc"))]
#[crate::stabby]
pub struct Arc<T, Alloc: IAlloc> {
    ptr: AllocPtr<T, Alloc>,
}
#[cfg(feature = "libc")]
#[crate::stabby]
pub struct Arc<T, Alloc: IAlloc = crate::realloc::libc_alloc::LibcAlloc> {
    ptr: AllocPtr<T, Alloc>,
}
unsafe impl<T: Send + Sync, Alloc: IAlloc + Send + Sync> Send for Arc<T, Alloc> {}
unsafe impl<T: Send + Sync, Alloc: IAlloc + Send + Sync> Sync for Arc<T, Alloc> {}
const USIZE_TOP_BIT: usize = 1 << (core::mem::size_of::<usize>() as i32 * 8 - 1);

impl<T, Alloc: IAlloc> Arc<T, Alloc> {
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
            ptr.prefix_mut().strong = AtomicUsize::new(1);
            ptr.prefix_mut().weak = AtomicUsize::new(1)
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
            ptr.prefix_mut().strong = AtomicUsize::new(1);
            ptr.prefix_mut().weak = AtomicUsize::new(1)
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

    /// Returns the pointer to the inner raw allocation, leaking `this`.
    ///
    /// Note that the pointer may be dangling if `T` is zero-sized.
    pub const fn into_raw(this: Self) -> AllocPtr<T, Alloc> {
        let inner = this.ptr;
        core::mem::forget(this);
        inner
    }
    /// Constructs `Self` from a raw allocation.
    /// # Safety
    /// `this` MUST not be dangling, and have been obtained through [`Self::into_inner`].
    pub const unsafe fn from_raw(this: AllocPtr<T, Alloc>) -> Self {
        Self { ptr: this }
    }

    /// Provides a mutable reference to the internals if the strong and weak counts are both 1.
    pub fn get_mut(this: &mut Self) -> Option<&mut T> {
        if Self::weak_count(this) == 1 && Self::strong_count(this) == 1 {
            Some(unsafe { Self::get_mut_unchecked(this) })
        } else {
            None
        }
    }

    /// Provides a mutable reference to the internals without checking.
    /// # Safety
    /// If used carelessly, this can cause mutable references and immutable references to the same value to appear,
    /// causing undefined behaviour.
    pub unsafe fn get_mut_unchecked(this: &mut Self) -> &mut T {
        unsafe { this.ptr.as_mut() }
    }

    /// Returns the strong count.
    pub fn strong_count(this: &Self) -> usize {
        unsafe { this.ptr.prefix() }.strong.load(Ordering::Relaxed)
    }
    /// Returns the weak count. Note that all Arcs to a same value share a Weak, so the weak count can never be 0.
    pub fn weak_count(this: &Self) -> usize {
        unsafe { this.ptr.prefix() }.weak.load(Ordering::Relaxed)
    }

    /// Returns a mutable reference to this `Arc`'s value, cloning that value into a new `Arc` if [`Self::get_mut`] would have failed.
    pub fn make_mut(&mut self) -> &mut T
    where
        T: Clone,
        Alloc: Clone,
    {
        if Self::strong_count(self) > 1 || Self::weak_count(self) > 1 {
            *self = Self::new_in(T::clone(self), unsafe { self.ptr.prefix() }.alloc.clone());
        }
        unsafe { Self::get_mut_unchecked(self) }
    }
    /// Attempts the value from the allocation, freeing said allocation, if the strong and weah counts are indeed 1.
    pub fn try_into_inner(this: Self) -> Result<T, Self> {
        if Self::strong_count(&this) > 1 || Self::weak_count(&this) > 1 {
            Err(this)
        } else {
            let ret = unsafe { core::ptr::read(&*this) };
            _ = unsafe { Weak::<T, Alloc>::from_raw(Arc::into_raw(this)) };
            Ok(ret)
        }
    }
}
impl<T, Alloc: IAlloc> Drop for Arc<T, Alloc> {
    fn drop(&mut self) {
        if unsafe { self.ptr.prefix() }
            .strong
            .fetch_sub(1, Ordering::Relaxed)
            != 1
        {
            return;
        }
        unsafe {
            core::ptr::drop_in_place(self.ptr.as_mut());
            _ = Weak::<T, Alloc>::from_raw(self.ptr);
        }
    }
}
impl<T, Alloc: IAlloc> Clone for Arc<T, Alloc> {
    fn clone(&self) -> Self {
        unsafe { self.ptr.prefix() }
            .strong
            .fetch_add(1, Ordering::Relaxed);
        Self { ptr: self.ptr }
    }
}
impl<T, Alloc: IAlloc> core::ops::Deref for Arc<T, Alloc> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

#[cfg(not(feature = "libc"))]
#[crate::stabby]
pub struct Weak<T, Alloc: IAlloc> {
    ptr: AllocPtr<T, Alloc>,
}
#[cfg(feature = "libc")]
#[crate::stabby]
pub struct Weak<T, Alloc: IAlloc = crate::realloc::libc_alloc::LibcAlloc> {
    ptr: AllocPtr<T, Alloc>,
}
unsafe impl<T: Send + Sync, Alloc: IAlloc + Send + Sync> Send for Weak<T, Alloc> {}
unsafe impl<T: Send + Sync, Alloc: IAlloc + Send + Sync> Sync for Weak<T, Alloc> {}
impl<T, Alloc: IAlloc> From<&Arc<T, Alloc>> for Weak<T, Alloc> {
    fn from(value: &Arc<T, Alloc>) -> Self {
        unsafe { value.ptr.prefix() }
            .weak
            .fetch_add(1, Ordering::Relaxed);
        Self { ptr: value.ptr }
    }
}
impl<T, Alloc: IAlloc> Weak<T, Alloc> {
    /// Returns the pointer to the inner raw allocation, leaking `this`.
    ///
    /// Note that the pointer may be dangling if `T` is zero-sized.
    pub const fn into_raw(this: Self) -> AllocPtr<T, Alloc> {
        let inner = this.ptr;
        core::mem::forget(this);
        inner
    }
    /// Constructs `Self` from a raw allocation.
    /// # Safety
    /// `this` MUST not be dangling, and have been obtained through [`Self::into_inner`].
    pub const unsafe fn from_raw(this: AllocPtr<T, Alloc>) -> Self {
        Self { ptr: this }
    }
    /// Attempts to upgrade self into an Arc.
    pub fn upgrade(&self) -> Option<Arc<T, Alloc>> {
        let strong = &unsafe { self.ptr.prefix() }.strong;
        let count = strong.fetch_or(USIZE_TOP_BIT, Ordering::Acquire);
        match count {
            0 | USIZE_TOP_BIT => {
                strong.store(0, Ordering::Release);
                None
            }
            _ => {
                strong.fetch_add(1, Ordering::Release);
                strong.fetch_and(!USIZE_TOP_BIT, Ordering::Release);
                Some(Arc { ptr: self.ptr })
            }
        }
    }
}
impl<T, Alloc: IAlloc> Clone for Weak<T, Alloc> {
    fn clone(&self) -> Self {
        unsafe { self.ptr.prefix() }
            .weak
            .fetch_add(1, Ordering::Relaxed);
        Self { ptr: self.ptr }
    }
}
impl<T, Alloc: IAlloc> Drop for Weak<T, Alloc> {
    fn drop(&mut self) {
        if unsafe { self.ptr.prefix() }
            .weak
            .fetch_sub(1, Ordering::Relaxed)
            != 1
        {
            return;
        }
        unsafe {
            _ = Box::<_, Alloc>::from_raw(self.ptr);
        }
    }
}

#[cfg(feature = "libc")]
#[crate::stabby]
pub struct ArcSlice<T, Alloc: IAlloc = super::libc_alloc::LibcAlloc>(
    pub(crate) AllocSlice<T, Alloc>,
);

#[cfg(not(feature = "libc"))]
#[crate::stabby]
pub struct ArcSlice<T, Alloc: IAlloc>(pub(crate) AllocSlice<T, Alloc>);

impl<T, Alloc: IAlloc> ArcSlice<T, Alloc> {
    pub const fn len(&self) -> usize {
        ptr_diff(self.0.end, self.0.start.ptr)
    }
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn as_slice(&self) -> &[T] {
        let start = self.0.start;
        unsafe { core::slice::from_raw_parts(start.as_ptr(), self.len()) }
    }
    /// # Safety
    /// This can easily create aliased mutable references, which would be undefined behaviour.
    pub unsafe fn as_slice_mut_unchecked(&mut self) -> &mut [T] {
        let start = self.0.start;
        unsafe { core::slice::from_raw_parts_mut(start.as_ptr(), self.len()) }
    }
    pub fn strong_count(this: &Self) -> usize {
        unsafe { this.0.start.prefix().strong.load(Ordering::Relaxed) }
    }
    pub fn as_slice_mut(&mut self) -> Option<&mut [T]> {
        match ArcSlice::strong_count(self) {
            1 => Some(unsafe { self.as_slice_mut_unchecked() }),
            _ => None,
        }
    }
}
impl<T, Alloc: IAlloc> core::ops::Deref for ArcSlice<T, Alloc> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}
impl<T, Alloc: IAlloc> Clone for ArcSlice<T, Alloc> {
    fn clone(&self) -> Self {
        unsafe { self.0.start.prefix() }
            .strong
            .fetch_add(1, Ordering::Relaxed);
        Self(self.0)
    }
}
impl<T, Alloc: IAlloc> From<Arc<T, Alloc>> for ArcSlice<T, Alloc> {
    fn from(mut value: Arc<T, Alloc>) -> Self {
        unsafe { value.ptr.prefix_mut() }.capacity = AtomicUsize::new(1);
        Self(AllocSlice {
            start: value.ptr,
            end: ptr_add(value.ptr.ptr, 1),
        })
    }
}
impl<T, Alloc: IAlloc> From<Vec<T, Alloc>> for ArcSlice<T, Alloc> {
    fn from(value: Vec<T, Alloc>) -> Self {
        let (mut slice, capacity, mut alloc) = value.into_raw_components();
        if capacity != 0 {
            unsafe {
                slice.start.prefix_mut().strong = AtomicUsize::new(1);
                slice.start.prefix_mut().weak = AtomicUsize::new(1);
                slice.start.prefix_mut().capacity = AtomicUsize::new(capacity);
                core::ptr::write(&mut slice.start.prefix_mut().alloc, alloc);
            }
            Self(AllocSlice {
                start: slice.start,
                end: slice.end,
            })
        } else {
            let mut start = AllocPtr::alloc_array(&mut alloc, 0).expect("Allocation failed");
            unsafe {
                start.prefix_mut().strong = AtomicUsize::new(1);
                start.prefix_mut().weak = AtomicUsize::new(1);
                start.prefix_mut().capacity = if core::mem::size_of::<T>() != 0 {
                    AtomicUsize::new(0)
                } else {
                    AtomicUsize::new(ptr_diff(
                        core::mem::transmute(usize::MAX),
                        start.ptr.cast::<u8>(),
                    ))
                };
                core::ptr::write(&mut slice.start.prefix_mut().alloc, alloc);
            }
            Self(AllocSlice {
                start,
                end: ptr_add(start.ptr.cast::<u8>(), slice.len()).cast(),
            })
        }
    }
}
impl<T, Alloc: IAlloc> TryFrom<ArcSlice<T, Alloc>> for Vec<T, Alloc> {
    type Error = ArcSlice<T, Alloc>;
    fn try_from(value: ArcSlice<T, Alloc>) -> Result<Self, Self::Error> {
        if core::mem::size_of::<T>() == 0 || ArcSlice::strong_count(&value) != 1 {
            Err(value)
        } else {
            unsafe {
                let ret = Vec(VecInner {
                    start: value.0.start,
                    end: value.0.end,
                    capacity: ptr_add(
                        value.0.start.ptr,
                        value.0.start.prefix().capacity.load(Ordering::Relaxed),
                    ),
                    alloc: core::ptr::read(&value.0.start.prefix().alloc),
                });
                core::mem::forget(value);
                Ok(ret)
            }
        }
    }
}
impl<T, Alloc: IAlloc> Drop for ArcSlice<T, Alloc> {
    fn drop(&mut self) {
        if unsafe { self.0.start.prefix() }
            .strong
            .fetch_sub(1, Ordering::Relaxed)
            != 1
        {
            return;
        }
        unsafe { core::ptr::drop_in_place(self.as_slice_mut_unchecked()) }
        _ = WeakSlice(self.0)
    }
}

#[cfg(feature = "libc")]
#[crate::stabby]
pub struct WeakSlice<T, Alloc: IAlloc = super::libc_alloc::LibcAlloc>(
    pub(crate) AllocSlice<T, Alloc>,
);

#[cfg(not(feature = "libc"))]
#[crate::stabby]
pub struct WeakSlice<T, Alloc: IAlloc>(pub(crate) AllocSlice<T, Alloc>);

impl<T, Alloc: IAlloc> From<&ArcSlice<T, Alloc>> for WeakSlice<T, Alloc> {
    fn from(value: &ArcSlice<T, Alloc>) -> Self {
        unsafe { value.0.start.prefix() }
            .weak
            .fetch_add(1, Ordering::Relaxed);
        Self(value.0)
    }
}
impl<T, Alloc: IAlloc> Drop for WeakSlice<T, Alloc> {
    fn drop(&mut self) {
        if unsafe { self.0.start.prefix() }
            .weak
            .fetch_sub(1, Ordering::Relaxed)
            != 1
        {
            return;
        }
        let mut alloc = unsafe { core::ptr::read(&self.0.start.prefix().alloc) };
        unsafe { self.0.start.free(&mut alloc) }
    }
}
