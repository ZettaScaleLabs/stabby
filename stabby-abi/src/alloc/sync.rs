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

use core::{
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    mem::ManuallyDrop,
    ptr::NonNull,
    sync::atomic::{AtomicPtr, AtomicUsize, Ordering},
};

use crate::{vtable::HasDropVt, Dyn, IStable, IntoDyn};

use super::{
    vec::{ptr_add, ptr_diff, Vec, VecInner},
    AllocPtr, AllocSlice, DefaultAllocator, IAlloc,
};

/// [`alloc::sync::Arc`](https://doc.rust-lang.org/stable/alloc/sync/struct.Arc.html), but ABI-stable.
#[crate::stabby]
pub struct Arc<T, Alloc: IAlloc = super::DefaultAllocator> {
    ptr: AllocPtr<T, Alloc>,
}
unsafe impl<T: Send + Sync, Alloc: IAlloc + Send + Sync> Send for Arc<T, Alloc> {}
unsafe impl<T: Send + Sync, Alloc: IAlloc + Send + Sync> Sync for Arc<T, Alloc> {}
const USIZE_TOP_BIT: usize = 1 << (core::mem::size_of::<usize>() as i32 * 8 - 1);

impl<T> Arc<T> {
    /// Attempts to allocate [`Self`], initializing it with `constructor`.
    ///
    /// Note that the allocation may or may not be zeroed.
    ///
    /// # Panics
    /// If the allocator fails to provide an appropriate allocation.
    pub fn make<F: FnOnce(&mut core::mem::MaybeUninit<T>)>(constructor: F) -> Self {
        Self::make_in(constructor, DefaultAllocator::new())
    }
    /// Attempts to allocate [`Self`] and store `value` in it.
    ///
    /// # Panics
    /// If the allocator fails to provide an appropriate allocation.
    pub fn new(value: T) -> Self {
        Self::new_in(value, DefaultAllocator::new())
    }
}

