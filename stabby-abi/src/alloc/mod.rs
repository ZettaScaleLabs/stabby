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

use core::{marker::PhantomData, ptr::NonNull, sync::atomic::AtomicUsize};

use self::vec::ptr_diff;

#[cfg(feature = "libc")]
pub mod libc_alloc;

#[crate::stabby]
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct AllocationError();
impl core::fmt::Display for AllocationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("AllocationError")
    }
}
#[cfg(feature = "std")]
impl std::error::Error for AllocationError {}

pub mod boxed;
pub mod string;
pub mod sync;
pub mod vec;

#[cfg(feature = "libc")]
pub type DefaultAllocator = libc_alloc::LibcAlloc;
#[cfg(not(feature = "libc"))]
pub type DefaultAllocator = core::convert::Intry;

#[crate::stabby]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// ABI-stable equivalent of std::mem::Layout
pub struct Layout {
    pub size: usize,
    pub align: usize,
}
impl Layout {
    /// Returns the [`Layout`] corresponding to `T`
    pub const fn of<T: Sized>() -> Self {
        Layout {
            size: core::mem::size_of::<T>(),
            align: core::mem::align_of::<T>(),
        }
    }
    /// Returns the [`Layout`] corresponding to `[T; n]`.
    ///
    /// Note that while this ensures that even if `T`'s size is not a multiple of its alignment,
    /// the layout will have sufficient memory to store `n` of `T` in an aligned fashion.
    pub const fn array<T: Sized>(n: usize) -> Self {
        let Self { mut size, align } = Self::of::<T>();
        let sizemodalign = size % align;
        if sizemodalign != 0 {
            size += align;
            size -= sizemodalign;
        }
        size *= n;
        Layout { size, align }
    }
    /// Concatenates a layout to `self`, ensuring that alignment padding is taken into account.
    pub const fn concat(mut self, other: Self) -> Self {
        let sizemodalign = self.size % other.align;
        if sizemodalign != 0 {
            self.size += other.align;
            self.size -= sizemodalign;
        }
        self.size += other.size;
        self
    }
    #[inline]
    pub fn next_matching<T>(self, ptr: *mut T) -> *mut T {
        fn next_matching(align: usize, ptr: *mut u8) -> *mut u8 {
            unsafe { ptr.add(ptr.align_offset(align)) }
        }
        next_matching(self.align, ptr.cast()).cast()
    }
}

/// An interface to an allocator.
///
/// Note that `stabby` often stores allocators inside allocations they made, so allocators that cannot allocate
/// more than their size on stack will systematically fail to construct common stabby types.
///
/// Since the allocator may be moved, it must also be safe to do so, including after it has performed allocations.
pub trait IAlloc: Unpin {
    /// Allocates at least as much memory as requested by layout, ensuring the requested alignment is respected.
    ///
    /// If the requested size is 0, or allocation failed, then a null pointer is returned.
    fn alloc(&mut self, layout: Layout) -> *mut ();
    /// Frees the allocation
    ///
    /// # Safety
    /// `ptr` MUST have been allocated through a succesful call to `Self::alloc` or `Self::realloc` with the same instance of `Self`
    unsafe fn free(&mut self, ptr: *mut ());
    /// Reallocates `ptr`, ensuring that it has enough memory for the newly requested layout.
    ///
    /// If the requested size is 0, or allocation failed, then a null pointer is returned, and `ptr` is not freed.
    ///
    /// # Safety
    /// `ptr` MUST have been allocated through a succesful call to `Self::alloc` with the same instance of `Self`
    unsafe fn realloc(&mut self, ptr: *mut (), new_layout: Layout) -> *mut () {
        let ret = self.alloc(new_layout);
        if !ret.is_null() {
            unsafe {
                core::ptr::copy_nonoverlapping(ptr.cast::<u8>(), ret.cast(), new_layout.size);
                self.free(ptr);
            }
        }
        ret
    }
}

