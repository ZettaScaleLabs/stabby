use stabby_macros::tyeval;

use super::*;
use core::ops::*;
pub trait NicheSpec {}
pub unsafe trait Stable {
    type Niches: NicheSpec;
    type Start;
    type Size;
    type Align;
}
pub trait StableExt: Stable {
    type End;
}
impl<T: Stable> StableExt for T
where
    T::Start: Add<T::Size>,
{
    type End = tyeval!(T::Start + T::Size);
}
pub struct Or<A, B>(core::marker::PhantomData<(A, B)>);
impl<A: Shift<By>, B: Shift<By>, By> Shift<By> for Or<A, B> {
    type Output = Or<<A as Shift<By>>::Output, <B as Shift<By>>::Output>;
}

pub struct Niche<Offset, Holes, And: NicheSpec>(core::marker::PhantomData<(Offset, Holes, And)>);
impl<Offset, Holes, And: NicheSpec> NicheSpec for Niche<Offset, Holes, And> {}
impl<Offset: Add<By>, By, Holes, And: NicheSpec + Shift<By>> Shift<By> for Niche<Offset, Holes, And>
where
    <And as Shift<By>>::Output: NicheSpec,
{
    type Output = Niche<tyeval!(Offset + By), Holes, <And as Shift<By>>::Output>;
}

pub struct End<Offset>(core::marker::PhantomData<Offset>);
impl<Offset> NicheSpec for End<Offset> {}
impl<Offset: Add<By>, By> Shift<By> for End<Offset> {
    type Output = tyeval!(Offset + By);
}

pub struct Shifted<T, By>(core::marker::PhantomData<(T, By)>);
pub trait Shift<By> {
    type Output;
}
unsafe impl<T: Stable, By> Stable for Shifted<T, By>
where
    T::Niches: Shift<By>,
    By: Rem<T::Align, Output = U0>,
    <T::Niches as Shift<By>>::Output: NicheSpec,
    T::Start: Add<By>,
{
    type Align = T::Align;
    type Start = tyeval!(T::Start + By);
    type Size = T::Size;
    type Niches = <T::Niches as Shift<By>>::Output;
}

pub trait After<T> {
    type Output;
}
impl<A, B> After<A> for B {
    type Output = SAfter<A, B>;
}
pub struct SAfter<A, B>(core::marker::PhantomData<(A, B)>);

unsafe impl<A: StableExt, B: StableExt> Stable for SAfter<A, B>
where
    A::End: Rem<B::Align>,
    tyeval!(A::End % B::Align): IsEqual<U0>,
    (Self, tyeval!((A::End % B::Align) == U0)): Stable,
{
    type Align = <(Self, tyeval!((A::End % B::Align) == U0)) as Stable>::Align;
    type Size = <(Self, tyeval!((A::End % B::Align) == U0)) as Stable>::Size;
    type Start = <(Self, tyeval!((A::End % B::Align) == U0)) as Stable>::Start;
    type Niches = <(Self, tyeval!((A::End % B::Align) == U0)) as Stable>::Niches;
}

unsafe impl<A: StableExt, B: StableExt> Stable for (SAfter<A, B>, B0)
where
    B::Start: Add<B::Size>,
    A::End: Rem<B::Align> + Add<tyeval!(B::Align - (A::End % B::Align))>,
    tyeval!(A::End + (B::Align - (A::End % B::Align))): Sub<U1>,
    B::Align: Sub<tyeval!(A::End % B::Align)>,
    tyeval!(B::Align - (A::End % B::Align)): Rem<B::Align>,
    tyeval!(A::End % B::Align): IsEqual<U0, Output = B1>,
{
    type Align = B::Align;
    type Size = B::Size;
    type Start = tyeval!(A::End + (B::Align - (A::End % B::Align)));
    type Niches = Niche<
        tyeval!((A::End + (B::Align - (A::End % B::Align))) - U1),
        FreeByte,
        End<tyeval!(B::Start + B::Size)>,
    >;
}
unsafe impl<A: StableExt, B: StableExt> Stable for (SAfter<A, B>, B1)
where
    B::Start: Add<B::Size>,
    A::End: Rem<B::Align> + Add<B::Size>,
    B::Align: Sub<tyeval!(A::End % B::Align)>,
    tyeval!(B::Align - (A::End % B::Align)): Rem<B::Align>,
    tyeval!(A::End % B::Align): IsEqual<U0, Output = B1>,
{
    type Align = B::Align;
    type Size = B::Size;
    type Start = A::End;
    type Niches = End<tyeval!(A::End + B::Size)>;
}

pub trait Ternary<A, B> {
    type Output;
}
impl<A, B> Ternary<A, B> for B0 {
    type Output = B;
}
impl<A, B> Ternary<A, B> for B1 {
    type Output = A;
}

pub type NonZeroHole = stabby_macros::holes!([1, 0, 0, 0]);
pub type FreeByte = stabby_macros::holes!([
    0xffffffffffffffff,
    0xffffffffffffffff,
    0xffffffffffffffff,
    0xffffffffffffffff
]);
