/// A simple cross-platform allocator implementation
///
/// This allocator is based on maintaining a btree of free memory blocks,
/// allowing rather predictable alloc/dealloc times.
// pub mod btree_alloc;
/// A simple cross-platform allocator implementation
///
/// This allocator is based on an ordered linked list of free memory blocks.
// #[cfg(any(target_arch = "wasm32", all(target_family = "unix", feature = "libc")))]
// mod freelist_alloc;
// #[cfg(any(target_arch = "wasm32", all(target_family = "unix", feature = "libc")))]
// pub use freelist_alloc::{FreelistAlloc, FreelistGlobalAlloc};

#[cfg(all(feature = "libc", not(target_arch = "wasm32")))]
/// [`IAlloc`](crate::alloc::IAlloc) bindings for `libc::malloc`
pub(crate) mod libc_alloc;
#[cfg(all(feature = "libc", not(target_arch = "wasm32")))]
pub use libc_alloc::LibcAlloc;

#[cfg(feature = "alloc-rs")]
/// Rust's GlobalAlloc, accessed through a vtable to ensure no incompatible function calls are performed
mod rust_alloc;
#[cfg(feature = "alloc-rs")]
pub use rust_alloc::RustAlloc;

// #[cfg(target_arch = "wasm32")]
// pub(crate) mod paging {
//     use core::ptr::NonNull;
//     pub(crate) const PAGESIZE: usize = 65536;
//     pub(crate) fn memmap(_hint: *const (), requested_capacity: &mut usize) -> Option<NonNull<u8>> {
//         let added_pages = (*requested_capacity / PAGESIZE) + 1;
//         let start = core::arch::wasm32::memory_grow(0, added_pages);
//         if start == usize::MAX {
//             return None;
//         }
//         *requested_capacity = added_pages * PAGESIZE;
//         unsafe { core::mem::transmute::<usize, Option<NonNull<u8>>>(start * PAGESIZE) }
//     }
//     pub(crate) fn memunmap(_addr: *mut (), _len: usize) {}
// }

// #[cfg(all(target_family = "unix", feature = "libc"))]
// pub(crate) mod paging {
//     use core::ptr::NonNull;
//     pub(crate) const PAGESIZE: usize = 65536;
//     pub(crate) fn memmap(hint: *const (), requested_capacity: &mut usize) -> Option<NonNull<u8>> {
//         let added_pages = (*requested_capacity / PAGESIZE) + 1;
//         *requested_capacity = added_pages * PAGESIZE;
//         let start = unsafe {
//             libc::mmap(
//                 hint.cast_mut().cast(),
//                 *requested_capacity,
//                 libc::PROT_READ | libc::PROT_WRITE,
//                 libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
//                 0,
//                 0,
//             )
//             .cast::<u8>()
//         };
//         if start as isize == -1 {
//             return None;
//         }
//         NonNull::new(start)
//     }
//     // pub(crate) fn memunmap(addr: *mut (), mut len: usize) {
//     //     len -= len % PAGESIZE;
//     //     unsafe { libc::munmap(addr.cast(), len) };
//     // }
// }

#[cfg(any(stabby_default_alloc = "RustAlloc", feature = "alloc-rs"))]
/// The default allocator, depending on which of the following is available:
/// - RustAlloc: Rust's `GlobalAlloc`, through a vtable that ensures FFI-safety.
/// - LibcAlloc: libc::malloc, which is 0-sized.
/// - None. I _am_ working on getting a 0-dependy allocator working, but you should probably go with `feature = "alloc-rs"` anyway.
///
/// You can also use the `stabby_default_alloc` cfg to override the default allocator regardless of feature flags.
pub(crate) type DefaultAllocator = RustAlloc;

#[cfg(any(
    stabby_default_alloc = "LibcAlloc",
    all(feature = "libc", not(feature = "alloc-rs"))
))]
/// The default allocator, depending on which of the following is available:
/// - RustAlloc: Rust's `GlobalAlloc`, through a vtable that ensures FFI-safety.
/// - LibcAlloc: libc::malloc, which is 0-sized.
/// - None. I _am_ working on getting a 0-dependy allocator working, but you should probably go with `feature = "alloc-rs"` anyway.
///
/// You can also use the `stabby_default_alloc` cfg to override the default allocator regardless of feature flags.
pub(crate) type DefaultAllocator = LibcAlloc;

#[cfg(not(any(stabby_default_alloc, feature = "alloc-rs", feature = "libc")))]
/// The default allocator, depending on which of the following is available:
/// - RustAlloc: Rust's `GlobalAlloc`, through a vtable that ensures FFI-safety.
/// - LibcAlloc: libc::malloc, which is 0-sized.
/// - None. I _am_ working on getting a 0-dependy allocator working, but you should probably go with `feature = "alloc-rs"` anyway.
///
/// You can also use the `stabby_default_alloc` cfg to override the default allocator regardless of feature flags.
pub(crate) type DefaultAllocator = core::convert::Infallible;
