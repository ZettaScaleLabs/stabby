# A Stable ABI for Rust with compact sum-types
`stabby` is your one-stop-shop to create stable binary interfaces for your shared libraries easily, without having your sum-types (enums) explode in size.

Your main vector of interraction with `stabby` will be the `#[stabby::stabby]` proc-macro, with which you can annotate a lot of things:

## Structures
When you annotate structs with `#[stabby::stabby]`, two things happen:
- The struct becomes `#[repr(C)]`. Unless you specify otherwise or your struct has generic fields, `stabby` will assert that you haven't ordered your fields in a suboptimal manner at compile time.
- `stabby::abi::IStable` will be implemented for your type. It is similar to `abi_stable::Stable`, but represents the layout (including niches) through associated types. This is key to being able to provide niche-optimization in enums (at least, until `#[feature(generic_const_exprs)]` becomes stable).

## Enums
When you annotate an enum with `#[stabby::stabby]`, you may select an existing stable representation (like you must with `abi_stable`), but you may also select `#[repr(stabby)]` (the default representation) to let `stabby` turn your enum into a tagged-union with a twist: the tag may be a ZST that inspects the union to emulate Rust's niche optimizations.

Note that `#[repr(stabby)]` does lose you the ability to pattern-match.

Due to limitations of the trait solver, `#[repr(stabby)]` enums have a few papercuts:
- Compilation times suffer from `#[repr(stabby)]` enums: on my machine, adding one typically adds about one second to compilation time.
- Additional trait bounds are required when writing `impl`-blocks generic enums. They will always be of the form of one or multiple `(A, B): stabby::abi::IDiscriminantProvider` bounds (although `rustc`'s error may suggest more complex tuples, the 2 element tuple will always be the one you should use).

`#[repr(stabby)]` enums are implemented as a balanced binary tree of `stabby::result::Result<Ok, Err>`, so discriminants are always computed between two types through the following process:
- If some of `Err`'s forbidden values (think `0` for non-zero types) fit inside the bits that `Ok` doesn't care for, that value is used to signify that we are in the `Ok` variant.
- The same thing is attempted with `Err` and `Ok`'s roles inverted.
- If no single value discriminant is found, `Ok` and `Err`'s unused bits are intersected. If the intersection exists, the least significant bit is used, while the others are kept as potential niches for sum-types that would contain a `Result<Ok, Err>` variant.
- Should no niche be found, the smallest of the two types is shifted right by its alignment, and the process is attempted again. This shifting process stops if the union would become bigger, or at the 8th time it has been attempted. If the process stops before a niche is found, a single bit will be used as the determinant (shifting the union right by its own alignment).

## Unions
If you want to make your own internally tagged unions, you can tag them with `#[stabby::stabby]` to let `stabby` check that you only used stably variants, and let it know the size and alignment of your union. Note that `stabby` will consider that unions have no niches.

## Traits
When you annotate a trait with `#[stabby::stabby]`, an ABI-stable vtable is generated for it. You can then use any of the following, replacing `VTable` by `stabby::vtable!(TraitA + TraitB<Output=u8> + Send + Sync)` to obtain the stable VTable for all of these traits (note that order is important here, appart from `Send`, `Sync` and other empty traits that may appear anywhere in the sum of traits):
- `DynRef<'a, vtable!(Traits)>` = `&'a dyn Traits`
- `Dyn<&'a mut (), vtable!(Traits)>` = `&'a mut dyn Traits`
- `Dyn<Box<()>, vtable!(Traits)>` = `Box<dyn Traits>`
- `Dyn<Arc<()>, vtable!(Traits)>` = `Arc<dyn Traits>`

## Functions
For now, annotating a function with `#[stabby::stabby]` merely makes it no-mangle and checks its signature to ensure all exchanged types are marked with `stabby::abi::IStable`.

Future plans include:
- `#[stabby::export]` will export a stably-mangled symbol which may be used to extract the function, but also obtain a report of its signature's layout.
- `stabby` would include a function similar to `libloading::Library::get`, which would also check that the signature you specified for a symbol matches the one encoded by the exporter.
- `#[stabby::import]` will act similarly to `#[link]`. Its exact behaviour is still to be defined, but the goal is to obtain the same reliability with shared-dependencies as what `stabby` will grant you with dynamically-loaded libraries.

## Async
`stabby` supports futures through the `stabby::future::Future` trait. Async functions (usable in traits) are turned into functions that return a `Dyn<Box<()>, vtable!(stabby::future::Future + Send + Sync)>` (the `Send` and `Sync` bounds may be removed by using `#[stabby::stabby(unsync, unsend)]`), which itself implements `core::future::Future`.

# The `stabby` "manifesto"
`stabby` was built in response to the lack of ABI-stability in the Rust ecosystem, which makes writing plugins and other dynamic linkage based programs painful. Currently, Rust's only stable ABI is the C ABI, which has no concept of sum-types, let alone niche exploitation.

However, our experience in software engineering has shown that type-size matters a lot to performance, and that sum-types should therefore be encoded in the least space-occupying manner.

My hope with `stabby` comes in two flavors:
- Adoption in the Rust ecosystem: this is my least favorite option, but this would at least let people have a better time with Rust in situations where they need dynamic linkage.
- Triggering a discussion about providing not a stable, but versionned ABI for Rust: `stabby` essentially provides a versionned ABI already through the selected version of the `stabby-abi` crate. In my opinion, a `abi = "1.xx"` (where `xx` would be a subset of `rustc`'s version that the compiler team is willing to support for a given amount of times) key in the cargo manifest would be a much better way to do this.