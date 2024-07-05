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

use core::marker::PhantomData;
use stabby_macros::tyeval;

use super::{
    istable::{IBitMask, IForbiddenValues, ISingleForbiddenValue, NicheExporter, Saturator},
    unsigned::NonZero,
    vtable::{H, T},
    IStable,
};
use crate::*;

/// A type that can inspect a union to detect if it's in `ok` or `err` state.
pub trait IDeterminant: IStable {
    /// Sets the union in `ok` state.
    /// # Safety
    /// This function MUST be called after setting `union` to a valid value for type `Ok`
    unsafe fn ok(union: *mut u8) -> Self;
    /// Sets the union in `err` state.
    /// # Safety
    /// This function MUST be called after setting `union` to a valid value for type `Err`
    unsafe fn err(union: *mut u8) -> Self;
    /// Returns the state of the union.
    fn is_det_ok(&self, union: *const u8) -> bool;
    /// Whether the determinant is explicit or implicit.
    type IsNicheTrick: Bit;
}

/// If no niche can be found, an external tag is used.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BitDeterminant {
    /// The union is in the `ok` state.
    Ok = 0,
    /// The union is in the `err` state.
    Err = 1,
}
unsafe impl IStable for BitDeterminant {
    type Size = U1;
    type Align = U1;
    type ForbiddenValues = End;
    type UnusedBits = Array<U0, U254, End>;
    type HasExactlyOneNiche = Saturator;
    type ContainsIndirections = B0;
    #[cfg(feature = "experimental-ctypes")]
    type CType = u8;
    primitive_report!("BitDeterminant");
}

impl IDeterminant for BitDeterminant {
    unsafe fn ok(_: *mut u8) -> Self {
        BitDeterminant::Ok
    }
    unsafe fn err(_: *mut u8) -> Self {
        BitDeterminant::Err
    }
    fn is_det_ok(&self, _: *const u8) -> bool {
        (*self as u8 & 1) == 0
    }
    type IsNicheTrick = B0;
}
impl IDeterminant for End {
    unsafe fn ok(_: *mut u8) -> Self {
        End
    }
    unsafe fn err(_: *mut u8) -> Self {
        End
    }
    fn is_det_ok(&self, _: *const u8) -> bool {
        false
    }
    type IsNicheTrick = B0;
}

/// Indicates that if the `Offset`th byte equals `Value`, and that the `Tail` also says so, `Err` is the current variant of the inspected union.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct ValueIsErr<Offset, Value, Tail: IStable>(PhantomData<(Offset, Value)>, Tail);
impl<Offset, Value, Tail: IStable> Unpin for ValueIsErr<Offset, Value, Tail> {}
unsafe impl<Offset, Value, Tail: IStable> IStable for ValueIsErr<Offset, Value, Tail> {
    type Size = Tail::Size;
    type Align = Tail::Align;
    type ForbiddenValues = Tail::ForbiddenValues;
    type UnusedBits = Tail::UnusedBits;
    type HasExactlyOneNiche = Tail::HasExactlyOneNiche;
    type ContainsIndirections = B0;
    #[cfg(feature = "experimental-ctypes")]
    type CType = ();
    primitive_report!("ValueIsErr");
}
impl<Offset: Unsigned, Value: Unsigned, Tail: IDeterminant + core::fmt::Debug> core::fmt::Debug
    for ValueIsErr<Offset, Value, Tail>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "ValIsErr(ptr[{}]={}, {:?})",
            Offset::USIZE,
            Value::U8,
            &self.1,
        )
    }
}
impl<Offset: Unsigned, Value: Unsigned, Tail: IDeterminant> IDeterminant
    for ValueIsErr<Offset, Value, Tail>
