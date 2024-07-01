/// A simple cross-platform allocator implementation
///
/// This allocator is based on maintaining a btree of free memory blocks,
/// allowing rather predictable alloc/dealloc times.
// pub mod btree_alloc;
/// A simple cross-platform allocator implementation
///
/// This allocator is based on an ordered linked list of free memory blocks.
// pub mod freelist_alloc;

#[cfg(not(any(target_arch = "wasm32")))]
/// [`IAlloc`](crate::alloc::IAlloc) bindings for `libc::malloc`
pub mod libc_alloc;

#[cfg(feature = "alloc-rs")]
/// Rust's GlobalAlloc, accessed through a vtable to ensure no incompatible function calls are performed
pub mod rust_alloc;

#[cfg(target_arch = "wasm32")]
pub(crate) mod paging {
    use core::ptr::NonNull;
    pub(crate) const PAGESIZE: usize = 65536;
    pub(crate) fn memmap(hint: *const (), requested_capacity: &mut usize) -> Option<NonNull<u8>> {
        let added_pages = (*requested_capacity / PAGESIZE) + 1;
        let start = core::arch::wasm32::memory_grow(0, added_pages);
        if start == usize::MAX {
            return None;
        }
        *requested_capacity = added_pages * PAGESIZE;
        unsafe { core::mem::transmute::<usize, Option<NonNull<u8>>>(start * PAGESIZE) }
    }
    pub(crate) fn memunmap(hint: *mut (), max_unmap: usize) {}
}

// #[cfg(all(target_family = "unix", feature = "libc"))]
// pub(crate) mod paging {
//     use core::ptr::NonNull;
//     pub(crate) const PAGESIZE: usize = 65536;
//     pub(crate) fn memmap(hint: *const (), requested_capacity: &mut usize) -> Option<NonNull<u8>> {
//         const PAGESIZE: usize = 65536;
//         let added_pages = (*requested_capacity / PAGESIZE) + 1;
//         *requested_capacity = added_pages * PAGESIZE;
//         let start = unsafe {
//             libc::mmap(
//                 hint.cast_mut().cast(),
//                 *requested_capacity,
//                 libc::PROT_READ | libc::PROT_WRITE,
//                 libc::MAP_PRIVATE,
//                 -1,
//                 0,
//             )
//         };
//         if start as isize == -1 {
//             return None;
//         }
//         NonNull::new(start.cast())
//     }
//     pub(crate) fn memunmap(addr: *mut (), mut len: usize) {
//         len -= len % PAGESIZE;
//         unsafe { libc::munmap(addr.cast(), len) };
//     }
// }
