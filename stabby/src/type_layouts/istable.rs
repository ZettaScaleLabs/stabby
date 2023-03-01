use super::{Tuple2, Union};
pub use ::typenum::*;
use core::ops::*;
use stabby_macros::tyeval;
/// A trait to describe the layout of a type.
///
/// Every layout is assumed to start at the type's first byte.
///
/// # Safety
/// Mis-implementing this trait can lead to memory corruption in sum tyoes
pub unsafe trait IStable: Sized {
    type Size: Unsigned;
    type Align: Unsigned;
    type IllegalValues;
    type UnusedBits;
    type HasExactlyOneNiche;
    fn size() -> usize {
        let size = Self::Size::USIZE;
        let align = Self::Align::USIZE;
        size + ((align - (size % align)) % align)
    }
    fn align() -> usize {
        Self::Align::USIZE
    }
}

pub struct End;
pub struct Array<Offset, T, Rest>(core::marker::PhantomData<(Offset, T, Rest)>);

unsafe impl<A: IStable, B: IStable> IStable for Tuple2<A, B>
where
    A::Align: Max<B::Align>,
    AlignedAfter<B, A::Size>: IStable,
    A::UnusedBits: IArrayPush<<AlignedAfter<B, A::Size> as IStable>::UnusedBits>,
    <A::Align as Max<B::Align>>::Output: Unsigned,
    A::HasExactlyOneNiche: SaturatingAdd<B::HasExactlyOneNiche>,
{
    type IllegalValues = End; // TODO
    type UnusedBits =
        <A::UnusedBits as IArrayPush<<AlignedAfter<B, A::Size> as IStable>::UnusedBits>>::Output;
    type Size = <AlignedAfter<B, A::Size> as IStable>::Size;
    type Align = <A::Align as Max<B::Align>>::Output;
    type HasExactlyOneNiche =
        <A::HasExactlyOneNiche as SaturatingAdd<B::HasExactlyOneNiche>>::Output;
}
pub trait SaturatingAdder: SaturatingAdd<B0> + SaturatingAdd<B1> + SaturatingAdd<B2> {}
impl<T: SaturatingAdd<B0> + SaturatingAdd<B1> + SaturatingAdd<B2>> SaturatingAdder for T {}
pub trait SaturatingAdd<T> {
    type Output;
}
pub struct B2;
impl<T> SaturatingAdd<T> for B0 {
    type Output = T;
}
impl SaturatingAdd<B0> for B1 {
    type Output = B1;
}
impl SaturatingAdd<B1> for B1 {
    type Output = B2;
}
impl SaturatingAdd<B2> for B1 {
    type Output = B2;
}
impl<T> SaturatingAdd<T> for B2 {
    type Output = B2;
}

unsafe impl<A: IStable + Copy, B: IStable + Copy> IStable for Union<A, B>
where
    A::Align: Max<B::Align>,
    A::Size: Max<B::Size>,
    <A::Size as Max<B::Size>>::Output: Unsigned,
    <A::Align as Max<B::Align>>::Output: Unsigned,
{
    type IllegalValues = End;
    type UnusedBits = End;
    type Size = <A::Size as Max<B::Size>>::Output;
    type Align = <A::Align as Max<B::Align>>::Output;
    type HasExactlyOneNiche = B0;
}

pub trait IArrayPush<T> {
    type Output;
}
impl<Arr> IArrayPush<Arr> for End {
    type Output = Arr;
}
impl<Arr, Offset, T, Rest: IArrayPush<Arr>> IArrayPush<Arr> for Array<Offset, T, Rest> {
    type Output = Array<Offset, T, <Rest as IArrayPush<Arr>>::Output>;
}

pub struct AlignedAfter<T, Start>(core::marker::PhantomData<(T, Start)>);
macro_rules! same_as {
    ($t: ty) => {
        type Align = <$t as IStable>::Align;
        type Size = <$t as IStable>::Size;
        type UnusedBits = <$t as IStable>::UnusedBits;
        type IllegalValues = <$t as IStable>::IllegalValues;
        type HasExactlyOneNiche = <$t as IStable>::HasExactlyOneNiche;
    };
}
// Check if T::Align == 0
unsafe impl<T: IStable, Start> IStable for AlignedAfter<T, Start>
where
    T::Align: IsEqual<U0>,
    (tyeval!(T::Align == U0), Self): IStable,
{
    same_as!((tyeval!(T::Align == U0), Self));
}
// T::Align == 0 => The layout doesn't change
unsafe impl<T: IStable, Start: Unsigned> IStable for (B1, AlignedAfter<T, Start>) {
    type Align = <T as IStable>::Align;
    type Size = Start;
    type UnusedBits = End;
    type IllegalValues = End;
    type HasExactlyOneNiche = B0;
}
// T::Align != 0 => Check if Start == 0
unsafe impl<T: IStable, Start> IStable for (B0, AlignedAfter<T, Start>)
where
    Start: IsEqual<U0>,
    (tyeval!(Start == U0), Self): IStable,
{
    same_as!((tyeval!(Start == U0), Self));
}
// Start == 0 => The layout doesn't change
unsafe impl<T: IStable, Start> IStable for (B1, (B0, AlignedAfter<T, Start>)) {
    same_as!(T);
}
unsafe impl<T: IStable, Start> IStable for (B0, (B0, AlignedAfter<T, Start>))
where
    Start: Rem<T::Align>,
    tyeval!(Start % T::Align): IsEqual<U0>,
    (AlignedAfter<T, Start>, tyeval!((Start % T::Align) == U0)): IStable,
{
    same_as!((AlignedAfter<T, Start>, tyeval!((Start % T::Align) == U0)));
}
unsafe impl<T: IStable, Start> IStable for (AlignedAfter<T, Start>, B1)
where
    Start: Add<T::Size>,
    T::UnusedBits: IShift<Start>,
    T::IllegalValues: IShift<Start>,
    tyeval!(Start + T::Size): Unsigned,
{
    type Align = T::Align;
    type Size = tyeval!(Start + T::Size);
    type UnusedBits = <T::UnusedBits as IShift<Start>>::Output;
    type IllegalValues = <T::IllegalValues as IShift<Start>>::Output;
    type HasExactlyOneNiche = T::HasExactlyOneNiche;
}
unsafe impl<T: IStable, Start> IStable for (AlignedAfter<T, Start>, B0)
where
    Start: Rem<T::Align> + Sub<U1> + Add<tyeval!(T::Align - (Start % T::Align))>,
    T::Align: Sub<tyeval!(Start % T::Align)>,
    tyeval!(Start + (T::Align - (Start % T::Align))): Add<T::Size>,
    T::UnusedBits: IShift<tyeval!(Start + (T::Align - (Start % T::Align)))>,
    T::IllegalValues: IShift<tyeval!(Start + (T::Align - (Start % T::Align)))>,
    tyeval!((Start + (T::Align - (Start % T::Align))) + T::Size): Unsigned,
{
    type Align = T::Align;
    type Size = tyeval!((Start + (T::Align - (Start % T::Align))) + T::Size);
    type UnusedBits = Array<
        tyeval!(Start - U1),
        U255,
        <T::UnusedBits as IShift<tyeval!(Start + (T::Align - (Start % T::Align)))>>::Output,
    >;
    type IllegalValues =
        <T::IllegalValues as IShift<tyeval!(Start + (T::Align - (Start % T::Align)))>>::Output;
    type HasExactlyOneNiche = B0;
}

pub trait IShift<By> {
    type Output;
}
impl<By> IShift<By> for End {
    type Output = End;
}
