use super::{padding::IPadding, FieldPair, Struct, Union};
pub use ::typenum2::*;
use stabby_macros::tyeval;
macro_rules! same_as {
    ($t: ty) => {
        type Align = <$t as IStable>::Align;
        type Size = <$t as IStable>::Size;
        type UnusedBits = <$t as IStable>::UnusedBits;
        type IllegalValues = <$t as IStable>::IllegalValues;
        type HasExactlyOneNiche = <$t as IStable>::HasExactlyOneNiche;
    };
}
/// A trait to describe the layout of a type.
///
/// Every layout is assumed to start at the type's first byte.
///
/// # Safety
/// Mis-implementing this trait can lead to memory corruption in sum tyoes
pub unsafe trait IStable: Sized {
    type Size: Unsigned;
    type Align: PowerOf2;
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

#[crate::stabby]
#[derive(Default)]
pub struct End;
pub struct Array<Offset: Unsigned, T, Rest>(core::marker::PhantomData<(Offset, T, Rest)>);
impl<Offset: Unsigned, T, Rest> Default for Array<Offset, T, Rest> {
    fn default() -> Self {
        Self(Default::default())
    }
}
pub struct IllegalValue<Value: Unsigned>(core::marker::PhantomData<Value>);
pub struct Or<A, B>(core::marker::PhantomData<(A, B)>);
pub trait IsEnd {
    type Output;
}
impl IsEnd for End {
    type Output = B1;
}
impl<O: Unsigned, T, R> IsEnd for Array<O, T, R> {
    type Output = B0;
}

unsafe impl<A: IStable, B: IStable> IStable for FieldPair<A, B>
where
    AlignedAfter<B, A::Size>: IStable,
    A::UnusedBits: IArrayPush<<AlignedAfter<B, A::Size> as IStable>::UnusedBits>,
    A::HasExactlyOneNiche: SaturatingAdd<<AlignedAfter<B, A::Size> as IStable>::HasExactlyOneNiche>,
{
    type IllegalValues = Or<A::IllegalValues, <AlignedAfter<B, A::Size> as IStable>::IllegalValues>;
    type UnusedBits =
        <A::UnusedBits as IArrayPush<<AlignedAfter<B, A::Size> as IStable>::UnusedBits>>::Output;
    type Size = <AlignedAfter<B, A::Size> as IStable>::Size;
    type Align = <A::Align as PowerOf2>::Max<B::Align>;
    type HasExactlyOneNiche = <A::HasExactlyOneNiche as SaturatingAdd<
        <AlignedAfter<B, A::Size> as IStable>::HasExactlyOneNiche,
    >>::Output;
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

pub trait Includes<SubSet> {
    type Output;
}
impl<T> Includes<End> for T {
    type Output = End;
}
impl<O: Unsigned, T, R> Includes<Array<O, T, R>> for End {
    type Output = End;
}
impl<O1: Unsigned, T1, R1, O2: Unsigned, T2, R2> Includes<Array<O1, T1, R1>> for Array<O2, T2, R2>
where
    Array<O2, T2, R2>: IncludesComputer<(O1, T1)> + Includes<R1>,
    R1: IsEnd,
    <Self as Includes<R1>>::Output: IsEnd,
    (
        <Self as IncludesComputer<(O1, T1)>>::Output,
        <Self as Includes<R1>>::Output,
        <<Self as Includes<R1>>::Output as IsEnd>::Output,
        <R1 as IsEnd>::Output,
    ): Arrayify,
{
    type Output = <(
        <Self as IncludesComputer<(O1, T1)>>::Output,
        <Self as Includes<R1>>::Output,
        <<Self as Includes<R1>>::Output as IsEnd>::Output,
        <R1 as IsEnd>::Output,
    ) as Arrayify>::Output;
}
impl<O1: Unsigned, T1> Arrayify for ((O1, T1), End, B1, B1) {
    type Output = Array<O1, T1, End>;
}
impl<O1: Unsigned, T1> Arrayify for ((O1, T1), End, B1, B0) {
    type Output = End;
}
impl<O1: Unsigned, T1, Tail> Arrayify for ((O1, T1), Tail, B0, B0) {
    type Output = Array<O1, T1, Tail>;
}
impl<Tail, T, U> Arrayify for (End, Tail, T, U) {
    type Output = End;
}
pub trait Arrayify {
    type Output;
}
pub trait IncludesComputer<SubSet> {
    type Output;
}
impl<O1: Unsigned, T1, O2: Unsigned, T2, R2> IncludesComputer<(O1, T1)> for Array<O2, T2, R2>
where
    Self: IncludesComputer<(O1, T1, tyeval!(O1 == O2))>,
{
    type Output = <Self as IncludesComputer<(O1, T1, tyeval!(O1 == O2))>>::Output;
}
impl<O1: Unsigned, T1, O2: Unsigned, T2, R2> IncludesComputer<(O1, T1, B0)> for Array<O2, T2, R2>
where
    R2: IncludesComputer<(O1, T1)>,
{
    type Output = <R2 as IncludesComputer<(O1, T1)>>::Output;
}
impl<O1: Unsigned, T1, O2: Unsigned, T2: Unsigned, R2> IncludesComputer<(O1, T1, B1)>
    for Array<O2, T2, R2>
where
    Self: IncludesComputer<(O1, T1, B1, tyeval!(T2 == U255))>,
{
    type Output = <Self as IncludesComputer<(O1, T1, B1, tyeval!(T2 == U255))>>::Output;
}
impl<O1: Unsigned, T1, O2: Unsigned, T2, R2> IncludesComputer<(O1, T1, B1, B1)>
    for Array<O2, T2, R2>
{
    type Output = (O1, T1);
}
impl<O1: Unsigned, T1, O2: Unsigned, T2, R2> IncludesComputer<(O1, T1, B1, B0)>
    for Array<O2, T2, R2>
{
    type Output = End;
}

unsafe impl<A: IStable, B: IStable> IStable for Union<A, B>
where
    (Self, tyeval!(A::Align == B::Align)): IStable,
{
    same_as!((Self, tyeval!(A::Align == B::Align)));
}
unsafe impl<A: IStable, B: IStable> IStable for (Union<A, B>, B1) {
    type IllegalValues = End;
    type UnusedBits = End;
    type Size = <A::Size as Unsigned>::Max<B::Size>;
    type Align = <A::Align as PowerOf2>::Max<B::Align>;
    type HasExactlyOneNiche = B0;
}
unsafe impl<A: IStable, B: IStable> IStable for (Union<A, B>, B0)
where
    Struct<(Union<A, B>, B1)>: IStable,
{
    same_as!(Struct<(Union<A, B>, B1)>);
}

pub trait IArrayPush<T> {
    type Output;
}
impl<Arr> IArrayPush<Arr> for End {
    type Output = Arr;
}
impl<Arr, Offset: Unsigned, T, Rest: IArrayPush<Arr>> IArrayPush<Arr> for Array<Offset, T, Rest> {
    type Output = Array<Offset, T, <Rest as IArrayPush<Arr>>::Output>;
}

pub struct AlignedAfter<T, Start: Unsigned>(core::marker::PhantomData<(T, Start)>);

// AlignedAfter a ZST
unsafe impl<T: IStable> IStable for AlignedAfter<T, U0> {
    same_as!(T);
}
// Aligned after a non-ZST
unsafe impl<T: IStable, B: Unsigned, Int: Bit> IStable for AlignedAfter<T, UInt<B, Int>>
where
    (Self, T::Align): IStable,
{
    same_as!((Self, T::Align));
}

unsafe impl<T: IStable, Start: Unsigned> IStable for (AlignedAfter<T, Start>, U1)
where
    tyeval!(Start + T::Size): Unsigned,
    T::UnusedBits: IShift<Start>,
    T::IllegalValues: IShift<Start>,
{
    type Align = U1;
    type Size = tyeval!(Start + T::Size);
    type UnusedBits = <T::UnusedBits as IShift<Start>>::Output;
    type IllegalValues = <T::IllegalValues as IShift<Start>>::Output;
    type HasExactlyOneNiche = T::HasExactlyOneNiche;
}
// non-ZST aligned after a non-ZST
unsafe impl<T: IStable, Start: Unsigned, TAlignB1: Bit, TAlignB2: Bit, TAlignInt: Unsigned> IStable
    for (
        AlignedAfter<T, Start>,
        UInt<UInt<TAlignInt, TAlignB1>, TAlignB2>,
    )
where
    UInt<UInt<TAlignInt, TAlignB1>, TAlignB2>: PowerOf2,
    (
        Self,
        tyeval!(Start % UInt<UInt<TAlignInt, TAlignB1>, TAlignB2>),
    ): IStable,
{
    same_as!((
        Self,
        tyeval!(Start % UInt<UInt<TAlignInt, TAlignB1>, TAlignB2>)
    ));
}
// non-ZST already aligned
unsafe impl<T: IStable, Start: Unsigned, TAlignB: Unsigned, TAlignInt: Bit> IStable
    for ((AlignedAfter<T, Start>, UInt<TAlignB, TAlignInt>), U0)
where
    T::UnusedBits: IShift<Start>,
    T::IllegalValues: IShift<Start>,
{
    type Align = T::Align;
    type Size = tyeval!(Start + T::Size);
    type UnusedBits = <T::UnusedBits as IShift<Start>>::Output;
    type IllegalValues = <T::IllegalValues as IShift<Start>>::Output;
    type HasExactlyOneNiche = T::HasExactlyOneNiche;
}
// non-ZST needs alignment
unsafe impl<T: IStable, Start: Unsigned, TAlignB: Unsigned, TAlignInt: Bit, B: Unsigned, Int: Bit>
    IStable
    for (
        (AlignedAfter<T, Start>, UInt<TAlignB, TAlignInt>),
        UInt<B, Int>,
    )
where
    T::UnusedBits: IShift<tyeval!(Start + (T::Align - UInt<B, Int>))>,
    T::IllegalValues: IShift<tyeval!(Start + (T::Align - UInt<B, Int>))>,
    tyeval!(T::Align - UInt<B, Int>): IPadding,
    <tyeval!(T::Align - UInt<B, Int>) as IPadding>::Padding: IStable,
    <<tyeval!(T::Align - UInt<B, Int>) as IPadding>::Padding as IStable>::UnusedBits: IShift<Start>,
    <<<tyeval!(T::Align - UInt<B, Int>) as IPadding>::Padding as IStable>::UnusedBits as IShift<
        Start,
    >>::Output:
        IArrayPush<<T::UnusedBits as IShift<tyeval!(Start + (T::Align - UInt<B, Int>))>>::Output>,
{
    type Align = T::Align;
    type Size = tyeval!((Start + (T::Align - UInt<B, Int>)) + T::Size);
    type UnusedBits = <<<<tyeval!(T::Align - UInt<B, Int>) as IPadding>::Padding as IStable>::UnusedBits as IShift<Start>>::Output as IArrayPush<
        <T::UnusedBits as IShift<tyeval!(Start + (T::Align - UInt<B, Int>))>>::Output,
    >>::Output;
    type IllegalValues =
        <T::IllegalValues as IShift<tyeval!(Start + (T::Align - UInt<B, Int>))>>::Output;
    type HasExactlyOneNiche = B2;
}

pub trait IShift<By: Unsigned> {
    type Output;
}
impl<By: Unsigned> IShift<By> for End {
    type Output = End;
}

impl<Offset: Unsigned, T, Rest: IShift<By>, By: Unsigned> IShift<By> for Array<Offset, T, Rest> {
    type Output = Array<tyeval!(Offset + By), T, Rest::Output>;
}
impl<A: IShift<By>, B: IShift<By>, By: Unsigned> IShift<By> for Or<A, B> {
    type Output = Or<A::Output, B::Output>;
}

unsafe impl<T: IStable> IStable for Struct<T>
where
    (Self, T::Align): IStable,
{
    same_as!((Self, T::Align));
}
unsafe impl<T: IStable> IStable for (Struct<T>, U0) {
    same_as!(T);
}
unsafe impl<T: IStable, B: Bit, Int: Unsigned> IStable for (Struct<T>, UInt<Int, B>)
where
    UInt<Int, B>: PowerOf2,
    (Self, tyeval!(T::Size % UInt<Int, B>)): IStable,
{
    same_as!((Self, tyeval!(T::Size % UInt<Int, B>)));
}
unsafe impl<T: IStable, Align> IStable for ((Struct<T>, Align), U0) {
    same_as!(T);
}
unsafe impl<T: IStable, Align, RemU: Unsigned, RemB: Bit> IStable for ((Struct<T>, Align), UInt<RemU, RemB>)
where
    tyeval!(T::Align - UInt<RemU, RemB>): IPadding,
    <tyeval!(T::Align - UInt<RemU, RemB>) as IPadding>::Padding: IStable,
    <<tyeval!(T::Align - UInt<RemU, RemB>) as IPadding>::Padding as IStable>::UnusedBits: IShift<T::Size>,
    T::UnusedBits: IArrayPush<
        <<<tyeval!(T::Align - UInt<RemU, RemB>) as IPadding>::Padding as IStable>::UnusedBits as IShift<T::Size>>::Output,
    >,
{
    type Size = tyeval!(T::Size + (T::Align - UInt<RemU, RemB>));
    type Align = T::Align;
    type IllegalValues = T::IllegalValues;
    type UnusedBits = <T::UnusedBits as IArrayPush<
        <<<tyeval!(T::Align - UInt<RemU, RemB>) as IPadding>::Padding as IStable>::UnusedBits as IShift<T::Size>>::Output,
    >>::Output;
    type HasExactlyOneNiche = B2;
}
