# The Repr-Stabby Specification
This file details how stabby lays out types, allowing for alternative implementations of `#[repr(stabby)]`. Note that `stabby` deriving from this specification in any way is considered a bug, and I would be grateful that you report it if you found such a case.

"Niches" refer to information about a type's representation. `#[repr(stabby)]` distinguishes two types of niches:
- Forbidden Values (fvs) are the ordered set of values that a type isn't allowed to occupy. These values may be represented over multiple bytes. A given type may have any amount of forbidden values. The addition for fvs is the concatenation of ordered sets. A single forbidden value is an array of `(byte_offset, value)` tuples. The set of forbidden values must be ordered by ascending LSB-first order.
- Unused Bits (unbits) is a bit-mask such that for any `value` of type `T`, `value ^ unbits` is strictly equivalent to `value` as long as it's treated as being of type `T`. The addtion for unbits is bitwise-or.

Note that unbits can never overlap with a forbidden value.

## Primitive types
### Unit `()`
Unit types have size 0, alignment 1, and no niches.

### Boolean
`bool` is a single byte type, `0` represents `false`, `1` represents `true`, and all other values are forbidden.

### Pointers and references
Pointers are considered nullables, whereas references are non-null types (`[0; size_of_ptr]` is their only forbidden value)

## Product types (`struct`)
Product type use the same representation as `#[repr(C)]` structs: fields are ordered in memory in the same order (increasing ptr offset) as in source code (increasing cursor offset at begining of field description).

Their niches are fully preserved: each field's niches are shifted by the field's pointer offset to the beginning offset, and are then added together.

## Max types (`unions`)
Max types have the same layout as C-unions. Max-types never have any niches, as responsibility of their memory is entirely left to the user.

## Sum types (Rust `enum`)
### `#[repr(u*/i*)]` enums
Explicitly tagged unions are treated like `struct {tag: u*, union}`: the tag's potential forbidden values are not exported, nor are potential niches within the union, but the padding between tag and union

### `#[repr(stabby)]` enums
Sum types are defined as a balanced binary tree of `Result<A, B>`. This binary tree is constructed by the following algorithm:
```python
buffer = [variant.ty for variant in enum.variants] # where variants are ordered by offset in source-code.
def binary_tree(buffer):
    if len(buffer) > 2:
        pivot = len(buffer)//2;
        return [binary_tree(buffer[:pivot]), binary_tree(buffer[pivot:])]
    return buffer
buffer = binary_tree(buffer)
# buffer is now a binary tree
```

## `Result<Ok, Err>`
For any pair of types `Ok`, `Err` the following algorithm is applied to compute the `(determinant, ok_shift, err_shift, remaining_unbits)` tuple:
```python
class Unbits:
    def __init__(self, mask: [int]):
        self.mask = mask
    def pad(self, target) -> Unbits:
        mask = copy(self.mask)
        while len(mask) < target:
            mask.append(0xff)
        return Unbits(mask)
    def shift(self, by: int) -> Unbits:
        mask = copy(self.mask)
        for i in range(by):
            mask.insert(0, 0xff)
        return Unbits(mask)
    def can_contain(self, fv_offset: int) -> bool:
        return self.mask[offset] == 0xff
    def extract_bit(self) -> ((int, int), Unbits)
        mask = copy(self.mask)
        for byte_offset in range(len(mask)):
            if mask[offset]:
                bit_offset = rightmost_one(mask[offset])
                mask[offset] ^= 1 << bit_offset
                return ((byte_offset, bit_offset), Unbits(mask))
        return (None, self)
def determinant(Ok, Err) -> (Determinant, int, int, Unbits):
    if Ok.size < Err.size:
        det, ok_shift, err_shift, remaining_niches = determinant(Err, Ok)
        return Not(det), err_shift, ok_shift, remaining_niches
    union_size = max(next_multiple(Ok.size, Err.align), next_multiple(Err.size, Ok.align))
    ok_unbits = Ok.unbits.pad(union_size)
    # this limit is a technical limitation of Rust's current type system, where this ABI was first defined.
    for i in range(8): 
        shift = i * Err.align
        err_unbits = Err.unbits.shift(shift).pad(union_size)
        unbits = ok_unbits & err_unbits
        for fv in Err.fvs:
            if ok_unbits.can_contain(fv.offset):
                return ValueIsErr(fv), 0, shift, unbits
        for fv in Ok.fvs:
            if err_unbits.can_contain(fv):
                return Not(ValueIsErr(fv)), 0, shift, unbits
        if unbits:
            bit, unbits = unbits.extract_bit()
            return BitIsErr(bit), 0, shift, unbits
        if Err.size + shift + Err.align > union_size:
            break
    return BitDeterminant(), 0, 0, Unbits([0 for _ in union_size])
```

`U` is defined as the union between `Ok` shifted by `ok_shift` bytes and `Err` shifted by `err_shift` bytes, with `remaining_unbits` as its unbits, and no forbidden values.

`Result<Ok, Err>`'s layout depends on determinant:
- `BitDeterminant()`: the Result is laid out as `struct {tag: Bit, union: U}`, where `bit == 1` signifies that `U` is `Err`.
- `ValueIsErr(fv)`: the Result is laid out as `U`, where `all(self[offset] == value for (offset, value) in fv)` signifies that `U` is `Err`.
- `BitIsErr((byte_offset, bit_offset))`: the Result is laid out as `U`, where `self[byte_offset] & (1<<bit_offset) != 0` signifies that `U` is `Err`.
- `Not(Determinant)`: the Result is laid out as with `Determinant`, but the `tag.is_ok(union) == true` signifies that `U` is `Err` instead of `Ok`

## `Option<T>`
`Option<T>` is laid out in memory as if it was `Result<T, ()>`.

# Future possibilities
At the moment, `Result<Ok, Err>` never has any forbidden values left, even if `Ok` had multiple fvs that could fit in `Err`'s unbits. This means that `Option<Option<bool>>` occupies 2 bytes, instead of 1 as it does with Rust's current layout.

Since extracting only the correct FV from FVs can be complex to implement in computation contexts such as Rust's trait-solver, the choice was made not to do it. Should this become feasible, a release process will have to be designed. If you implement `#[repr(stabby)]`, please [file an issue on `stabby`'s original repository'](https://github.com/ZettaScaleLabs/stabby/issues/new) to be notified.
