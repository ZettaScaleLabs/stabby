# 2.0.1
- Update constness to fit 1.72
- Ensure that 1.66 MSRV is respected

# 2.0.0
- BREAKING CHANGE:
	- `std::boxed::Box` and `std::sync::Arc` were originally marked as `IStable` because their representation was indeed stable provided they pointed to sized types. However, Rust has historically changed the default global allocator, and since it can be overriden, it's also possible to create two binaries with mismatching allocators on each side. This meant that these types didn't have "invariant stability": moving one over FFI wasn't guaranteed to not introduce UB.
- Introducing `stabby::alloc`:
	- `IAlloc`: a trait that defines an allocator. It's basically just an ABI-stable equivalent to `std::alloc::GlobalAllocator`.
	- `Box<T, Alloc>`, `Arc<T, Alloc>`, and `Vec<T, Alloc>`, which emulate their `std` counterparts. `Alloc` defaults to `LibcAlloc`, which is built atop `posix_memalign`/`aligned_malloc`. They have been built such that converting between them never causes a reallocation (converting from an empty `Vec` to a `Box` or `Arc` will allocate, since these two types must always be allocated).
	- `BoxedSlice<T, Alloc>`, `BoxedStr<Alloc>`, `ArcSlice<T, Alloc>`, `ArcStr<Alloc>`, exist to emulate `Box<[T]>` &co, and are also built to be convertible from `Vec<T>` without reallocating.
	- All of these duplicate all operations that may allocate with `try` variants that will return an error instead of panicking on allocation failures.
- Better test coverage: the correct implementation of the spec is now fully verified.
- Better documentation: `stabby` now uses the `deny(missing_(safety|errors|panics)_doc)` lints to ensure all failure conditions are always documented, and documents all of its macros outputs (often based on your own documentation) to allow `stabby` to be used in `deny(missing_docs)` environments.
- `[T; N]` is now marked as `IStable` for `N` in `0..=128`.
- `SingleOrVec<T, Alloc>` is a `Vec`-like container that will avoid allocating until you attempt to push a second element in it.
- Introducing `NonMaxUx`, `NonXUx<const X: ux>` and `NonXIx<const X: ux>`: equivalents to `NonZero` that allow you to have another value as the niche.

# 1.0.10
- Make bound deduction better for enums.
- Introduce `MaybeResolved`: a future that may already be resolved to handle "maybe async" functions.
- `stabby` now has support for custom allocators, and uses that to define truly ABI stable allocated types in the `realloc` module.
	- While Rust's standard `Box` and `Arc` have stable layout, the default global allocator may change without `stabby` noticing,
	they are therefore not truly ABI stable.
	- `stabby::realloc`'s `Box`, `Arc` and `Vec` all support custom allocators, and prefix all allocations with the same layout,
	this allows conversions between those types to never require a reallocation unless the target requires an allocation that the source
	type didn't, like converting a `Vec` to an `Arc`.

# 1.0.9
- Introduce better matchers for pattern-matching emulations when at the borrrow checker would forbid the previously available ones:
 `match_ref_ctx`, `match_mut_ctx` and `match_owned_ctx` all take a context, and one closure per variant; and only call the closure corresponding to the current variant, passing the context as first argument.

# 1.0.8
- Fix duplicated bounds on structures that would cause compile errors when a structure had several fields of the same type

# 1.0.7
- Actually expose `stabby::time::{Instant, SystemTime}`

# 1.0.6
- Add trait implementations to `stabby::time::{Duration, Instant, SystemTime}`.
- Improve release process (releases are now based on changelogs, which should become more accurate)

# 1.0.5
- Marked `std::os::fd::{OwnedFd, BorrowedFd}` as stable.
- Added support for `core::time::Duration` and `std::time::{Instant, SystemTime}` through equivalent types.

# 1.0.4
- Added support for `core::iter::Iterator`.
- Made release process more reliable.

# 1.0.3
- Added support for some of `abi_stable`'s types
- Made checks for potential ABI misreports better

# 1.0.2: Accidental repeat of 1.0.1
# 1.0.1
- Fix cyclic trait bounds arising when a stabby trait depended on a dyn-self

# 1.0.0
This is the base release of this CHANGELOG. Please refer to its README for more information.