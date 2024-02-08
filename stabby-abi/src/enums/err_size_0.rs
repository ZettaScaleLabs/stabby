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

pub use super::*;
// BRANCH Ok::ForbiddenValues
impl<Ok: IStable, Err: IStable> IDeterminantProviderInner for (Ok, Err, UTerm)
where
    DeterminantProviderWithUnit<
        <Ok::ForbiddenValues as IForbiddenValues>::SelectOne,
        Ok::UnusedBits,
    >: IDeterminantProviderInner,
{
    same_as!(DeterminantProviderWithUnit<
        <Ok::ForbiddenValues as IForbiddenValues>::SelectOne,
        Ok::UnusedBits,
    >);
}

pub struct DeterminantProviderWithUnit<ForbiddenValues, UnusedBits: IBitMask>(
    core::marker::PhantomData<(ForbiddenValues, UnusedBits)>,
);
// IF Ok::ForbiddenValues
impl<
        Offset: Unsigned,
        V: Unsigned,
        Tail: IForbiddenValues + IntoValueIsErr,
        UnusedBits: IBitMask,
    > IDeterminantProviderInner
    for DeterminantProviderWithUnit<Array<Offset, V, Tail>, UnusedBits>
{
    type ErrShift = U0;
    type Determinant = <Array<Offset, V, Tail> as IntoValueIsErr>::ValueIsErr;
    type NicheExporter = NicheExporter<End, UnusedBits, Saturator>;
    // type Debug = Self;
}
// ELSE IF Ok::UnusedBits
impl<Offset: Unsigned, V: NonZero, Rest: IBitMask> IDeterminantProviderInner
    for DeterminantProviderWithUnit<End, Array<Offset, V, Rest>>
{
    type ErrShift = U0;
    type Determinant = BitIsErr<
        <Array<Offset, V, Rest> as IBitMask>::ExtractedBitByteOffset,
        <Array<Offset, V, Rest> as IBitMask>::ExtractedBitMask,
    >;
    type NicheExporter =
        NicheExporter<End, <Array<Offset, V, Rest> as IBitMask>::ExtractBit, Saturator>;
    // type Debug = Self;
}
// ELSE
impl IDeterminantProviderInner for DeterminantProviderWithUnit<End, End> {
    type Determinant = BitDeterminant;
    type ErrShift = U0;
    type NicheExporter = ();
    // type Debug = Self;
}
