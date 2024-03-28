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

Finally, `stabby` is perfectly happy to annotate unit and tuple structs.

### `stabby::stabby` on enums