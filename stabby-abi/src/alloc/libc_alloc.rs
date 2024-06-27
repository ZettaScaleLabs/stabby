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

#[cfg(not(any(windows, target_arch = "wasm32")))]
use libc::posix_memalign;
#[cfg(windows)]
unsafe fn posix_memalign(this: &mut *mut core::ffi::c_void, size: usize, align: usize) -> i32 {
    let ptr = unsafe { libc::aligned_malloc(size, align) };
    if ptr.is_null() {
        return libc::ENOMEM;
    }
    *this = ptr;
    0
}
#[cfg(windows)]
use libc::aligned_free;
#[cfg(not(any(windows, target_arch = "wasm32")))]
use libc::free as aligned_free;
#[cfg(not(target_arch = "wasm32"))]
use libc::realloc;
#[cfg(target_arch = "wasm32")]
use wasm32_alloc::{free as aligned_free, posix_memalign, realloc};

/// An allocator based on `libc::posix_memalign` or `libc::aligned_malloc` depending on the platform.
///
/// It has all of `malloc`'s usual properties.
#[crate::stabby]
#[derive(Clone, Copy, Debug, Default)]
pub struct LibcAlloc {
    inner: [u8; 0],
}
impl LibcAlloc {
    /// Constructs the allocator.
    pub const fn new() -> Self {
        Self { inner: [] }
    }
}

