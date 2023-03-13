use core::marker::PhantomData;

use stabby_macros::tyeval;

use super::{istable::B2, IStable};
use crate::abi::*;

#[repr(transparent)]
#[derive(Debug, Default, Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PadByte(u8);
pub trait IPadding {
    type Padding: Default + Copy;
}
unsafe impl IStable for PadByte {
    type Size = U1;
    type Align = U1;
    type IllegalValues = End;
    type UnusedBits = Array<U0, U255, End>;
    type HasExactlyOneNiche = B0;
}
impl IPadding for UTerm {
    type Padding = ();
}
impl<B, Tail> IPadding for UInt<B, Tail>
where
    Self: Sub<U1>,
    <Self as Sub<U1>>::Output: IPadding,
{
    type Padding = Tuple2<PadByte, <<Self as Sub<U1>>::Output as IPadding>::Padding>;
}

#[crate::stabby]
#[derive(Clone, Copy)]
pub struct Shifted<T, Shift: IPadding> {
    pub shift: Shift::Padding,
    pub value: T,
}
#[crate::stabby]
#[derive(Clone, Copy)]
pub struct BackPadded<T, Pad: IPadding> {
    pub value: T,
    pub shift: Pad::Padding,
}
#[crate::stabby]
#[derive(Clone, Copy)]
pub struct Surrounded<Shift: IPadding, T, Pad: IPadding>(BackPadded<Shifted<T, Shift>, Pad>);

#[repr(C)]
pub union ShiftUnion<Ok, Err, OkShift: IPadding, ErrShift: IPadding> {
    pub ok: core::mem::ManuallyDrop<Shifted<Ok, OkShift>>,
    pub err: core::mem::ManuallyDrop<Shifted<Err, ErrShift>>,
}
impl<Ok, Err, OkShift: IPadding, ErrShift: IPadding> Clone
    for ShiftUnion<Ok, Err, OkShift, ErrShift>
{
    fn clone(&self) -> Self {
        unsafe { core::ptr::read(self) }
    }
}

pub trait IDiscriminant {
    /// # Safety
    /// This function MUST be called after setting `union` to a valid value for type `Ok`
    unsafe fn ok(union: *mut u8) -> Self;
    /// # Safety
    /// This function MUST be called after setting `union` to a valid value for type `Err`
    unsafe fn err(union: *mut u8) -> Self;
    fn is_ok(&self, union: *const u8) -> bool;
}

pub trait IDiscriminantProvider {
    type OkShift: IPadding;
    type ErrShift: IPadding;
    type Discriminant: IDiscriminant + Clone + Copy;
}

