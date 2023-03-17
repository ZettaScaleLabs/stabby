use core::marker::PhantomData;

use stabby_macros::tyeval;

use super::{
    istable::{IArrayPush, Includes, B2},
    padding::Padded,
    IStable,
};
use crate::abi::{padding::IPadding, *};

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
#[crate::stabby]
#[derive(Debug, Clone, Copy)]
pub struct ValueIsErr<Offset, Value, Tail>(PhantomData<(Offset, Value)>, Tail);

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
#[crate::stabby]
#[derive(Debug, Clone, Copy)]
pub struct Not<Discriminant>(Discriminant);
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
unsafe impl<Left: IPadding, This, Other> IStable for UnionMember<Left, This, Other>
where
    Padded<Left, This>: IStable,
    Union<Padded<Left, This>, Other>: IStable,
    <Union<Padded<Left, This>, Other> as IStable>::Size: Sub<<Padded<Left, This> as IStable>::Size>,
    tyeval!(
        <Union<Padded<Left, This>, Other> as IStable>::Size - <Padded<Left, This> as IStable>::Size
    ): IPadding,
    <tyeval!(
        <Union<Padded<Left, This>, Other> as IStable>::Size - <Padded<Left, This> as IStable>::Size
    ) as IPadding>::Padding: IStable,
    <<tyeval!(
        <Union<Padded<Left, This>, Other> as IStable>::Size - <Padded<Left, This> as IStable>::Size
    ) as IPadding>::Padding as IStable>::UnusedBits:
        IArrayPush<<Padded<Left, This> as IStable>::IllegalValues>,
{
    type Size = <Union<Padded<Left, This>, Other> as IStable>::Size;
    type Align = <Union<Padded<Left, This>, Other> as IStable>::Align;
    type IllegalValues = <Padded<Left, This> as IStable>::Size;
    type UnusedBits = <<<tyeval!(
        <Union<Padded<Left, This>, Other> as IStable>::Size - <Padded<Left, This> as IStable>::Size
    ) as IPadding>::Padding as IStable>::UnusedBits as IArrayPush<
        <Padded<Left, This> as IStable>::IllegalValues,
    >>::Output;
    type HasExactlyOneNiche = B2;
}

pub trait IDiscriminantProvider {
    type OkShift: IPadding;
    type ErrShift: IPadding;
    type Discriminant: IDiscriminant;
}

pub struct Eval;
impl<Ok: IStable, Err: IStable> IDiscriminantProvider for (Ok, Err)
where
    (UnionMember<U0, Ok, Err>, UnionMember<U0, Err, Ok>, Eval): IDiscriminantProvider,
{
    type OkShift = <(UnionMember<U0, Ok, Err>, UnionMember<U0, Err, Ok>, Eval) as IDiscriminantProvider>::OkShift;
    type ErrShift = <(UnionMember<U0, Ok, Err>, UnionMember<U0, Err, Ok>, Eval) as IDiscriminantProvider>::ErrShift;
    type Discriminant = <(UnionMember<U0, Ok, Err>, UnionMember<U0, Err, Ok>, Eval) as IDiscriminantProvider>::Discriminant;
}

impl<Ok: IStable, Err: IStable, OkS, ErrS> IDiscriminantProvider
    for (UnionMember<OkS, Ok, Err>, UnionMember<ErrS, Err, Ok>, Eval)