/// An ABI stable equivalent to [`IAlloc`].
#[crate::stabby]
pub trait IStableAlloc: Unpin {
    /// Allocates at least as much memory as requested by layout, ensuring the requested alignment is respected.
    ///
    /// If the requested size is 0, or allocation failed, then a null pointer is returned.
    extern "C" fn alloc(&mut self, layout: Layout) -> *mut ();
    /// Frees the allocation
    ///
    /// # Safety
    /// `ptr` MUST have been allocated through a succesful call to `Self::alloc` or `Self::realloc` with the same instance of `Self`
    extern "C" fn free(&mut self, ptr: *mut ());
    /// Reallocates `ptr`, ensuring that it has enough memory for the newly requested layout.
    ///
    /// If the requested size is 0, or allocation failed, then a null pointer is returned, and `ptr` is not freed.
    ///
    /// # Safety
    /// `ptr` MUST have been allocated through a succesful call to `Self::alloc` with the same instance of `Self`
    extern "C" fn realloc(&mut self, ptr: *mut (), new_layout: Layout) -> *mut () {
        let ret = self.alloc(new_layout);
        if !ret.is_null() {
            unsafe {
                core::ptr::copy_nonoverlapping(ptr.cast::<u8>(), ret.cast(), new_layout.size);
                self.free(ptr);
            }
        }
        ret
    }
}
#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl<T: IAlloc> IStableAlloc for T {
    extern "C" fn alloc(&mut self, layout: Layout) -> *mut () {
        IAlloc::alloc(self, layout)
    }
    extern "C" fn free(&mut self, ptr: *mut ()) {
        unsafe { IAlloc::free(self, ptr) }
    }
    extern "C" fn realloc(&mut self, ptr: *mut (), layout: Layout) -> *mut () {
        unsafe { IAlloc::realloc(self, ptr, layout) }
    }
}

impl<T: IStableAllocDynMut<crate::vtable::H> + Unpin> IAlloc for T {
    fn alloc(&mut self, layout: Layout) -> *mut () {
        IStableAllocDynMut::alloc(self, layout)
    }
    unsafe fn free(&mut self, ptr: *mut ()) {
        IStableAllocDynMut::free(self, ptr)
    }
    unsafe fn realloc(&mut self, ptr: *mut (), new_layout: Layout) -> *mut () {
        IStableAllocDynMut::realloc(self, ptr, new_layout)
    }
}
impl IAlloc for core::convert::Infallible {
    fn alloc(&mut self, _layout: Layout) -> *mut () {
        unreachable!()
    }
    unsafe fn free(&mut self, _ptr: *mut ()) {
        unreachable!()
    }
}

/// The prefix common to all allocations in [`stabby::realloc`].
///
/// This allows reuse of allocations when converting between container types.
#[crate::stabby]
pub struct AllocPrefix<Alloc> {
    pub strong: core::sync::atomic::AtomicUsize,
    pub weak: core::sync::atomic::AtomicUsize,
    pub capacity: core::sync::atomic::AtomicUsize,
    pub alloc: Alloc,
}
impl<Alloc> AllocPrefix<Alloc> {
    pub const fn skip_to<T>() -> usize {
        let mut size = core::mem::size_of::<Self>();
        let align = core::mem::align_of::<T>();
        let sizemodalign = size % align;
        if sizemodalign != 0 {
            size += align;
            size -= sizemodalign;
        }
        size
    }
}

