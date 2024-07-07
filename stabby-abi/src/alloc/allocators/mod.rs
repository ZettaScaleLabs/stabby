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
