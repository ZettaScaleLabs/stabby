use core::marker::PhantomData;
use stabby_macros::tyeval;

use super::{
    istable::{IBitMask, IForbiddenValues, ISingleForbiddenValue, Saturator},
    padding::Padded,
    unsigned::Equal,
    IStable,
};
use crate::abi::*;

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
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ValueIsErr<Offset, Value, Tail: IStable>(PhantomData<(Offset, Value)>, Tail);
unsafe impl<Offset, Value, Tail: IStable> IStable for ValueIsErr<Offset, Value, Tail> {
    type Size = Tail::Size;
    type Align = Tail::Align;
    type ForbiddenValues = Tail::ForbiddenValues;
    type UnusedBits = Tail::UnusedBits;
    type HasExactlyOneNiche = Tail::HasExactlyOneNiche;
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

pub struct UnionMember<Left, This, Other>(core::marker::PhantomData<(Left, This, Other)>);
unsafe impl<Left: Unsigned, This, Other> IStable for UnionMember<Left, This, Other>
where
    Padded<Left, This>: IStable,
    Union<Padded<Left, This>, Other>: IStable,
{
    type Size = <Union<Padded<Left, This>, Other> as IStable>::Size;
    type Align = <Union<Padded<Left, This>, Other> as IStable>::Align;
    type ForbiddenValues = <Padded<Left, This> as IStable>::ForbiddenValues;
    type UnusedBits = <<<tyeval!(
        <Union<Padded<Left, This>, Other> as IStable>::Size - <Padded<Left, This> as IStable>::Size
    ) as Unsigned>::Padding as IStable>::UnusedBits as IBitMask>::BitOr<
        <Padded<Left, This> as IStable>::UnusedBits,
    >;
    type HasExactlyOneNiche = Saturator;
}

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

type UnionAlign<Ok, Err> = <<Ok as IStable>::Align as PowerOf2>::Max<<Err as IStable>::Align>;
type UnionSize<Ok, Err, OkShift, ErrShift> =
    <<tyeval!(<Ok as IStable>::Size + OkShift) as Unsigned>::Max<
        tyeval!(<Err as IStable>::Size + ErrShift),
    > as Unsigned>::NextMultipleOf<UnionAlign<Ok, Err>>;
type PaddedSize<T, Shift> = <<T as IStable>::Size as Unsigned>::Add<Shift>;
type ShiftedForbiddenValues<T, Shift> =
    <<T as IStable>::ForbiddenValues as IForbiddenValues>::Shift<Shift>;
type ShiftedUnusedBits<T, Shift> = <<T as IStable>::UnusedBits as IBitMask>::Shift<Shift>;

type UnionMemberUnusedBits<Ok, Err, OkShift> =
    <<<<OkShift as Unsigned>::Padding as IStable>::UnusedBits as IBitMask>::BitOr<
        ShiftedUnusedBits<Ok, OkShift>,
    > as IBitMask>::BitOr<
        ShiftedUnusedBits<
            <tyeval!(UnionSize<Ok, Err, OkShift, U0> - PaddedSize<Ok, OkShift>) as Unsigned>::Padding,
            PaddedSize<Ok, OkShift>,
        >,
    >;

impl<Ok: IStable, Err: IStable> IDiscriminantProvider for (Ok, Err)
where
    (Ok, Err, U0, U0): IDiscriminantProvider,
{
    same_as!((Ok, Err, U0, U0));
}

impl<Ok: IStable, Err: IStable, OkS: Unsigned, ErrS: Unsigned> IDiscriminantProvider
    for (Ok, Err, OkS, ErrS)
where
    (
        Ok,
        Err,
        OkS,
        ErrS,
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
        <<ShiftedForbiddenValues<Err, ErrS> as IForbiddenValues>::SelectFrom<
            UnionMemberUnusedBits<Ok, Err, OkS>,
        > as ISingleForbiddenValue>::Resolve,
    ));
}

