//
// Copyright (c) 2023 ZettaScale Technology
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   Pierre Avital, <pierre.avital@me.com>
//

/// Name the first great batch of numbers.
pub mod typenames {
    use super::{UInt, UTerm, B0, B1};
    include!(concat!(env!("OUT_DIR"), "/unsigned.rs"));
}
use stabby_macros::tyeval;
use typenames::*;

use crate::{
    istable::{IBitMask, IForbiddenValues, ISaturatingAdd, ISingleForbiddenValue, Saturator},
    Array, End, IStable, Tuple,
};
/// (unsigned)0
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct UTerm;
/// An unsigned number as a list of digits.
/// Ordering the generics this way makes reading types less painful, as the bits appear
/// MSB-first in text form.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct UInt<Msbs: IUnsignedBase, Bit: IBit>(Msbs, Bit);

/// A type to generate paddings.
#[repr(transparent)]
#[derive(Debug, Default, Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PadByte(u8);
// SAFETY: See fields
unsafe impl IStable for PadByte {
    // Trivial
    type Size = U1;
    // Trivial
    type Align = U1;
    // Trivial
    type ForbiddenValues = End;
    // A padding byte uses none of its bits
    type UnusedBits = Array<U0, UxFF, End>;
    // Rust doesn't know there's a niche here
    type HasExactlyOneNiche = B0;
    type ContainsIndirections = B0;
    #[cfg(feature = "experimental-ctypes")]
    type CType = u8;
    primitive_report!("PadByte");
}