impl<Ok: IStable, Err: IStable> IDiscriminantProvider for (Ok, Err)
where
    Ok::Size: Min<Err::Size>,
    Ok::Size: Sub<<Ok::Size as Min<Err::Size>>::Output>,
    Err::Size: Sub<<Ok::Size as Min<Err::Size>>::Output>,
    <Ok::Size as Sub<<Ok::Size as Min<Err::Size>>::Output>>::Output: IPadding,
    <Err::Size as Sub<<Ok::Size as Min<Err::Size>>::Output>>::Output: IPadding,
    (
        Surrounded<U0, Ok, <Err::Size as Sub<<Ok::Size as Min<Err::Size>>::Output>>::Output>,
        Surrounded<U0, Err, <Ok::Size as Sub<<Ok::Size as Min<Err::Size>>::Output>>::Output>,
    ): IDiscriminantComputer,
    (
        Surrounded<U0, Ok, <Err::Size as Sub<<Ok::Size as Min<Err::Size>>::Output>>::Output>,
        Surrounded<U0, Err, <Ok::Size as Sub<<Ok::Size as Min<Err::Size>>::Output>>::Output>,
        <(
            Surrounded<U0, Ok, <Err::Size as Sub<<Ok::Size as Min<Err::Size>>::Output>>::Output>,
            Surrounded<U0, Err, <Ok::Size as Sub<<Ok::Size as Min<Err::Size>>::Output>>::Output>,
        ) as IDiscriminantComputer>::CoerceErr,
    ): IDiscriminantProvider,
{
    type OkShift = <(
        Surrounded<U0, Ok, <Err::Size as Sub<<Ok::Size as Min<Err::Size>>::Output>>::Output>,
        Surrounded<U0, Err, <Ok::Size as Sub<<Ok::Size as Min<Err::Size>>::Output>>::Output>,
        <(
            Surrounded<U0, Ok, <Err::Size as Sub<<Ok::Size as Min<Err::Size>>::Output>>::Output>,
            Surrounded<U0, Err, <Ok::Size as Sub<<Ok::Size as Min<Err::Size>>::Output>>::Output>,
        ) as IDiscriminantComputer>::CoerceErr,
    ) as IDiscriminantProvider>::OkShift;
    type ErrShift = <(
        Surrounded<U0, Ok, <Err::Size as Sub<<Ok::Size as Min<Err::Size>>::Output>>::Output>,
        Surrounded<U0, Err, <Ok::Size as Sub<<Ok::Size as Min<Err::Size>>::Output>>::Output>,
        <(
            Surrounded<U0, Ok, <Err::Size as Sub<<Ok::Size as Min<Err::Size>>::Output>>::Output>,
            Surrounded<U0, Err, <Ok::Size as Sub<<Ok::Size as Min<Err::Size>>::Output>>::Output>,
        ) as IDiscriminantComputer>::CoerceErr,
    ) as IDiscriminantProvider>::ErrShift;
    type Discriminant = <(
        Surrounded<U0, Ok, <Err::Size as Sub<<Ok::Size as Min<Err::Size>>::Output>>::Output>,
        Surrounded<U0, Err, <Ok::Size as Sub<<Ok::Size as Min<Err::Size>>::Output>>::Output>,
        <(
            Surrounded<U0, Ok, <Err::Size as Sub<<Ok::Size as Min<Err::Size>>::Output>>::Output>,
            Surrounded<U0, Err, <Ok::Size as Sub<<Ok::Size as Min<Err::Size>>::Output>>::Output>,
        ) as IDiscriminantComputer>::CoerceErr,
    ) as IDiscriminantProvider>::Discriminant;
}
impl<
        Ok: IStable,
        OkShift: IPadding,
        OkPad: IPadding,
        Err: IStable,
        ErrShift: IPadding,
        ErrPad: IPadding,
    > IDiscriminantProvider
    for (
        Surrounded<OkShift, Ok, OkPad>,
        Surrounded<ErrShift, Err, ErrPad>,
        End,
    )
where
    (
        Surrounded<OkShift, Ok, OkPad>,
        Surrounded<ErrShift, Err, ErrPad>,
    ): IDiscriminantComputer,
    (
        Surrounded<OkShift, Ok, OkPad>,
        Surrounded<ErrShift, Err, ErrPad>,
        End,
        <(
            Surrounded<OkShift, Ok, OkPad>,
            Surrounded<ErrShift, Err, ErrPad>,
        ) as IDiscriminantComputer>::CoerceOk,
    ): IDiscriminantProvider,
{
    type OkShift = <(
        Surrounded<OkShift, Ok, OkPad>,
        Surrounded<ErrShift, Err, ErrPad>,
        End,
        <(
            Surrounded<OkShift, Ok, OkPad>,
            Surrounded<ErrShift, Err, ErrPad>,
        ) as IDiscriminantComputer>::CoerceOk,
    ) as IDiscriminantProvider>::OkShift;
    type ErrShift = <(
        Surrounded<OkShift, Ok, OkPad>,
        Surrounded<ErrShift, Err, ErrPad>,
        End,
        <(
            Surrounded<OkShift, Ok, OkPad>,
            Surrounded<ErrShift, Err, ErrPad>,
        ) as IDiscriminantComputer>::CoerceOk,
    ) as IDiscriminantProvider>::ErrShift;
    type Discriminant = <(
        Surrounded<OkShift, Ok, OkPad>,
        Surrounded<ErrShift, Err, ErrPad>,
        End,
        <(
            Surrounded<OkShift, Ok, OkPad>,
            Surrounded<ErrShift, Err, ErrPad>,
        ) as IDiscriminantComputer>::CoerceOk,
    ) as IDiscriminantProvider>::Discriminant;
}
impl<
        Ok: IStable,
        OkShift: IPadding,
        OkPad: IPadding,
        Err: IStable,
        ErrShift: IPadding,
        ErrPad: IPadding,
    > IDiscriminantProvider
    for (
        Surrounded<OkShift, Ok, OkPad>,
        Surrounded<ErrShift, Err, ErrPad>,
        End,
        End,
    )
