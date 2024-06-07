# 5.1.0
- Introducing `stabby::collections::arc_btree`, a set of copy-on-write btrees:
	- `ArcBtreeSet` and `ArcBtreeMap` behave like you would expect, but share nodes with their clones.
	- When mutating one of their nodes (either because an entry is changed, or because a child node required a carry-over operation to complete its own mutation), one of two cases will happen:
		- The node wasn't shared: it is mutated in place.
		- The node was shared: it is copied (increasing its children's reference counts), mutated, and the parent node is mutated to replace the shared node by its mutated copy. This behaviour keeps recursing until the root if necessary.
	- `AtomicArcBtreeSet` is a lock-free set based on `ArcBtreeSet`: when a node is inserted, the root pointer is cloned, the clone is mutated (causing its root pointer to change), and replaced. If the root pointer changed since reading it, the process is tried again.
		- This is notably how `stabby` global set of vtables is implemented to support stable Rust from version 1.78 onward, until the [static-promotion regression](https://github.com/rust-lang/rust/issues/123281) is [fixed](https://github.com/rust-lang/rfcs/pull/3633), and this global set can be removed.
- Add some missing `Send` and `Sync` implementations for container types.
- Fix a lot of nightly lints.
- Officially switch to [Humane SemVer](https://p-avital.github.io/humane-semver)

# 5.0.1
- Fix a regression in MSRV

# 5.0.0
- BREAKING CHANGE:
	- Due to a soundness hole in the previous implementation of `stabby::result::Result`, its representation was overhauled. While it's still technically compatible binary-wise (meaning it still follows the original spec), the soundness hole could possibly lead to UB, so `stabby` will treat them as incompatible.
- Add support for Rust 1.78:
	- With 1.78, [Rust merged a breaking change](https://github.com/rust-lang/rust/issues/123281) which impacts `stabby`. This change prevents consts from refering to generics entirely, which was key to `stabby`'s implementation of vtables.
	- More accurately, it prevents consts from refering to generics that aren't bound by `core::marker::Freeze`, but that trait hasn't been stabilized at the same time as the new error has.
	- While the team was aware that this would be a breaking change, `crater` failed to report that `stabby` was impacted by the regression, as it tried compiling an obsolete version of `stabby` that could only build with pre-1.77 versions of Rust due to the `u128` ABI-break on x86. This led them to judge that the breaking change was acceptable.
	- To compensate for this, `stabby` will (for non-nighly `>=1.78` versions of Rust) draw its vtable references from a heap-allocated, lazily populated, global set of vtables. This is in opposition to the `<1.78` and `nightly` behaviour where it'll keep on drawing these vtable references straight from the binary.
		- From this release onwards, a new priority for `stabby` will be to improve the performance of this behaviour; or better yet find a new way to obtain the previous behaviour that compiles.
	- While I can't hide that I am very annoyed at this development, I must also state that I understand the Rust Team's choice to ship this breaking change: they considered this potential window for a soundness hole a bug, and even though `crater` didn't report any use of this bug that was unsound, it also failed to report `stabby` as a legitimate user of it. I do wish they'd have waited for `Freeze`'s stabilization to make the breaking change however, as the sound pattern it would prevent, as well as the fact that it couldn't be replicated without `Freeze`, _was_ known.

# 4.0.5
- Fix for 1.72: `AllocPtr::prefix` is `const fn` from 1.73 onwards rather than 1.72 onwards (@yellowhatter).

# 4.0.4
- Introduce a tutorial to help onboard new users.
	- Available as `stabby/TUTORIAL.md` in sources.
	- Inserted as the documentation to a docs-only `stabby::_tutorial_` module. This ensures that codeblocks in it compile and that links to the doc are checked.
- Allow `stabby::sync::Weak` to function as a pointer-type for fat pointers, allowing the `stabby::dynptr!(Weak<dyn Trait>)` pattern.
	- This can be helpful if you're building a plugin that needs to refer to its host weakly to avoid cycles, for example.

# 4.0.3
- Ensure `stabby` compiles on `nightly` by using `core::marker::Freeze` to reassure Rust that a bound that will eventually become required for const static references is respected by v-tables.
- Small documentation pass on internals to make nightly clippy happy

# 4.0.2
- Fix lifetimes seeping in code generation in traits, allowing more valid code to compile.

# 4.0.1
- Add constructors from slices for `Vec<T, A>`, `BoxedSlice<T, A>` and `ArcSlice<T, A>` where `T` is `Copy` and `A` is a default constructible allocator.

# 4.0.0
- With Rust 1.77, `u128`'s alignment changes to 16 bytes. This version of `stabby` supports both and is able to tell them appart.
- Fix a soundness hole in `Result`, contaminating all `#[repr(stabby)]` enums: mutable references to a variant can no longer be held past the closure they originate from. This is needed because assigning to such a reference may override the determinant, which `stabby` reinserts at the end of the match. Passing a continuation in `match_mut_ctx` is the proper way to use a reference that may have originated from several variants.
- Introduce the `uX` and `iX` types. These types are implemented as a newtype on the smallest integer type that is larger than them, but expose niches that are exclusive to `stabby`.
- Some benchmarks have been built to measure `stabby`'s impact on performance. The global result is that `stabby` generally has similar performances to `std`, being marginally faster and marginally slower depending on cases. Some specific exceptions exist:
	- `repr(stabby)` enums get much slower than their `std` versions, due to not being able to constify some of its niche handling. They can however be more compact, letting them be faster when memory access become the bottleneck.
	- `stabby`'s `Vec` is faster at growing than Rust's thanks to a growth factor that minimizes memory partitioning. Note that your mileage may vary here, as a PR in the stdlib was attempted to use the same trick and did not yield better performances.
	- `stabby` is much faster at converting between `Vec` and `ArcSlice`, and between large `Box`es and `Arc`s. This is thanks to all `stabby::alloc` types sharing a slot in front of their payload to allow converting between them without reallocating.
- MIRI passes are being made to ensure that `stabby` stays safely within defined behaviour.
	- Some UB may now no longer occur.
	- Some UB is still detected in certain tests. Work is ongoing to remove said UB.

# 3.0.3
- Ensure docrs can properly build docs

# 3.0.2
- Add support for `AtomicDuration` and `AtomicInstant`: these allow storing `Duration`s and `Instant`s in an atomic manner, at the cost of their resolution being limited to 1μs and their range limited to ~270000 years.
- Add many convenience methods to `stabby::time::Duration`.
- Fix typo in symbol mangling of report fetchers in `libloading` integration: this typo meant that if a symbol was loaded, but the reports mismatched, the loader would be unable to extract the full report. While this wouldn't panic or cause UB, it would make the experience worse than expected.
- Improve reliability by ensuring that the CI tests both load-time and runtime linkage with `stabby::import` and `libloading::Library::get_stabbied`.

# 3.0.1
- Change the symbol mangling of stabbied functions to ensure an ABI-incompatible reports are never mixed.

# 3.0.0
- BREAKING CHANGE:
	- From now on, unless `#[repr(stabby)]` is specified, stabbied `enum`s will throw a compile error if using `#[repr(u8)]` would yield a better or equal layout size. The default layout remains that produced when selecting `#[repr(stabby)]`, so any crate that didn't use either annotation and wishes to keep the same ABI should annotate types that now throw that error with `#[repr(stabby)]`.
	- Some internal systems that were previously exposed because they needed to be public to appear in trait bounds have been sealed to avoid over-exposing the ABI.
	- `stabby::report::TypeReport` now uses a single `u32` to describe versions, as this is by far sufficient, and avoids clashes when ABI changes (such as invariants) appear between release versions.
	- For all allocated containers, the `new` and `make` methods are now specific to the default allocator. Alternative allocators will have to use `new_in(Default::default())` instead, but this makes using these constructors without type hints possible.
- DOCUMENT ALL THE THINGS: `stabby` and `stabby-abi` will now both fail to compile if some of their elements are undocumented.
- Introduce the `IPod` trait to help prove that a type is "Plain Old Data" and safe to transfer between processes that don't share memory or even file-system. This is notably meant to be used in [`zenoh`](https://crates.io/crates/zenoh)'s shared-memory API.

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