/// The basis on which [`IBit`] stands.
pub trait IBitBase {
    /// Support for [`IBit`]
    const _BOOL: bool;
    /// Support for [`IBit`]
    type _And<T: IBit>: IBit;
    /// Support for [`IBit`]
    type _Or<T: IBit>: IBit;
    /// Support for [`IBit`]
    type _Not: IBit;
    /// Support for [`IBit`]
    type _Ternary<A, B>;
    /// Support for [`IBit`]
    type _UTernary<A: IUnsigned, B: IUnsigned>: IUnsigned;
    /// Support for [`IBit`]
    type _NzTernary<A: NonZero, B: NonZero>: NonZero;
    /// Support for [`IBit`]
    type _BTernary<A: IBit, B: IBit>: IBit;
    /// Support for [`IBit`]
    type _BmTernary<A: IBitMask, B: IBitMask>: IBitMask;
    /// Support for [`IBit`]
    type _PTernary<A: IPowerOf2, B: IPowerOf2>: IPowerOf2;
    /// Support for [`IBit`]
    type _FvTernary<A: IForbiddenValues, B: IForbiddenValues>: IForbiddenValues;
    /// Support for [`IBit`]
    type _SfvTernary<A: ISingleForbiddenValue, B: ISingleForbiddenValue>: ISingleForbiddenValue;
    /// Support for [`IBit`]
    type _SaddTernary<A: ISaturatingAdd, B: ISaturatingAdd>: ISaturatingAdd;
    /// Support for [`IBit`]
    type _StabTernary<A: IStable, B: IStable>: IStable;
    /// Ternary for Aligments
    type _ATernary<A: Alignment, B: Alignment>: Alignment;
    /// Support for [`IBit`]
    type AsForbiddenValue: ISingleForbiddenValue;
    /// u8 if B1, () otherwise
    type _Padding: IStable<Align = U1> + Default + Copy + Unpin;
}
/// false
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct B0;
impl IBitBase for B0 {
    const _BOOL: bool = false;
    type _And<T: IBit> = Self;
    type _Or<T: IBit> = T;
    type _Not = B1;
    type _Ternary<A, B> = B;
    type _UTernary<A: IUnsigned, B: IUnsigned> = B;
    type _NzTernary<A: NonZero, B: NonZero> = B;
    type _BTernary<A: IBit, B: IBit> = B;
    type _BmTernary<A: IBitMask, B: IBitMask> = B;
    type _PTernary<A: IPowerOf2, B: IPowerOf2> = B;
    type _FvTernary<A: IForbiddenValues, B: IForbiddenValues> = B;
    type _SfvTernary<A: ISingleForbiddenValue, B: ISingleForbiddenValue> = B;
    type _SaddTernary<A: ISaturatingAdd, B: ISaturatingAdd> = B;
    type _StabTernary<A: IStable, B: IStable> = B;
    type _ATernary<A: Alignment, B: Alignment> = B;
    type _Padding = ();
    type AsForbiddenValue = Saturator;
}
/// true
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct B1;
impl IBitBase for B1 {
    const _BOOL: bool = true;
    type _And<T: IBit> = T;
    type _Or<T: IBit> = Self;
    type _Not = B0;
    type _Ternary<A, B> = A;
    type _UTernary<A: IUnsigned, B: IUnsigned> = A;
    type _NzTernary<A: NonZero, B: NonZero> = A;
    type _BTernary<A: IBit, B: IBit> = A;
    type _BmTernary<A: IBitMask, B: IBitMask> = A;
    type _PTernary<A: IPowerOf2, B: IPowerOf2> = A;
    type _FvTernary<A: IForbiddenValues, B: IForbiddenValues> = A;
    type _SfvTernary<A: ISingleForbiddenValue, B: ISingleForbiddenValue> = A;
    type _SaddTernary<A: ISaturatingAdd, B: ISaturatingAdd> = A;
    type _StabTernary<A: IStable, B: IStable> = A;
    type _ATernary<A: Alignment, B: Alignment> = A;
    type _Padding = PadByte;
    type AsForbiddenValue = End;
}
/// Equivalent to `B0` and `B1`, but can be constructed with a `const` expression.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Bool<const B: bool>;
macro_rules! same_as_bitbase {
    ($src: ty) => {
        const _BOOL: bool = <$src as IBitBase>::_BOOL;
        type _And<T: IBit> = <$src as IBitBase>::_And<T>;
        type _Or<T: IBit> = <$src as IBitBase>::_Or<T>;
        type _Not = <$src as IBitBase>::_Not;
        type _Ternary<A, B> = <$src as IBitBase>::_Ternary<A, B>;
        type _UTernary<A: IUnsigned, B: IUnsigned> = <$src as IBitBase>::_UTernary<A, B>;
        type _NzTernary<A: NonZero, B: NonZero> = <$src as IBitBase>::_NzTernary<A, B>;
        type _BTernary<A: IBit, B: IBit> = <$src as IBitBase>::_BTernary<A, B>;
        type _BmTernary<A: IBitMask, B: IBitMask> = <$src as IBitBase>::_BmTernary<A, B>;
        type _PTernary<A: IPowerOf2, B: IPowerOf2> = <$src as IBitBase>::_PTernary<A, B>;
        type _FvTernary<A: IForbiddenValues, B: IForbiddenValues> =
            <$src as IBitBase>::_FvTernary<A, B>;
        type _SfvTernary<A: ISingleForbiddenValue, B: ISingleForbiddenValue> =
            <$src as IBitBase>::_SfvTernary<A, B>;
        type _SaddTernary<A: ISaturatingAdd, B: ISaturatingAdd> =
            <$src as IBitBase>::_SaddTernary<A, B>;
        type _StabTernary<A: IStable, B: IStable> = <$src as IBitBase>::_StabTernary<A, B>;
        type _ATernary<A: Alignment, B: Alignment> = <$src as IBitBase>::_ATernary<A, B>;
        type _Padding = <$src as IBitBase>::_Padding;
        type AsForbiddenValue = <$src as IBitBase>::AsForbiddenValue;
    };
}
impl IBitBase for Bool<false> {
    same_as_bitbase!(B0);
}
impl IBitBase for Bool<true> {
    same_as_bitbase!(B1);
}
/// A boolean. [`B0`] and [`B1`] are the canonical members of this type-class
pub trait IBit: IBitBase {
    /// Converts from type to value
    const BOOL: bool;
    /// Self & T
    type And<T: IBit>: IBit + Sized;
    /// Self | T
    type Or<T: IBit>: IBit + Sized;
    /// !Self
    type Not: IBit + Sized;
    /// Self ? A : B
    type Ternary<A, B>;
    /// Self ? A : B, preserving bounds.
    type UTernary<A: IUnsigned + Sized, B: IUnsigned + Sized>: IUnsigned + Sized;
    /// Self ? A : B, preserving bounds.
    type NzTernary<A: NonZero, B: NonZero>: NonZero + Sized;
    /// Self ? A : B, preserving bounds.
    type BTernary<A: IBit, B: IBit>: IBit + Sized;
    /// Self ? A : B, preserving bounds.
    type BmTernary<A: IBitMask, B: IBitMask>: IBitMask + Sized;
    /// Self ? A : B, preserving bounds.
    type PTernary<A: IPowerOf2, B: IPowerOf2>: IPowerOf2 + Sized;
    /// Self ? A : B, preserving bounds.
    type FvTernary<A: IForbiddenValues, B: IForbiddenValues>: IForbiddenValues;
    /// Self ? A : B, preserving bounds.
    type SaddTernary<A: ISaturatingAdd, B: ISaturatingAdd>: ISaturatingAdd;
    /// Self ? A : B, preserving bounds.
    type StabTernary<A: IStable, B: IStable>: IStable;
    /// !(Self & Other)
    type Nand<T: IBit>: IBit + Sized;
    /// Self ^ Other
    type Xor<T: IBit>: IBit + Sized;
    /// Self == Other
    type Equals<T: IBit>: IBit + Sized;
    /// The sum bit of a full adder.
    type AdderSum<Rhs: IBit, Carry: IBit>: IBit + Sized;
    /// The carry bit of a full adder.
    type AdderCarry<Rhs: IBit, Carry: IBit>: IBit + Sized;
    /// The sum bit of a substractor.
    type SuberSum<Rhs: IBit, Carry: IBit>: IBit + Sized;
    /// The carry bit of a substractor.
    type SuberCarry<Rhs: IBit, Carry: IBit>: IBit + Sized;
}
impl<Bit: IBitBase> IBit for Bit {
    const BOOL: bool = Self::_BOOL;
    type And<T: IBit> = Self::_And<T>;
    type Or<T: IBit> = Self::_Or<T>;
    type Not = Self::_Not;
    type Ternary<A, B> = Self::_Ternary<A, B>;
    type UTernary<A: IUnsigned, B: IUnsigned> = Self::_UTernary<A, B>;
    type NzTernary<A: NonZero, B: NonZero> = Self::_NzTernary<A, B>;
    type BTernary<A: IBit, B: IBit> = Self::_BTernary<A, B>;
    type BmTernary<A: IBitMask, B: IBitMask> = Self::_BmTernary<A, B>;
    type PTernary<A: IPowerOf2, B: IPowerOf2> = Self::_PTernary<A, B>;
    type FvTernary<A: IForbiddenValues, B: IForbiddenValues> = Self::_FvTernary<A, B>;
    type SaddTernary<A: ISaturatingAdd, B: ISaturatingAdd> = Self::_SaddTernary<A, B>;
    type StabTernary<A: IStable, B: IStable> = Self::_StabTernary<A, B>;
    type Nand<T: IBit> = <Self::_And<T> as IBitBase>::_Not;
    type Xor<T: IBit> = <Self::_And<T::_Not> as IBitBase>::_Or<T::_And<Self::_Not>>;
    type Equals<T: IBit> = <Self::Xor<T> as IBitBase>::_Not;
    type AdderSum<Rhs: IBit, Carry: IBit> = <Self::Xor<Rhs> as IBit>::Xor<Carry>;
    type AdderCarry<Rhs: IBit, Carry: IBit> =
        <Rhs::_And<Carry> as IBitBase>::_Or<Self::_And<Rhs::Xor<Carry>>>;
    type SuberSum<Rhs: IBit, Carry: IBit> = Self::AdderSum<Rhs, Carry>;
    type SuberCarry<Rhs: IBit, Carry: IBit> =
        <<Self::_Not as IBitBase>::_And<Rhs::_Or<Carry>> as IBitBase>::_Or<
            Self::_And<Rhs::_And<Carry>>,
        >;
}
/// The basis for [`IUnsigned`].
pub trait IUnsignedBase {
    /// Support for [`IUnsigned`]
    const _U128: u128;
    /// Support for [`IUnsigned`]
    type Bit: IBitBase;
    /// Support for [`IUnsigned`]
    type Msb: IUnsigned;
    /// Support for [`IUnsigned`]
    type _BitAndInner<T: IUnsigned>: IUnsigned;
    /// Support for [`IUnsigned`]
    type _IsUTerm: IBit;
    /// Support for [`IUnsigned`]
    type _BitOrInner<T: IUnsigned>: IUnsigned;
    /// Support for [`IUnsigned`]
    type _Simplified: IUnsigned;
    /// Support for [`IUnsigned`]
    type _Equals<T: IUnsigned>: IBit;
    /// Support for [`IUnsigned`]
    type _Add<T: IUnsigned, Carry: IBit>: IUnsigned;
    /// Support for [`IUnsigned`]
    type _Sub<T: IUnsigned, Carry: IBit>: IUnsigned;
    /// Support for [`IUnsigned`]
    type _Greater<T: IUnsigned, Hint: IBit>: IBit;
    /// Support for [`IUnsigned`]
    type _Truncate<T: IUnsigned>: IUnsigned;
    /// Support for [`IUnsigned`]
    type NextPow2: IUnsigned;
    /// Support for [`IUnsigned`]
    type Increment: IUnsigned;
    /// Support for [`IUnsigned`]
    type _Padding: IStable<Align = U1> + Default + Copy + Unpin;
    /// Support for [`IUnsigned`]
    type _SatDecrement: IUnsigned;
    /// Support for [`IUnsigned`]
    type _TruncateAtRightmostOne: NonZero;
    /// Support for [`IUnsigned`]
    type _NonZero: NonZero;
    /// Support for [`IUnsigned`]
    type _Mul<T: IUnsigned>: IUnsigned;
    /// Generates the bitmask for a Self bytes long padding.
    type PaddingBitMask: IBitMask;
    /// Helper for [`IUnsignedBase::Array`]
    type _AddToArray<LsbArray: IStable, ArrayStack: IStable>: IStable;
    /// Generates a type that has the same layout as `[T; Self]`
    type Array<T: IStable>: IStable;
}
/// A is smaller than B if `A::Cmp<B>` = Lesser.
pub struct Lesser;
/// A equals B if `A::Cmp<B>` = Equal.
pub struct Equal;
/// A is greater than B if `A::Cmp<B>` = Greater.
pub struct Greater;