where
    (
        Surrounded<OkShift, Ok, OkPad>,
        Surrounded<ErrShift, Err, ErrPad>,
    ): IDiscriminantComputer,
    (
        Surrounded<OkShift, Ok, OkPad>,
        Surrounded<ErrShift, Err, ErrPad>,
        End,
        End,
        <(
            Surrounded<OkShift, Ok, OkPad>,
            Surrounded<ErrShift, Err, ErrPad>,
        ) as IDiscriminantComputer>::CommonUnusedBits,
    ): IDiscriminantProvider,
{
    type OkShift = <(
        Surrounded<OkShift, Ok, OkPad>,
        Surrounded<ErrShift, Err, ErrPad>,
        End,
        End,
        <(
            Surrounded<OkShift, Ok, OkPad>,
            Surrounded<ErrShift, Err, ErrPad>,
        ) as IDiscriminantComputer>::CommonUnusedBits,
    ) as IDiscriminantProvider>::OkShift;
    type ErrShift = <(
        Surrounded<OkShift, Ok, OkPad>,
        Surrounded<ErrShift, Err, ErrPad>,
        End,
        End,
        <(
            Surrounded<OkShift, Ok, OkPad>,
            Surrounded<ErrShift, Err, ErrPad>,
        ) as IDiscriminantComputer>::CommonUnusedBits,
    ) as IDiscriminantProvider>::ErrShift;
    type Discriminant = <(
        Surrounded<OkShift, Ok, OkPad>,
        Surrounded<ErrShift, Err, ErrPad>,
        End,
        End,
        <(
            Surrounded<OkShift, Ok, OkPad>,
            Surrounded<ErrShift, Err, ErrPad>,
        ) as IDiscriminantComputer>::CommonUnusedBits,
    ) as IDiscriminantProvider>::Discriminant;
}

impl<Ok: IStable, OkShift: IPadding, Err: IStable, ErrShift: IPadding> IDiscriminantProvider
    for (
        Surrounded<OkShift, Ok, U0>,
        Surrounded<ErrShift, Err, U0>,
        End,
        End,
        End,
    )
{
    type OkShift = U0;
    type ErrShift = U0;
    type Discriminant = BitDiscriminant;
}
impl<Ok: IStable, OkShift: IPadding, Err: IStable, ErrShift: IPadding, B, Tail>
    IDiscriminantProvider
    for (
        Surrounded<OkShift, Ok, U0>,
        Surrounded<ErrShift, Err, UInt<B, Tail>>,
        End,
        End,
        End,
    )
