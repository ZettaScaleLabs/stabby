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

use crate::{unreachable_unchecked, IntoDyn};

use super::{vec::*, AllocPtr, AllocSlice, IAlloc};
use core::{
    fmt::Debug,
    mem::{ManuallyDrop, MaybeUninit},
    ptr::NonNull,
};

/// An ABI-stable Box, provided `Alloc` is ABI-stable.
#[crate::stabby]
pub struct Box<T, Alloc: IAlloc = super::DefaultAllocator> {
    ptr: AllocPtr<T, Alloc>,
}
// SAFETY: Same constraints as `std::boxed::Box`
unsafe impl<T: Send, Alloc: IAlloc + Send> Send for Box<T, Alloc> {}
// SAFETY: Same constraints as `std::boxed::Box`
unsafe impl<T: Sync, Alloc: IAlloc> Sync for Box<T, Alloc> {}
// SAFETY: Same constraints as `std::boxed::Box`
unsafe impl<T: Send, Alloc: IAlloc + Send> Send for BoxedSlice<T, Alloc> {}
// SAFETY: Same constraints as `std::boxed::Box`
unsafe impl<T: Sync, Alloc: IAlloc> Sync for BoxedSlice<T, Alloc> {}

#[cfg(not(stabby_default_alloc = "disabled"))]
impl<T> Box<T> {
    /// Attempts to allocate [`Self`], initializing it with `constructor`.
    ///
    /// Note that the allocation may or may not be zeroed.
    ///
    /// If the allocation fails, the `constructor` will not be run.
    ///
    /// # Safety
    /// `constructor` MUST return `Err(())` if it failed to initialize the passed argument.
    ///
    /// # Errors
    /// Returns the uninitialized allocation if the constructor declares a failure.
    ///
    /// # Panics
    /// If the allocator fails to provide an appropriate allocation.
    pub unsafe fn make<
        F: for<'a> FnOnce(&'a mut core::mem::MaybeUninit<T>) -> Result<&'a mut T, ()>,
    >(
        constructor: F,
    ) -> Result<Self, Box<MaybeUninit<T>>> {
        // SAFETY: Ensured by parent fn
        unsafe { Self::make_in(constructor, super::DefaultAllocator::new()) }
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
    /// Note that the allocation may or may not be zeroed.
    ///
    /// If the `constructor` panics, the allocated memory will be leaked.
    ///
    /// # Errors
    /// - Returns the `constructor` and the allocator in case of allocation failure.
    /// - Returns the uninitialized allocated memory if `constructor` fails.
    ///
    /// # Safety
    /// `constructor` MUST return `Err(())` if it failed to initialize the passed argument.
    ///
    /// # Notes
    /// Note that the allocation may or may not be zeroed.
    #[allow(clippy::type_complexity)]
    pub unsafe fn try_make_in<
        F: for<'a> FnOnce(&'a mut core::mem::MaybeUninit<T>) -> Result<&'a mut T, ()>,
    >(
        constructor: F,
        mut alloc: Alloc,
    ) -> Result<Self, Result<Box<MaybeUninit<T>, Alloc>, (F, Alloc)>> {
        let mut ptr = match AllocPtr::alloc(&mut alloc) {
            Some(mut ptr) => {
                // SAFETY: `ptr` just got allocated via `AllocPtr::alloc`.
                unsafe { ptr.prefix_mut() }.alloc.write(alloc);
                ptr
            }
            None => return Err(Err((constructor, alloc))),
        };
        // SAFETY: We are the sole owners of `ptr`
        constructor(unsafe { ptr.as_mut() }).map_or_else(
            |()| Err(Ok(Box { ptr })),
            |_| {
                Ok(Self {
                    // SAFETY: `constructor` reported success.
                    ptr: unsafe { ptr.assume_init() },
                })
            },
        )
    }
    /// Attempts to allocate a [`Self`] and store `value` in it
    /// # Errors
    /// Returns `value` and the allocator in case of failure.
    pub fn try_new_in(value: T, alloc: Alloc) -> Result<Self, (T, Alloc)> {
        // SAFETY: `ctor` is a valid constructor, always initializing the value.
        let this = unsafe {
            Self::try_make_in(
                |slot: &mut core::mem::MaybeUninit<T>| {
                    // SAFETY: `value` will be forgotten if the allocation succeeds and `read` is called.
                    Ok(slot.write(core::ptr::read(&value)))
                },
                alloc,
            )
        };
        match this {
            Ok(this) => {
                core::mem::forget(value);
                Ok(this)
            }
            Err(Err((_, a))) => Err((value, a)),
            // SAFETY: the constructor is infallible.
            Err(Ok(_)) => unsafe { unreachable_unchecked!() },
        }
    }
    /// Attempts to allocate [`Self`], initializing it with `constructor`.
    ///
    /// Note that the allocation may or may not be zeroed.
    ///
    /// # Errors
    /// Returns the uninitialized allocated memory if `constructor` fails.
    ///
    /// # Safety
    /// `constructor` MUST return `Err(())` if it failed to initialize the passed argument.
    ///
    /// # Panics
    /// If the allocator fails to provide an appropriate allocation.
    pub unsafe fn make_in<
        F: for<'a> FnOnce(&'a mut core::mem::MaybeUninit<T>) -> Result<&'a mut T, ()>,
    >(
        constructor: F,
        alloc: Alloc,
    ) -> Result<Self, Box<MaybeUninit<T>, Alloc>> {
        Self::try_make_in(constructor, alloc).map_err(|e| match e {
            Ok(uninit) => uninit,
            Err(_) => panic!("Allocation failed"),
        })
    }
    /// Attempts to allocate [`Self`] and store `value` in it.
    ///
    /// # Panics
    /// If the allocator fails to provide an appropriate allocation.
    pub fn new_in(value: T, alloc: Alloc) -> Self {
        // SAFETY: `constructor` fits the spec.
        let this = unsafe { Self::make_in(move |slot| Ok(slot.write(value)), alloc) };
        // SAFETY: `constructor` is infallible.
        unsafe { this.unwrap_unchecked() }
    }
    /// Extracts the value from the allocation, freeing said allocation.
    pub fn into_inner(this: Self) -> T {
        let mut this = core::mem::ManuallyDrop::new(this);
        // SAFETY: `this` will not be dropped, preventing double-frees.
        let ret = ManuallyDrop::new(unsafe { core::ptr::read(&**this) });
        // SAFETY: `Box::free` only frees the memory allocation, without calling the destructor for `ret`'s source.
        unsafe { this.free() };
        ManuallyDrop::into_inner(ret)
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
    /// No other container must own (even partially) `this`.
    pub const unsafe fn from_raw(this: AllocPtr<T, Alloc>) -> Self {
        Self { ptr: this }
    }
}