/// An unsigned number.
pub trait IUnsigned: IUnsignedBase {
    /// Convert type to value
    const U128: u128;
    /// Convert type to value
    const USIZE: usize;
    /// Convert type to value
    const U64: u64;
    /// Convert type to value
    const U32: u32;
    /// Convert type to value
    const U16: u16;
    /// Convert type to value
    const U8: u8;
    /// Self & T
    type BitAnd<T: IUnsigned>: IUnsigned;
    /// Self | T
    type BitOr<T: IUnsigned>: IUnsigned;
    /// Self == T
    type Equal<T: IUnsigned>: IBit;
    /// Self != T
    type NotEqual<T: IUnsigned>: IBit;
    /// Self > T
    type Greater<T: IUnsigned>: IBit;
    /// Self >= T
    type GreaterOrEq<T: IUnsigned>: IBit;
    /// Self < T
    type Smaller<T: IUnsigned>: IBit;
    /// Self <= T
    type SmallerOrEq<T: IUnsigned>: IBit;
    /// Self + T
    type Add<T: IUnsigned>: IUnsigned;
    /// |Self - T|
    type AbsSub<T: IUnsigned>: IUnsigned;
    /// min(Self, T)
    type Min<T: IUnsigned>: IUnsigned;
    /// max(Self, T)
    type Max<T: IUnsigned>: IUnsigned;
    /// Support for modular operations.
    type Truncate<T: IUnsigned>: IUnsigned;
    /// Self % T
    type Mod<T: IPowerOf2>: IUnsigned;
    /// Constructs a type with alignment 1 and size Self.
    type Padding: IStable<Align = U1> + Sized + Default + Copy + Unpin;
    /// Coerces Self into [`NonZero`]
    type NonZero: NonZero;
    /// Finds the smallest `n` such that `n = T * k` and `n >= Self`
    type NextMultipleOf<T: IPowerOf2>: IUnsigned;
    /// Self.cmp(T)
    type Cmp<T: IUnsigned>;
    /// Self * T
    type Mul<T: IUnsigned>: IUnsigned;
}

