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

use super::Layout;

/// An allocator based on `libc::malloc`.
///
/// It has all of `malloc`'s usual properties.
#[crate::stabby]
#[derive(Clone, Copy, Debug, Default)]
pub struct LibcAlloc;
impl LibcAlloc {
    pub fn errno(&self) -> i32 {
        unsafe { *libc::__errno_location() }
    }
}
impl super::IAlloc for LibcAlloc {
    fn alloc(&mut self, layout: Layout) -> *mut () {
        if layout.size == 0 {
            return core::ptr::null_mut();
        }
        let mut ptr = unsafe { libc::memalign(layout.align, layout.size).cast::<()>() };
        if (!ptr.is_null()) && (ptr as usize % layout.align != 0) {
            ptr = core::ptr::null_mut();
        }
        ptr
    }
    unsafe fn free(&mut self, ptr: *mut ()) {
        unsafe { libc::free(ptr.cast()) }
    }
    unsafe fn realloc(&mut self, ptr: *mut (), new_layout: Layout) -> *mut () {
        if new_layout.size == 0 {
            return core::ptr::null_mut();
        }
        let mut new_ptr = unsafe { libc::realloc(ptr.cast(), new_layout.size).cast::<()>() };
        if new_ptr as usize % new_layout.align != 0 {
            let ptr = unsafe { libc::memalign(new_layout.align, new_layout.size).cast::<()>() };
            unsafe {
                core::ptr::copy_nonoverlapping(
                    new_ptr.cast::<u8>(),
                    ptr.cast::<u8>(),
                    new_layout.size,
                )
            }
            self.free(new_ptr);
            new_ptr = ptr;
        }
        new_ptr
    }
}
