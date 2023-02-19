use stabby_macros::{tyeval, tybound};

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
    type End = <T::Start as Add<T::Size>>::Output;
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
    type Output = Niche<<Offset as Add<By>>::Output, Holes, <And as Shift<By>>::Output>;
}

pub struct End<Offset>(core::marker::PhantomData<Offset>);
impl<Offset> NicheSpec for End<Offset> {}
impl<Offset: Add<By>, By> Shift<By> for End<Offset> {
    type Output = <Offset as Add<By>>::Output;
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
    type Start = <T::Start as Add<By>>::Output;
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

type Test = tyeval!(!B1);

unsafe impl<A: StableExt, B: StableExt> Stable for SAfter<A, B>
where
tybound!(+ A::End (% (- B::Align (% A::End B::Align)) B::Align))
 {
    type Align = B::Align;
    type Size = B::Size;
    type Start = tyeval!(+ A::End (% (- B::Align (% A::End B::Align)) B::Align));
}
// unsafe impl<A: StableExt, B: StableExt> Stable for SAfter<A, B>
// where
//     A::End: Rem<B::Align>,
//     B::Align: Sub<tyeval!(A::End % B::Align)>,
//     tyeval!(B::Align - tyeval!(A::End % B::Align)): Rem<B::Align>,
//     tyeval!(tyeval!(B::Align - tyeval!(A::End % B::Align)) % B::Align): Add<A::End>,
//     tyeval!(tyeval!(tyeval!(B::Align - tyeval!(A::End % B::Align)) % B::Align) + A::End):
//         IsEqual<A::End> + Sub<B::Start>,
//     <tyeval!(tyeval!(tyeval!(B::Align - tyeval!(A::End % B::Align)) % B::Align) + A::End) as
//         IsEqual<A::End>>::Output: Ternary<
//         Shifted<B::Niches, <tyeval!(tyeval!(tyeval!(B::Align - tyeval!(A::End % B::Align)) % B::Align) + A::End) as Sub<B::Start>>::Output>,
//         Shifted<B::Niches, <tyeval!(tyeval!(tyeval!(B::Align - tyeval!(A::End % B::Align)) % B::Align) + A::End) as Sub<B::Start>>::Output>>,
//     <<tyeval!(tyeval!(tyeval!(B::Align - tyeval!(A::End % B::Align)) % B::Align) + A::End) as
//         IsEqual<A::End>>::Output as Ternary<
//         Shifted<B::Niches, <tyeval!(tyeval!(tyeval!(B::Align - tyeval!(A::End % B::Align)) % B::Align) + A::End) as Sub<B::Start>>::Output>,
//         Shifted<B::Niches, <tyeval!(tyeval!(tyeval!(B::Align - tyeval!(A::End % B::Align)) % B::Align) + A::End) as Sub<B::Start>>::Output>>>::Output: NicheSpec,
// {
//     type Align = B::Align;
//     type Size = B::Size;
//     type Start =
//         tyeval!(tyeval!(tyeval!(B::Align - tyeval!(A::End % B::Align)) % B::Align) + A::End);
//     type Niches = <<Self::Start as IsEqual<A::End>>::Output as Ternary<
//         Shifted<B::Niches, <Self::Start as Sub<B::Start>>::Output>,
//         Shifted<B::Niches, <Self::Start as Sub<B::Start>>::Output>,
//     >>::Output;
// }

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
