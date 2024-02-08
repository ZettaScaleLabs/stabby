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

#![allow(clippy::type_complexity)] // I mean, what did you expect?

use self::err_size_0::DeterminantProviderWithUnit;

pub use super::*;

pub struct Layout<
    UnionSize: Unsigned,
    Budget,
    OkFv: IForbiddenValues,
    OkUb: IBitMask,
    ErrFv: IForbiddenValues,
    ErrUb: IBitMask,
    ErrSize: Unsigned,
    ErrAlign: PowerOf2,
    ErrOffset: Unsigned,
>(
    core::marker::PhantomData<(
        UnionSize,
        Budget,
        OkFv,
        OkUb,
        ErrFv,
        ErrUb,
        ErrSize,
        ErrAlign,
        ErrOffset,
    )>,
);
pub struct DeterminantProvider<
    Layout,
    ErrFvInOkUb: ISingleForbiddenValue,
    OkFvInErrUb: ISingleForbiddenValue,
    UbIntersect: IBitMask,
>(core::marker::PhantomData<(Layout, ErrFvInOkUb, OkFvInErrUb, UbIntersect)>);

/// Prevents the compiler from doing infinite recursion when evaluating `IDeterminantProvider`
type DefaultRecursionBudget = T<T<T<T<T<T<T<T<H>>>>>>>>;
// ENTER LOOP ON Budget
impl<Ok: IStable, Err: IStable, EI: Unsigned, EB: Bit> IDeterminantProviderInner
    for (Ok, Err, UInt<EI, EB>)
where
    Layout<
        UnionSize<Ok, Err, U0, U0>,
        DefaultRecursionBudget,
        Ok::ForbiddenValues,
        UnionMemberUnusedBits<Ok, Err, U0>,
        Err::ForbiddenValues,
        Err::UnusedBits,
        Err::Size,
        Err::Align,
        U0,
    >: IDeterminantProviderInner,
{
    same_as!(
        Layout<
            UnionSize<Ok, Err, U0, U0>,
            DefaultRecursionBudget,
            Ok::ForbiddenValues,
            UnionMemberUnusedBits<Ok, Err, U0>,
            Err::ForbiddenValues,
            Err::UnusedBits,
            Err::Size,
            Err::Align,
            U0,
        >
    );
}

// EXIT LOOP
impl<
        UnionSize: Unsigned,
        OkFv: IForbiddenValues,
        OkUb: IBitMask,
        ErrFv: IForbiddenValues,
        ErrUb: IBitMask,
        ErrSize: Unsigned,
        ErrAlign: PowerOf2,
        ErrOffset: Unsigned,
    > IDeterminantProviderInner
    for Layout<UnionSize, H, OkFv, OkUb, ErrFv, ErrUb, ErrSize, ErrAlign, ErrOffset>
{
    same_as!(DeterminantProviderWithUnit<End, End>);
}

type UnusedBits<UnionSize, ErrUb, ErrSize, ErrOffset> = <<<ErrOffset as Unsigned>::Padding as IStable>::UnusedBits as IBitMask>::BitOr<<<ErrUb as IBitMask>::Shift<ErrOffset> as IBitMask>::BitOr<<<<tyeval!(UnionSize - (ErrSize + ErrOffset)) as Unsigned>::Padding as IStable>::UnusedBits as IBitMask>::Shift<tyeval!(ErrSize + ErrOffset)>>>;

/// Branch on whether some forbidden values for Err fit inside Ok's unused bits
impl<
        UnionSize: Unsigned,
        Budget,
        OkFv: IForbiddenValues,
        OkUb: IBitMask,
        ErrFv: IForbiddenValues,
        ErrUb: IBitMask,
        ErrSize: Unsigned,
        ErrAlign: PowerOf2,
        ErrOffset: Unsigned,
    > IDeterminantProviderInner
    for Layout<UnionSize, T<Budget>, OkFv, OkUb, ErrFv, ErrUb, ErrSize, ErrAlign, ErrOffset>