where
    UnionMember<OkS, Ok, Err>: IStable,
    UnionMember<ErrS, Err, Ok>: IStable,
    <UnionMember<OkS, Ok, Err> as IStable>::UnusedBits:
        Includes<<UnionMember<ErrS, Err, Ok> as IStable>::IllegalValues>,
    (
        UnionMember<OkS, Ok, Err>,
        UnionMember<ErrS, Err, Ok>,
        <<UnionMember<OkS, Ok, Err> as IStable>::UnusedBits as Includes<
            <UnionMember<ErrS, Err, Ok> as IStable>::IllegalValues,
        >>::Output,
    ): IDiscriminantProvider,
{
    type OkShift = <(
        UnionMember<OkS, Ok, Err>,
        UnionMember<ErrS, Err, Ok>,
        <<UnionMember<OkS, Ok, Err> as IStable>::UnusedBits as Includes<
            <UnionMember<ErrS, Err, Ok> as IStable>::IllegalValues,
        >>::Output,
    ) as IDiscriminantProvider>::OkShift;
    type ErrShift = <(
        UnionMember<OkS, Ok, Err>,
        UnionMember<ErrS, Err, Ok>,
        <<UnionMember<OkS, Ok, Err> as IStable>::UnusedBits as Includes<
            <UnionMember<ErrS, Err, Ok> as IStable>::IllegalValues,
        >>::Output,
    ) as IDiscriminantProvider>::ErrShift;
    type Discriminant = <(
        UnionMember<OkS, Ok, Err>,
        UnionMember<ErrS, Err, Ok>,
        <<UnionMember<OkS, Ok, Err> as IStable>::UnusedBits as Includes<
            <UnionMember<ErrS, Err, Ok> as IStable>::IllegalValues,
        >>::Output,
    ) as IDiscriminantProvider>::Discriminant;
}

impl<Ok: IStable, Err: IStable, OkS, ErrS> IDiscriminantProvider
    for (UnionMember<OkS, Ok, Err>, UnionMember<ErrS, Err, Ok>, End)
where
    UnionMember<OkS, Ok, Err>: IStable,
    UnionMember<ErrS, Err, Ok>: IStable,
    <UnionMember<ErrS, Err, Ok> as IStable>::UnusedBits:
        Includes<<UnionMember<OkS, Ok, Err> as IStable>::IllegalValues>,
    (
        UnionMember<OkS, Ok, Err>,
        UnionMember<ErrS, Err, Ok>,
        End,
        <<UnionMember<ErrS, Err, Ok> as IStable>::UnusedBits as Includes<
            <UnionMember<OkS, Ok, Err> as IStable>::IllegalValues,
        >>::Output,
    ): IDiscriminantProvider,
{
    type OkShift = <(
        UnionMember<OkS, Ok, Err>,
        UnionMember<ErrS, Err, Ok>,
        End,
        <<UnionMember<ErrS, Err, Ok> as IStable>::UnusedBits as Includes<
            <UnionMember<OkS, Ok, Err> as IStable>::IllegalValues,
        >>::Output,
    ) as IDiscriminantProvider>::OkShift;
    type ErrShift = <(
        UnionMember<OkS, Ok, Err>,
        UnionMember<ErrS, Err, Ok>,
        End,
        <<UnionMember<ErrS, Err, Ok> as IStable>::UnusedBits as Includes<
            <UnionMember<OkS, Ok, Err> as IStable>::IllegalValues,
        >>::Output,
    ) as IDiscriminantProvider>::ErrShift;
    type Discriminant = <(
        UnionMember<OkS, Ok, Err>,
        UnionMember<ErrS, Err, Ok>,
        End,
        <<UnionMember<ErrS, Err, Ok> as IStable>::UnusedBits as Includes<
            <UnionMember<OkS, Ok, Err> as IStable>::IllegalValues,
        >>::Output,
    ) as IDiscriminantProvider>::Discriminant;
}

impl<Ok: IStable, Err: IStable, OkS, ErrS> IDiscriminantProvider
    for (
        UnionMember<OkS, Ok, Err>,
        UnionMember<ErrS, Err, Ok>,
        End,
        End,
    )