where
    ValueIsErr<Offset, Value, Tail>: IStable,
{
    unsafe fn ok(union: *mut u8) -> Self {
        ValueIsErr(PhantomData, Tail::ok(union))
    }
    unsafe fn err(union: *mut u8) -> Self {
        let ptr = union;
        *ptr.add(Offset::USIZE) = Value::U8;
        ValueIsErr(PhantomData, Tail::err(union))
    }
    fn is_det_ok(&self, union: *const u8) -> bool {
        let ptr = union;
        unsafe { *ptr.add(Offset::USIZE) != Value::U8 || self.1.is_det_ok(union) }
    }
    type IsNicheTrick = B1;
}
/// Coerces a type into a [`ValueIsErr`].
pub trait IntoValueIsErr {
    /// The coerced type.
    type ValueIsErr: IDeterminant + IStable + Unpin;
}
impl IntoValueIsErr for End {
    type ValueIsErr = End;
}
impl<Offset: Unsigned, Value: Unsigned, Tail: IForbiddenValues + IntoValueIsErr> IntoValueIsErr
    for Array<Offset, Value, Tail>
{
    type ValueIsErr = ValueIsErr<Offset, Value, Tail::ValueIsErr>;
}
/// Indicates that if the `Offset`th byte bitanded with `Mask` is non-zero, `Err` is the current variant of the inspected union.
#[crate::stabby]
#[derive(Clone, Copy)]
pub struct BitIsErr<Offset, Mask>(PhantomData<(Offset, Mask)>);
impl<Offset: Unsigned, Mask: Unsigned> core::fmt::Debug for BitIsErr<Offset, Mask> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "BitIsErr(ptr[{}] & {})", Offset::USIZE, Mask::U8)
    }
}
impl<Offset, Mask> Unpin for BitIsErr<Offset, Mask> {}
impl<Offset: Unsigned, Mask: Unsigned> IDeterminant for BitIsErr<Offset, Mask> {
    unsafe fn ok(union: *mut u8) -> Self {
        let ptr = union;
        if Mask::U8 == 1 {
            *ptr.add(Offset::USIZE) = 0;
        }
        *ptr.add(Offset::USIZE) &= u8::MAX ^ Mask::U8;
        BitIsErr(PhantomData)
    }
    unsafe fn err(union: *mut u8) -> Self {
        let ptr = union;
        if Mask::U8 == 1 {
            *ptr.add(Offset::USIZE) = 0;
        }
        *ptr.add(Offset::USIZE) |= Mask::U8;
        BitIsErr(PhantomData)
    }
    fn is_det_ok(&self, union: *const u8) -> bool {
        let ptr = union;
        unsafe { *ptr.add(Offset::USIZE) & Mask::U8 == 0 }
    }
    type IsNicheTrick = B1;
}
/// Inverts the return value of `Determinant`'s inspection.
#[derive(Debug, Clone, Copy)]
pub struct Not<Determinant>(Determinant);
impl<Determinant> Unpin for Not<Determinant> {}
unsafe impl<Determinant: IStable> IStable for Not<Determinant> {
    type Size = Determinant::Size;
    type Align = Determinant::Align;
    type ForbiddenValues = Determinant::ForbiddenValues;
    type UnusedBits = Determinant::UnusedBits;
    type HasExactlyOneNiche = Determinant::HasExactlyOneNiche;
    type ContainsIndirections = Determinant::ContainsIndirections;
    #[cfg(feature = "experimental-ctypes")]
    type CType = Determinant::CType;
    primitive_report!("Not", Determinant);
}
impl<Determinant: IDeterminant> IDeterminant for Not<Determinant>
where
    Not<Determinant>: IStable,
{
    unsafe fn ok(union: *mut u8) -> Self {
        Not(Determinant::err(union))
    }
    unsafe fn err(union: *mut u8) -> Self {
        Not(Determinant::ok(union))
    }
    fn is_det_ok(&self, union: *const u8) -> bool {
        !self.0.is_det_ok(union)
    }
    type IsNicheTrick = Determinant::IsNicheTrick;
}

// "And now for the tricky bit..."
///Proof that stabby can construct a [`crate::Result`] based on `Self` and `Other`'s niches.
pub trait IDeterminantProvider<Other>: IStable {
    /// How much the `Ok` variant must be shifted.
    type OkShift: Unsigned;
    /// How much the `Err` variant must be shifted.
    type ErrShift: Unsigned;
    /// The discriminant.
    type Determinant: IDeterminant + Unpin;
    /// The remaining niches.
    type NicheExporter: IStable + Default + Copy + Unpin;
}
mod seal {
    use super::*;
    pub trait IDeterminantProviderInnerRev {
        type OkShift: Unsigned;
        type ErrShift: Unsigned;
        type Determinant: IDeterminant + Unpin;
        type NicheExporter: IStable + Default + Copy + Unpin;
    }
    pub trait IDeterminantProviderInner {
        type ErrShift: Unsigned;
        type Determinant: IDeterminant + Unpin;
        type NicheExporter: IStable + Default + Copy + Unpin;
    }
}
pub(crate) use seal::*;