impl<T, Alloc: IAlloc> Arc<T, Alloc> {
    /// Attempts to allocate [`Self`], initializing it with `constructor`.
    ///
    /// # Errors
    /// Returns the constructor and the allocator in case of allocation failure.
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
            constructor(ptr.as_mut());
            ptr.prefix_mut().strong = AtomicUsize::new(1);
            ptr.prefix_mut().weak = AtomicUsize::new(1)
        }
        Ok(Self {
            ptr: unsafe { ptr.assume_init() },
        })
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
            constructor(ptr.as_mut());
            ptr.prefix_mut().strong = AtomicUsize::new(1);
            ptr.prefix_mut().weak = AtomicUsize::new(1)
        }
        Self {
            ptr: unsafe { ptr.assume_init() },
        }
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
    /// `this` MUST not be dangling, and have been obtained through [`Self::into_raw`].
    pub const unsafe fn from_raw(this: AllocPtr<T, Alloc>) -> Self {
        Self { ptr: this }
    }

    /// Provides a mutable reference to the internals if the strong and weak counts are both 1.
    pub fn get_mut(this: &mut Self) -> Option<&mut T> {
        if Self::is_unique(this) {
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
    /// Increments the strong count.
    /// # Safety
    /// `this` MUST be a valid pointer derived from `Self`
    pub unsafe fn increment_strong_count(this: *const T) -> usize {
        let ptr: AllocPtr<T, Alloc> = AllocPtr {
            ptr: NonNull::new_unchecked(this.cast_mut()),
            marker: core::marker::PhantomData,
        };
        unsafe { ptr.prefix() }
            .strong
            .fetch_add(1, Ordering::Relaxed)
    }
    /// Returns the weak count. Note that all Arcs to a same value share a Weak, so the weak count can never be 0.
    pub fn weak_count(this: &Self) -> usize {
        unsafe { this.ptr.prefix() }.weak.load(Ordering::Relaxed)
    }
    /// Increments the weak count, returning its previous value.
    pub fn increment_weak_count(this: &Self) -> usize {
        unsafe { this.ptr.prefix() }
            .weak
            .fetch_add(1, Ordering::Relaxed)
    }

    /// Returns a mutable reference to this `Arc`'s value, cloning that value into a new `Arc` if [`Self::get_mut`] would have failed.
    pub fn make_mut(&mut self) -> &mut T
    where
        T: Clone,
        Alloc: Clone,
    {
        if !Self::is_unique(self) {
            *self = Self::new_in(T::clone(self), unsafe { self.ptr.prefix() }.alloc.clone());
        }
        unsafe { Self::get_mut_unchecked(self) }
    }

    /// Whether or not `this` is the sole owner of its data, including weak owners.
    pub fn is_unique(this: &Self) -> bool {
        Self::strong_count(this) == 1 && Self::weak_count(this) == 1
    }
    /// Attempts the value from the allocation, freeing said allocation.
    /// # Errors
    /// Returns `this` if it's not the sole owner of its value.
    pub fn try_into_inner(this: Self) -> Result<T, Self> {
        if !Self::is_unique(&this) {
            Err(this)
        } else {
            let ret = unsafe { core::ptr::read(&*this) };
            _ = unsafe { Weak::<T, Alloc>::from_raw(Arc::into_raw(this)) };
            Ok(ret)
        }
    }

    /// Constructs an additional [`Weak`] pointer to `this`.
    pub fn downgrade(this: &Self) -> Weak<T, Alloc> {
        this.into()
    }
    /// Returns a reference to the allocator used to construct `this`
    pub const fn allocator(this: &Self) -> &Alloc {
        unsafe { &this.ptr.prefix().alloc }
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

/// [`alloc::sync::Weak`](https://doc.rust-lang.org/stable/alloc/sync/struct.Weak.html), but ABI-stable.
#[crate::stabby]
pub struct Weak<T, Alloc: IAlloc = super::DefaultAllocator> {
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
    /// `this` MUST not be dangling, and have been obtained through [`Self::into_raw`].
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
            let mut alloc = core::ptr::read(&self.ptr.prefix().alloc);
            self.ptr.free(&mut alloc)
        }
    }
}

/// A strong reference to a fixed size slice of elements.
///
/// Equivalent to `alloc::sync::Arc<[T]>`
#[crate::stabby]
pub struct ArcSlice<T, Alloc: IAlloc = super::DefaultAllocator> {
    pub(crate) inner: AllocSlice<T, Alloc>,
}
unsafe impl<T: Send + Sync, Alloc: IAlloc + Send + Sync> Send for ArcSlice<T, Alloc> {}
unsafe impl<T: Send + Sync, Alloc: IAlloc + Send + Sync> Sync for ArcSlice<T, Alloc> {}
unsafe impl<T: Send + Sync, Alloc: IAlloc + Send + Sync> Send for WeakSlice<T, Alloc> {}
unsafe impl<T: Send + Sync, Alloc: IAlloc + Send + Sync> Sync for WeakSlice<T, Alloc> {}

impl<T, Alloc: IAlloc> ArcSlice<T, Alloc> {
    /// Returns the number of elements in the slice.
    pub const fn len(&self) -> usize {
        ptr_diff(self.inner.end, self.inner.start.ptr)
    }
    /// Returns true if the slice is empty.
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Returns a borrow to the slice.
    pub fn as_slice(&self) -> &[T] {
        let start = self.inner.start;
        unsafe { core::slice::from_raw_parts(start.as_ptr(), self.len()) }
    }
    /// Returns a mutable borrow to the slice if no other references to it may exist.
    pub fn as_slice_mut(&mut self) -> Option<&mut [T]> {
        (ArcSlice::strong_count(self) == 1 && ArcSlice::weak_count(self) == 1)
            .then(|| unsafe { self.as_slice_mut_unchecked() })
    }
    /// Returns a mutable borrow to the slice.
    /// # Safety
    /// This can easily create aliased mutable references, which would be undefined behaviour.
    pub unsafe fn as_slice_mut_unchecked(&mut self) -> &mut [T] {
        let start = self.inner.start;
        unsafe { core::slice::from_raw_parts_mut(start.as_ptr(), self.len()) }
    }
    /// Returns the strong count to the slice.
    pub fn strong_count(this: &Self) -> usize {
        unsafe { this.inner.start.prefix().strong.load(Ordering::Relaxed) }
    }
    /// Returns the weak count to the slice.
    pub fn weak_count(this: &Self) -> usize {
        unsafe { this.inner.start.prefix().weak.load(Ordering::Relaxed) }
    }
    /// Whether or not `this` is the sole owner of its data, including weak owners.
    pub fn is_unique(this: &Self) -> bool {
        Self::strong_count(this) == 1 && Self::weak_count(this) == 1
    }
    /// Returns the slice's raw representation, without altering the associated reference counts.
    ///
    /// Failing to reconstruct the `this` using [`Self::from_raw`] will result in the associated `this` being effectively leaked.
    pub fn into_raw(this: Self) -> AllocSlice<T, Alloc> {
        let inner = this.inner;
        core::mem::forget(this);
        inner
    }
    /// Reconstructs an [`ArcSlice`] from its raw representation, without altering the associated reference counts.
    ///
    /// # Safety
    /// `this` MUST have been obtained using [`Self::into_raw`], and not have been previously used to reconstruct an [`ArcSlice`].
    pub unsafe fn from_raw(this: AllocSlice<T, Alloc>) -> Self {
        Self { inner: this }
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
        unsafe { self.inner.start.prefix() }
            .strong
            .fetch_add(1, Ordering::Relaxed);
        Self { inner: self.inner }
    }
}
impl<T, Alloc: IAlloc> From<Arc<T, Alloc>> for ArcSlice<T, Alloc> {
    fn from(mut value: Arc<T, Alloc>) -> Self {
        unsafe { value.ptr.prefix_mut() }.capacity = AtomicUsize::new(1);
        Self {
            inner: AllocSlice {
                start: value.ptr,
                end: ptr_add(value.ptr.ptr, 1),
            },
        }
    }
}
impl<T: Copy, Alloc: IAlloc + Default> From<&[T]> for ArcSlice<T, Alloc> {
    fn from(value: &[T]) -> Self {
        Vec::from(value).into()
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
            Self {
                inner: AllocSlice {
                    start: slice.start,
                    end: slice.end,
                },
            }
        } else {
            let mut start = AllocPtr::alloc_array(&mut alloc, 0).expect("Allocation failed");
            unsafe {
                start.prefix_mut().strong = AtomicUsize::new(1);
                start.prefix_mut().weak = AtomicUsize::new(1);
                start.prefix_mut().capacity = if core::mem::size_of::<T>() != 0 {
                    AtomicUsize::new(0)
                } else {
                    AtomicUsize::new(ptr_diff(
                        core::mem::transmute::<usize, NonNull<u8>>(usize::MAX),
                        start.ptr.cast::<u8>(),
                    ))
                };
                core::ptr::write(&mut slice.start.prefix_mut().alloc, alloc);
            }
            Self {
                inner: AllocSlice {
                    start,
                    end: ptr_add(start.ptr.cast::<u8>(), slice.len()).cast(),
                },
            }
        }
    }
}
impl<T, Alloc: IAlloc> TryFrom<ArcSlice<T, Alloc>> for Vec<T, Alloc> {
    type Error = ArcSlice<T, Alloc>;
    fn try_from(value: ArcSlice<T, Alloc>) -> Result<Self, Self::Error> {
        if core::mem::size_of::<T>() == 0 || !ArcSlice::is_unique(&value) {
            Err(value)
        } else {
            unsafe {
                let ret = Vec {
                    inner: VecInner {
                        start: value.inner.start,
                        end: value.inner.end,
                        capacity: ptr_add(
                            value.inner.start.ptr,
                            value.inner.start.prefix().capacity.load(Ordering::Relaxed),
                        ),
                        alloc: core::ptr::read(&value.inner.start.prefix().alloc),
                    },
                };
                core::mem::forget(value);
                Ok(ret)
            }
        }
    }
}
impl<T: Eq, Alloc: IAlloc> Eq for ArcSlice<T, Alloc> {}
impl<T: PartialEq, Alloc: IAlloc> PartialEq for ArcSlice<T, Alloc> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}
impl<T: Ord, Alloc: IAlloc> Ord for ArcSlice<T, Alloc> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_slice().cmp(other.as_slice())
    }
}
impl<T: PartialOrd, Alloc: IAlloc> PartialOrd for ArcSlice<T, Alloc> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.as_slice().partial_cmp(other.as_slice())
    }
}
impl<T: Hash, Alloc: IAlloc> Hash for ArcSlice<T, Alloc> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state)
    }
}
impl<T, Alloc: IAlloc> Drop for ArcSlice<T, Alloc> {
    fn drop(&mut self) {
        if unsafe { self.inner.start.prefix() }
            .strong
            .fetch_sub(1, Ordering::Relaxed)
            != 1
        {
            return;
        }
        unsafe { core::ptr::drop_in_place(self.as_slice_mut_unchecked()) }
        _ = WeakSlice { inner: self.inner };
    }
}
impl<T: Debug, Alloc: IAlloc> Debug for ArcSlice<T, Alloc> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.as_slice().fmt(f)
    }
}
impl<T: core::fmt::LowerHex, Alloc: IAlloc> core::fmt::LowerHex for ArcSlice<T, Alloc> {
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
impl<T: core::fmt::UpperHex, Alloc: IAlloc> core::fmt::UpperHex for ArcSlice<T, Alloc> {
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
impl<'a, T, Alloc: IAlloc> IntoIterator for &'a ArcSlice<T, Alloc> {
    type Item = &'a T;
    type IntoIter = core::slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

/// A weak reference counted slice.
#[crate::stabby]
pub struct WeakSlice<T, Alloc: IAlloc = super::DefaultAllocator> {
    pub(crate) inner: AllocSlice<T, Alloc>,
}

impl<T, Alloc: IAlloc> WeakSlice<T, Alloc> {
    /// Return a strong reference to the slice if it hasn't been destroyed yet.
    pub fn upgrade(&self) -> Option<ArcSlice<T, Alloc>> {
        let strong = &unsafe { self.inner.start.prefix() }.strong;
        let count = strong.fetch_or(USIZE_TOP_BIT, Ordering::Acquire);
        match count {
            0 | USIZE_TOP_BIT => {
                strong.store(0, Ordering::Release);
                None
            }
            _ => {
                strong.fetch_add(1, Ordering::Release);
                strong.fetch_and(!USIZE_TOP_BIT, Ordering::Release);
                Some(ArcSlice { inner: self.inner })
            }
        }
    }
    /// For types that are [`Copy`], the slice actually remains valid even after all strong references
    /// have been dropped as long as at least a weak reference lives on.
    ///
    /// If you're using this, there are probably design issues in your program...
    pub fn force_upgrade(&self) -> ArcSlice<T, Alloc>
    where
        T: Copy,
    {
        let strong = &unsafe { self.inner.start.prefix() }.strong;
        match strong.fetch_add(1, Ordering::Release) {
            0 | USIZE_TOP_BIT => {
                unsafe { self.inner.start.prefix() }
                    .weak
                    .fetch_add(1, Ordering::Relaxed);
            }
            _ => {}
        }
        ArcSlice { inner: self.inner }
    }
}
impl<T, Alloc: IAlloc> Clone for WeakSlice<T, Alloc> {
    fn clone(&self) -> Self {
        unsafe { self.inner.start.prefix() }
            .weak
            .fetch_add(1, Ordering::Relaxed);
        Self { inner: self.inner }
    }
}
impl<T, Alloc: IAlloc> From<&ArcSlice<T, Alloc>> for WeakSlice<T, Alloc> {
    fn from(value: &ArcSlice<T, Alloc>) -> Self {
        unsafe { value.inner.start.prefix() }
            .weak
            .fetch_add(1, Ordering::Relaxed);
        Self { inner: value.inner }
    }
}
impl<T, Alloc: IAlloc> Drop for WeakSlice<T, Alloc> {
    fn drop(&mut self) {
        if unsafe { self.inner.start.prefix() }
            .weak
            .fetch_sub(1, Ordering::Relaxed)
            != 1
        {
            return;
        }
        let mut alloc = unsafe { core::ptr::read(&self.inner.start.prefix().alloc) };
        unsafe { self.inner.start.free(&mut alloc) }
    }
}
pub use super::string::{ArcStr, WeakStr};

impl<T, Alloc: IAlloc> crate::IPtr for Arc<T, Alloc> {
    unsafe fn as_ref<U: Sized>(&self) -> &U {
        self.ptr.cast().as_ref()
    }
}
impl<T, Alloc: IAlloc> crate::IPtrClone for Arc<T, Alloc> {
    fn clone(this: &Self) -> Self {
        this.clone()
    }
}

impl<T, Alloc: IAlloc> crate::IPtrTryAsMut for Arc<T, Alloc> {
    unsafe fn try_as_mut<U: Sized>(&mut self) -> Option<&mut U> {
        Self::get_mut(self).map(|r| unsafe { core::mem::transmute::<&mut T, &mut U>(r) })
    }
}
impl<T, Alloc: IAlloc> crate::IPtrOwned for Arc<T, Alloc> {
    fn drop(this: &mut core::mem::ManuallyDrop<Self>, drop: unsafe extern "C" fn(&mut ())) {
        if unsafe { this.ptr.prefix() }
            .strong
            .fetch_sub(1, Ordering::Relaxed)
            != 1
        {
            return;
        }
        unsafe {
            drop(this.ptr.cast().as_mut());
            _ = Weak::<T, Alloc>::from_raw(this.ptr);
        }
    }
}

impl<T, Alloc: IAlloc> IntoDyn for Arc<T, Alloc> {
    type Anonymized = Arc<(), Alloc>;
    type Target = T;
    fn anonimize(self) -> Self::Anonymized {
        let original_prefix = self.ptr.prefix_ptr();
        let anonymized = unsafe { core::mem::transmute::<Self, Self::Anonymized>(self) };
        let anonymized_prefix = anonymized.ptr.prefix_ptr();
        assert_eq!(anonymized_prefix, original_prefix, "The allocation prefix was lost in anonimization, this is definitely a bug, please report it.");
        anonymized
    }
}

impl<T, Alloc: IAlloc> crate::IPtrOwned for Weak<T, Alloc> {
    fn drop(this: &mut core::mem::ManuallyDrop<Self>, drop: unsafe extern "C" fn(&mut ())) {
        if unsafe { this.ptr.prefix() }
            .strong
            .fetch_sub(1, Ordering::Relaxed)
            != 1
        {
            return;
        }
        unsafe {
            drop(this.ptr.cast().as_mut());
            _ = Weak::<T, Alloc>::from_raw(this.ptr);
        }
    }
}

impl<T, Alloc: IAlloc> crate::IPtrClone for Weak<T, Alloc> {
    fn clone(this: &Self) -> Self {
        this.clone()
    }
}

impl<T, Alloc: IAlloc> IntoDyn for Weak<T, Alloc> {
    type Anonymized = Weak<(), Alloc>;
    type Target = T;
    fn anonimize(self) -> Self::Anonymized {
        let original_prefix = self.ptr.prefix_ptr();
        let anonymized = unsafe { core::mem::transmute::<Self, Self::Anonymized>(self) };
        let anonymized_prefix = anonymized.ptr.prefix_ptr();
        assert_eq!(anonymized_prefix, original_prefix, "The allocation prefix was lost in anonimization, this is definitely a bug, please report it.");
        anonymized
    }
}

impl<'a, Vt: HasDropVt, Alloc: IAlloc> From<&'a Dyn<'a, Arc<(), Alloc>, Vt>>
    for Dyn<'a, Weak<(), Alloc>, Vt>
{
    fn from(value: &'a Dyn<'a, Arc<(), Alloc>, Vt>) -> Self {
        Self {
            ptr: ManuallyDrop::new(Arc::downgrade(&value.ptr)),
            vtable: value.vtable,
            unsend: core::marker::PhantomData,
        }
    }
}
impl<'a, Vt: HasDropVt + IStable, Alloc: IAlloc> Dyn<'a, Weak<(), Alloc>, Vt> {
    /// Attempts to upgrade a weak trait object to a strong one.
    pub fn upgrade(self) -> crate::option::Option<Dyn<'a, Arc<(), Alloc>, Vt>> {
        let Some(ptr) = self.ptr.upgrade() else {
            return crate::option::Option::None();
        };
        crate::option::Option::Some(Dyn {
            ptr: ManuallyDrop::new(ptr),
            vtable: self.vtable,
            unsend: core::marker::PhantomData,
        })
    }
}

#[crate::stabby]
/// An owner of an [`Arc<T, Alloc>`] whose pointee can be atomically changed.
pub struct AtomicArc<T, Alloc: IAlloc> {
    ptr: AtomicPtr<T>,
    alloc: core::marker::PhantomData<*const Alloc>,
}
unsafe impl<T: Send + Sync, Alloc: IAlloc + Send + Sync> Send for AtomicArc<T, Alloc> {}
unsafe impl<T: Send + Sync, Alloc: IAlloc + Send + Sync> Sync for AtomicArc<T, Alloc> {}

impl<T, Alloc: IAlloc> Drop for AtomicArc<T, Alloc> {
    fn drop(&mut self) {
        let ptr = self.ptr.load(Ordering::Relaxed);
        if let Some(ptr) = NonNull::new(ptr) {
            unsafe {
                Arc::<T, Alloc>::from_raw(AllocPtr {
                    ptr,
                    marker: PhantomData,
                })
            };
        }
    }
}

type MaybeArc<T, Alloc> = Option<Arc<T, Alloc>>;
impl<T, Alloc: IAlloc> AtomicArc<T, Alloc> {
    /// Constructs a new [`AtomicArc`] set to the provided value.
    pub const fn new(value: MaybeArc<T, Alloc>) -> Self {
        Self {
            ptr: AtomicPtr::new(unsafe {
                core::mem::transmute::<Option<Arc<T, Alloc>>, *mut T>(value)
            }),
            alloc: PhantomData,
        }
    }
    /// Atomically load the current value.
    pub fn load(&self, order: Ordering) -> MaybeArc<T, Alloc> {
        let ptr = NonNull::new(self.ptr.load(order))?;
        unsafe {
            Arc::<T, Alloc>::increment_strong_count(ptr.as_ptr());
            Some(Arc::from_raw(AllocPtr {
                ptr,
                marker: PhantomData,
            }))
        }
    }
    /// Atomically store a new value.
    pub fn store(&self, value: MaybeArc<T, Alloc>, order: Ordering) {
        let ptr = value.map_or(core::ptr::null_mut(), |value| Arc::into_raw(value).as_ptr());
        self.ptr.store(ptr, order)
    }
    /// Compares `self` with `current` by pointer.
    /// # Errors
    /// Returns the new value of `self` if it differs from `current`.
    pub fn is(
        &self,
        current: Option<&Arc<T, Alloc>>,
        order: Ordering,
    ) -> Result<(), MaybeArc<T, Alloc>> {
        let ptr = NonNull::new(self.ptr.load(order));
        match (ptr, current) {
            (None, None) => Ok(()),
            (None, _) => Err(None),
            (Some(ptr), Some(current)) if core::ptr::eq(ptr.as_ptr(), current.ptr.as_ptr()) => {
                Ok(())
            }
            (Some(ptr), _) => unsafe {
                Arc::<T, Alloc>::increment_strong_count(ptr.as_ptr());
                Err(Some(Arc::from_raw(AllocPtr {
                    ptr,
                    marker: PhantomData,
                })))
            },
        }
    }
    /// Replace the current value with the new value.
    /// # Errors
    /// If `current` no longer points to the same value as `self`, it
    pub fn compare_exchange(
        &self,
        current: Option<&Arc<T, Alloc>>,
        new: MaybeArc<T, Alloc>,
        success: Ordering,
        failure: Ordering,
    ) -> Result<MaybeArc<T, Alloc>, MaybeArc<T, Alloc>> {
        let current = current.map_or(core::ptr::null_mut(), |value| value.ptr.ptr.as_ptr());
        let new = new.map_or(core::ptr::null_mut(), |value| Arc::into_raw(value).as_ptr());
        match self.ptr.compare_exchange(current, new, success, failure) {
            Ok(ptr) => Ok(NonNull::new(ptr).map(|ptr| unsafe {
                Arc::from_raw(AllocPtr {
                    ptr,
                    marker: PhantomData,
                })
            })),
            Err(ptr) => Err(NonNull::new(ptr).map(|ptr| unsafe {
                Arc::<T, Alloc>::increment_strong_count(ptr.as_ptr());
                Arc::from_raw(AllocPtr {
                    ptr,
                    marker: PhantomData,
                })
            })),
        }
    }
}