where
    UnionMember<OkS, Ok, Err>: IStable,
    UnionMember<ErrS, Err, Ok>: IStable,
    <UnionMember<ErrS, Err, Ok> as IStable>::UnusedBits:
        BitAnd<<UnionMember<OkS, Ok, Err> as IStable>::UnusedBits>,
    (
        UnionMember<OkS, Ok, Err>,
        UnionMember<ErrS, Err, Ok>,
        End,
        End,
        <<UnionMember<ErrS, Err, Ok> as IStable>::UnusedBits as BitAnd<
            <UnionMember<OkS, Ok, Err> as IStable>::UnusedBits,
        >>::Output,
    ): IDiscriminantProvider,
{
    type OkShift = <(
        UnionMember<OkS, Ok, Err>,
        UnionMember<ErrS, Err, Ok>,
        End,
        End,
        <<UnionMember<ErrS, Err, Ok> as IStable>::UnusedBits as BitAnd<
            <UnionMember<OkS, Ok, Err> as IStable>::UnusedBits,
        >>::Output,
    ) as IDiscriminantProvider>::OkShift;
    type ErrShift = <(
        UnionMember<OkS, Ok, Err>,
        UnionMember<ErrS, Err, Ok>,
        End,
        End,
        <<UnionMember<ErrS, Err, Ok> as IStable>::UnusedBits as BitAnd<
            <UnionMember<OkS, Ok, Err> as IStable>::UnusedBits,
        >>::Output,
    ) as IDiscriminantProvider>::ErrShift;
    type Discriminant = <(
        UnionMember<OkS, Ok, Err>,
        UnionMember<ErrS, Err, Ok>,
        End,
        End,
        <<UnionMember<ErrS, Err, Ok> as IStable>::UnusedBits as BitAnd<
            <UnionMember<OkS, Ok, Err> as IStable>::UnusedBits,
        >>::Output,
    ) as IDiscriminantProvider>::Discriminant;
}

impl<Ok: IStable, Err: IStable, OkS, ErrS> IDiscriminantProvider
    for (
        UnionMember<OkS, Ok, Err>,
        UnionMember<ErrS, Err, Ok>,
        End,
        End,
        End,
    )
where
    Ok::Size: Cmp<Err::Size>,
    (
        UnionMember<OkS, Ok, Err>,
        UnionMember<ErrS, Err, Ok>,
        End,
        End,
        End,
        <Ok::Size as Cmp<Err::Size>>::Output,
    ): IDiscriminantProvider,
{
    type OkShift = <(
        UnionMember<OkS, Ok, Err>,
        UnionMember<ErrS, Err, Ok>,
        End,
        End,
        End,
        <Ok::Size as Cmp<Err::Size>>::Output,
    ) as IDiscriminantProvider>::OkShift;
    type ErrShift = <(
        UnionMember<OkS, Ok, Err>,
        UnionMember<ErrS, Err, Ok>,
        End,
        End,
        End,
        <Ok::Size as Cmp<Err::Size>>::Output,
    ) as IDiscriminantProvider>::ErrShift;
    type Discriminant = <(
        UnionMember<OkS, Ok, Err>,
        UnionMember<ErrS, Err, Ok>,
        End,
        End,
        End,
        <Ok::Size as Cmp<Err::Size>>::Output,
    ) as IDiscriminantProvider>::Discriminant;
}
impl<Ok: IStable, Err: IStable, OkS, ErrS> IDiscriminantProvider
    for (
        UnionMember<OkS, Ok, Err>,
        UnionMember<ErrS, Err, Ok>,
        End,
        End,
        End,
        Equal,
    )
{
    type OkShift = U0;
    type ErrShift = U0;
    type Discriminant = BitDiscriminant;
}
impl<Ok: IStable, Err: IStable, OkS, ErrS, T> IDiscriminantProvider
    for (
        UnionMember<OkS, Ok, Err>,
        UnionMember<ErrS, Err, Ok>,
        End,
        End,
        End,
        T,
        B0,
    )
{
    type OkShift = U0;
    type ErrShift = U0;
    type Discriminant = BitDiscriminant;
}
impl<Ok: IStable, Err: IStable, OkS, ErrS, T> IDiscriminantProvider
    for (
        UnionMember<OkS, Ok, Err>,
        UnionMember<ErrS, Err, Ok>,
        End,
        End,
        End,
        T,
        B1,
    )
