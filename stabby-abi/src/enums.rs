use core::marker::PhantomData;
use stabby_macros::tyeval;

use super::{
    istable::{IBitMask, IForbiddenValues, ISingleForbiddenValue, NicheExporter, Saturator},
    unsigned::{Equal, Greater, Lesser, NonZero},
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
        true
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
        unsafe { *ptr.add(Offset::USIZE) != Value::U8 && self.1.is_ok(union) }
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
pub trait IDiscriminantProvider {
    type OkShift: Unsigned;
    type ErrShift: Unsigned;
    type Discriminant: IDiscriminant;
    type NicheExporter: IStable + Default + Copy;
}
macro_rules! same_as {
    ($T: ty) => {
        type OkShift = <$T as IDiscriminantProvider>::OkShift;
        type ErrShift = <$T as IDiscriminantProvider>::ErrShift;
        type Discriminant = <$T as IDiscriminantProvider>::Discriminant;
        type NicheExporter = <$T as IDiscriminantProvider>::NicheExporter;
    };
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

/// Prevents the compiler from doing infinite recursion when evaluating `IDiscriminantProvider`
type DefaultRecursionBudget = T<T<T<T<T<T<T<T<H>>>>>>>>;

/// Enter the type-fu recursion
impl<Ok: IStable, Err: IStable> IDiscriminantProvider for (Ok, Err)
where
    (Ok, Err, U0, U0, DefaultRecursionBudget): IDiscriminantProvider,
{
    same_as!((Ok, Err, U0, U0, DefaultRecursionBudget));
}

/// Branch on whether some forbidden values for Err fit inside Ok's unused bits
impl<Ok: IStable, Err: IStable, OkS: Unsigned, ErrS: Unsigned, Budget> IDiscriminantProvider
    for (Ok, Err, OkS, ErrS, T<Budget>)
where
    (
        Ok,
        Err,
        OkS,
        ErrS,
        T<Budget>,
        <<ShiftedForbiddenValues<Err, ErrS> as IForbiddenValues>::SelectFrom<
            UnionMemberUnusedBits<Ok, Err, OkS>,
        > as ISingleForbiddenValue>::Resolve,
    ): IDiscriminantProvider,
{
    same_as!((
        Ok,
        Err,
        OkS,
        ErrS,
        T<Budget>,
        <<ShiftedForbiddenValues<Err, ErrS> as IForbiddenValues>::SelectFrom<
            UnionMemberUnusedBits<Ok, Err, OkS>,
        > as ISingleForbiddenValue>::Resolve,
    ));
}
impl<Ok: IStable, Err: IStable, OkS: Unsigned, ErrS: Unsigned> IDiscriminantProvider
    for (Ok, Err, OkS, ErrS, H)
{
    type Discriminant = BitDiscriminant;
    type ErrShift = U0;
    type OkShift = U0;
    type NicheExporter = ();
}

/// If some forbidden values for Err fit inside Ok's unused bits, exit the recursion
impl<
        Ok: IStable,
        Err: IStable,
        OkS: Unsigned,
        ErrS: Unsigned,
        Offset: Unsigned,
        V: Unsigned,
        Tail: IForbiddenValues + IntoValueIsErr,
        Budget,
    > IDiscriminantProvider for (Ok, Err, OkS, ErrS, T<Budget>, Array<Offset, V, Tail>)
{
    type OkShift = OkS;
    type ErrShift = ErrS;
    type Discriminant = Not<<Array<Offset, V, Tail> as IntoValueIsErr>::ValueIsErr>;
    type NicheExporter = NicheExporter<
        End,
        <UnionMemberUnusedBits<Ok, Err, OkS> as IBitMask>::BitAnd<
            UnionMemberUnusedBits<Err, Ok, ErrS>,
        >,
        Saturator,
    >;
}

/// None of Err's forbidden values fit into Ok's unused bits, so branch on wherther
/// some of Ok's forbidden values fit into Err's forbidden value
impl<Ok: IStable, Err: IStable, OkS: Unsigned, ErrS: Unsigned, Budget> IDiscriminantProvider
    for (Ok, Err, OkS, ErrS, T<Budget>, End)
where
    (
        Ok,
        Err,
        OkS,
        ErrS,
        T<Budget>,
        End,
        <<ShiftedForbiddenValues<Ok, OkS> as IForbiddenValues>::SelectFrom<
            UnionMemberUnusedBits<Err, Ok, ErrS>,
        > as ISingleForbiddenValue>::Resolve,
    ): IDiscriminantProvider,
{
    same_as!((
        Ok,
        Err,
        OkS,
        ErrS,
        T<Budget>,
        End,
        <<ShiftedForbiddenValues<Ok, OkS> as IForbiddenValues>::SelectFrom<
            UnionMemberUnusedBits<Err, Ok, ErrS>,
        > as ISingleForbiddenValue>::Resolve,
    ));
}

/// If some forbidden values for Ok fit inside Err's unused bits, exit the recursion
impl<
        Ok: IStable,
        Err: IStable,
        OkS: Unsigned,
        ErrS: Unsigned,
        Offset: Unsigned,
        V: Unsigned,
        Tail: IForbiddenValues + IntoValueIsErr,
        Budget,
    > IDiscriminantProvider for (Ok, Err, OkS, ErrS, T<Budget>, End, Array<Offset, V, Tail>)
{
    type OkShift = OkS;
    type ErrShift = ErrS;
    type Discriminant = <Array<Offset, V, Tail> as IntoValueIsErr>::ValueIsErr;
    type NicheExporter = NicheExporter<
        End,
        <UnionMemberUnusedBits<Ok, Err, OkS> as IBitMask>::BitAnd<
            UnionMemberUnusedBits<Err, Ok, ErrS>,
        >,
        Saturator,
    >;
}

/// If neither Err nor Ok's unused bits can fit any of the other's forbidden value,
/// check if their unused bits have an intersection
impl<Ok: IStable, Err: IStable, OkS: Unsigned, ErrS: Unsigned, Budget> IDiscriminantProvider
    for (Ok, Err, OkS, ErrS, T<Budget>, End, End)
where
    (
        Ok,
        Err,
        OkS,
        ErrS,
        T<Budget>,
        End,
        End,
        <UnionMemberUnusedBits<Ok, Err, OkS> as IBitMask>::BitAnd<
            UnionMemberUnusedBits<Err, Ok, ErrS>,
        >,
    ): IDiscriminantProvider,
{
    same_as!((
        Ok,
        Err,
        OkS,
        ErrS,
        T<Budget>,
        End,
        End,
        <UnionMemberUnusedBits<Ok, Err, OkS> as IBitMask>::BitAnd<
            UnionMemberUnusedBits<Err, Ok, ErrS>,
        >,
    ));
}
/// If Ok and Err's unused bits have an intersection, use it.
impl<
        Ok: IStable,
        Err: IStable,
        OkS: Unsigned,
        ErrS: Unsigned,
        Offset: Unsigned,
        V: NonZero,
        Rest: IBitMask,
        Budget,
    > IDiscriminantProvider
    for (
        Ok,
        Err,
        OkS,
        ErrS,
        T<Budget>,
        End,
        End,
        Array<Offset, V, Rest>,
    )
{
    type OkShift = OkS;
    type ErrShift = ErrS;
    type Discriminant = BitIsErr<
        <Array<Offset, V, Rest> as IBitMask>::ExtractedBitByteOffset,
        <Array<Offset, V, Rest> as IBitMask>::ExtractedBitMask,
    >;
    type NicheExporter =
        NicheExporter<End, <Array<Offset, V, Rest> as IBitMask>::ExtractBit, Saturator>;
}
/// If no niche was found, compare Ok and Err's sizes to push the smallest to the right
impl<Ok: IStable, Err: IStable, OkS: Unsigned, ErrS: Unsigned, Budget> IDiscriminantProvider
    for (Ok, Err, OkS, ErrS, T<Budget>, End, End, End)
where
    (
        Ok,
        Err,
        OkS,
        ErrS,
        T<Budget>,
        End,
        End,
        End,
        <Ok::Size as Unsigned>::Cmp<Err::Size>,
    ): IDiscriminantProvider,
{
    same_as!((
        Ok,
        Err,
        OkS,
        ErrS,
        T<Budget>,
        End,
        End,
        End,
        <Ok::Size as Unsigned>::Cmp<Err::Size>,
    ));
}
/// If Ok and Err are the same size, give up on niche optimization and just place a bit-discriminant
impl<Ok: IStable, Err: IStable, OkS: Unsigned, ErrS: Unsigned, Budget> IDiscriminantProvider
    for (Ok, Err, OkS, ErrS, T<Budget>, End, End, End, Equal)
{
    type Discriminant = BitDiscriminant;
    type ErrShift = U0;
    type OkShift = U0;
    type NicheExporter = ();
}
/// If Ok is smaller, check if shifting it by its aligment would fit in the current union size
impl<Ok: IStable, Err: IStable, OkS: Unsigned, Budget> IDiscriminantProvider
    for (Ok, Err, OkS, U0, T<Budget>, End, End, End, Lesser)
where
    (
        Ok,
        Err,
        OkS,
        U0,
        T<Budget>,
        End,
        End,
        End,
        Lesser,
        <tyeval!((Ok::Size + Ok::Align) + OkS) as Unsigned>::SmallerOrEq<
            UnionSize<Ok, Err, U0, U0>,
        >,
    ): IDiscriminantProvider,
{
    same_as!((
        Ok,
        Err,
        OkS,
        U0,
        T<Budget>,
        End,
        End,
        End,
        Lesser,
        <tyeval!((Ok::Size + Ok::Align) + OkS) as Unsigned>::SmallerOrEq<
            UnionSize<Ok, Err, U0, U0>,
        >,
    ));
}
impl<Ok: IStable, Err: IStable, OkS: Unsigned, Budget> IDiscriminantProvider
    for (Ok, Err, OkS, U0, T<Budget>, End, End, End, Lesser, B0)
{
    type Discriminant = BitDiscriminant;
    type ErrShift = U0;
    type OkShift = U0;
    type NicheExporter = ();
}
impl<Ok: IStable, Err: IStable, OkS: Unsigned, Budget> IDiscriminantProvider
    for (Ok, Err, OkS, U0, T<Budget>, End, End, End, Lesser, B1)
where
    (Ok, Err, tyeval!(OkS + Ok::Align), U0, Budget): IDiscriminantProvider,
{
    same_as!((Ok, Err, tyeval!(OkS + Ok::Align), U0, Budget));
}

/// If Err is bigger, check shift
impl<Ok: IStable, Err: IStable, ErrS: Unsigned, Budget> IDiscriminantProvider
    for (Ok, Err, U0, ErrS, T<Budget>, End, End, End, Greater)
where
    (
        Ok,
        Err,
        U0,
        ErrS,
        T<Budget>,
        End,
        End,
        End,
        Greater,
        <tyeval!((Err::Size + Err::Align) + ErrS) as Unsigned>::SmallerOrEq<
            UnionSize<Ok, Err, U0, U0>,
        >,
    ): IDiscriminantProvider,
{
    same_as!((
        Ok,
        Err,
        U0,
        ErrS,
        T<Budget>,
        End,
        End,
        End,
        Greater,
        <tyeval!((Err::Size + Err::Align) + ErrS) as Unsigned>::SmallerOrEq<
            UnionSize<Ok, Err, U0, U0>,
        >,
    ));
}
impl<Ok: IStable, Err: IStable, ErrS: Unsigned, Budget> IDiscriminantProvider
    for (Ok, Err, U0, ErrS, T<Budget>, End, End, End, Greater, B0)
{
    type Discriminant = BitDiscriminant;
    type ErrShift = U0;
    type OkShift = U0;
    type NicheExporter = ();
}

impl<Ok: IStable, Err: IStable, ErrS: Unsigned, Budget> IDiscriminantProvider
    for (Ok, Err, U0, ErrS, T<Budget>, End, End, End, Greater, B1)
where
    (Ok, Err, U0, tyeval!(ErrS + Err::Align), Budget): IDiscriminantProvider,
{
    same_as!((Ok, Err, U0, tyeval!(ErrS + Err::Align), Budget));
}