/// An unsigned number that's a power of 2
pub trait IPowerOf2: IUnsigned {
    /// log2(Self)
    type Log2: IUnsigned;
    /// min(Self, T)
    type Min<T: IPowerOf2>: IPowerOf2;
    /// max(Self, T)
    type Max<T: IPowerOf2>: IPowerOf2;
    /// T % Self
    type Modulate<T: IUnsigned>: IUnsigned;
    /// T / Self
    type Divide<T: IUnsigned>: IUnsigned;
}
impl<U: IUnsignedBase> IUnsigned for U {
    const U128: u128 = Self::_U128;
    const USIZE: usize = Self::_U128 as usize;
    const U64: u64 = Self::_U128 as u64;
    const U32: u32 = Self::_U128 as u32;
    const U16: u16 = Self::_U128 as u16;
    const U8: u8 = Self::_U128 as u8;
    type BitAnd<T: IUnsigned> = <Self::_BitAndInner<T> as IUnsignedBase>::_Simplified;
    type BitOr<T: IUnsigned> = <Self::_BitOrInner<T> as IUnsignedBase>::_Simplified;
    type Equal<T: IUnsigned> = Self::_Equals<T>;
    type NotEqual<T: IUnsigned> = <Self::Equal<T> as IBit>::Not;
    type Greater<T: IUnsigned> = Self::_Greater<T, B0>;
    type GreaterOrEq<T: IUnsigned> = <Self::Greater<T> as IBit>::Or<Self::Equal<T>>;
    type SmallerOrEq<T: IUnsigned> = <Self::Greater<T> as IBit>::Not;
    type Smaller<T: IUnsigned> = <Self::GreaterOrEq<T> as IBit>::Not;
    type Add<T: IUnsigned> = Self::_Add<T, B0>;
    type AbsSub<T: IUnsigned> = <<Self::Greater<T> as IBit>::UTernary<
        Self::_Sub<T, B0>,
        T::_Sub<Self, B0>,
    > as IUnsignedBase>::_Simplified;
    type Min<T: IUnsigned> = <Self::Greater<T> as IBit>::UTernary<T, Self>;
    type Max<T: IUnsigned> = <Self::Greater<T> as IBit>::UTernary<Self, T>;
    type Truncate<T: IUnsigned> = <Self::_Truncate<T> as IUnsignedBase>::_Simplified;
    type Mod<T: IPowerOf2> = T::Modulate<Self>;
    type Padding = Self::_Padding;
    type NonZero = Self::_NonZero;
    type NextMultipleOf<T: IPowerOf2> =
        tyeval!(((Self % T) == U0) ? Self : (Self + (T - (Self % T))));
    type Cmp<T: IUnsigned> = <Self::Equal<T> as IBit>::Ternary<
        Equal,
        <Self::Greater<T> as IBit>::Ternary<Greater, Lesser>,
    >;
    type Mul<T: IUnsigned> = Self::_Mul<T>;
}
impl IUnsignedBase for UTerm {
    const _U128: u128 = 0;
    type Bit = B0;
    type Msb = UTerm;
    type _IsUTerm = B1;
    type _BitAndInner<T: IUnsignedBase> = UTerm;
    type _BitOrInner<T: IUnsigned> = T;
    type _Simplified = UTerm;
    type _Equals<T: IUnsigned> = T::_IsUTerm;
    type Increment = U1;
    type _Add<T: IUnsigned, Carry: IBit> = Carry::UTernary<T::Increment, T>;
    type _Greater<T: IUnsigned, Hint: IBit> = Hint::And<T::_IsUTerm>;
    type _Sub<T: IUnsigned, Carry: IBit> = UTerm;
    type _Truncate<T: IUnsigned> = UTerm;
    type _Padding = ();
    type _SatDecrement = U0;
    type NextPow2 = U0;
    type _TruncateAtRightmostOne = Saturator;
    type _NonZero = Saturator;
    type _Mul<T: IUnsigned> = UTerm;
    type PaddingBitMask = End;
    type _AddToArray<LsbArray: IStable, ArrayStack: IStable> = LsbArray;
    type Array<T: IStable> = ();
}
impl IUnsignedBase for Saturator {
    #[cfg(not(doc))]
    const _U128: u128 = { panic!("Attempted to convert Saturator into u128") };
    #[cfg(doc)]
    const _U128: u128 = u128::MAX;
    type Bit = B0;
    type Msb = Saturator;
    type _IsUTerm = B1;
    type _BitAndInner<T: IUnsignedBase> = Saturator;
    type _BitOrInner<T: IUnsigned> = Saturator;
    type _Simplified = Saturator;
    type _Equals<T: IUnsigned> = T::_IsUTerm;
    type Increment = Saturator;
    type _Add<T: IUnsigned, Carry: IBit> = Carry::UTernary<T::Increment, T>;
    type _Greater<T: IUnsigned, Hint: IBit> = Hint::And<T::_IsUTerm>;
    type _Sub<T: IUnsigned, Carry: IBit> = Saturator;
    type _Truncate<T: IUnsigned> = Saturator;
    type _Padding = ();
    type _SatDecrement = Saturator;
    type NextPow2 = Saturator;
    type _TruncateAtRightmostOne = Saturator;
    type _NonZero = Saturator;
    type _Mul<T: IUnsigned> = Saturator;
    type PaddingBitMask = End;
    type _AddToArray<LsbArray: IStable, ArrayStack: IStable> = ();
    type Array<T: IStable> = ();
}