where
    UnionMember<OkS, Ok, Err>: IStable,
    UnionMember<ErrS, Err, Ok>: IStable,
    <UnionMember<OkS, Ok, Err> as IStable>::UnusedBits:
        Includes<<UnionMember<ErrS, Err, Ok> as IStable>::IllegalValues>,
    (
        UnionMember<OkS, Ok, Err>,
        UnionMember<ErrS, Err, Ok>,
        <<UnionMember<OkS, Ok, Err> as IStable>::UnusedBits as Includes<
            <UnionMember<ErrS, Err, Ok> as IStable>::IllegalValues,
        >>::Output,
    ): IDiscriminantProvider,
{
    type OkShift = <(
        UnionMember<OkS, Ok, Err>,
        UnionMember<ErrS, Err, Ok>,
        <<UnionMember<OkS, Ok, Err> as IStable>::UnusedBits as Includes<
            <UnionMember<ErrS, Err, Ok> as IStable>::IllegalValues,
        >>::Output,
    ) as IDiscriminantProvider>::OkShift;
    type ErrShift = <(
        UnionMember<OkS, Ok, Err>,
        UnionMember<ErrS, Err, Ok>,
        <<UnionMember<OkS, Ok, Err> as IStable>::UnusedBits as Includes<
            <UnionMember<ErrS, Err, Ok> as IStable>::IllegalValues,
        >>::Output,
    ) as IDiscriminantProvider>::ErrShift;
    type Discriminant = <(
        UnionMember<OkS, Ok, Err>,
        UnionMember<ErrS, Err, Ok>,
        <<UnionMember<OkS, Ok, Err> as IStable>::UnusedBits as Includes<
            <UnionMember<ErrS, Err, Ok> as IStable>::IllegalValues,
        >>::Output,
    ) as IDiscriminantProvider>::Discriminant;
}
impl<Ok: IStable, Err: IStable, OkS, ErrS> IDiscriminantProvider
    for (
        UnionMember<OkS, Ok, Err>,
        UnionMember<ErrS, Err, Ok>,
        End,
        End,
        End,
        Greater,
    )
where
    ErrS: Add<Err::Align> + Add<tyeval!(Err::Align + Err::Size)>,
    Err::Align: Add<Err::Size>,
    UnionMember<ErrS, Err, Ok>: IStable,
    <UnionMember<ErrS, Err, Ok> as IStable>::Size:
        IsGreater<tyeval!(ErrS + (Err::Align + Err::Size))>,
    (
        UnionMember<OkS, Ok, Err>,
        UnionMember<tyeval!(ErrS + Err::Align), Err, Ok>,
        End,
        End,
        End,
        Greater,
        <<UnionMember<ErrS, Err, Ok> as IStable>::Size as IsGreater<
            tyeval!(ErrS + (Err::Align + Err::Size)),
        >>::Output,
    ): IDiscriminantProvider,
{
    type OkShift = <(
        UnionMember<OkS, Ok, Err>,
        UnionMember<tyeval!(ErrS + Err::Align), Err, Ok>,
        End,
        End,
        End,
        Greater,
        <<UnionMember<ErrS, Err, Ok> as IStable>::Size as IsGreater<
            tyeval!(ErrS + (Err::Align + Err::Size)),
        >>::Output,
    ) as IDiscriminantProvider>::OkShift;
    type ErrShift = <(
        UnionMember<OkS, Ok, Err>,
        UnionMember<tyeval!(ErrS + Err::Align), Err, Ok>,
        End,
        End,
        End,
        Greater,
        <<UnionMember<ErrS, Err, Ok> as IStable>::Size as IsGreater<
            tyeval!(ErrS + (Err::Align + Err::Size)),
        >>::Output,
    ) as IDiscriminantProvider>::ErrShift;
    type Discriminant = <(
        UnionMember<OkS, Ok, Err>,
        UnionMember<tyeval!(ErrS + Err::Align), Err, Ok>,
        End,
        End,
        End,
        Greater,
        <<UnionMember<ErrS, Err, Ok> as IStable>::Size as IsGreater<
            tyeval!(ErrS + (Err::Align + Err::Size)),
        >>::Output,
    ) as IDiscriminantProvider>::Discriminant;
}
