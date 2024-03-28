# The Stabby Tutorial

`stabby` provides a set of tools to use all of Rust's power accross FFI with as little runtime overhead as possible. This page is meant to guide you through its various features and how you're expected to use it.



## Defining ABI stable types

`stabby` leverages the trait system to find niches in your types that can be exploited when defining sum-types (`enum`s).

Most of the magic comes from the [`IStable`] trait which acts as a proof that a given type's representation is stable, as well as carries information about said representation (size, alignment, available niches, whether contains indirections, what its fields are...).

Lucky for you, you're _never_ expected to implement it yourself, relying instead on the [`stabby`] attribute macro to implement it for you!

If you're experienced in Rust, you may now wonder why this is an attribute macro and not a derive macro: the answer is that this macro does not limit itself to generating a trait implementation...

### `stabby::stabby` on structs

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

### `stabby::stabby` on enums

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

#### `repr(uX)` and `repr(iX)`

These are `Rust`'s standard way of making ABI-stable sum types: an external tag of type `uX`/`iX` is added in front of the `union` of each
variant's data.

This is actually optimal for `AllInts`, as there isn't any niche to exploit in the largest variants' data (`u64` and `i64` both are types whose range of valid values covers all the values that the memory they occupy can take).

However, `Poll<T>` could have more efficient representations if `T` has "niches": binary patterns that do not correspond to any valid value of `T`. For example, `core::num::NonZeroU64` occupies 64 bits in memory, but is never allowed to be
`0`, which means that we could set all 64 bits to `0` as a way of indicating `Pending`. This is generally refered to as "niche optimizations" in Rust, and is something that normal Rust enums perform, but not `repr(uX/iX/C)`.

#### `repr(stabby)`

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

### ABI-stability must go all the way down

For a type to be ABI-stable, not only must its components be assembled in stable ways, but these components must also be ABI-stable.

The [`IStable`] trait is already implemented for most ABI-stable types in Rust's `core`, including all integers\*, non-zero integers\*,
floats, thin pointers, thin references, and transparent types (\*`u128` and `i128` are considered ABI-stable, despite their ABI having changed between `1.76` and `1.77` due to LLVM changing their alignment on x86. `stabby` is able to tell appart these types when they've been compiled with either alignment.).

On top of this, `stabby` can tell when `core::option::Option<T>` is ABI-stable (when `T` is ABI-stable and known to have only one possible niche, like thin references and `NonZero` types).

`stabby` also provides ABI-stable equivalents to a few of the core allocated types from `alloc`, as none of `alloc`'s types are ABI-stable. If you need more than the types currently offered by `stabby`, contributions are always welcome. However, for more complex types,
[trait objects](#defining-abi-stable-traits) might be a better solution than stabilizing a given representation.

## Defining ABI stable traits.
