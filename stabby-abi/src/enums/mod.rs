use core::marker::PhantomData;
use stabby_macros::tyeval;

use super::{
    istable::{IBitMask, IForbiddenValues, ISingleForbiddenValue, NicheExporter, Saturator},
    unsigned::NonZero,
    vtable::{H, T},
    IStable,
};
use crate::*;

pub trait IDiscriminant: IStable {
    /// # Safety
    /// This function MUST be called after setting `union` to a valid value for type `Ok`
    unsafe fn ok(union: *mut u8) -> Self;
    /// # Safety
    /// This function MUST be called after setting `union` to a valid value for type `Err`
    unsafe fn err(union: *mut u8) -> Self;
    fn is_ok(&self, union: *const u8) -> bool;
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
    type ForbiddenValues = End;
    type UnusedBits = Array<U0, U254, End>;
    type HasExactlyOneNiche = Saturator;
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
        false
    }
}
#[derive(Clone, Copy)]
#[repr(C)]
pub struct ValueIsErr<Offset, Value, Tail: IStable>(PhantomData<(Offset, Value)>, Tail);
unsafe impl<Offset, Value, Tail: IStable> IStable for ValueIsErr<Offset, Value, Tail> {
    type Size = Tail::Size;
    type Align = Tail::Align;
    type ForbiddenValues = Tail::ForbiddenValues;
    type UnusedBits = Tail::UnusedBits;
    type HasExactlyOneNiche = Tail::HasExactlyOneNiche;
}
impl<Offset: Unsigned, Value: Unsigned, Tail: IDiscriminant + core::fmt::Debug> core::fmt::Debug
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
impl<Offset: Unsigned, Value: Unsigned, Tail: IDiscriminant> IDiscriminant
    for ValueIsErr<Offset, Value, Tail>
