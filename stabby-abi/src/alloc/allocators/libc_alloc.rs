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

use crate::alloc::Layout;

#[cfg(not(windows))]
use libc::posix_memalign;
#[cfg(windows)]
unsafe fn posix_memalign(this: &mut *mut core::ffi::c_void, size: usize, align: usize) -> i32 {
    // SAFETY: `aligned_malloc` is always safe.
    let ptr = unsafe { libc::aligned_malloc(size, align) };
    if ptr.is_null() {
        return libc::ENOMEM;
    }
    *this = ptr;
    0
}
#[cfg(windows)]
use libc::aligned_free;
#[cfg(not(windows))]
use libc::free as aligned_free;
use libc::realloc;

/// An allocator based on `libc::posix_memalign` or `libc::aligned_malloc` depending on the platform.
///
/// It has all of `malloc`'s usual properties.
#[crate::stabby]
#[derive(Clone, Copy, Default)]
pub struct LibcAlloc {
    inner: [u8; 0],
}
impl core::fmt::Debug for LibcAlloc {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("LibcAlloc")
    }
}
impl LibcAlloc {
    /// Constructs the allocator.
    pub const fn new() -> Self {
        Self { inner: [] }
    }
}

impl crate::alloc::IAlloc for LibcAlloc {
    fn alloc(&mut self, layout: Layout) -> *mut () {
        if layout.size == 0 {
            return core::ptr::null_mut();
        }
        let mut ptr = core::ptr::null_mut();
        // SAFETY: `posix_memalign` is always safe.
        let err = unsafe { posix_memalign(&mut ptr, layout.align, layout.size) };
        if err != 0 && (ptr as usize % layout.align != 0) {
            ptr = core::ptr::null_mut();
        }
        ptr.cast()
    }
    unsafe fn free(&mut self, ptr: *mut ()) {
        // SAFETY: `aligned_free` must be called by a pointer allocated by the corresponding allocator, which is already a safety condition of `IAlloc::free`
        unsafe { aligned_free(ptr.cast()) }
    }
    unsafe fn realloc(&mut self, ptr: *mut (), prev: Layout, new_size: usize) -> *mut () {
        if new_size == 0 {
            return core::ptr::null_mut();
        }
        let mut new_ptr = core::ptr::null_mut::<core::ffi::c_void>();
        if prev.align <= 8 {
            // SAFETY: `realloc` must be called by a pointer allocated by the corresponding allocator, which is already a safety condition of `IAlloc::free`. It may also fail if the alignment is not correct on some systems, so we avoid calling it if it's too great.
            new_ptr = unsafe { realloc(ptr.cast(), new_size) };
        };
        if new_ptr.is_null() {
            // SAFETY: `posix_memalign` is always safe.
            let err = unsafe { posix_memalign(&mut new_ptr, prev.align, new_size) };
            if err == 0 {
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        ptr.cast::<u8>(),
                        new_ptr.cast::<u8>(),
                        prev.size,
                    )
                }
                self.free(ptr.cast());
            }
        }
        new_ptr.cast()
    }
}