/// The alignment of `Union<Ok, Err>`
type UnionAlign<Ok, Err> = <<Ok as IStable>::Align as PowerOf2>::Max<<Err as IStable>::Align>;
/// The size of `Union<Ok, Err>`
type UnionSize<Ok, Err, OkShift, ErrShift> =
    <<tyeval!(<Ok as IStable>::Size + OkShift) as Unsigned>::Max<
        tyeval!(<Err as IStable>::Size + ErrShift),
    > as Unsigned>::NextMultipleOf<UnionAlign<Ok, Err>>;
/// T::Size + Shift
type PaddedSize<T, Shift> = <<T as IStable>::Size as Unsigned>::Add<Shift>;
/// T's unused bits, shifted by Shift bytes
type ShiftedUnusedBits<T, Shift> = <<T as IStable>::UnusedBits as IBitMask>::Shift<Shift>;

/// The unused bits of the Ok variant in a Ok-Err union where the Ok is placed OkShift bytes from the left
pub(crate) type UnionMemberUnusedBits<Ok, Err, OkShift> =
    <<<<OkShift as Unsigned>::Padding as IStable>::UnusedBits as IBitMask>::BitOr<
        ShiftedUnusedBits<Ok, OkShift>,
    > as IBitMask>::BitOr<
        ShiftedUnusedBits<
            <tyeval!(UnionSize<Ok, Err, OkShift, U0> - PaddedSize<Ok, OkShift>) as Unsigned>::Padding,
            PaddedSize<Ok, OkShift>,
        >,
    >;

macro_rules! same_as {
    ($T: ty) => {
        type ErrShift = <$T as IDeterminantProviderInner>::ErrShift;
        type Determinant = <$T as IDeterminantProviderInner>::Determinant;
        type NicheExporter = <$T as IDeterminantProviderInner>::NicheExporter;
        // type Debug = <$T as IDeterminantProviderInner>::Debug;
    };
    ($T: ty, $Trait: ty) => {
        type OkShift = <$T as $Trait>::OkShift;
        type ErrShift = <$T as $Trait>::ErrShift;
        type Determinant = <$T as $Trait>::Determinant;
        type NicheExporter = <$T as $Trait>::NicheExporter;
        // type Debug = <$T as $Trait>::Debug;
    };
}

impl<A: IStable, B: IStable> IDeterminantProvider<B> for A
where
    (A, B, <A::Size as Unsigned>::GreaterOrEq<B::Size>): IDeterminantProviderInnerRev,
{
    same_as!(
        (A, B, <A::Size as Unsigned>::GreaterOrEq<B::Size>),
        IDeterminantProviderInnerRev
    );
}

// IF Ok::Size < Err::Size
impl<Ok: IStable, Err: IStable> IDeterminantProviderInnerRev for (Ok, Err, B0)
where
    (Err, Ok, Ok::Size): IDeterminantProviderInner,
{
    type OkShift = <(Err, Ok, Ok::Size) as IDeterminantProviderInner>::ErrShift;
    type ErrShift = U0;
    type Determinant = Not<<(Err, Ok, Ok::Size) as IDeterminantProviderInner>::Determinant>;
    type NicheExporter = <(Err, Ok, Ok::Size) as IDeterminantProviderInner>::NicheExporter;
    // type Debug = <(Err, Ok, Ok::Size) as IDeterminantProviderInner>::Debug;
}
// ELSE
impl<Ok: IStable, Err: IStable> IDeterminantProviderInnerRev for (Ok, Err, B1)
where
    (Ok, Err, Err::Size): IDeterminantProviderInner,
{
    type OkShift = U0;
    same_as!((Ok, Err, Err::Size));
}

// IF Err::Size == 0
mod err_non_empty;
mod err_size_0;
