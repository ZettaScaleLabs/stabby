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

use crate::*;

use super::istable::{IBitMask, IForbiddenValues, Saturator};

// #[crate::stabby]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Padded<Left: Unsigned, T> {
    pub lpad: Left::Padding,
    pub value: T,
}
unsafe impl<Left: Unsigned, T: IStable> IStable for Padded<Left, T> {
    type Size = Left::Add<T::Size>;
    type Align = T::Align;
    type ForbiddenValues = <T::ForbiddenValues as IForbiddenValues>::Shift<Left>;
    type UnusedBits = <<Left::Padding as IStable>::UnusedBits as IBitMask>::BitOr<
        <T::UnusedBits as IBitMask>::Shift<Left>,
    >;
    type HasExactlyOneNiche = Saturator;
    const REPORT: &'static report::TypeReport = T::REPORT;
}
impl<Left: Unsigned, T> From<T> for Padded<Left, T> {
    fn from(value: T) -> Self {
        Self {
            lpad: Default::default(),
            value,
        }
    }
}
impl<Left: Unsigned, T> core::ops::Deref for Padded<Left, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl<Left: Unsigned, T> core::ops::DerefMut for Padded<Left, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

// pub trait IPadding: Unsigned {
//     type Padding: Default + Sized + Copy;
// }
// impl IPadding for UTerm {
//     type Padding = ();
// }
// impl<A: IPadding, B: Bit> IPadding for UInt<A, B>
// where
//     Self::AbsSub<U1>: IPadding,
// {
//     type Padding = FieldPair<<Self::AbsSub<U1> as IPadding>::Padding, PadByte>;
// }
// // impl<A: IPadding> IPadding for UInt<A, B0> {
// //     type Padding = A::Padding;
// // }
// // impl IPadding for UInt<UTerm, B1> {
// //     type Padding = PadByte;
// // }
// // impl<A: Bit> IPadding for UInt<UInt<UTerm, A>, B1>
// // where
// //     UInt<UTerm, A>: IPadding,
// // {
// //     type Padding = Tuple2<
// //         <UInt<UTerm, A> as IPadding>::Padding,
// //         Tuple2<<U1 as IPadding>::Padding, <U1 as IPadding>::Padding>,
// //     >;
// // }
// // impl<A: Bit, B: Bit> IPadding for UInt<UInt<UInt<UTerm, A>, B>, B1>
// // where
// //     UInt<UInt<UTerm, A>, B>: IPadding,
// // {
// //     type Padding = Tuple2<
// //         <UInt<UInt<UTerm, A>, B> as IPadding>::Padding,
// //         Tuple2<<U2 as IPadding>::Padding, <U2 as IPadding>::Padding>,
// //     >;
// // }
// // impl<A: Bit, B: Bit, C: Bit> IPadding for UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, B1>
// // where
// //     UInt<UInt<UInt<UTerm, A>, B>, C>: IPadding,
// // {
// //     type Padding = Tuple2<
// //         <UInt<UInt<UInt<UTerm, A>, B>, C> as IPadding>::Padding,
// //         Tuple2<<U4 as IPadding>::Padding, <U4 as IPadding>::Padding>,
// //     >;
// // }
// // impl<A: Bit, B: Bit, C: Bit, D: Bit> IPadding
// //     for UInt<UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D>, B1>
// // where
// //     UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D>: IPadding,
// // {
// //     type Padding = Tuple2<
// //         <UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D> as IPadding>::Padding,
// //         Tuple2<<U8 as IPadding>::Padding, <U8 as IPadding>::Padding>,
// //     >;
// // }
// // impl<A: Bit, B: Bit, C: Bit, D: Bit, E: Bit> IPadding
// //     for UInt<UInt<UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D>, E>, B1>
// // where
// //     UInt<UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D>, E>: IPadding,
// // {
// //     type Padding = Tuple2<
// //         <UInt<UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D>, E> as IPadding>::Padding,
// //         Tuple2<<U16 as IPadding>::Padding, <U16 as IPadding>::Padding>,
// //     >;
// // }
// // impl<A: Bit, B: Bit, C: Bit, D: Bit, E: Bit, F: Bit> IPadding
// //     for UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D>, E>, F>, B1>
// // where
// //     UInt<UInt<UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D>, E>, F>: IPadding,
// // {
// //     type Padding = Tuple2<
// //         <UInt<UInt<UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D>, E>, F> as IPadding>::Padding,
// //         Tuple2<<U32 as IPadding>::Padding, <U32 as IPadding>::Padding>,
// //     >;
// // }
// // impl<A: Bit, B: Bit, C: Bit, D: Bit, E: Bit, F: Bit, G: Bit> IPadding
// //     for UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D>, E>, F>, G>, B1>
// // where
// //     UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D>, E>, F>, G>: IPadding,
// // {
// //     type Padding = Tuple2<
// //         <UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D>, E>, F>, G> as IPadding>::Padding,
// //         Tuple2<<U64 as IPadding>::Padding, <U64 as IPadding>::Padding>,
// //     >;
// // }
// // impl<A: Bit, B: Bit, C: Bit, D: Bit, E: Bit, F: Bit, G: Bit, H: Bit> IPadding
// //     for UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D>, E>, F>, G>, H>, B1>
// // where
// //     UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D>, E>, F>, G>, H>: IPadding,
// // {
// //     type Padding = Tuple2<
// //         <UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D>, E>, F>, G>, H> as IPadding>::Padding,
// //         Tuple2<<U128 as IPadding>::Padding, <U128 as IPadding>::Padding>,
// //     >;
// // }
// // impl<A: Bit, B: Bit, C: Bit, D: Bit, E: Bit, F: Bit, G: Bit, H: Bit, I: Bit> IPadding
// //     for UInt<
// //         UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D>, E>, F>, G>, H>, I>,
// //         B1,
// //     >
// // where
// //     UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D>, E>, F>, G>, H>, I>:
// //         IPadding,
// // {
// //     type Padding = Tuple2<
// //         <UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D>, E>, F>, G>, H>, I> as IPadding>::Padding,
// //         Tuple2<<U256 as IPadding>::Padding, <U256 as IPadding>::Padding>,
// //     >;
// // }
// // impl<A: Bit, B: Bit, C: Bit, D: Bit, E: Bit, F: Bit, G: Bit, H: Bit, I: Bit, J: Bit> IPadding
// //     for UInt<
// //         UInt<
// //             UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D>, E>, F>, G>, H>, I>,
// //             J,
// //         >,
// //         B1,
// //     >
// // where
// //     UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D>, E>, F>, G>, H>, I>, J>:
// //         IPadding,
// // {
// //     type Padding = Tuple2<
// //         <UInt<
// //             UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D>, E>, F>, G>, H>, I>,
// //             J,
// //         > as IPadding>::Padding,
// //         Tuple2<<U512 as IPadding>::Padding, <U512 as IPadding>::Padding>,
// //     >;
// // }