where
DeterminantProvider<
        Self,
        <<ErrFv::Shift<ErrOffset> as IForbiddenValues>::SelectFrom<
            OkUb
        > as ISingleForbiddenValue>::Resolve,
        <OkFv::SelectFrom<
            UnusedBits<UnionSize, ErrUb, ErrSize, ErrOffset>
        > as ISingleForbiddenValue>::Resolve,
        OkUb::BitAnd<UnusedBits<UnionSize, ErrUb, ErrSize, ErrOffset>>
 >: IDeterminantProviderInner,
{
    same_as!(DeterminantProvider<
        Self,
        <<ErrFv::Shift<ErrOffset> as IForbiddenValues>::SelectFrom<
            OkUb
        > as ISingleForbiddenValue>::Resolve,
        <OkFv::SelectFrom<
            UnusedBits<UnionSize, ErrUb, ErrSize, ErrOffset>
        > as ISingleForbiddenValue>::Resolve,
        OkUb::BitAnd<UnusedBits<UnionSize, ErrUb, ErrSize, ErrOffset>>
 >);
}

/// If some forbidden values for Err fit inside Ok's unused bits, exit the recursion
impl<
        UnionSize: Unsigned,
        Budget,
        OkFv: IForbiddenValues,
        OkUb: IBitMask,
        ErrFv: IForbiddenValues,
        ErrUb: IBitMask,
        ErrSize: Unsigned,
        ErrAlign: PowerOf2,
        ErrOffset: Unsigned,
        Offset: Unsigned,
        V: Unsigned,
        Tail: IForbiddenValues + ISingleForbiddenValue + IntoValueIsErr,
        OkFvInErrUb: ISingleForbiddenValue,
        UbIntersect: IBitMask,
    > IDeterminantProviderInner
    for DeterminantProvider<
        Layout<UnionSize, Budget, OkFv, OkUb, ErrFv, ErrUb, ErrSize, ErrAlign, ErrOffset>,
        Array<Offset, V, Tail>,
        OkFvInErrUb,
        UbIntersect,
    >
{
    type ErrShift = ErrOffset;
    type Determinant = Not<<Array<Offset, V, Tail> as IntoValueIsErr>::ValueIsErr>;
    type NicheExporter = NicheExporter<End, UbIntersect, Saturator>;
    // type Debug = Self;
}

/// None of Err's forbidden values fit into Ok's unused bits, so branch on wherther
/// some of Ok's forbidden values fit into Err's forbidden value
///
/// If some forbidden values for Ok fit inside Err's unused bits, exit the recursion

impl<
        UnionSize: Unsigned,
        Budget,
        OkFv: IForbiddenValues,
        OkUb: IBitMask,
        ErrFv: IForbiddenValues,
        ErrUb: IBitMask,
        ErrSize: Unsigned,
        ErrAlign: PowerOf2,
        ErrOffset: Unsigned,
        Offset: Unsigned,
        V: Unsigned,
        Tail: IForbiddenValues + ISingleForbiddenValue + IntoValueIsErr,
        UbIntersect: IBitMask,
    > IDeterminantProviderInner
    for DeterminantProvider<
        Layout<UnionSize, Budget, OkFv, OkUb, ErrFv, ErrUb, ErrSize, ErrAlign, ErrOffset>,
        End,
        Array<Offset, V, Tail>,
        UbIntersect,
    >
{
    type ErrShift = ErrOffset;
    type Determinant = <Array<Offset, V, Tail> as IntoValueIsErr>::ValueIsErr;
    type NicheExporter = NicheExporter<End, UbIntersect, Saturator>;
    // type Debug = Self;
}