impl<
        Ok: IStable,
        Err: IStable,
        OkS: Unsigned,
        ErrS: Unsigned,
        Offset: Unsigned,
        T: Unsigned,
        Tail: IForbiddenValues + IntoValueIsErr,
    > IDiscriminantProvider for (Ok, Err, OkS, ErrS, Array<Offset, T, Tail>)
{
    type OkShift = OkS;
    type ErrShift = ErrS;
    type Discriminant = Not<<Array<Offset, T, Tail> as IntoValueIsErr>::ValueIsErr>;
    type NicheExporter = ();
}

impl<Ok: IStable, Err: IStable, OkS: Unsigned, ErrS: Unsigned> IDiscriminantProvider
    for (Ok, Err, OkS, ErrS, End)
where
    (
        Ok,
        Err,
        OkS,
        ErrS,
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
        End,
        <<ShiftedForbiddenValues<Ok, OkS> as IForbiddenValues>::SelectFrom<
            UnionMemberUnusedBits<Err, Ok, ErrS>,
        > as ISingleForbiddenValue>::Resolve,
    ));
}

impl<Ok: IStable, Err: IStable, OkS: Unsigned, ErrS: Unsigned> IDiscriminantProvider
    for (Ok, Err, OkS, ErrS, End, End)
where
    (
        Ok,
        Err,
        OkS,
        ErrS,
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
        End,
        End,
        <UnionMemberUnusedBits<Ok, Err, OkS> as IBitMask>::BitAnd<
            UnionMemberUnusedBits<Err, Ok, ErrS>,
        >,
    ));
}
impl<Ok: IStable, Err: IStable, OkS: Unsigned, ErrS: Unsigned> IDiscriminantProvider
    for (Ok, Err, OkS, ErrS, End, End, End)
where
    (
        Ok,
        Err,
        OkS,
        ErrS,
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
        End,
        End,
        End,
        <Ok::Size as Unsigned>::Cmp<Err::Size>,
    ));
}
impl<Ok: IStable, Err: IStable, OkS: Unsigned, ErrS: Unsigned> IDiscriminantProvider
    for (Ok, Err, OkS, ErrS, End, End, End, Equal)
{
    type Discriminant = BitDiscriminant;
    type ErrShift = U0;
    type OkShift = U0;
    type NicheExporter = ();
}

// impl<Ok: IStable, Err: IStable, OkS, ErrS> IDiscriminantProvider
//     for (UnionMember<OkS, Ok, Err>, UnionMember<ErrS, Err, Ok>, End)
// where
//     UnionMember<OkS, Ok, Err>: IStable,
//     UnionMember<ErrS, Err, Ok>: IStable,
//     <UnionMember<ErrS, Err, Ok> as IStable>::UnusedBits:
//         Includes<<UnionMember<OkS, Ok, Err> as IStable>::ForbiddenValues>,
//     (
//         UnionMember<OkS, Ok, Err>,
//         UnionMember<ErrS, Err, Ok>,
//         End,
//         <<UnionMember<ErrS, Err, Ok> as IStable>::UnusedBits as Includes<
//             <UnionMember<OkS, Ok, Err> as IStable>::ForbiddenValues,
//         >>::Output,
//     ): IDiscriminantProvider,
// {
//     type OkShift = <(
//         UnionMember<OkS, Ok, Err>,
//         UnionMember<ErrS, Err, Ok>,
//         End,
//         <<UnionMember<ErrS, Err, Ok> as IStable>::UnusedBits as Includes<
//             <UnionMember<OkS, Ok, Err> as IStable>::ForbiddenValues,
//         >>::Output,
//     ) as IDiscriminantProvider>::OkShift;
//     type ErrShift = <(
//         UnionMember<OkS, Ok, Err>,
//         UnionMember<ErrS, Err, Ok>,
//         End,
//         <<UnionMember<ErrS, Err, Ok> as IStable>::UnusedBits as Includes<
//             <UnionMember<OkS, Ok, Err> as IStable>::ForbiddenValues,
//         >>::Output,
//     ) as IDiscriminantProvider>::ErrShift;
//     type Discriminant = <(
//         UnionMember<OkS, Ok, Err>,
//         UnionMember<ErrS, Err, Ok>,
//         End,
//         <<UnionMember<ErrS, Err, Ok> as IStable>::UnusedBits as Includes<
//             <UnionMember<OkS, Ok, Err> as IStable>::ForbiddenValues,
//         >>::Output,
//     ) as IDiscriminantProvider>::Discriminant;
// }

