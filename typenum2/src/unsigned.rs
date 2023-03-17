pub mod typenames {
    use super::{UInt, UTerm, B0, B1};
    include!(concat!(env!("OUT_DIR"), "/unsigned.rs"));
}
use typenames::*;
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct UTerm;
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct UInt<Msbs: IUnsignedBase, Bit: IBit>(Msbs, Bit);

pub trait IBitBase {
    const BOOL: bool;
    type And<T: IBit>: IBit;
    type Or<T: IBit>: IBit;
    type Not: IBit;
    type Ternary<A: IUnsigned, B: IUnsigned>: IUnsigned;
    type BTernary<A: IBit, B: IBit>: IBit;
    type PTernary<A: IPowerOf2, B: IPowerOf2>: IPowerOf2;
}
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct B0;
impl IBitBase for B0 {
    const BOOL: bool = false;
    type And<T: IBit> = Self;
    type Or<T: IBit> = T;
    type Not = B1;
    type Ternary<A: IUnsigned, B: IUnsigned> = B;
    type BTernary<A: IBit, B: IBit> = B;
    type PTernary<A: IPowerOf2, B: IPowerOf2> = B;
}
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct B1;
impl IBitBase for B1 {
    const BOOL: bool = true;
    type And<T: IBit> = T;
    type Or<T: IBit> = Self;
    type Not = B0;
    type Ternary<A: IUnsigned, B: IUnsigned> = A;
    type BTernary<A: IBit, B: IBit> = A;
    type PTernary<A: IPowerOf2, B: IPowerOf2> = A;
}
pub trait IBit: IBitBase {
    type Nand<T: IBit>: IBit;
    type Xor<T: IBit>: IBit;
    type Equals<T: IBit>: IBit;
    type AdderSum<Rhs: IBit, Carry: IBit>: IBit;
    type AdderCarry<Rhs: IBit, Carry: IBit>: IBit;
    type SuberSum<Rhs: IBit, Carry: IBit>: IBit;
    type SuberCarry<Rhs: IBit, Carry: IBit>: IBit;
}
impl<B: IBitBase> IBit for B {
    type Nand<T: IBit> = <Self::And<T> as IBitBase>::Not;
    type Xor<T: IBit> = <Self::And<T::Not> as IBitBase>::Or<T::And<Self::Not>>;
    type Equals<T: IBit> = <Self::Xor<T> as IBitBase>::Not;
    type AdderSum<Rhs: IBit, Carry: IBit> = <Self::Xor<Rhs> as IBit>::Xor<Carry>;
    type AdderCarry<Rhs: IBit, Carry: IBit> =
        <Rhs::And<Carry> as IBitBase>::Or<Self::And<Rhs::Xor<Carry>>>;
    type SuberSum<Rhs: IBit, Carry: IBit> = Self::AdderSum<Rhs, Carry>;
    type SuberCarry<Rhs: IBit, Carry: IBit> =
        <<Self::Not as IBitBase>::And<Rhs::Or<Carry>> as IBitBase>::Or<Self::And<Rhs::And<Carry>>>;
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
}
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
    type NotEqual<T: IUnsigned> = <Self::Equal<T> as IBitBase>::Not;
    type Greater<T: IUnsigned> = Self::_Greater<T, B0>;
    type GreaterOrEq<T: IUnsigned> = <Self::Greater<T> as IBitBase>::Or<Self::Equal<T>>;
    type SmallerOrEq<T: IUnsigned> = <Self::Greater<T> as IBitBase>::Not;
    type Smaller<T: IUnsigned> = <Self::GreaterOrEq<T> as IBitBase>::Not;
    type Add<T: IUnsigned> = Self::_Add<T, B0>;
    type AbsSub<T: IUnsigned> = <<Self::Greater<T> as IBitBase>::Ternary<
        Self::_Sub<T, B0>,
        T::_Sub<Self, B0>,
    > as IUnsignedBase>::_Simplified;
    type Min<T: IUnsigned> = <Self::Greater<T> as IBitBase>::Ternary<T, Self>;
    type Max<T: IUnsigned> = <Self::Greater<T> as IBitBase>::Ternary<Self, T>;
    type Truncate<T: IUnsigned> = <Self::_Truncate<T> as IUnsignedBase>::_Simplified;
    type Mod<T: IPowerOf2> = Self::Truncate<T::Log2>;
    //     <Self::GreaterOrEq<T> as IBitBase>::Ternary<<Self::AbsSub<T> as IUnsigned>::Mod<T>, Self>;
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
    type _Add<T: IUnsigned, Carry: IBit> = Carry::Ternary<T::Increment, T>;
    type _Greater<T: IUnsigned, Hint: IBit> = Hint::And<T::_IsUTerm>;
    type _Sub<T: IUnsigned, Carry: IBit> = UTerm;
    type _Truncate<T: IUnsigned> = UTerm;
    type NextPow2 = U0;
}
impl<Msb: IUnsigned, Bit: IBit> IUnsignedBase for UInt<Msb, Bit> {
    const _U128: u128 = (Msb::_U128 << 1) | (<Self::Bit as IBitBase>::BOOL as u128);
    type Bit = Bit;
    type Msb = Msb;
    type _IsUTerm = <Bit::Not as IBitBase>::And<Msb::_IsUTerm>;
    type _Simplified = <Self::_IsUTerm as IBitBase>::Ternary<UTerm, UInt<Msb::_Simplified, Bit>>;
    type _BitAndInner<T: IUnsigned> = UInt<Msb::_BitAndInner<T::Msb>, Bit::And<T::Bit>>;
    type _BitOrInner<T: IUnsigned> = UInt<Msb::_BitOrInner<T::Msb>, Bit::Or<T::Bit>>;
    type _Equals<T: IUnsigned> = <Bit::Equals<T::Bit> as IBitBase>::And<Msb::Equal<T::Msb>>;
    type _Greater<T: IUnsigned, Hint: IBit> =
        Msb::_Greater<T::Msb, <T::Bit as IBitBase>::BTernary<Hint::And<Bit>, Hint::Or<Bit>>>;
    type Increment = Self::_Add<UTerm, B1>;
    type _Add<T: IUnsigned, Carry: IBit> =
        UInt<Msb::_Add<T::Msb, Bit::AdderCarry<T::Bit, Carry>>, Bit::AdderSum<T::Bit, Carry>>;
    type _Sub<T: IUnsigned, Carry: IBit> =
        UInt<Msb::_Sub<T::Msb, Bit::SuberCarry<T::Bit, Carry>>, Bit::SuberSum<T::Bit, Carry>>;
    type _Truncate<T: IUnsigned> =
        <T::_IsUTerm as IBitBase>::Ternary<UTerm, UInt<Msb::_Truncate<T::AbsSub<U1>>, Bit>>;
    type NextPow2 =
        <Msb::NextPow2 as IUnsigned>::Add<<Self::_IsUTerm as IBitBase>::Ternary<U0, U1>>;
}
impl<Msb: IUnsigned<_IsUTerm = B1>> IPowerOf2 for UInt<Msb, B1> {
    type Log2 = U0;
    type Min<T: IPowerOf2> = <Self::Greater<T> as IBitBase>::PTernary<T, Self>;
    type Max<T: IPowerOf2> = <Self::Greater<T> as IBitBase>::PTernary<Self, T>;
}
impl<Msb: IPowerOf2> IPowerOf2 for UInt<Msb, B0> {
    type Log2 = <Msb::Log2 as IUnsignedBase>::Increment;
    type Min<T: IPowerOf2> = <Self::Greater<T> as IBitBase>::PTernary<T, Self>;
    type Max<T: IPowerOf2> = <Self::Greater<T> as IBitBase>::PTernary<Self, T>;
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
            <A::Greater<B> as IBitBase>::BOOL,
            A::U128 > B::U128,
            "{} > {}",
            A::U128,
            B::U128,
        );
        assert_eq!(
            <A::Smaller<B> as IBitBase>::BOOL,
            A::U128 < B::U128,
            "{} < {}",
            A::U128,
            B::U128,
        );
        assert_eq!(
            <A::Equal<B> as IBitBase>::BOOL,
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
    assert_eq!(U0::_U128, 0);
    assert_eq!(U1::_U128, 1);
    assert_eq!(U2::_U128, 2);
    assert_eq!(U3::_U128, 3);
    assert_eq!(U4::_U128, 4);
    assert_eq!(U5::_U128, 5);
    assert_eq!(U10::_U128, 10);
}
