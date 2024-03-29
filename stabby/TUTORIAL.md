# The Stabby Tutorial

`stabby` provides a set of tools to use all of Rust's power accross FFI with as little runtime overhead as possible. This page is meant to guide you through its various features and how you're expected to use it.
<!-- TOC -->

- [Defining ABI stable types](#defining-abi-stable-types)
	- [Product types or structs](#product-types-or-structs)
	- [Sum types or enums](#sum-types-or-enums)
	- [ABI-stability must go all the way down...](#abi-stability-must-go-all-the-way-down)
	- [Unless you opacify types to their dependents](#unless-you-opacify-types-to-their-dependents)
- [Defining ABI stable traits](#defining-abi-stable-traits)
	- [Common pitfalls](#common-pitfalls)
	- [Multi-trait objects](#multi-trait-objects)
- [Creating shared libraries](#creating-shared-libraries)
	- [Exporting functions in shared objects](#exporting-functions-in-shared-objects)
	- [Exporting functions with checks](#exporting-functions-with-checks)
	- [Importing functions at runtime](#importing-functions-at-runtime)
	- [Importing functions at runtime _with checks_](#importing-functions-at-runtime-_with-checks_)

<!-- /TOC -->


## Defining ABI stable types

`stabby` leverages the trait system to find niches in your types that can be exploited when defining sum-types (`enum`s).

Most of the magic comes from the [`IStable`] trait which acts as a proof that a given type's representation is stable, as well as carries information about said representation (size, alignment, available niches, whether contains indirections, what its fields are...).

Lucky for you, you're _never_ expected to implement it yourself, relying instead on the [`stabby`] attribute macro to implement it for you!

If you're experienced in Rust, you may now wonder why this is an attribute macro and not a derive macro: the answer is that this macro does not limit itself to generating a trait implementation...

### Product types (or `struct`s)

When you annotate a structure with `stabby` like so
```rust
#[stabby::stabby]
pub struct MyStruct {
	hi: u8,
	there: u16,
}
```
`stabby` will actually turn it into the following
```ignore
#[repr(C)] // guarantees that the layout stays fixed
pub struct MyStruct {
	hi: u8,
	there: u16,
}
impl stabby::IStable for MyStruct { 
	// Dark magiks that compute MyStruct's layout,
	// Taking note that an entire byte is unused at offset 1 from the structure's start.
	...
}
const _: () = {	assert!(MyStruct::has_optimal_layout()) };
```

This last `const` here will prevent your code from compiling unless you've laid out your fields in an order that doesn't introduce unnecessary padding. 
If you're not familiar with the concept of alignment padding, [I've done a talk on the subject at RustConf 2023](https://www.youtube.com/watch?v=qkh8Fs2c4mY).

The core thing to retain about alignment padding is that it's necessary to uphold alignment constraints, which CPUs care about a lot: a value of a type must
always be at an address in memory that's a multiple of its alignment. In structures, padding will be inserted between fields to ensure this constraint in respected.
For example, here's `MyStruct`'s memory layout (considering u8 and u16's respective alignments to be 1 and 2 bytes, as is the case on all architectures I know of):
```text
bytes	|0       |1       |2       |3       |
fields	|hi      |--------|there            |
```

The dashes here are padding. Padding is also inserted at the end of structures to ensure that their size is a multiple of their alignment (the maximum of its fields' alignments).

If we were to add a `hello: u8` field to `MyStruct`, here are the layouts you'd get depending on where you insert it:

```text
if hello is inserted in first position
bytes	|0       |1       |2       |3       |
fields	|hello   |hi      |there            |
```
```text
if hello is appended at the end of MyStruct
bytes	|0       |1       |2       |3       |4       |5       |
fields	|hi      |--------|there            |hello   |--------|
```

Note how the second representation is 33% padding, while the first is much smaller. Without `repr(C)`, Rust is free to reorder fields to make
your type as compact as possible; but `repr(C)` forces fields to be ordered the same way in memory as in code, which means you have to do the reordering
to stay optimal.

Note that `stabby` can only help you automatically if your structure has no generics that may affect layout. Here's how this help looks in code:
```compile_fail
// This doesn't compile because reordering the fields would yield a better layout.
#[stabby::stabby]
pub struct MyStruct {
	hi: u8,
	there: u16,
	hello: u8,
}
```
```rust
// Here, we have an optimal layout memory-wise.
#[stabby::stabby]
pub struct MyStructProperlyOrdered {
	hi: u8,
	hello: u8,
	there: u16,
}
// You can also opt-out of `stabby`'s help if you need an explicitly sub-memory-optimal layout.
#[stabby::stabby(no_opt)]
pub struct MyStructExplicitlySuboptimal {
	hi: u8,
	there: u16,
	hello: u8,
}
```

Finally, `stabby` is perfectly happy to annotate unit and tuple structs.

### Sum types (or `enum`s)

`stabby`'s core reason for wanting to compute things about your types is so that it can do enum layout optimizations.

However, doing these layout optimizations in an ABI-stable manner does change how your code looks, and the tradeoffs between type-size
and speed are very close, meaning you might prefer to only use these optimized layouts for certain use cases.

Therefore, `stabby` leaves you the choice of representation. For the following subsections, we'll use the following 2 example types to
highlight differences between representations.

```rust
#[stabby::stabby]
#[repr(stabby)]
pub enum Poll<T> {
	Pending,
	Ready(T),
}

#[stabby::stabby]
#[repr(stabby)]
pub enum AllInts {
	U8(u8),
	U16(u16),
	U32(u32),
	U64(u64),
	I8(i8),
	I16(i16),
	I32(i32),
	I64(i64),
}
```

#### External tagging: `repr(uX)` and `repr(iX)`

These are `Rust`'s standard way of making ABI-stable sum types: an external tag of type `uX`/`iX` is added in front of the `union` of each
variant's data.

This is actually optimal for `AllInts`, as there isn't any niche to exploit in the largest variants' data (`u64` and `i64` both are types whose range of valid values covers all the values that the memory they occupy can take).

However, `Poll<T>` could have more efficient representations if `T` has "niches": binary patterns that do not correspond to any valid value of `T`. For example, `core::num::NonZeroU64` occupies 64 bits in memory, but is never allowed to be
`0`, which means that we could set all 64 bits to `0` as a way of indicating `Pending`. This is generally refered to as "niche optimizations" in Rust, and is something that normal Rust enums perform, but not `repr(uX/iX/C)`.

#### Stable niche optimizations: `repr(stabby)`

`stabby` was originally created as PoC that through [type system shenanigans](https://www.youtube.com/watch?v=g6mUtBVESb0), one could keep track of a type's layout at compile time, and use that information to define niche optimized layouts in a deterministic way,
granting ABI-stability without sacrificing memory-efficiency.

The way `stabby` does this is by having its own `stabby::Result<Ok, Err>` type which serves as the basis for its sum types. Through dark magiks, `stabby::Result` can find a niche to encode the determinant between `Ok` and `Err` without adding an external tag if that niche exists.
When you define a `repr(stabby)` enum (or just mark your enum with `stabby::stabby` without specifying your desired `repr`), `stabby` will represent it as a binary tree of `Results`:
```rust
struct AllInts(
	Result<
		Result<
			Result<
				u8, 
				u16,
			>,
			Result<
				u32,
				u64,
			>,
		>,
		Result<
			Result<
				i8,
				i16,
			>,
			Result<
				i32,
				i64,
			>,
		>,
	>
);
```

Of course, you don't have to deal with `stabby`'s exactions: `stabby` will also define a lot of accessors and constructors to let you interract with `AllInts` in ways that mostly resemble how you would interract with normal enums, with a few exceptions:
- Due to the current limitations of `const fn` and traits, the constructors for each variant cannot be `const`.
- Pattern matching is no longer available, but is instead emulated using the `match_ref`, `match_mut`, `match_owned` and their `_ctx` variants which require you to provide one closure for each variant of your enum.

Note that in the `AllInts` example, you should definitely _not_ use `repr(stabby)` in hopes of getting better performance, as it will not be able to provide any layout optimization benefits; and contrary to `repr(u8)`,
will not be able to have its matches be optimized to lookup tables. `stabby` will notably force you to explicitly pick your representation explicitly in this case, as it will realize that its default representation will
not provide you with the benefits it was designed to give you.
```compile_fail
#[stabby::stabby]
// without an explicit `repr`, this doesn't compile,
// as the default `repr(stabby)` is found not to be beneficial.
pub enum AllInts {
	U8(u8),
	U16(u16),
	U32(u32),
	U64(u64),
	I8(i8),
	I16(i16),
	I32(i32),
	I64(i64),
}
```

### ABI-stability must go all the way down...

For a type to be ABI-stable, not only must its components be assembled in stable ways, but these components must also be ABI-stable.

The [`IStable`] trait is already implemented for most ABI-stable types in Rust's `core`, including all integers\*, non-zero integers\*,
floats, thin pointers, thin references, and transparent types (\*`u128` and `i128` are considered ABI-stable, despite their ABI having changed between `1.76` and `1.77` due to LLVM changing their alignment on x86. `stabby` is able to tell appart these types when they've been compiled with either alignment).

On top of this, `stabby` can tell when `core::option::Option<T>` is ABI-stable (when `T` is ABI-stable and known to have only one possible niche, like thin references and `NonZero` types).

`stabby` also provides ABI-stable equivalents to a few of the core allocated types from `alloc`, as none of `alloc`'s types are ABI-stable, notably because their default allocator is not guaranteed to stay the same ([it has changed before](https://internals.rust-lang.org/t/jemalloc-was-just-removed-from-the-standard-library/8759)), despite the allocator being a type invariant for any type containing owning pointers.

Allocator choice is a great excuse to introduce another important concept in ABI-stability: type invariants are part of your ABI. This is important because this means that if you decide to include (or not to include) and invariant in your type at any point, changing your mind on this
is an ABI breakage, as passing memory from a binary that doesn't uphold the invariant to one that expects it to be upheld may lead to undefined behaviour.

### ... Unless you opacify types to their dependents

Sometimes, you might decide that you'd rather not commit to a given representation for your type, or for a given set of invariants. That's perfectly normal, especially if that type is expected to allow complex behaviour.

Lucky for you, a solution for that has existed since times immemorial: opaque types.

The core principle with opaque types is to make consumer code completely unaware of their internal representation, limiting interraction to functions that return and accept pointers to them. The `FILE` and `socket`APIs in C are prime examples of this.

Opaque types are typically used when only one implementation of their API is expected to be loaded at any given time. The moment you expect more, what you probably want is trait objects.

While this isn't yet implemented, as I'm still looking for ways to do it both conveniently and reliably, `stabby` will eventually try to provide a way to define opaque types with minimal boilerplate such that their binary code is only included when built as a shared object, while
dependents on them would instead get bindings to interract with said shared objects as if they were standard Rust code. In the meantime, [trait objects](defining-abi-stable-traits) can fulfill the same role at a slightly higher runtime cost, but with greater flexibility yet.

## Defining ABI stable traits

`stabby::stabby` can also be applied to traits! Doing so will let you use these traits in ABI-stable trait objects using a rather familiar syntax.
```rust
use stabby::boxed::Box;
#[stabby::stabby(checked)]
// `checked` verifies that all function signatures are ABI-stable.
// A following example will show why it is disabled by default.
pub trait Volume {
	extern "C" fn in_liters(&self) -> f32;
}
#[stabby::export]
pub extern "C" fn teaspoons(n: f32) -> stabby::dynptr!(Box<dyn Volume>) {
	struct Teaspoon(f32);
	impl Volume for Teaspoon {
		extern "C" fn in_liters(&self) -> f32 { 0.004928 * self.0 }
	}
	Box::new(Teaspoon(n)).into()
}
```

`stabby::dynptr` allows you to obtain the type of the `stabby`-defined trait object with the familiar trait object syntax. Here, it actually expands to the pretty horrid 
```text
stabby::abi::Dyn<
	'static,
	Box<()>,
	stabby::abi::VTable<VtVolume, VtDrop>
>
```

### Common pitfalls
An error you may quickly run into when working with multiple trait objects is that you can't just use `stabby::dynptr` (or any other macro, for that matter) in the function signatures, because `stabby` cannot see through macros:

```compile_fail
# use stabby::boxed::Box;
# #[stabby::stabby(checked)]
# pub trait Volume {
# 	extern "C" fn in_liters(&self) -> f32;
# }
#[stabby::stabby(checked)]
pub trait Engine {
	extern "C" fn volume(&self) -> stabby::dynptr!(Box<dyn Volume>);
}
```
This is, however, easy to circumvent using a type alias.
```rust
# use stabby::boxed::Box;
# #[stabby::stabby(checked)]
# pub trait Volume {
# 	extern "C" fn in_liters(&self) -> f32;
# }
type BoxedVolume = stabby::dynptr!(Box<dyn Volume>);
#[stabby::stabby(checked)]
pub trait Engine {
	extern "C" fn volume(&self) -> BoxedVolume;
}
```

`stabby` originally checked all traits for ABI-stability by default. This behaviour was changed in `1.0.1` as it would lead to the `cargo check` looping forever when trying to evaluate the trait's stability when one of its methods mentioned it:
```rust
type RefVolume<'a> = stabby::dynptr!(&'a dyn Volume);
#[stabby::stabby] // Adding `checked` here would provoke an infinite loop.
pub trait Volume {
	extern "C" fn in_liters(&self) -> f32;
	extern "C" fn cmp(&self, rhs: RefVolume<'_>) -> core::cmp::Ordering;
}
```

Note however that until I wrote this example, `stabby` didn't know that `core::cmp::Ordering` was actually ABI-stable, yet accepted it as such because the checks were bypassed. I would therefore advise to enable the check whenever possible, as leaving it disabled does punch a hole in stabby's safety net.

Alternatively, if you need to disable checks to prevent a loop, but still want to guarantee stability, you could add the following lines to the previous example:
```rust
const _: () = {
	stabby::abi::assert_stable::<f32>();
	stabby::abi::assert_stable::<core::cmp::Ordering>();
};
```

### Multi-trait objects

`stabby`'s trait objects differ from Rust's in two points:
- While Rust's trait objects automatically take on all of their supertrait's methods into their v-table, `stabby`'s trait objects don't.
- Contrary to Rust, `stabby` allows your trait objects to refer to multiple traits.

Let's imagine you wanted a trait object with all of `Volume` and `Engine`'s methods. Rust's way of handling trait objects would mean you'd need to create a trait
that inherits them both:
```rust
# use stabby::boxed::Box;
# #[stabby::stabby(checked)]
# pub trait Volume {
# 	extern "C" fn in_liters(&self) -> f32;
# }
# type BoxedVolume = stabby::dynptr!(Box<dyn Volume>);
# #[stabby::stabby(checked)]
# pub trait Engine {
# 	extern "C" fn volume(&self) -> BoxedVolume;
# }
pub trait EngineAndVolume: Engine + Volume {}
impl<T: Engine + Volume> EngineAndVolume for T {}
type BoxedEngineAndVolume = Box<dyn EngineAndVolume>;
```
while in stabby, you would instead to the following:
```rust
# use stabby::boxed::Box;
# #[stabby::stabby(checked)]
# pub trait Volume {
# 	extern "C" fn in_liters(&self) -> f32;
# }
# type BoxedVolume = stabby::dynptr!(Box<dyn Volume>);
# #[stabby::stabby(checked)]
# pub trait Engine {
# 	extern "C" fn volume(&self) -> BoxedVolume;
# }
type BoxedEngineAndVolume = stabby::dynptr!(Box<dyn Engine + Volume>);
```

## Creating shared libraries

Now that we have ways to define types that will work accross the shared library boundary, let's see how we can put them to good use!

If you'd rather just look at code, [these examples](https://github.com/ZettaScaleLabs/stabby/tree/main/examples) exist solely to show you how to make a shared [library](https://github.com/ZettaScaleLabs/stabby/tree/main/examples/library), and how to link it [at load-time](https://github.com/ZettaScaleLabs/stabby/tree/main/examples/dynlinkage) or [at runtime with `libloading`](https://github.com/ZettaScaleLabs/stabby/tree/main/examples/libloading).

### Exporting functions in shared objects

Before we can load functions from shared objects, we need to create them. To do so, we need to ask `cargo` to produce one by adding the `cdylib` crate type to our library:
```toml
[lib]
crate-type = ["cdylib"]
```

By doing so, `cargo` will now build an appropriately named artifact for your system, `lib<crate_name>.so` on Linux, which you'll be able to find in the target directory corresponding to your build profile. If you start inspecting it with `nm`, you'll find that it's home to a multitude of oddly named symbols: that's because unless you explicitly ask Rust not to do so, it will mangle symbol names so that symbols that share the same identifier, but were defined in different modules or with different generic parameters, can coexist in the binary and be told appart.

If you've done any C before, you might be used to "manually mangling" your function names: prefixing them with a prefix systematically to avoid your `intvec_push` from clashing with someone else's. That's because C doesn't do mangling (leaving you to mangle your mind and fingers doing it yourself), but that's also what makes it easy to work with shared objects in C.

I just said "unless you explicitly ask Rust not to do so", though. Here's how that looks:
```rust
#[no_mangle]
pub extern "C" fn my_self_mangled_function(param1: u8, param2: u16) {}
```

`#[no_mangle]` is really explicit: the symbol it annotates will not have its name mangled, meaning it will appear as-is in the list of symbols `nm` will now give you.

You might also have spotted that `extern "C"` here, and possibly earlier when we were talking about traits. `extern "X"` is how you tell Rust to use the `X` calling convention for a givent function. Calling conventions are complicated, but the gist of them is this:
- A calling convention defines how a function should be called: where its arguments will be in memory/registers; how the caller should recover their return value; which of the `caller` and `callee` is supposed to save which registers to prevent `caller`'s intermediate results from getting erased by `callee` doing its own job.
- Rust's default calling convention is not "stable": it may change depending on which version of the compiler you're using, or which settings you used for it. This means that calling a function from a different binary that wasn't explicitly annotated with a stable calling convention may result in undefined behaviour.
- The `C` calling convention is one of the most commonly used stable calling conventions. This is also the calling convention you should use when importing symbols from a binary produced by C when no explicit calling conventions have been specified.

Rust will also take that as enough reason not to "tree shake" your function out of the final shared object: even if nothing calls your function in your object's code, it will still be included in the resulting binary. Tree shaking is a common feature in compiled languages where code that's unused will be removed from the final binary (possibly very early in the compilation process to avoid wasting time building that code at all).

### Exporting functions with checks

The previous example is the default way of exporting symbols in Rust when you're planning on dynamically linking to them in other binaries.

However, `stabby` attempts to provide a better way: `#[stabby::export]`, which comes in two flavours:
- The default flavour will export an additional symbol which lets the importer inquire on the exported function's signature, allowing the detection of incompatibilities between the exported function's signature and the signature you're trying to import it as. It is by far `stabby`'s favoured way of exporting symbols, but does require all function parameters to be proven ABI-stable with the [`IStable`] trait. This means that this will only compile if your function's ABI is indeed entirely stable.
- The `canaries` flavour, will export additional symbols which let the importer inquire on the version and settings of the compiler used to compile the shared object. You can use this exporter when you want your function to use types that aren't provably stable in their signature. Doing so is still risky, but the canaries make it less risky than just straight up ignoring the risks. From my previous experience with [Zenoh](https://zenoh.io)'s plugin system, we've never detected any undefined behaviour in doing things without guaranteeing ABI-stability as long as we did check the parameters checked by these canaries.

In either case, `#[stabby::export]` will automatically imply `#[no_mangle]`, and will force you to pick a stable calling convention (which, suprisingly, `#[no_mangle]` doesn't warn you about forgetting).

### Importing functions at runtime

To import functions at runtime, you will first need to link the shared object that contains them at runtime.

No need to panic, it's not that hard: the most common way to do that in Rust is to rely on the [`libloading`](https://crates.io/crates/libloading) crate to interface with whatever your OS provides to load libraries (`dlopen` on POSIX compliant OSes, `LoadLibrary` on Windows). 
```no_run
extern crate libloading;
let lib = unsafe { libloading::Library::new("my_library").unwrap() };
```

Once you've loaded your shared object (or `Library`), you'll have to actually import the symbols you want from it:

```no_run
# extern crate libloading;
# let lib = unsafe { libloading::Library::new("my_library").unwrap() };
let my_imported_function = unsafe { 
		lib.get::<extern "C" fn(u8, u16)>(b"my_self_mangled_function").unwrap()
	};
```

And from this point on, you can use your function! Hurray!

### Importing functions at runtime _with checks_

```text
But wait, how was `stabby` involved in the last example?
```

Very observant of you, imaginary reader: the previous example shows how you'd do it without `stabby`, but then you wouldn't take any advantage from using `#[stabby::export]` instead of `#[no_mangle]`.

`stabby` offers two alternative ways to `get` a symbol from a [`libloading::Library`](::libloading::Library), provided through the [`StabbyLibrary`](crate::libloading::StabbyLibrary) trait.

The trait adds the [`get_stabbied`](crate::libloading::StabbyLibrary::get_stabbied) and [`get_canaried`](crate::libloading::StabbyLibrary::get_canaried) methods to [`libloading::Library`](::libloading::Library). These can be used to load symbols that were exported with `#[stabby::export]` and `#[stabby::export(canaries)]` respectively, and will return errors if the signatures or canaries respectively mismatch the ones you expected.