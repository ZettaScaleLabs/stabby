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
    let alloc_start = unsafe { alloc_rs::alloc::alloc(layout) };
    let ret =
        unsafe { alloc_start.add(layout.align().max(core::mem::size_of::<RustAllocPrefix>())) };
    unsafe {
        ret.cast::<RustAllocPrefix>().sub(1).write(RustAllocPrefix {
            layout: requested,
            vtable: VTABLE,
        })
    };
    ret.cast()
}
extern "C" fn realloc(ptr: *mut (), prev_layout: crate::alloc::Layout, new_size: usize) -> *mut () {
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
    unsafe {
        let requested = Layout::of::<RustAllocPrefix>().concat(Layout {
            size: new_size,
            align: prev_layout.align,
        });
        let alloc_start = alloc_rs::alloc::realloc(realloc_start, layout, requested.size);
        let ret = alloc_start.add(layout.align().max(core::mem::size_of::<RustAllocPrefix>()));
        ret.cast::<RustAllocPrefix>().sub(1).write(RustAllocPrefix {
            layout: requested,
            vtable: VTABLE,
        });
        ret.cast()
    }
}
extern "C" fn free(ptr: *mut (), prev_layout: crate::alloc::Layout) {
    let dealloc_start = unsafe {
        ptr.cast::<u8>().sub(
            prev_layout
                .align
                .max(core::mem::size_of::<RustAllocPrefix>()),
        )
    };
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
        let RustAllocPrefix { layout, vtable } =
            unsafe { ptr.cast::<RustAllocPrefix>().sub(1).read() };
        (vtable.free)(ptr, layout)
    }

    unsafe fn realloc(
        &mut self,
        ptr: *mut (),
        _prev_layout: crate::alloc::Layout,
        new_size: usize,
    ) -> *mut () {
        let RustAllocPrefix { layout, vtable } =
            unsafe { ptr.cast::<RustAllocPrefix>().sub(1).read() };
        (vtable.realloc)(ptr, layout, new_size)
    }
}
