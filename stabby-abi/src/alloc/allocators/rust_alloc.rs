use crate::alloc::{IAlloc, Layout};

/// Rust's GlobalAlloc, called via an FFI-safe vtable.
#[crate::stabby]
#[derive(Clone, Copy)]
pub struct RustAlloc {
    vtable: &'static RustAllocVt,
}
#[crate::stabby]
/// The VTable for [`RustAlloc`]
pub struct RustAllocVt {
    alloc: extern "C" fn(crate::alloc::Layout) -> *mut (),
    free: extern "C" fn(*mut ()),
    realloc: extern "C" fn(*mut (), crate::alloc::Layout, usize) -> *mut (),
}
extern "C" fn alloc(requested: crate::alloc::Layout) -> *mut () {
    let requested = Layout::of::<Layout>().concat(requested);
    let Ok(layout) = core::alloc::Layout::from_size_align(requested.size, requested.align) else {
        return core::ptr::null_mut();
    };
    let alloc_start = unsafe { alloc_rs::alloc::alloc(layout) };
    let ret = unsafe { alloc_start.add(layout.align().max(core::mem::size_of::<Layout>())) };
    unsafe { ret.cast::<Layout>().sub(1).write(requested) };
    ret.cast()
}
extern "C" fn realloc(
    ptr: *mut (),
    _prev_layout: crate::alloc::Layout,
    new_size: usize,
) -> *mut () {
    let prev_layout = unsafe { ptr.cast::<Layout>().sub(1).read() };
    let realloc_start = unsafe {
        ptr.cast::<u8>()
            .sub(prev_layout.align.max(core::mem::size_of::<Layout>()))
    };
    let Ok(layout) = core::alloc::Layout::from_size_align(prev_layout.size, prev_layout.align)
    else {
        return core::ptr::null_mut();
    };
    unsafe {
        let requested = Layout::of::<Layout>().concat(Layout {
            size: new_size,
            align: prev_layout.align,
        });
        let alloc_start = alloc_rs::alloc::realloc(realloc_start, layout, requested.size);
        let ret = alloc_start.add(layout.align().max(core::mem::size_of::<Layout>()));
        ret.cast::<Layout>().sub(1).write(requested);
        ret.cast()
    }
}
extern "C" fn free(ptr: *mut ()) {
    let prev_layout = unsafe { ptr.cast::<Layout>().sub(1).read() };
    let dealloc_start = unsafe {
        ptr.cast::<u8>()
            .sub(prev_layout.align.max(core::mem::size_of::<Layout>()))
    };
    unsafe {
        alloc_rs::alloc::dealloc(
            dealloc_start,
            core::alloc::Layout::from_size_align_unchecked(prev_layout.size, prev_layout.align),
        )
    }
}
const VTABLE: RustAllocVt = RustAllocVt {
    alloc: alloc as extern "C" fn(crate::alloc::Layout) -> *mut (),
    free: free as extern "C" fn(*mut ()),
    realloc: realloc as extern "C" fn(*mut (), crate::alloc::Layout, usize) -> *mut (),
};
static VT: RustAllocVt = VTABLE;
impl core::fmt::Debug for RustAlloc {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("RustAlloc")
    }
}
impl RustAlloc {
    /// Constructs the allocator.
    pub const fn new() -> Self {
        Self { vtable: &VTABLE }
    }
}
impl Default for RustAlloc {
    fn default() -> Self {
        Self { vtable: &VT }
    }
}
impl IAlloc for RustAlloc {
    fn alloc(&mut self, layout: crate::alloc::Layout) -> *mut () {
        (self.vtable.alloc)(layout)
    }

    unsafe fn free(&mut self, ptr: *mut ()) {
        (self.vtable.free)(ptr)
    }

    unsafe fn realloc(
        &mut self,
        ptr: *mut (),
        prev_layout: crate::alloc::Layout,
        new_size: usize,
    ) -> *mut () {
        (self.vtable.realloc)(ptr, prev_layout, new_size)
    }
}