impl<T, Alloc: IAlloc> Box<T, Alloc> {
    /// Frees the allocation without destroying the value in it.
    /// # Safety
    /// `self` is in an invalid state after this and MUST be forgotten immediately.
    unsafe fn free(&mut self) {
        // SAFETY: `Box` guarantees that `alloc` is stored in the prefix, and it won't be reused after this.
        let mut alloc = unsafe { self.ptr.prefix().alloc.assume_init_read() };
        // SAFETY: `self.ptr` was definitely allocated in `alloc`
        unsafe { self.ptr.free(&mut alloc) }
    }
}

impl<T: Clone, Alloc: IAlloc + Clone> Clone for Box<T, Alloc> {
    fn clone(&self) -> Self {
        Box::new_in(
            T::clone(self),
            unsafe { self.ptr.prefix().alloc.assume_init_ref() }.clone(),
        )
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
        // SAFETY: This is evil casting shenanigans, but `IPtrOwned` is a type anonimization primitive.
        unsafe {
            drop(core::mem::transmute::<&mut T, &mut ()>(rthis));
        }
        // SAFETY: `this` is immediately forgotten.
        unsafe { this.free() }
    }
}
impl<T, Alloc: IAlloc> Drop for Box<T, Alloc> {
    fn drop(&mut self) {
        // SAFETY: We own the target of `ptr` and guarantee it is initialized.
        unsafe {
            core::ptr::drop_in_place(self.ptr.as_mut());
        }
        // SAFETY: `this` is immediately forgotten.
        unsafe { self.free() }
    }
}
impl<T, Alloc: IAlloc> IntoDyn for Box<T, Alloc> {
    type Anonymized = Box<(), Alloc>;
    type Target = T;
    fn anonimize(self) -> Self::Anonymized {
        let original_prefix = self.ptr.prefix_ptr();
        // SAFETY: Evil anonimization.
        let anonymized = unsafe { core::mem::transmute::<Self, Self::Anonymized>(self) };
        let anonymized_prefix = anonymized.ptr.prefix_ptr();
        assert_eq!(anonymized_prefix, original_prefix, "The allocation prefix was lost in anonimization, this is definitely a bug, please report it.");
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
    /// Constructs an empty boxed slice with a given capacity.
    pub fn with_capacity_in(capacity: usize, alloc: Alloc) -> Self {
        Vec::with_capacity_in(capacity, alloc).into()
    }
    /// The number of elements in the boxed slice.
    pub const fn len(&self) -> usize {
        ptr_diff(self.slice.end, self.slice.start.ptr)
    }
    /// Returns `true` if the slice is empty.
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Cast into a standard slice.
    #[rustversion::attr(since(1.86), const)]
    pub fn as_slice(&self) -> &[T] {
        // SAFETY: we own this slice.
        unsafe { core::slice::from_raw_parts(self.slice.start.ptr.as_ptr(), self.len()) }
    }
    /// Cast into a standard mutable slice.
    #[rustversion::attr(since(1.86), const)]
    pub fn as_slice_mut(&mut self) -> &mut [T] {
        // SAFETY: we own this slice.
        unsafe { core::slice::from_raw_parts_mut(self.slice.start.ptr.as_ptr(), self.len()) }
    }
    /// Attempts to add an element to the boxed slice without reallocating.
    /// # Errors
    /// Returns the value if pushing would require reallocating.
    pub fn try_push(&mut self, value: T) -> Result<(), T> {
        // SAFETY: the prefix must be initialized for this type to exist.
        if self.slice.len()
            >= unsafe { self.slice.start.prefix() }
                .capacity
                .load(core::sync::atomic::Ordering::Relaxed)
        {
            return Err(value);
        }
        // SAFETY: we've acertained that we have enough space to push an element.
        unsafe {
            core::ptr::write(self.slice.end.as_ptr(), value);
            self.slice.end = NonNull::new_unchecked(self.slice.end.as_ptr().add(1));
        }
        Ok(())
    }
    pub(crate) fn into_raw_components(self) -> (AllocSlice<T, Alloc>, usize, Alloc) {
        let slice = self.slice;
        // SAFETY: We forget `alloc` immediately.
        let alloc = unsafe { core::ptr::read(&self.alloc) };
        core::mem::forget(self);
        let capacity = if core::mem::size_of::<T>() == 0 || slice.is_empty() {
            0
        } else {
            // SAFETY: we store the capacity in the prefix when constructed.
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
impl<T, Alloc: IAlloc> core::ops::Deref for BoxedSlice<T, Alloc> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, Alloc: IAlloc> core::ops::DerefMut for BoxedSlice<T, Alloc> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_slice_mut()
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
            // SAFETY: the AllocSlice is initialized, storing to it is safe.
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
                        unsafe { core::mem::transmute::<usize, NonNull<T>>(usize::MAX) }
                    } else {
                        slice.start.ptr
                    },
                    alloc,
                },
            }
        }
    }
}
impl<T: Copy, Alloc: IAlloc + Default> From<&[T]> for BoxedSlice<T, Alloc> {
    fn from(value: &[T]) -> Self {
        Vec::from(value).into()
    }
}
impl<T, Alloc: IAlloc + Default> FromIterator<T> for BoxedSlice<T, Alloc> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Vec::from_iter(iter).into()
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

#[cfg(feature = "serde")]
mod serde_impl {
    use super::*;
    use crate::alloc::IAlloc;
    use serde::{Deserialize, Serialize};
    impl<T: Serialize, Alloc: IAlloc> Serialize for BoxedSlice<T, Alloc> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let slice: &[T] = self;
            slice.serialize(serializer)
        }
    }
    impl<'a, T: Deserialize<'a>, Alloc: IAlloc + Default> Deserialize<'a> for BoxedSlice<T, Alloc> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'a>,
        {
            crate::alloc::vec::Vec::deserialize(deserializer).map(Into::into)
        }
    }
    impl<Alloc: IAlloc> Serialize for BoxedStr<Alloc> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let slice: &str = self;
            slice.serialize(serializer)
        }
    }
    impl<'a, Alloc: IAlloc + Default> Deserialize<'a> for BoxedStr<Alloc> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'a>,
        {
            crate::alloc::string::String::deserialize(deserializer).map(Into::into)
        }
    }
}