/// A non-zero unsigned number.
pub trait NonZero: IUnsigned {
    /// Self--
    type Decrement: IUnsigned;
    /// Self >> (Self.trailing_zeros())
    type TruncateAtRightmostOne: IUnsigned;
}
impl NonZero for Saturator {
    type Decrement = Saturator;
    type TruncateAtRightmostOne = Saturator;
}
/// Adds a byte to a padding `L`
#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct OneMoreByte<L: IStable<Align = U1> + Copy + Default> {
    l: L,
    r: PadByte,
}
// SAFETY: See each field
unsafe impl<L: IStable<Align = U1> + Copy + Default> IStable for OneMoreByte<L> {
    // L::Align = U1 => adding a single byte increments the size
    type Size = <L::Size as IUnsignedBase>::Increment;
    // L::Align = U1, adding a single byte doesn't change that.
    type Align = U1;
    // The added byte has no forbidden values, so we only keep L's
    type ForbiddenValues = L::ForbiddenValues;
    // The enhtire added byte is unused, since it's padding.
    type UnusedBits = <L::UnusedBits as IBitMask>::BitOr<Array<L::Size, UxFF, End>>;
    // Rust doesn't know the padding byte is padding.
    type HasExactlyOneNiche = L::HasExactlyOneNiche;
    // The padding byte doesn't contain indirections.
    type ContainsIndirections = L::ContainsIndirections;
    #[cfg(feature = "experimental-ctypes")]
    type CType = Tuple<L, u8>;
    primitive_report!("OneMoreByte");
}
impl<Msb: IUnsigned, Bit: IBit> NonZero for UInt<Msb, Bit> {
    type Decrement =
        <Bit::UTernary<UInt<Msb, B0>, UInt<Msb::_SatDecrement, B1>> as IUnsignedBase>::_Simplified;
    type TruncateAtRightmostOne = Self::_TruncateAtRightmostOne;
}
/// A helper to generate padding of appropriate size.
#[repr(C)]
pub struct PaddingHelper<Double: IStable<Align = U1>, Bit: IBitBase>(Double, Double, Bit::_Padding);
impl<Double: IStable<Align = U1> + Default, Bit: IBitBase> Default for PaddingHelper<Double, Bit> {
    fn default() -> Self {
        Self(Default::default(), Default::default(), Default::default())
    }
}
impl<Double: IStable<Align = U1> + Copy, Bit: IBitBase> Copy for PaddingHelper<Double, Bit> {}
impl<Double: IStable<Align = U1> + Copy, Bit: IBitBase> Clone for PaddingHelper<Double, Bit> {
    fn clone(&self) -> Self {
        *self
    }
}
unsafe impl<Double: IStable<Align = U1>, Bit: IBitBase> IStable for PaddingHelper<Double, Bit> {
    type Align = U1;
    type Size = <<U2 as IUnsigned>::Mul<Double::Size> as IUnsigned>::Add<Bit::_UTernary<U1, U0>>;
    type ForbiddenValues = End;
    type ContainsIndirections = B0;
    type HasExactlyOneNiche = B0;
    type UnusedBits = <crate::tuple::Tuple3<Double, Double, Bit::_Padding> as IStable>::UnusedBits;
    #[cfg(feature = "experimental-ctypes")]
    type CType = crate::tuple::Tuple3<Double, Double, Bit::_Padding>;
    primitive_report!("Padding");
}
impl<Msb: IUnsigned, Bit: IBit> IUnsignedBase for UInt<Msb, Bit> {
    const _U128: u128 = (Msb::_U128 << 1) | (<Self::Bit as IBit>::BOOL as u128);
    type Bit = Bit;
    type Msb = Msb;
    type _IsUTerm = <Bit::Not as IBit>::And<Msb::_IsUTerm>;
    type _Simplified = <Self::_IsUTerm as IBit>::UTernary<UTerm, UInt<Msb::_Simplified, Bit>>;
    type _BitAndInner<T: IUnsigned> = UInt<Msb::_BitAndInner<T::Msb>, Bit::And<T::Bit>>;
    type _BitOrInner<T: IUnsigned> = UInt<Msb::_BitOrInner<T::Msb>, Bit::Or<T::Bit>>;
    type _Equals<T: IUnsigned> = <Bit::Equals<T::Bit> as IBit>::And<Msb::Equal<T::Msb>>;
    type _Greater<T: IUnsigned, Hint: IBit> =
        Msb::_Greater<T::Msb, <T::Bit as IBit>::BTernary<Hint::And<Bit>, Hint::Or<Bit>>>;
    type Increment = Self::_Add<UTerm, B1>;
    type _Add<T: IUnsigned, Carry: IBit> =
        UInt<Msb::_Add<T::Msb, Bit::AdderCarry<T::Bit, Carry>>, Bit::AdderSum<T::Bit, Carry>>;
    type _Sub<T: IUnsigned, Carry: IBit> =
        UInt<Msb::_Sub<T::Msb, Bit::SuberCarry<T::Bit, Carry>>, Bit::SuberSum<T::Bit, Carry>>;
    type _Truncate<T: IUnsigned> =
        <T::_IsUTerm as IBit>::UTernary<UTerm, UInt<Msb::_Truncate<T::AbsSub<U1>>, Bit>>;
    type _SatDecrement =
        <Bit::UTernary<UInt<Msb, B0>, UInt<Msb::_SatDecrement, B1>> as IUnsignedBase>::_Simplified;
    type _Padding = PaddingHelper<Msb::Padding, Bit>;
    type NextPow2 = <Msb::NextPow2 as IUnsigned>::Add<<Self::_IsUTerm as IBit>::UTernary<U0, U1>>;
    type _TruncateAtRightmostOne = Bit::NzTernary<U1, UInt<Msb::_TruncateAtRightmostOne, B0>>;
    type _NonZero = Self;
    type _Mul<T: IUnsigned> = <Bit::UTernary<T, UTerm> as IUnsigned>::Add<
        <UInt<Msb::Mul<T>, B0> as IUnsignedBase>::_Simplified,
    >;
    type PaddingBitMask = Array<
        U0,
        U255,
        <<Self::_SatDecrement as IUnsignedBase>::PaddingBitMask as IBitMask>::Shift<U1>,
    >;
    type _AddToArray<LsbArray: IStable, ArrayStack: IStable> = Msb::_AddToArray<
        Bit::StabTernary<
            <<LsbArray::Size as IUnsignedBase>::_IsUTerm as IBit>::StabTernary<
                ArrayStack,
                Tuple<ArrayStack, LsbArray>,
            >,
            LsbArray,
        >,
        Tuple<ArrayStack, ArrayStack>,
    >;
    type Array<T: IStable> = Self::_AddToArray<(), T>;
}
impl<Msb: IUnsigned<_IsUTerm = B1>> IPowerOf2 for UInt<Msb, B1> {
    type Log2 = U0;
    type Min<T: IPowerOf2> = <Self::Greater<T> as IBit>::PTernary<T, Self>;
    type Max<T: IPowerOf2> = <Self::Greater<T> as IBit>::PTernary<Self, T>;
    type Modulate<T: IUnsigned> = UTerm;
    type Divide<T: IUnsigned> = T;
}
impl<Msb: IPowerOf2> IPowerOf2 for UInt<Msb, B0> {
    type Log2 = <Msb::Log2 as IUnsignedBase>::Increment;
    type Min<T: IPowerOf2> = <Self::Greater<T> as IBit>::PTernary<T, Self>;
    type Max<T: IPowerOf2> = <Self::Greater<T> as IBit>::PTernary<Self, T>;
    type Modulate<T: IUnsigned> =
        <UInt<Msb::Modulate<T::Msb>, T::Bit> as IUnsignedBase>::_Simplified;
    type Divide<T: IUnsigned> = Msb::Divide<T::Msb>;
}

