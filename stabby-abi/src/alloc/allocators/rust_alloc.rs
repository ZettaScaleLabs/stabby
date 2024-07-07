use crate::alloc::{IAlloc, Layout};

/// Rust's GlobalAlloc, annotating its yielded pointers in such a way that the allocated pointers can be safely freed from other binaries.
#[crate::stabby]
#[derive(Clone, Copy)]
pub struct RustAlloc {
    inner: [u8; 0],
}
#[crate::stabby]
/// The VTable for [`RustAlloc`]
pub struct RustAllocVt {
    free: extern "C" fn(*mut (), crate::alloc::Layout),
    realloc: extern "C" fn(*mut (), crate::alloc::Layout, usize) -> *mut (),
}
#[crate::stabby]
/// The Prefix for [`RustAlloc`]
pub struct RustAllocPrefix {
    layout: Layout,
    vtable: RustAllocVt,
}

extern "C" fn alloc(requested: crate::alloc::Layout) -> *mut () {
    let requested = Layout::of::<RustAllocPrefix>().concat(requested);
    let Ok(layout) = core::alloc::Layout::from_size_align(requested.size, requested.align) else {
        return core::ptr::null_mut();
    };
    // SAFETY: The layout is always non-zero-sized
    let alloc_start = unsafe { alloc_rs::alloc::alloc(layout) };
    let ret = // SAFETY: the addition is indeed in-bound.
        unsafe { alloc_start.add(layout.align().max(core::mem::size_of::<RustAllocPrefix>())) };
    // SAFETY: `ret` is allocated and _at least_ one `RustAllocPrefix` greater than the start of the allocation, so writing there is safe.
    unsafe {
        ret.cast::<RustAllocPrefix>().sub(1).write(RustAllocPrefix {
            layout: requested,
            vtable: VTABLE,
        })
    };
    ret.cast()
}
extern "C" fn realloc(ptr: *mut (), prev_layout: crate::alloc::Layout, new_size: usize) -> *mut () {
    // SAFETY: The corresponding `alloc` returns the allocation offset by this much (see the line where `ret` is constructed in both the `alloc` and `realloc` functions)
    let realloc_start = unsafe {
        ptr.cast::<u8>().sub(
            prev_layout
                .align
                .max(core::mem::size_of::<RustAllocPrefix>()),
        )
    };
    let Ok(layout) = core::alloc::Layout::from_size_align(prev_layout.size, prev_layout.align)
    else {
        return core::ptr::null_mut();
    };
    let requested = Layout::of::<RustAllocPrefix>().concat(Layout {
        size: new_size,
        align: prev_layout.align,
    });
    // SAFETY: See each line
    unsafe {
        // If `ptr` was indeed allocated on by this allocator, then `realloc_start` was indeed allocated by _our_ GlobalAlloc.
        let alloc_start = alloc_rs::alloc::realloc(realloc_start, layout, requested.size);
        // We follow the same return-value shifting as in `alloc`
        let ret = alloc_start.add(layout.align().max(core::mem::size_of::<RustAllocPrefix>()));
        // And prepend the same prefix
        ret.cast::<RustAllocPrefix>().sub(1).write(RustAllocPrefix {
            layout: requested,
            vtable: VTABLE,
        });
        ret.cast()
    }
}
extern "C" fn free(ptr: *mut (), prev_layout: crate::alloc::Layout) {
    // SAFETY: The corresponding `alloc` returns the allocation offset by this much (see the line where `ret` is constructed in both the `alloc` and `realloc` functions)
    let dealloc_start = unsafe {
        ptr.cast::<u8>().sub(
            prev_layout
                .align
                .max(core::mem::size_of::<RustAllocPrefix>()),
        )
    };
    // If `ptr` was indeed allocated on by this allocator, then `dealloc_start` was indeed allocated by _our_ GlobalAlloc.
    unsafe {
        alloc_rs::alloc::dealloc(
            dealloc_start,
            core::alloc::Layout::from_size_align_unchecked(prev_layout.size, prev_layout.align),
        )
    }
}
const VTABLE: RustAllocVt = RustAllocVt {
    free: free as extern "C" fn(*mut (), crate::alloc::Layout),
    realloc: realloc as extern "C" fn(*mut (), crate::alloc::Layout, usize) -> *mut (),
};
impl RustAlloc {
    /// Constructs the allocator.
    pub const fn new() -> Self {
        Self { inner: [] }
    }
}
impl Default for RustAlloc {
    fn default() -> Self {
        Self::new()
    }
}
impl IAlloc for RustAlloc {
    fn alloc(&mut self, layout: crate::alloc::Layout) -> *mut () {
        alloc(layout)
    }

    unsafe fn free(&mut self, ptr: *mut ()) {
        let RustAllocPrefix { layout, vtable } = // SAFETY: if called with a `ptr` allocated by an instance of `self`, this read is valid.
            unsafe { ptr.cast::<RustAllocPrefix>().sub(1).read() };
        (vtable.free)(ptr, layout)
    }

    unsafe fn realloc(
        &mut self,
        ptr: *mut (),
        _prev_layout: crate::alloc::Layout,
        new_size: usize,
    ) -> *mut () {
        let RustAllocPrefix { layout, vtable } = // SAFETY: if called with a `ptr` allocated by an instance of `self`, this read is valid.
            unsafe { ptr.cast::<RustAllocPrefix>().sub(1).read() };
        (vtable.realloc)(ptr, layout, new_size)
    }
}