impl super::IAlloc for LibcAlloc {
    fn alloc(&mut self, layout: Layout) -> *mut () {
        if layout.size == 0 {
            return core::ptr::null_mut();
        }
        let mut ptr = core::ptr::null_mut();
        let err = unsafe { posix_memalign(&mut ptr, layout.align, layout.size) };
        if err != 0 && (ptr as usize % layout.align != 0) {
            ptr = core::ptr::null_mut();
        }
        ptr.cast()
    }
    unsafe fn free(&mut self, ptr: *mut ()) {
        unsafe { aligned_free(ptr.cast()) }
    }
    unsafe fn realloc(&mut self, ptr: *mut (), new_layout: Layout) -> *mut () {
        if new_layout.size == 0 {
            return core::ptr::null_mut();
        }
        let mut new_ptr = unsafe { realloc(ptr.cast(), new_layout.size) };
        if new_ptr.is_null() || new_ptr as usize % new_layout.align != 0 {
            let mut ptr = core::ptr::null_mut();
            let err = unsafe { posix_memalign(&mut ptr, new_layout.align, new_layout.size) };
            if err == 0 {
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        new_ptr.cast::<u8>(),
                        ptr.cast::<u8>(),
                        new_layout.size,
                    )
                }
                self.free(new_ptr.cast());
                new_ptr = ptr;
            }
        }
        new_ptr.cast()
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm32_alloc {
    use core::{
        ffi::c_void,
        mem::MaybeUninit,
        sync::atomic::{AtomicPtr, Ordering},
    };

    #[repr(C)]
    struct Slot {
        size: usize,
        lower: Option<&'static mut Slot>,
        padding: usize,
        _reserved: usize,
    }
    impl core::cmp::Ord for Slot {
        fn cmp(&self, other: &Self) -> core::cmp::Ordering {
            (self as *const Self).cmp(&(other as *const Self))
        }
    }
    impl core::cmp::PartialOrd for Slot {
        fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }
    impl core::cmp::Eq for Slot {}
    impl core::cmp::PartialEq for Slot {
        fn eq(&self, other: &Self) -> bool {
            core::ptr::eq(self, other)
        }
    }
    impl Slot {
        const fn full_size(&self) -> usize {
            core::mem::size_of::<Self>() + self.size
        }
        const fn start(&self) -> *const u8 {
            unsafe { (self as *const Self).cast::<u8>().sub(self.padding) }
        }
        const fn end(&self) -> *const u8 {
            unsafe { (self as *const Self).cast::<u8>().add(self.full_size()) }
        }
        fn shift(&'static mut self, target_align: usize) -> &'static mut Self {
            let required_padding = target_align - core::mem::size_of::<Self>();
            let padding = self.padding;
            if padding == required_padding {
                return self;
            }
            self.size += padding;
            self.padding = 0;
            let new_addr = unsafe {
                (self as *mut Self)
                    .cast::<u8>()
                    .offset(padding as isize - required_padding as isize)
            };
            unsafe {
                core::ptr::copy(
                    (self as *const Self).cast(),
                    new_addr,
                    core::mem::size_of::<Self>(),
                );
                &mut *new_addr.cast()
            }
        }
        fn split(self: &mut &'static mut Self, at: usize) -> Option<&'static mut Self> {
            let size = self.size;
            (size > at + core::mem::size_of::<Self>()).then(move || {
                self.size = at;
                let slot = unsafe { &mut *(self.end() as *mut MaybeUninit<Slot>) };
                slot.write(Slot {
                    size: size - at + core::mem::size_of::<Self>(),
                    lower: None,
                    padding: 0,
                    _reserved: 0,
                })
            })
        }
    }

    const PAGESIZE: usize = 65536;
    #[repr(C)]
    struct Allocator {
        free_list: AtomicPtr<Slot>,
    }
    struct Slots {
        list: Option<&'static mut Slot>,
    }
    impl Drop for Slots {
        fn drop(&mut self) {
            ALLOC.free_list.store(
                unsafe {
                    core::mem::transmute::<Option<&'static mut Slot>, *mut Slot>(self.list.take())
                },
                Ordering::Release,
            );
        }
    }
    impl Slots {
        fn insert(&mut self, mut slot: &'static mut Slot) {
            slot = slot.shift(core::mem::size_of::<Slot>());
            let mut head = &mut self.list;
            while let Some(h) = head {
                if *h < slot {
                    if core::ptr::eq(h.end(), slot.start()) {
                        h.size += slot.full_size();
                        return;
                    }
                    break;
                }
                head = unsafe {
                    core::mem::transmute::<
                        &mut Option<&'static mut Slot>,
                        &mut Option<&'static mut Slot>,
                    >(&mut h.lower)
                };
            }
            slot.lower = head.take();
            *head = Some(slot)
        }
        fn take(&mut self, size: usize, align: usize) -> Option<&'static mut Slot> {
            let req = size + align;
            let slot_owner = self.select_slot(req)?;
            let mut slot = slot_owner.take()?;
            let lower = slot.lower.take();
            *slot_owner = slot.split(size);
            match slot_owner {
                Some(owner) => owner.lower = lower,
                None => *slot_owner = lower,
            }
            Some(slot)
        }
        fn select_slot(&mut self, size: usize) -> Option<&mut Option<&'static mut Slot>> {
            let mut head = unsafe {
                core::mem::transmute::<&mut Option<&'static mut Slot>, &mut Option<&'static mut Slot>>(
                    &mut self.list,
                )
            };
            while let Some(h) = head {
                if h.size < size {
                    head = unsafe {
                        core::mem::transmute::<
                            &mut Option<&'static mut Slot>,
                            &mut Option<&'static mut Slot>,
                        >(&mut h.lower)
                    };
                } else {
                    return Some(head);
                }
            }
            self.grow_take(size)
        }
        fn grow_take(&mut self, size: usize) -> Option<&mut Option<&'static mut Slot>> {
            let added_pages = (size / PAGESIZE) + 2;
            let start = core::arch::wasm32::memory_grow(0, added_pages);
            if start == usize::MAX {
                return None;
            }
            let slot = unsafe { &mut *((start * PAGESIZE) as *mut MaybeUninit<Slot>) };
            let slot = slot.write(Slot {
                size: added_pages * PAGESIZE - core::mem::size_of::<Slot>(),
                lower: None,
                padding: 0,
                _reserved: 0,
            });
            self.insert(slot);
            Some(&mut self.list)
        }
    }
    impl Allocator {
        const fn new() -> Self {
            Self {
                free_list: AtomicPtr::new(core::ptr::null_mut()),
            }
        }
        fn lock(&self) -> Slots {
            loop {
                let list = self
                    .free_list
                    .swap(usize::MAX as *mut Slot, Ordering::AcqRel);
                if list as usize != usize::MAX {
                    return Slots {
                        list: unsafe { list.as_mut() },
                    };
                }
                core::hint::spin_loop();
            }
        }
    }
    static ALLOC: Allocator = Allocator::new();
    pub unsafe fn posix_memalign(
        this: &mut *mut core::ffi::c_void,
        mut size: usize,
        mut align: usize,
    ) -> i32 {
        size = size.max(64);
        align = align.max(8);
        match ALLOC.lock().take(size, align) {
            Some(slot) => {
                *this = (slot as *mut Slot).add(1).cast();
                0
            }
            None => -1,
        }
    }
    pub unsafe fn realloc(p: *mut c_void, new_size: usize) -> *mut c_void {
        let mut this = core::ptr::null_mut();
        if posix_memalign(&mut this, new_size, 8) != 0 {
            return core::ptr::null_mut();
        }
        let slot = p.cast::<Slot>().sub(1);
        unsafe { core::ptr::copy_nonoverlapping(p.cast::<u8>(), this.cast(), (*slot).size) };
        this
    }
    pub unsafe fn free(p: *mut c_void) {
        let slot = p.cast::<Slot>().sub(1);
        ALLOC.lock().insert(&mut *slot);
    }
}