/// An alignment that `stabby` can build an array arround.
pub trait Alignment: IPowerOf2 {
    /// max(Self, T)
    type Max<T: Alignment>: Alignment;
    /// A type with size and aligment equal to Self
    type AsUint: Copy + Default + IStable;
}
impl Alignment for U1 {
    type Max<T: Alignment> = <Self::Greater<T> as IBitBase>::_ATernary<Self, T>;
    type AsUint = u8;
}
impl Alignment for U2 {
    type Max<T: Alignment> = <Self::Greater<T> as IBitBase>::_ATernary<Self, T>;
    type AsUint = u16;
}
impl Alignment for U4 {
    type Max<T: Alignment> = <Self::Greater<T> as IBitBase>::_ATernary<Self, T>;
    type AsUint = u32;
}
impl Alignment for U8 {
    type Max<T: Alignment> = <Self::Greater<T> as IBitBase>::_ATernary<Self, T>;
    type AsUint = u64;
}
impl Alignment for U16 {
    type Max<T: Alignment> = <Self::Greater<T> as IBitBase>::_ATernary<Self, T>;
    type AsUint = u128;
}
macro_rules! gen_align {
    ($n: literal, $path: ident, $u: ident, $backing: ty) => {
        /// A type with the alignment specified in its name.
        ///
        /// This type is also valid as a contiguous buffer.
        #[crate::stabby]
        #[repr(align($n))]
        #[derive(Debug, Default, Copy, Clone)]
        pub struct $path(pub $backing);
        impl Alignment for $u {
            type Max<T: Alignment> = <Self::Greater<T> as IBitBase>::_ATernary<Self, T>;
            type AsUint = $path;
        }
    };
}
gen_align!(32, Align32, U32, [u8; 32]);
gen_align!(64, Align64, U64, [[u8; 32]; 2]);
gen_align!(128, Align128, U128, [[u8; 32]; 4]);
gen_align!(256, Align256, U256, [[u8; 32]; 8]);
gen_align!(512, Align512, U512, [[u8; 32]; 16]);
gen_align!(1024, Align1024, U1024, [[u8; 32]; 32]);
gen_align!(2048, Align2048, U2048, [[[u8; 32]; 32]; 2]);
gen_align!(4096, Align4096, U4096, [[[u8; 32]; 32]; 4]);
gen_align!(8192, Align8192, U8192, [[[u8; 32]; 32]; 8]);
gen_align!(16384, Align16384, U16384, [[[u8; 32]; 32]; 16]);
gen_align!(32768, Align32768, U32768, [[[u8; 32]; 32]; 32]);
gen_align!(65536, Align65536, U65536, [[[[u8; 32]; 32]; 32]; 2]);