// impl<Ok: IStable, Err: IStable, OkS, ErrS> IDiscriminantProvider
//     for (
//         UnionMember<OkS, Ok, Err>,
//         UnionMember<ErrS, Err, Ok>,
//         End,
//         End,
//     )
// where
//     UnionMember<OkS, Ok, Err>: IStable,
//     UnionMember<ErrS, Err, Ok>: IStable,
//     <UnionMember<ErrS, Err, Ok> as IStable>::UnusedBits:
//         BitAnd<<UnionMember<OkS, Ok, Err> as IStable>::UnusedBits>,
//     (
//         UnionMember<OkS, Ok, Err>,
//         UnionMember<ErrS, Err, Ok>,
//         End,
//         End,
//         <<UnionMember<ErrS, Err, Ok> as IStable>::UnusedBits as BitAnd<
//             <UnionMember<OkS, Ok, Err> as IStable>::UnusedBits,
//         >>::Output,
//     ): IDiscriminantProvider,
// {
//     type OkShift = <(
//         UnionMember<OkS, Ok, Err>,
//         UnionMember<ErrS, Err, Ok>,
//         End,
//         End,
//         <<UnionMember<ErrS, Err, Ok> as IStable>::UnusedBits as BitAnd<
//             <UnionMember<OkS, Ok, Err> as IStable>::UnusedBits,
//         >>::Output,
//     ) as IDiscriminantProvider>::OkShift;
//     type ErrShift = <(
//         UnionMember<OkS, Ok, Err>,
//         UnionMember<ErrS, Err, Ok>,
//         End,
//         End,
//         <<UnionMember<ErrS, Err, Ok> as IStable>::UnusedBits as BitAnd<
//             <UnionMember<OkS, Ok, Err> as IStable>::UnusedBits,
//         >>::Output,
//     ) as IDiscriminantProvider>::ErrShift;
//     type Discriminant = <(
//         UnionMember<OkS, Ok, Err>,
//         UnionMember<ErrS, Err, Ok>,
//         End,
//         End,
//         <<UnionMember<ErrS, Err, Ok> as IStable>::UnusedBits as BitAnd<
//             <UnionMember<OkS, Ok, Err> as IStable>::UnusedBits,
//         >>::Output,
//     ) as IDiscriminantProvider>::Discriminant;
// }

// impl<Ok: IStable, Err: IStable, OkS, ErrS> IDiscriminantProvider
//     for (
//         UnionMember<OkS, Ok, Err>,
//         UnionMember<ErrS, Err, Ok>,
//         End,
//         End,
//         End,
//     )
// where
//     (
//         UnionMember<OkS, Ok, Err>,
//         UnionMember<ErrS, Err, Ok>,
//         End,
//         End,
//         End,
//         <Ok::Size as Unsigned>::Equal<Err::Size>,
//     ): IDiscriminantProvider,
// {
//     type OkShift = <(
//         UnionMember<OkS, Ok, Err>,
//         UnionMember<ErrS, Err, Ok>,
//         End,
//         End,
//         End,
//         <Ok::Size as Unsigned>::Equal<Err::Size>,
//     ) as IDiscriminantProvider>::OkShift;
//     type ErrShift = <(
//         UnionMember<OkS, Ok, Err>,
//         UnionMember<ErrS, Err, Ok>,
//         End,
//         End,
//         End,
//         <Ok::Size as Unsigned>::Equal<Err::Size>,
//     ) as IDiscriminantProvider>::ErrShift;
//     type Discriminant = <(
//         UnionMember<OkS, Ok, Err>,
//         UnionMember<ErrS, Err, Ok>,
//         End,
//         End,
//         End,
//         <Ok::Size as Unsigned>::Equal<Err::Size>,
//     ) as IDiscriminantProvider>::Discriminant;
// }