where
    ValueIsErr<Offset, Value, Tail>: IStable,
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
        unsafe { dbg!(*ptr.add(Offset::USIZE)) != Value::U8 || self.1.is_ok(union) }
    }
}
pub trait IntoValueIsErr {
    type ValueIsErr: IDiscriminant + IStable;
}
impl IntoValueIsErr for End {
    type ValueIsErr = End;
}
impl<Offset: Unsigned, Value: Unsigned, Tail: IForbiddenValues + IntoValueIsErr> IntoValueIsErr
    for Array<Offset, Value, Tail>
{
    type ValueIsErr = ValueIsErr<Offset, Value, Tail::ValueIsErr>;
}
#[crate::stabby]
#[derive(Clone, Copy)]
pub struct BitIsErr<Offset, Mask>(PhantomData<(Offset, Mask)>);
impl<Offset: Unsigned, Mask: Unsigned> core::fmt::Debug for BitIsErr<Offset, Mask> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "BitIsErr(ptr[{}] & {})", Offset::USIZE, Mask::U8)
    }
}
impl<Offset: Unsigned, Mask: Unsigned> IDiscriminant for BitIsErr<Offset, Mask> {
    unsafe fn ok(union: *mut u8) -> Self {
        let ptr = union as *mut _ as *mut u8;
        *ptr.add(Offset::USIZE) &= u8::MAX ^ Mask::U8;
        BitIsErr(PhantomData)
    }
    unsafe fn err(union: *mut u8) -> Self {
        let ptr = union as *mut _ as *mut u8;
        *ptr.add(Offset::USIZE) |= Mask::U8;
        BitIsErr(PhantomData)
    }
    fn is_ok(&self, union: *const u8) -> bool {
        let ptr = union as *const _ as *const u8;
        unsafe { *ptr.add(Offset::USIZE) & Mask::U8 == 0 }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct Not<Discriminant>(Discriminant);
unsafe impl<Discriminant: IStable> IStable for Not<Discriminant> {
    type Size = Discriminant::Size;
    type Align = Discriminant::Align;
    type ForbiddenValues = Discriminant::ForbiddenValues;
    type UnusedBits = Discriminant::UnusedBits;
    type HasExactlyOneNiche = Discriminant::HasExactlyOneNiche;
}
impl<Discriminant: IDiscriminant> IDiscriminant for Not<Discriminant>
where
    Not<Discriminant>: IStable,
{
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

// "And now for the tricky bit..."
pub trait IDiscriminantProvider<Other>: IStable {
    type OkShift: Unsigned;
    type ErrShift: Unsigned;
    type Discriminant: IDiscriminant;
    type NicheExporter: IStable + Default + Copy;
}
pub trait IDiscriminantProviderInnerRev {
    type OkShift: Unsigned;
    type ErrShift: Unsigned;
    type Discriminant: IDiscriminant;
    type NicheExporter: IStable + Default + Copy;
}
pub trait IDiscriminantProviderInner {
    type ErrShift: Unsigned;
    type Discriminant: IDiscriminant;
    type NicheExporter: IStable + Default + Copy;
}

/// The alignment of `Union<Ok, Err>`
type UnionAlign<Ok, Err> = <<Ok as IStable>::Align as PowerOf2>::Max<<Err as IStable>::Align>;
/// The size of `Union<Ok, Err>`
type UnionSize<Ok, Err, OkShift, ErrShift> =
    <<tyeval!(<Ok as IStable>::Size + OkShift) as Unsigned>::Max<
        tyeval!(<Err as IStable>::Size + ErrShift),
    > as Unsigned>::NextMultipleOf<UnionAlign<Ok, Err>>;
/// T::Size + Shift
type PaddedSize<T, Shift> = <<T as IStable>::Size as Unsigned>::Add<Shift>;
/// T's forbidden values, shifted by Shift bytes
type ShiftedForbiddenValues<T, Shift> =
    <<T as IStable>::ForbiddenValues as IForbiddenValues>::Shift<Shift>;
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
        type ErrShift = <$T as IDiscriminantProviderInner>::ErrShift;
        type Discriminant = <$T as IDiscriminantProviderInner>::Discriminant;
        type NicheExporter = <$T as IDiscriminantProviderInner>::NicheExporter;
    };
    ($T: ty, $Trait: ty) => {
        type OkShift = <$T as $Trait>::OkShift;
        type ErrShift = <$T as $Trait>::ErrShift;
        type Discriminant = <$T as $Trait>::Discriminant;
        type NicheExporter = <$T as $Trait>::NicheExporter;
    };
}

impl<A: IStable, B: IStable> IDiscriminantProvider<B> for A
where
    (A, B, <A::Size as Unsigned>::GreaterOrEq<B::Size>): IDiscriminantProviderInnerRev,
{
    same_as!(
        (A, B, <A::Size as Unsigned>::GreaterOrEq<B::Size>),
        IDiscriminantProviderInnerRev
    );
}

// IF Ok::Size >= Err::Size
impl<Ok: IStable, Err: IStable> IDiscriminantProviderInnerRev for (Ok, Err, B0)
where
    (Err, Ok, Ok::Size): IDiscriminantProviderInner,
{
    type OkShift = <(Err, Ok, Ok::Size) as IDiscriminantProviderInner>::ErrShift;
    type ErrShift = U0;
    type Discriminant = Not<<(Err, Ok, Ok::Size) as IDiscriminantProviderInner>::Discriminant>;
    type NicheExporter = <(Err, Ok, Ok::Size) as IDiscriminantProviderInner>::NicheExporter;
}
// ELSE
impl<Ok: IStable, Err: IStable> IDiscriminantProviderInnerRev for (Ok, Err, B1)
where
    (Ok, Err, Err::Size): IDiscriminantProviderInner,
{
    type OkShift = U0;
    same_as!((Ok, Err, Err::Size));
}

// IF Err::Size == 0
mod err_non_empty;
mod err_size_0;
