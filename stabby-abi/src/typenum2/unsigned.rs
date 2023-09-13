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

pub mod typenames {
    use super::{UInt, UTerm, B0, B1};
    include!(concat!(env!("OUT_DIR"), "/unsigned.rs"));
}
use stabby_macros::tyeval;
use typenames::*;

use crate::{
    istable::{IBitMask, ISingleForbiddenValue, Saturator},
    Array, End, IStable,
};
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct UTerm;
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct UInt<Msbs: IUnsignedBase, Bit: IBit>(Msbs, Bit);

#[repr(transparent)]
#[derive(Debug, Default, Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PadByte(u8);
unsafe impl IStable for PadByte {
    type Size = U1;
    type Align = U1;
    type ForbiddenValues = End;
    type UnusedBits = Array<U0, UxFF, End>;
    type HasExactlyOneNiche = B0;
    primitive_report!("PadByte");
}

pub trait IBitBase {
    const _BOOL: bool;
    type _And<T: IBit>: IBit;
    type _Or<T: IBit>: IBit;
    type _Not: IBit;
    type _Ternary<A, B>;
    type _UTernary<A: IUnsigned, B: IUnsigned>: IUnsigned;
    type _NzTernary<A: NonZero, B: NonZero>: NonZero;
    type _BTernary<A: IBit, B: IBit>: IBit;
    type _BmTernary<A: IBitMask, B: IBitMask>: IBitMask;
    type _PTernary<A: IPowerOf2, B: IPowerOf2>: IPowerOf2;
    type AsForbiddenValue: ISingleForbiddenValue;
}
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
    type AsForbiddenValue = Saturator;
}
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
    type AsForbiddenValue = End;
}
pub trait IBit: IBitBase {
    const BOOL: bool;
    type And<T: IBit>: IBit + Sized;
    type Or<T: IBit>: IBit + Sized;
    type Not: IBit + Sized;
    type Ternary<A, B>;
    type UTernary<A: IUnsigned + Sized, B: IUnsigned + Sized>: IUnsigned + Sized;
    type NzTernary<A: NonZero, B: NonZero>: NonZero + Sized;
    type BTernary<A: IBit, B: IBit>: IBit + Sized;
    type BmTernary<A: IBitMask, B: IBitMask>: IBitMask + Sized;
    type PTernary<A: IPowerOf2, B: IPowerOf2>: IPowerOf2 + Sized;
    type Nand<T: IBit>: IBit + Sized;
    type Xor<T: IBit>: IBit + Sized;
    type Equals<T: IBit>: IBit + Sized;
    type AdderSum<Rhs: IBit, Carry: IBit>: IBit + Sized;
    type AdderCarry<Rhs: IBit, Carry: IBit>: IBit + Sized;
    type SuberSum<Rhs: IBit, Carry: IBit>: IBit + Sized;
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
pub trait IUnsignedBase {
    const _U128: u128;
    type Bit: IBitBase;
    type Msb: IUnsigned;
    type _BitAndInner<T: IUnsigned>: IUnsigned;
    type _IsUTerm: IBit;
    type _BitOrInner<T: IUnsigned>: IUnsigned;
    type _Simplified: IUnsigned;
    type _Equals<T: IUnsigned>: IBit;
    type _Add<T: IUnsigned, Carry: IBit>: IUnsigned;
    type _Sub<T: IUnsigned, Carry: IBit>: IUnsigned;
    type _Greater<T: IUnsigned, Hint: IBit>: IBit;
    type _Truncate<T: IUnsigned>: IUnsigned;
    type NextPow2: IUnsigned;
    type Increment: IUnsigned;
    type _Padding: IStable + Default + Copy + Unpin;
    type _SatDecrement: IUnsigned;
    type _TruncateAtRightmostOne: NonZero;
    type _NonZero: NonZero;
    type _Mul<T: IUnsigned>: IUnsigned;
}
pub struct Lesser;
pub struct Equal;
pub struct Greater;

pub trait IUnsigned: IUnsignedBase {
    const U128: u128;
    const USIZE: usize;
    const U64: u64;
    const U32: u32;
    const U16: u16;
    const U8: u8;
    type BitAnd<T: IUnsigned>: IUnsigned;
    type BitOr<T: IUnsigned>: IUnsigned;
    type Equal<T: IUnsigned>: IBit;
    type NotEqual<T: IUnsigned>: IBit;
    type Greater<T: IUnsigned>: IBit;
    type GreaterOrEq<T: IUnsigned>: IBit;
    type Smaller<T: IUnsigned>: IBit;
    type SmallerOrEq<T: IUnsigned>: IBit;
    type Add<T: IUnsigned>: IUnsigned;
    type AbsSub<T: IUnsigned>: IUnsigned;
    type Min<T: IUnsigned>: IUnsigned;
    type Max<T: IUnsigned>: IUnsigned;
    type Truncate<T: IUnsigned>: IUnsigned;
    type Mod<T: IPowerOf2>: IUnsigned;
    type Padding: IStable + Sized + Default + Copy + Unpin;
    type NonZero: NonZero;
    type NextMultipleOf<T: IPowerOf2>: IUnsigned;
    type Cmp<T: IUnsigned>;
    type Mul<T: IUnsigned>: IUnsigned;
}

pub trait IPowerOf2: IUnsigned {
    type Log2: IUnsigned;
    type Min<T: IPowerOf2>: IPowerOf2;
    type Max<T: IPowerOf2>: IPowerOf2;
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
    type Mod<T: IPowerOf2> = Self::Truncate<T::Log2>;
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
}
impl IUnsignedBase for Saturator {
    const _U128: u128 = { panic!("Attempted to convert Saturator into u128") };
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
}
pub trait NonZero: IUnsigned {
    type Decrement: IUnsigned;
    type TruncateAtRightmostOne: IUnsigned;
}
impl NonZero for Saturator {
    type Decrement = Saturator;
    type TruncateAtRightmostOne = Saturator;
}
#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct OneMoreByte<L: IStable + Copy + Default> {
    l: L,
    r: PadByte,
}
unsafe impl<L: IStable + Copy + Default> IStable for OneMoreByte<L> {
    type Size = <L::Size as IUnsignedBase>::Increment;
    type Align = L::Align;
    type ForbiddenValues = L::ForbiddenValues;
    type UnusedBits = <L::UnusedBits as IBitMask>::BitOr<Array<L::Size, UxFF, End>>;
    type HasExactlyOneNiche = L::HasExactlyOneNiche;
    primitive_report!("OneMoreByte");
}
impl<Msb: IUnsigned, Bit: IBit> NonZero for UInt<Msb, Bit> {
    type Decrement =
        <Bit::UTernary<UInt<Msb, B0>, UInt<Msb::_SatDecrement, B1>> as IUnsignedBase>::_Simplified;
    type TruncateAtRightmostOne = Self::_TruncateAtRightmostOne;
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
    type _Padding =
        OneMoreByte<<<Self as IUnsignedBase>::_SatDecrement as IUnsignedBase>::_Padding>;
    type NextPow2 = <Msb::NextPow2 as IUnsigned>::Add<<Self::_IsUTerm as IBit>::UTernary<U0, U1>>;
    type _TruncateAtRightmostOne = Bit::NzTernary<U1, UInt<Msb::_TruncateAtRightmostOne, B0>>;
    type _NonZero = Self;
    type _Mul<T: IUnsigned> = <Bit::UTernary<T, UTerm> as IUnsigned>::Add<
        <UInt<Msb::Mul<T>, B0> as IUnsignedBase>::_Simplified,
    >;
}
impl<Msb: IUnsigned<_IsUTerm = B1>> IPowerOf2 for UInt<Msb, B1> {
    type Log2 = U0;
    type Min<T: IPowerOf2> = <Self::Greater<T> as IBit>::PTernary<T, Self>;
    type Max<T: IPowerOf2> = <Self::Greater<T> as IBit>::PTernary<Self, T>;
}
impl<Msb: IPowerOf2> IPowerOf2 for UInt<Msb, B0> {
    type Log2 = <Msb::Log2 as IUnsignedBase>::Increment;
    type Min<T: IPowerOf2> = <Self::Greater<T> as IBit>::PTernary<T, Self>;
    type Max<T: IPowerOf2> = <Self::Greater<T> as IBit>::PTernary<Self, T>;
}

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
    assert_eq!(U0::_U128, 0);
    assert_eq!(U1::_U128, 1);
    assert_eq!(U2::_U128, 2);
    assert_eq!(U3::_U128, 3);
    assert_eq!(U4::_U128, 4);
    assert_eq!(U5::_U128, 5);
    assert_eq!(U10::_U128, 10);
}