where
    ErrShift: Add<Err::Align>,
    UInt<B, Tail>: IPadding + Sub<Err::Align>,
    tyeval!(ErrShift + Err::Align): IPadding,
    <UInt<B, Tail> as Sub<Err::Align>>::Output: IPadding,
    (
        Surrounded<OkShift, Ok, U0>,
        Surrounded<tyeval!(ErrShift + Err::Align), Err, <UInt<B, Tail> as Sub<Err::Align>>::Output>,
    ): IDiscriminantComputer,
    (
        Surrounded<OkShift, Ok, U0>,
        Surrounded<tyeval!(ErrShift + Err::Align), Err, <UInt<B, Tail> as Sub<Err::Align>>::Output>,
        <(
            Surrounded<OkShift, Ok, U0>,
            Surrounded<
                tyeval!(ErrShift + Err::Align),
                Err,
                <UInt<B, Tail> as Sub<Err::Align>>::Output,
            >,
        ) as IDiscriminantComputer>::CoerceErr,
    ): IDiscriminantProvider,
{
    type OkShift = <(
        Surrounded<OkShift, Ok, U0>,
        Surrounded<tyeval!(ErrShift + Err::Align), Err, <UInt<B, Tail> as Sub<Err::Align>>::Output>,
        <(
            Surrounded<OkShift, Ok, U0>,
            Surrounded<
                tyeval!(ErrShift + Err::Align),
                Err,
                <UInt<B, Tail> as Sub<Err::Align>>::Output,
            >,
        ) as IDiscriminantComputer>::CoerceErr,
    ) as IDiscriminantProvider>::OkShift;
    type ErrShift = <(
        Surrounded<OkShift, Ok, U0>,
        Surrounded<tyeval!(ErrShift + Err::Align), Err, <UInt<B, Tail> as Sub<Err::Align>>::Output>,
        <(
            Surrounded<OkShift, Ok, U0>,
            Surrounded<
                tyeval!(ErrShift + Err::Align),
                Err,
                <UInt<B, Tail> as Sub<Err::Align>>::Output,
            >,
        ) as IDiscriminantComputer>::CoerceErr,
    ) as IDiscriminantProvider>::ErrShift;
    type Discriminant = <(
        Surrounded<OkShift, Ok, U0>,
        Surrounded<tyeval!(ErrShift + Err::Align), Err, <UInt<B, Tail> as Sub<Err::Align>>::Output>,
        <(
            Surrounded<OkShift, Ok, U0>,
            Surrounded<
                tyeval!(ErrShift + Err::Align),
                Err,
                <UInt<B, Tail> as Sub<Err::Align>>::Output,
            >,
        ) as IDiscriminantComputer>::CoerceErr,
    ) as IDiscriminantProvider>::Discriminant;
}
impl<Ok: IStable, OkShift: IPadding, Err: IStable, ErrShift: IPadding, B, Tail>
    IDiscriminantProvider
    for (
        Surrounded<OkShift, Ok, UInt<B, Tail>>,
        Surrounded<ErrShift, Err, U0>,
        End,
        End,
        End,
    )
where
    OkShift: Add<Ok::Align>,
    UInt<B, Tail>: IPadding + Sub<Ok::Align>,
    tyeval!(OkShift + Ok::Align): IPadding,
    <UInt<B, Tail> as Sub<Ok::Align>>::Output: IPadding,
    (
        Surrounded<tyeval!(OkShift + Ok::Align), Ok, <UInt<B, Tail> as Sub<Ok::Align>>::Output>,
        Surrounded<ErrShift, Err, U0>,
    ): IDiscriminantComputer,
    (
        Surrounded<tyeval!(OkShift + Ok::Align), Ok, <UInt<B, Tail> as Sub<Ok::Align>>::Output>,
        Surrounded<ErrShift, Err, U0>,
        <(
            Surrounded<tyeval!(OkShift + Ok::Align), Ok, <UInt<B, Tail> as Sub<Ok::Align>>::Output>,
            Surrounded<ErrShift, Err, U0>,
        ) as IDiscriminantComputer>::CoerceErr,
    ): IDiscriminantProvider,
{
    type OkShift = <(
        Surrounded<tyeval!(OkShift + Ok::Align), Ok, <UInt<B, Tail> as Sub<Ok::Align>>::Output>,
        Surrounded<ErrShift, Err, U0>,
        <(
            Surrounded<tyeval!(OkShift + Ok::Align), Ok, <UInt<B, Tail> as Sub<Ok::Align>>::Output>,
            Surrounded<ErrShift, Err, U0>,
        ) as IDiscriminantComputer>::CoerceErr,
    ) as IDiscriminantProvider>::OkShift;
    type ErrShift = <(
        Surrounded<tyeval!(OkShift + Ok::Align), Ok, <UInt<B, Tail> as Sub<Ok::Align>>::Output>,
        Surrounded<ErrShift, Err, U0>,
        <(
            Surrounded<tyeval!(OkShift + Ok::Align), Ok, <UInt<B, Tail> as Sub<Ok::Align>>::Output>,
            Surrounded<ErrShift, Err, U0>,
        ) as IDiscriminantComputer>::CoerceErr,
    ) as IDiscriminantProvider>::ErrShift;
    type Discriminant = <(
        Surrounded<tyeval!(OkShift + Ok::Align), Ok, <UInt<B, Tail> as Sub<Ok::Align>>::Output>,
        Surrounded<ErrShift, Err, U0>,
        <(
            Surrounded<tyeval!(OkShift + Ok::Align), Ok, <UInt<B, Tail> as Sub<Ok::Align>>::Output>,
            Surrounded<ErrShift, Err, U0>,
        ) as IDiscriminantComputer>::CoerceErr,
    ) as IDiscriminantProvider>::Discriminant;
}