#[allow(unknown_lints)]
#[allow(clippy::missing_transmute_annotations)]
#[test]
fn ops() {
    fn test_pair<A: IUnsigned, B: IUnsigned>() {
        assert_eq!(
            <A::BitAnd<B> as IUnsigned>::U128,
            A::U128 & B::U128,
            "{} & {} ({} & {})",
            A::U128,
            B::U128,
            core::any::type_name::<A>(),
            core::any::type_name::<B>(),
        );
        assert_eq!(
            <A::BitOr<B> as IUnsigned>::U128,
            A::U128 | B::U128,
            "{} | {} ({} | {})",
            A::U128,
            B::U128,
            core::any::type_name::<A>(),
            core::any::type_name::<B>(),
        );
        assert_eq!(
            <A::Add<B> as IUnsigned>::U128,
            A::U128 + B::U128,
            "{} + {} ({} + {})",
            A::U128,
            B::U128,
            core::any::type_name::<A>(),
            core::any::type_name::<B>(),
        );

        let mask = if B::U32 == 0 {
            0
        } else {
            u128::MAX.wrapping_shr(128 - B::U32)
        };
        assert_eq!(
            <A::_Truncate<B> as IUnsigned>::U128,
            A::U128 & mask,
            "{} trunc {} ({mask:x}) ({} trunc {})",
            A::U128,
            B::U128,
            core::any::type_name::<A>(),
            core::any::type_name::<B>(),
        );
        assert_eq!(
            <A::Greater<B> as IBitBase>::_BOOL,
            A::U128 > B::U128,
            "{} > {}",
            A::U128,
            B::U128,
        );
        assert_eq!(
            <A::Smaller<B> as IBitBase>::_BOOL,
            A::U128 < B::U128,
            "{} < {}",
            A::U128,
            B::U128,
        );
        assert_eq!(
            <A::Equal<B> as IBitBase>::_BOOL,
            A::U128 == B::U128,
            "{} == {}",
            A::U128,
            B::U128,
        );
        assert_eq!(
            <A::Max<B> as IUnsigned>::U128,
            A::U128.max(B::U128),
            "{} max {}",
            A::U128,
            B::U128,
        );
        assert_eq!(
            <A::Min<B> as IUnsigned>::U128,
            A::U128.min(B::U128),
            "{} min {}",
            A::U128,
            B::U128,
        );
        assert_eq!(
            <A::AbsSub<B> as IUnsigned>::U128,
            A::U128.abs_diff(B::U128),
            "|{} - {}| (|{} - {}|)",
            A::U128,
            B::U128,
            core::any::type_name::<A>(),
            core::any::type_name::<B>(),
        );
        assert_eq!(
            <A::NextPow2 as IUnsigned>::U32,
            128 - A::U128.leading_zeros(),
            "nextpow2 {}",
            A::U128,
        );
    }
    test_pair::<U0, U0>();
    test_pair::<U0, U1>();
    test_pair::<U1, U0>();
    test_pair::<U1, U1>();
    test_pair::<U1, U2>();
    test_pair::<U2, U2>();
    test_pair::<U3, U1>();
    test_pair::<U4, U1>();
    test_pair::<U2, U1>();
    test_pair::<U4, U2>();
    test_pair::<U4, U3>();
    test_pair::<U4, U4>();
    test_pair::<U4, U5>();
    test_pair::<U5, U1>();
    test_pair::<U1, U5>();
    test_pair::<U5, U4>();
    test_pair::<U2, U3>();
    test_pair::<U3, U2>();
    test_pair::<U10, U0>();
    test_pair::<U10, U5>();
    test_pair::<U10, U4>();
    let _: <U0 as IUnsigned>::BitOr<U1> = <<U1 as IUnsigned>::BitOr<U0>>::default();
    let _: B0 = <<U0 as IUnsigned>::NotEqual<U0>>::default();
    let _: B1 = <<U1 as IUnsigned>::NotEqual<U0>>::default();
    let _: B1 = <<U2 as IUnsigned>::NotEqual<U0>>::default();
    let _: B1 = <<U3 as IUnsigned>::NotEqual<U0>>::default();
    let _: B1 = <<U4 as IUnsigned>::NotEqual<U0>>::default();
    let _: U2 = <<U10 as IUnsigned>::BitAnd<U6>>::default();
    let _: B1 = <<U2 as IUnsigned>::Equal<<U10 as IUnsigned>::BitAnd<U6>>>::default();
    let _: B0 = <<U3 as IUnsigned>::Equal<<U10 as IUnsigned>::BitAnd<U6>>>::default();
    let _: U11 = <<U16 as IPowerOf2>::Modulate<U11>>::default();
    let _: U11 = <<U11 as IUnsigned>::Mod<U16>>::default();
    let _: U10 = <<U10 as IUnsigned>::Mod<U16>>::default();
    let _: U3 = <<U11 as IUnsigned>::Mod<U8>>::default();
    let _: U2 = <<U10 as IUnsigned>::Mod<U8>>::default();
    let _: U3 = <<U11 as IUnsigned>::Mod<U4>>::default();
    let _: U2 = <<U10 as IUnsigned>::Mod<U4>>::default();
    let _: U1 = <<U11 as IUnsigned>::Mod<U2>>::default();
    let _: U0 = <<U10 as IUnsigned>::Mod<U2>>::default();
    let _: U0 = <<U10 as IUnsigned>::Mod<U1>>::default();
    let _: U255 = UxFF::default();
    let _: Ub111100 = <<Ub11111100 as IUnsigned>::BitAnd<Ub111111>>::default();
    let _: U0 = <<U0 as IUnsigned>::NextMultipleOf<U8>>::default();
    let _: U16 = <<U10 as IUnsigned>::NextMultipleOf<U8>>::default();
    let _: U16 = <<U16 as IUnsigned>::NextMultipleOf<U8>>::default();
    let _: U32 = <<U26 as IUnsigned>::NextMultipleOf<U8>>::default();
    let _: U32 = <<U26 as IUnsigned>::NextMultipleOf<U32>>::default();
    assert_eq!(U0::_U128, 0);
    assert_eq!(U1::_U128, 1);
    assert_eq!(U2::_U128, 2);
    assert_eq!(U3::_U128, 3);
    assert_eq!(U4::_U128, 4);
    assert_eq!(U5::_U128, 5);
    assert_eq!(U10::_U128, 10);
    // SAFETY: These are actually compile-time safety checks.
    unsafe { core::mem::transmute::<_, <U22 as IUnsignedBase>::Array<u8>>([0u8; 22]) };
    // SAFETY: These are actually compile-time safety checks.
    unsafe { core::mem::transmute::<_, <U122 as IUnsignedBase>::Array<u16>>([0u16; 122]) };
    // SAFETY: These are actually compile-time safety checks.
    unsafe { core::mem::transmute::<[u8; 0], <U0 as IUnsignedBase>::Array<u8>>([]) };
}