/// If neither Err nor Ok's unused bits can fit any of the other's forbidden value,
/// check if their unused bits have an intersection
///
/// If Ok and Err's unused bits have an intersection, use it.
impl<
        UnionSize: Unsigned,
        Budget,
        OkFv: IForbiddenValues,
        OkUb: IBitMask,
        ErrFv: IForbiddenValues,
        ErrUb: IBitMask,
        ErrSize: Unsigned,
        ErrAlign: PowerOf2,
        ErrOffset: Unsigned,
        Offset: Unsigned,
        V: NonZero,
        Tail: IBitMask,
    > IDeterminantProviderInner
    for DeterminantProvider<
        Layout<UnionSize, Budget, OkFv, OkUb, ErrFv, ErrUb, ErrSize, ErrAlign, ErrOffset>,
        End,
        End,
        Array<Offset, V, Tail>,
    >
{
    type ErrShift = ErrOffset;
    type Determinant = BitIsErr<
        <Array<Offset, V, Tail> as IBitMask>::ExtractedBitByteOffset,
        <Array<Offset, V, Tail> as IBitMask>::ExtractedBitMask,
    >;
    type NicheExporter =
        NicheExporter<End, <Array<Offset, V, Tail> as IBitMask>::ExtractBit, Saturator>;
    // type Debug = Self;
}
/// If no niche was found, check if Err can still be shifted to the right by its alignment.
impl<
        UnionSize: Unsigned,
        Budget,
        OkFv: IForbiddenValues,
        OkUb: IBitMask,
        ErrFv: IForbiddenValues,
        ErrUb: IBitMask,
        ErrSize: Unsigned,
        ErrAlign: PowerOf2,
        ErrOffset: Unsigned,
    > IDeterminantProviderInner
    for DeterminantProvider<
        Layout<UnionSize, Budget, OkFv, OkUb, ErrFv, ErrUb, ErrSize, ErrAlign, ErrOffset>,
        End,
        End,
        End,
    >
where
    (
        Layout<UnionSize, Budget, OkFv, OkUb, ErrFv, ErrUb, ErrSize, ErrAlign, ErrOffset>,
        <tyeval!((ErrSize + ErrAlign) + ErrOffset) as Unsigned>::SmallerOrEq<UnionSize>,
    ): IDeterminantProviderInner,
{
    same_as!((
        Layout<UnionSize, Budget, OkFv, OkUb, ErrFv, ErrUb, ErrSize, ErrAlign, ErrOffset>,
        <tyeval!((ErrSize + ErrAlign) + ErrOffset) as Unsigned>::SmallerOrEq<UnionSize>
    ));
}
/// If it can't be shifted
impl<
        UnionSize: Unsigned,
        Budget,
        OkFv: IForbiddenValues,
        OkUb: IBitMask,
        ErrFv: IForbiddenValues,
        ErrUb: IBitMask,
        ErrSize: Unsigned,
        ErrAlign: PowerOf2,
        ErrOffset: Unsigned,
    > IDeterminantProviderInner
    for (
        Layout<UnionSize, Budget, OkFv, OkUb, ErrFv, ErrUb, ErrSize, ErrAlign, ErrOffset>,
        B0,
    )
{
    same_as!(DeterminantProviderWithUnit<End, End>);
}

/// If it can be shifted
impl<
        UnionSize: Unsigned,
        Budget,
        OkFv: IForbiddenValues,
        OkUb: IBitMask,
        ErrFv: IForbiddenValues,
        ErrUb: IBitMask,
        ErrSize: Unsigned,
        ErrAlign: PowerOf2,
        ErrOffset: Unsigned,
    > IDeterminantProviderInner
    for (
        Layout<UnionSize, T<Budget>, OkFv, OkUb, ErrFv, ErrUb, ErrSize, ErrAlign, ErrOffset>,
        B1,
    )
where
    Layout<
        UnionSize,
        Budget,
        OkFv,
        OkUb,
        ErrFv,
        ErrUb,
        ErrSize,
        ErrAlign,
        ErrAlign::Add<ErrOffset>,
    >: IDeterminantProviderInner,
{
    same_as!(Layout<
        UnionSize,
        Budget,
        OkFv,
        OkUb,
        ErrFv,
        ErrUb,
        ErrSize,
        ErrAlign,
        ErrAlign::Add<ErrOffset>,
    >);
}
