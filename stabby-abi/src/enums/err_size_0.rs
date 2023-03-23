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
//   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
//

pub use super::*;
// BRANCH Ok::ForbiddenValues
impl<Ok: IStable, Err: IStable> IDiscriminantProviderInner for (Ok, Err, UTerm)
where
    (
        Ok,
        Err,
        UTerm,
        <Ok::ForbiddenValues as IForbiddenValues>::SelectOne,
    ): IDiscriminantProviderInner,
{
    same_as!((
        Ok,
        Err,
        UTerm,
        <Ok::ForbiddenValues as IForbiddenValues>::SelectOne
    ));
}
// IF Ok::ForbiddenValues
impl<
        Ok: IStable,
        Err: IStable,
        Offset: Unsigned,
        V: Unsigned,
        Tail: IForbiddenValues + IntoValueIsErr,
    > IDiscriminantProviderInner for (Ok, Err, UTerm, Array<Offset, V, Tail>)
{
    type ErrShift = U0;
    type Discriminant = <Array<Offset, V, Tail> as IntoValueIsErr>::ValueIsErr;
    type NicheExporter = NicheExporter<End, Ok::UnusedBits, Saturator>;
}
// ELSE BRANCH Ok::UnusedBits
impl<Ok: IStable, Err: IStable> IDiscriminantProviderInner for (Ok, Err, UTerm, End)
where
    (Ok, Err, UTerm, End, Ok::UnusedBits): IDiscriminantProviderInner,
{
    same_as!((Ok, Err, UTerm, End, Ok::UnusedBits));
}
// IF Ok::UnusedBits
impl<Ok: IStable, Err: IStable, Offset: Unsigned, V: NonZero, Rest: IBitMask>
    IDiscriminantProviderInner for (Ok, Err, UTerm, End, Array<Offset, V, Rest>)
{
    type ErrShift = U0;
    type Discriminant = BitIsErr<
        <Array<Offset, V, Rest> as IBitMask>::ExtractedBitByteOffset,
        <Array<Offset, V, Rest> as IBitMask>::ExtractedBitMask,
    >;
    type NicheExporter =
        NicheExporter<End, <Array<Offset, V, Rest> as IBitMask>::ExtractBit, Saturator>;
}
// ELSE
impl<Ok: IStable, Err: IStable> IDiscriminantProviderInner for (Ok, Err, UTerm, End, End) {
    type Discriminant = BitDiscriminant;
    type ErrShift = U0;
    type NicheExporter = ();
}