/// A non-null pointer guaranteed to be preceded by a valid
/// [`AllocPrefix`] unless the pointer is dangling.
#[crate::stabby]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AllocPtr<T, Alloc> {
    pub ptr: NonNull<T>,
    pub marker: PhantomData<Alloc>,
}
impl<T, Alloc> Copy for AllocPtr<T, Alloc> {}
impl<T, Alloc> Clone for AllocPtr<T, Alloc> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T, Alloc> core::ops::Deref for AllocPtr<T, Alloc> {
    type Target = NonNull<T>;
    fn deref(&self) -> &Self::Target {
        &self.ptr
    }
}
impl<T, Alloc> core::ops::DerefMut for AllocPtr<T, Alloc> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ptr
    }
}
impl<T, Alloc> AllocPtr<T, Alloc> {
    /// Constructs a dangling pointer.
    pub const fn dangling() -> Self {
        Self {
            ptr: NonNull::dangling(),
            marker: PhantomData,
        }
    }
    /// The offset between `self.ptr` and the prefix.
    pub const fn prefix_skip() -> usize {
        AllocPrefix::<Alloc>::skip_to::<T>()
    }
    ///The pointer to the prefix for this allocation
    const fn prefix_ptr(&self) -> NonNull<AllocPrefix<Alloc>> {
        unsafe {
            NonNull::new_unchecked(
                self.ptr
                    .as_ptr()
                    .cast::<u8>()
                    .sub(Self::prefix_skip())
                    .cast(),
            )
        }
    }
    /// A reference to the prefix for this allocation.
    /// # Safety
    /// `self` must not be dangling, and have been properly allocated, using [`Self::alloc`] or [`Self::realloc`] for example.
    pub unsafe fn prefix(&self) -> &AllocPrefix<Alloc> {
        unsafe { self.prefix_ptr().as_ref() }
    }
    /// A mutable reference to the prefix for this allocation.
    /// # Safety
    /// `self` must not be dangling, and have been properly allocated, using [`Self::alloc`] or [`Self::realloc`] for example.
    /// Since this type is [`Copy`], the `&mut self` is not a sufficient guarantee of uniqueness.
    pub unsafe fn prefix_mut(&mut self) -> &mut AllocPrefix<Alloc> {
        unsafe { self.prefix_ptr().as_mut() }
    }
}
impl<T, Alloc: IAlloc> AllocPtr<T, Alloc> {
    /// Allocates a pointer to a single element of `T`, prefixed by an [`AllocPrefix`]
    pub fn alloc(alloc: &mut Alloc) -> Option<Self> {
        let ptr = alloc.alloc(Layout::of::<AllocPrefix<Alloc>>().concat(Layout::of::<T>()));
        NonNull::new(ptr).map(|ptr| unsafe {
            ptr.cast::<AllocPrefix<Alloc>>().as_mut().capacity = AtomicUsize::new(1);
            Self {
                ptr: NonNull::new_unchecked(
                    ptr.as_ptr().cast::<u8>().add(Self::prefix_skip()).cast(),
                ),
                marker: PhantomData,
            }
        })
    }
    /// Allocates a pointer to an array of `capacity` `T`, prefixed by an [`AllocPrefix`]
    pub fn alloc_array(alloc: &mut Alloc, capacity: usize) -> Option<Self> {
        let ptr =
            alloc.alloc(Layout::of::<AllocPrefix<Alloc>>().concat(Layout::array::<T>(capacity)));
        NonNull::new(ptr).map(|ptr| unsafe {
            ptr.cast::<AllocPrefix<Alloc>>().as_mut().capacity = AtomicUsize::new(capacity);
            Self {
                ptr: NonNull::new_unchecked(
                    ptr.as_ptr().cast::<u8>().add(Self::prefix_skip()).cast(),
                ),
                marker: PhantomData,
            }
        })
    }
    /// Reallocates a pointer to an array of `capacity` `T`, prefixed by an [`AllocPrefix`].
    ///
    /// In case of failure of the allocator, this will return `None` and `self` will not have been freed.
    ///
    /// # Safety
    /// `self` must not be dangling
    pub unsafe fn realloc(self, alloc: &mut Alloc, capacity: usize) -> Option<Self> {
        let ptr = alloc.realloc(
            self.prefix() as *const _ as *mut _,
            Layout::of::<AllocPrefix<Alloc>>().concat(Layout::array::<T>(capacity)),
        );
        NonNull::new(ptr).map(|ptr| unsafe {
            ptr.cast::<AllocPrefix<Alloc>>().as_mut().capacity = AtomicUsize::new(capacity);
            Self {
                ptr: NonNull::new_unchecked(
                    ptr.as_ptr().cast::<u8>().add(Self::prefix_skip()).cast(),
                ),
                marker: PhantomData,
            }
        })
    }
    /// Reallocates a pointer to an array of `capacity` `T`, prefixed by an [`AllocPrefix`]
    /// # Safety
    /// `self` must not be dangling, and is freed after this returns.
    pub unsafe fn free(self, alloc: &mut Alloc) {
        alloc.free(self.prefix() as *const _ as *mut _)
    }
}

/// A helper to work with allocated slices.
#[crate::stabby]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AllocSlice<T, Alloc> {
    pub start: AllocPtr<T, Alloc>,
    pub end: NonNull<T>,
}
impl<T, Alloc> AllocSlice<T, Alloc> {
    pub const fn len(&self) -> usize {
        ptr_diff(self.end, self.start.ptr)
    }
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
impl<T, Alloc> Copy for AllocSlice<T, Alloc> {}
impl<T, Alloc> Clone for AllocSlice<T, Alloc> {
    fn clone(&self) -> Self {
        *self
    }
}
