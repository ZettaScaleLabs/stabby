[![Crates.io (latest)](https://img.shields.io/crates/v/stabby)](https://lib.rs/crates/stabby)
[![docs.rs](https://img.shields.io/docsrs/stabby)](https://docs.rs/stabby/latest/stabby/)

> [!WARNING]  
> Due to a breaking change in Rust 1.78, `stabby`'s implementation of trait objects may raise performance issues:
> - __Only non-nightly, >= 1.78 versions of Rust are affected__
> - The v-tables backing trait objects are now inserted in a global lock-free set.
> - This set is leaked: `valgrind` _will_ be angry at you.
> - This set grows with the number of distinct `(type, trait-set)` pairs. Its current implementation is a vector:
>   - Lookup is done through linear search (O(n)), which stays the fastest for <100 number of elements.
>   - Insertion is done by cloning the vector (O(n)) and replacing it atomically, repeating the operation in case of collision.
>   - Efforts to replace this implementation with immutable b-tree maps are ongoing (they will be scrapped if found to be much slower than the current implementation).
>
> This note will be updated as the situation evolves. In the meantime, if your project uses many `stabby`-defined trait objects,
> I suggest using either `nightly` or a `< 1.78` version of the compiler.

# A Stable ABI for Rust with compact sum-types
`stabby` is your one-stop-shop to create stable binary interfaces for your shared libraries easily, without having your sum-types (enums) explode in size.

Your main vector of interaction with `stabby` will be the `#[stabby::stabby]` proc-macro, with which you can annotate a lot of things.

## Why would I _want_ a stable ABI? And what even _is_ an ABI?
ABI stands for _Application Binary Interface_, and is like API's more detail focused sibling. While an API defines what type of data a function expects, and what properties these types should have; ABI defines _how_ this data should be laid out in memory, and how a function call even works.

How data is laid out in memory is often called "representation": field ordering, how variants of enums are distinguished, padding, size... In order to communicate using certain types, two software units _must_ agree on what these types look like in memory.

Function calls are also highly complex under the hood (although it's rare for developers to need to think about them): is the callee or caller responsible for protecting the caller's register from callee's operations? In which registers / order on the stack should arguments be passed? What CPU instruction is used to actually trigger the call? A set of replies to these questions (and a few more) is called "calling convention".

In Rust, unless you explicitly select a known representation for your types through `#[repr(_)]`, or an explicit calling convention for your functions with `extern "_"`, the compiler is free to do whatever it pleases with these aspects of your software: the process by which it does that is explicitly unstable, and depends on your compiler version, the optimization level you selected, some llama's mood in a wool farm near Berkshire... who knows?

The problem with that comes when dynamic linkage is involved: since the ABI for most things in Rust is unstable, software units (such as a dynamic library and the executable that requires it) that have been built through different compiler calls may disagree on these decisions about ABI, even though there's no way for the linker to know that they did.

Concretely, this could mean that your executable thinks the leftmost 8 bytes of `Vec<Potato>` is the pointer to the heap allocation, while the library believes them to be its length. This could also mean the library thinks it's free to clobber registers when its functions are called, while the executable relied on it to save them and restore them before returning.

`stabby` seeks to help you solve these issues by helping you pin the ABI for a subset of your program, while helping you retain some of the layout optimizations `rustc` provides when using its unstable ABI. On top of this, stabby allows you to annotate function exports and imports in a way that also serves as a check of your dependency versioning for types that are `stabby::abi::IStable`.

## Structures
When you annotate structs with `#[stabby::stabby]`, two things happen:
- The struct becomes `#[repr(C)]`. Unless you specify otherwise or your struct has generic fields, `stabby` will assert that you haven't ordered your fields in a suboptimal manner at compile time.
- `stabby::abi::IStable` will be implemented for your type. It is similar to `abi_stable::Stable`, but represents the layout (including niches) through associated types. This is key to being able to provide niche-optimization in enums (at least, until `#[feature(generic_const_exprs)]` becomes stable).

## Enums
When you annotate an enum with `#[stabby::stabby]`, you may select an existing stable representation (like you must with `abi_stable`), but you may also select `#[repr(stabby)]` (the default representation) to let `stabby` turn your enum into a tagged-union with a twist: the tag may be a ZST that inspects the union to emulate Rust's niche optimizations.

Note that `#[repr(stabby)]` does lose you the ability to pattern-match.

Due to limitations of the trait solver, `#[repr(stabby)]` enums have a few paper-cuts:
- Compilation times suffer from `#[repr(stabby)]` enums.
- Additional trait bounds are required when writing `impl`-blocks for generic enums. They will always be of the form of one or multiple `A: stabby::abi::IDeterminantProvider<B>` bounds (although `rustc`'s error may suggest more complex bounds, the bounds should always be of this `IDeterminantProvider` shape).

`#[repr(stabby)]` enums are implemented as a balanced binary tree of `stabby::result::Result<Ok, Err>`, so discriminants are always computed between two types through the following process:
- If some of `Err`'s forbidden values (think `0` for non-zero types) fit inside the bits that `Ok` doesn't care for, that value is used to signify that we are in the `Ok` variant.
- The same thing is attempted with `Err` and `Ok`'s roles inverted.
- If no single value discriminant is found, `Ok` and `Err`'s unused bits are intersected. If the intersection exists, the least significant bit is used, while the others are kept as potential niches for sum-types that would contain a `Result<Ok, Err>` variant.
- Should no niche be found, the smallest of the two types is shifted right by its alignment, and the process is attempted again. This shifting process stops if the union would become bigger, or at the 8th time it has been attempted. If the process stops before a niche is found, a single bit will be used as the determinant (shifting the union right by its own alignment, with `1` representing `Ok`).

## Unions
If you want to make your own internally tagged unions, you can tag them with `#[stabby::stabby]` to let `stabby` check that you only used stable variants, and let it know the size and alignment of your unions. Note that `stabby` will always consider that unions have no niches.

## Traits
When you annotate a trait with `#[stabby::stabby]`, an ABI-stable v-table is generated for it. You can then use any of the following type equivalence:
- `&'a dyn Traits` → `DynRef<'a, vtable!(Traits)>` __or__ `dynptr!(&'a dyn Trait)`
- `&'a mut dyn Traits` → `Dyn<&'a mut (), vtable!(Traits)>` __or__ `dynptr!(&'a mut dyn Traits)`
- `Box<dyn Traits + 'a>` → `Dyn<'a, Box<()>, vtable!(Traits)>` __or__ `dynptr!(Box<dyn Traits + 'a>)`
- `Arc<dyn Traits + 'a>` → `Dyn<'a, Arc<()>, vtable!(Traits)>` __or__ `dynptr!(Arc<dyn Traits + 'a>)`

Note that `vtable!(Traits)` and `dynptr!(..dyn Traits..)` support any number of traits: `vtable!(TraitA + TraitB<Output = u8>)` or `dynptr!(Box<dyn TraitA + TraitB<Output = u8>>)` are perfectly valid, but ordering must remain consistent.

However, the v-tables generated by stabby will not take super-traits into account.

In order for `stabby::dynptr!(Box<dyn Traits + 'a>)` to have `Trait`'s methods, you will need to `use trait::{TraitDyn, TraitDynMut};`, so make sure you don't accidentally seal these traits which are automatically declared with the same visibility as your `Trait`.

`stabby::closure` exports the `CallN`, `CallMutN` and `CallOnceN` traits, where `N` (in `0..=9`) is the number of arguments, as ABI-stable equivalents of `Fn`, `FnMut` and `FnOnce` respectively.

Since version `1.0.1`, the v-tables generated by `#[stabby::stabby]` always assume all of their method arguments to be ABI-stable, to prevent the risk of freezing `rustc`.
Unless your trait has methods referencing its own v-table, it's advised to use `#[stabby::stabby(checked)]` instead to avoid the v-table being marked as stable despite some types in its
interface not actually being stable.

## Functions
### `#[stabby::stabby]`
Annotating a function with `#[stabby::stabby]` makes it `extern "C"` (but not `#[no_mangle]`) and checks its signature to ensure all exchanged types are marked with `stabby::abi::IStable`. You may also specify the calling convention of your choice.

### `#[stabby::export]`
Works just like `#[stabby::stabby]`, but will add `#[no_mangle]` to the annotated function, and produce two other no-mangle functions:
- `extern "C" fn <fn_name>_stabbied(&stabby::abi::report::TypeReport) -> Option<...>`, will return `<fn_name>` as a function pointer if the type-report matches that of `<fn_name>`'s signature, ensuring that they indeed have the same signature.
- `extern "C" fn <fn_name>_stabbied_report() -> &'static stabby::abi::report::TypeReport` will return `<fn_name>`'s type report, allowing debugging if the previous function returned `None`.

### `#[stabby::export(canaries)]`
Works on any function, including ones that would be FFI-unsafe. On top of adding `#[no_mangle]` to the original function, it will add a small set of `<fn_name>_<canary>` symbols to the produced shared libraries. These canaries include `rustc`'s version, the optimization level, and other properties that may cause the compiler to use a different ABI for `<fn_name>`.

The presence of these symbols can then be checked for by the linker when loading the shared library, preventing linkage when the loader requests canaries with incompatible versions.

### `#[stabby::import(...)]`
Annotating an `extern` block with this is equivalent to `#[link(...)]`, except the symbols will be lazy-initialized by using `<fn_name>_stabbied`, ensuring that the reports on the functions parameters match before letting you call it.

If you want to handle potential mismatch errors without panicking, you can call `<fn_name>.as_ref()`, which will let you inspect the reports for `<fn_name>` in case of failure.

### `#[stabby::import(canaries="rustc, opt_level", ...)]`
Annotating an `extern` block with this is equivalent to `#[link(...)]`, but the canaries corresponding to your spec will be required for linkage to be possible. This mirrors `export(canaries)`, which always exports all available canaries, but you can choose which canaries you want to enable from the following set:
- `paranoid`: enables all canaries, this is also what is selected if you use `canaries=""`.
- `rustc`: enables the canary on `rustc` version (always up to the `commit` version).
- `opt_level`: enables the canary on `opt_level` (necessary for `extern "rust" fn`, as optimization level may change the calling convention).
- `target`: enables the canary on the compiler target triple which was used to build the objects.
- `num_jobs` (paranoid): enables the canary on the number of jobs used to build the objects. This can affect optimizations, and thus might affect ABI (unproven).
- `debug` (paranoid): enables the canary on whether the objects were built with debug symbols. The effects on ABI are unproven, but not excluded.
- `host` (paranoid): enables the canary on the compiler host triple which was set by the compiler. The effects on ABI are unproven, but not excluded.
- `none`: mostly here to let you fully disable canaries, at your own risks.

### The `stabby::libloading::StabbyLibrary` trait
Additional methods for `libloading::Library` that expose symbol getters which will fail if the canaries are absent, or in case of a report mismatch.

These methods are still considered unsafe, but they will reduce the risks of accidentally loading ABI-incompatible code. Reports also act as a runtime type-check, reducing the risk of mistyping a symbol.

## Async
Any implementation of `core::future::Future` on a stable type will work regardless of which side of the FFI-boundary that stable type was constructed. However, futures created by async blocks and async functions aren't ABI-stable, so they must be used through trait objects.

`stabby` supports futures through the `stabby::future::Future` trait. Async functions are turned by `#[stabby::stabby]` into functions that return a `Dyn<Box<()>, vtable!(stabby::future::Future + Send + Sync)>` (the `Send` and `Sync` bounds may be removed by using `#[stabby::stabby(unsync, unsend)]`), which itself implements `core::future::Future`.

`stabby` doesn't support async traits yet, but you can use the following pattern to implement them:
```rust
use stabby::{slice::SliceMut, future::DynFuture};
#[stabby::stabby]
pub trait AsyncRead {
	fn read<'a>(&'a mut self, buffer: SliceMut<'a, [u8]>) -> DynFuture<'a, usize>;
}
impl MyAsyncTrait for SocketReader {
	extern "C" fn read<'a>(&'a mut self, mut buffer: SliceMut<'a, [u8]>) -> DynFuture<'a, usize> {
		Box::new(
			async move {
				let slice = buffer.deref_mut();
				let read = SocketReader::read_async(&mut self.socket, slice).await;
				buffer = slice.into();
				read
			}
		).into()
	}
}
```

## Incremental stability
`stabby` also lets you tell it that something is ABI-stable even if you couldn't chain `#[stabby::stabby]` all along using `stabby::abi::StableLike`.

Combined with the ZSTs in `stabby::compiler_version` that implement `stabby::IStable` however you tell them to, but only when compiled with their respective versions of the compiler, this lets you state that some types are only stable if compiled with the appropriate compiler version. But the ZSTs will still exist even if not, so the types will still be usable anywhere that doesn't have a `stabby::IStable` bound.

# The `stabby` "manifesto"
`stabby` was built in response to the lack of ABI-stability in the Rust ecosystem, which makes writing plugins and other dynamic linkage based programs painful. Currently, Rust's only stable ABI is the C ABI, which has no concept of sum-types, let alone niche exploitation.

However, our experience in software engineering has shown that type-size matters a lot to performance, and that sum-types should therefore be encoded in the least space-occupying manner.

My hope with `stabby` comes in two flavors:
- Adoption in the Rust ecosystem: this is my least favorite option, but this would at least let people have a better time with Rust in situations where they need dynamic linkage.
- Triggering a discussion about providing not a stable, but versioned ABI for Rust: `stabby` essentially provides a versioned ABI already through the selected version of the `stabby-abi` crate. However, having a library implement type-layout, which is normally the compiler's job, forces abi-stability to be per-type explicit, instead of applicable to a whole compilation unit. In my opinion, a `abi = "<stabby/crabi/c>"` key in the cargo manifest would be a much better way to do this. Better yet, merging that idea with [RFC 3435](https://github.com/rust-lang/rfcs/pull/3435) to allow selecting an ABI on a per-function basis, and letting the compiler contaminate the types at the annotated functions' interfaces with the selected stable ABI, would be much more granular, but would still allow end users to become ABI-stable by committing to a single version of their dependencies. 

# `stabby`'s SemVer policy
Stabby includes all of its `stabby_abi::IStable` implementation in its public API: any change to an `IStable` type's memory representation is a breaking change which will lead to a `MAJOR` version change.

From `6.1.1` onwards, Stabby follows [SemVer Prime](https://p-avital.github.io/semver-prime), using the `api, abi` as the key. Here's a few ways you can interpret that:
- `stabby.version[level] = 2^(api[level]) * 3^(abi[level])` lets you compute the exact versions of stabby's ABI and API.
- When upgrading stabby, you can check what has changed by dividing the new version by the previous one: if the division result is a multiple of 2, the change affected API; and it affected ABI if it's a multiple of 3.
	- ABI versioning:
		- Adding a new type to the set of ABI stable type will bump ABI patch.
		- Modifying an existing type's ABI in any way will bump the ABI major.
	- API versioning strictly follows SemVer policy. Any API visible in the docs is considered public, as well as whatever contracts are mentioned in said docs.