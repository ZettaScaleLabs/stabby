# A Stable ABI for Rust with compact sum-types
`stabby` is your one-stop-shop to create stable binary interfaces for your shared libraries easily, without having your sum-types (enums) explode in size.

When you annotate structs with `#[stabby::stabby]`, two things happen:
- The struct becomes `#[repr(C)]`. Unless you specify otherwise or your struct has generic fields, `stabby` will assert that you haven't ordered your fields in a suboptimal manner at compile time.
- `stabby::abi::IStable` will be implemented for your type. It is similar to `abi_stable::Stable`, but represents the layout (including niches) through associated types. This is key to being able to provide niche-optimization in enums (at least, until `#[feature(generic_const_exprs)]` becomes stable).

When you annotate an enum with `#[stabby::stabby]`, you may select an existing stable representation (like you must with `abi_stable`), but you may also select `#[repr(stabby)]` (the default representation) to let `stabby` turn your enum into a tagged-union with a twist: the tag may be a ZST that inspects the union to emulate Rust's niche optimizations.
Note that `#[repr(stabby)]` does lose you the ability to pattern-match