pub trait IDiscriminantComputer {
    type CoerceErr;
    type CoerceOk;
    type CommonUnusedBits;
}
pub use sealed::{IShiftTarget, Neither, ShiftErr, ShiftOk};
mod sealed {
    pub struct Neither;
    pub struct ShiftOk;
    pub struct ShiftErr;
    pub trait IShiftTarget {}
    impl IShiftTarget for Neither {}
    impl IShiftTarget for ShiftOk {}
    impl IShiftTarget for ShiftErr {}
}

#[repr(u8)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BitDiscriminant {
    Err = 0,
    Ok = 1,
}
unsafe impl IStable for BitDiscriminant {
    type Size = U1;
    type Align = U1;
    type IllegalValues = End;
    type UnusedBits = Array<U0, U254, End>;
    type HasExactlyOneNiche = B2;
}

impl IDiscriminant for BitDiscriminant {
    unsafe fn ok(_: *mut u8) -> Self {
        BitDiscriminant::Ok
    }
    unsafe fn err(_: *mut u8) -> Self {
        BitDiscriminant::Err
    }
    fn is_ok(&self, _: *const u8) -> bool {
        matches!(self, BitDiscriminant::Ok)
    }
}
impl IDiscriminant for End {
    unsafe fn ok(_: *mut u8) -> Self {
        End
    }
    unsafe fn err(_: *mut u8) -> Self {
        End
    }
    fn is_ok(&self, _: *const u8) -> bool {
        true
    }
}
#[derive(Debug, Clone, Copy)]
pub struct ValueIsErr<Offset, Value, Tail>(PhantomData<(Offset, Value)>, Tail);
impl<Offset: Unsigned, Value: Unsigned, Tail: IDiscriminant> IDiscriminant
    for ValueIsErr<Offset, Value, Tail>
{
    unsafe fn ok(union: *mut u8) -> Self {
        ValueIsErr(PhantomData, Tail::ok(union))
    }
    unsafe fn err(union: *mut u8) -> Self {
        let ptr = union as *mut _ as *mut u8;
        *ptr.add(Offset::USIZE) = Value::U8;
        ValueIsErr(PhantomData, Tail::err(union))
    }
    fn is_ok(&self, union: *const u8) -> bool {
        let ptr = union as *const _ as *const u8;
        unsafe { *ptr.add(Offset::USIZE) != Value::U8 && self.1.is_ok(union) }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct BitIsErr<Offset, Mask>(PhantomData<(Offset, Mask)>);
impl<Offset: Unsigned, Mask: Unsigned> IDiscriminant for BitIsErr<Offset, Mask> {
    unsafe fn ok(_: *mut u8) -> Self {
        BitIsErr(PhantomData)
    }
    unsafe fn err(union: *mut u8) -> Self {
        let ptr = union as *mut _ as *mut u8;
        *ptr.add(Offset::USIZE) |= Mask::U8;
        BitIsErr(PhantomData)
    }
    fn is_ok(&self, union: *const u8) -> bool {
        let ptr = union as *const _ as *const u8;
        unsafe { *ptr.add(Offset::USIZE) & Mask::U8 != 0 }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct Not<Discriminant>(Discriminant);
impl<Discriminant: IDiscriminant> IDiscriminant for Not<Discriminant> {
    unsafe fn ok(union: *mut u8) -> Self {
        Not(Discriminant::err(union))
    }
    unsafe fn err(union: *mut u8) -> Self {
        Not(Discriminant::ok(union))
    }
    fn is_ok(&self, union: *const u8) -> bool {
        !self.0.is_ok(union)
    }
}
