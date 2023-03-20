use super::typenum2::*;
use super::unsigned::{IBitBase, NonZero};
use super::{FieldPair, Struct, Union};
use stabby_macros::tyeval;
macro_rules! same_as {
    ($t: ty) => {
        type Align = <$t as IStable>::Align;
        type Size = <$t as IStable>::Size;
        type UnusedBits = <$t as IStable>::UnusedBits;
        type ForbiddenValues = <$t as IStable>::ForbiddenValues;
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
    type ForbiddenValues: IForbiddenValues;
    type UnusedBits: IBitMask;
    type HasExactlyOneNiche: ISaturatingAdd;
    fn size() -> usize {
        let size = Self::Size::USIZE;
        let align = Self::Align::USIZE;
        size + ((align - (size % align)) % align)
    }
    fn align() -> usize {
        Self::Align::USIZE
    }
}

/// DO NOT PUT THIS IN YOUR OWN STRUCTURE! NOT EVER!!!
/// IF UNSAFE STRUCTS WERE A THING, THIS WOULD BE IT!!
///
/// This structure is used by `#[repr(stabby)]` enums to re-export their niches.
/// You could theoretically use this to export niches from your own internally tagged unions,
/// but this is the ONLY pertinent use-case for this struct, and failing to do so properly WILL
/// make your sum types containing this memory-corruptors.
pub struct NicheExporter<
    ForbiddenValues: IForbiddenValues,
    UnusedBits: IBitMask,
    HasExactlyOneNiche: ISaturatingAdd,
>(core::marker::PhantomData<(ForbiddenValues, UnusedBits, HasExactlyOneNiche)>);

impl<
        ForbiddenValues: IForbiddenValues,
        UnusedBits: IBitMask,
        HasExactlyOneNiche: ISaturatingAdd,
    > Clone for NicheExporter<ForbiddenValues, UnusedBits, HasExactlyOneNiche>
{
    fn clone(&self) -> Self {
        Self::default()
    }
}
impl<
        ForbiddenValues: IForbiddenValues,
        UnusedBits: IBitMask,
        HasExactlyOneNiche: ISaturatingAdd,
    > Copy for NicheExporter<ForbiddenValues, UnusedBits, HasExactlyOneNiche>
{
}
impl<
        ForbiddenValues: IForbiddenValues,
        UnusedBits: IBitMask,
        HasExactlyOneNiche: ISaturatingAdd,
    > Default for NicheExporter<ForbiddenValues, UnusedBits, HasExactlyOneNiche>
{
    fn default() -> Self {
        Self(core::marker::PhantomData)
    }
}
unsafe impl<
        ForbiddenValues: IForbiddenValues,
        UnusedBits: IBitMask,
        HasExactlyOneNiche: ISaturatingAdd,
    > IStable for NicheExporter<ForbiddenValues, UnusedBits, HasExactlyOneNiche>
{
    type Size = U0;
    type Align = U1;
    type ForbiddenValues = ForbiddenValues;
    type UnusedBits = UnusedBits;
    type HasExactlyOneNiche = HasExactlyOneNiche;
}

#[crate::stabby]
#[derive(Default)]
pub struct End;
pub struct Array<Offset: Unsigned, T, Rest>(core::marker::PhantomData<(Offset, T, Rest)>);

pub trait IBitMask {
    const TUPLE: Self::Tuple;
    type Tuple: core::fmt::Debug;
    type ByteAt<O: Unsigned>: Unsigned;
    type BitOr<T: IBitMask>: IBitMask;
    type Shift<O: Unsigned>: IBitMask;
    type BitAnd<T: IBitMask>: IBitMask;
    type HasFreeByteAt<O: Unsigned>: Bit;
    type ExtractBit: IBitMask;
    type ExtractedBitByteOffset: Unsigned;
    type ExtractedBitMask: Unsigned;
}
impl IBitMask for End {
    const TUPLE: Self::Tuple = ();
    type Tuple = ();
    type ByteAt<O: Unsigned> = U0;
    type BitOr<T: IBitMask> = T;
    type Shift<O: Unsigned> = End;
    type BitAnd<T: IBitMask> = End;
    type HasFreeByteAt<O: Unsigned> = B0;
    type ExtractBit = End;
    type ExtractedBitMask = Saturator;
    type ExtractedBitByteOffset = Saturator;
}
impl<Offset: Unsigned, T: NonZero, Rest: IBitMask> IBitMask for Array<Offset, T, Rest> {
    const TUPLE: Self::Tuple = ((Offset::USIZE, T::USIZE), Rest::TUPLE);
    type Tuple = ((usize, usize), Rest::Tuple);
    type ByteAt<O: Unsigned> = <Offset::Equal<O> as Bit>::UTernary<T, Rest::ByteAt<O>>;
    type BitAnd<Mask: IBitMask> =
        <<T::BitAnd<Mask::ByteAt<Offset>> as Unsigned>::Equal<U0> as Bit>::BmTernary<
            Rest::BitAnd<Mask>,
            Array<
                Offset,
                <T::BitAnd<Mask::ByteAt<Offset>> as Unsigned>::NonZero,
                Rest::BitAnd<Mask>,
            >,
        >;
    type BitOr<Arr: IBitMask> = Array<Offset, T, Rest::BitOr<Arr>>;
    type Shift<O: Unsigned> = Array<Offset::Add<O>, T, Rest::Shift<O>>;
    type HasFreeByteAt<O: Unsigned> =
        <<O::Equal<Offset> as Bit>::Or<T::Equal<UxFF>> as Bit>::Or<Rest::HasFreeByteAt<O>>;
    type ExtractBit =
        <<T::AbsSub<T::TruncateAtRightmostOne> as Unsigned>::Greater<U0> as Bit>::BmTernary<
            Array<Offset, <T::AbsSub<T::TruncateAtRightmostOne> as Unsigned>::NonZero, Rest>,
            Rest,
        >;
    type ExtractedBitByteOffset = Offset;
    type ExtractedBitMask = T::TruncateAtRightmostOne;
}
pub trait IForbiddenValues {
    type Shift<O: Unsigned>: IForbiddenValues;
    type Or<T: IForbiddenValues>: IForbiddenValues;
    type SelectFrom<Mask: IBitMask>: ISingleForbiddenValue;
}
pub trait ISingleForbiddenValue {
    type Push<O: Unsigned, T>: ISingleForbiddenValue;
    type Or<T: ISingleForbiddenValue>: ISingleForbiddenValue;
    type Resolve: ISingleForbiddenValue;
    type And<T: ISingleForbiddenValue>: ISingleForbiddenValue;
}
impl IForbiddenValues for End {
    type Shift<O: Unsigned> = End;
    type Or<T: IForbiddenValues> = T;
    type SelectFrom<Mask: IBitMask> = End;
}
impl ISingleForbiddenValue for Saturator {
    type Push<O: Unsigned, T> = Saturator;
    type Or<T: ISingleForbiddenValue> = T;
    type And<T: ISingleForbiddenValue> = Saturator;
    type Resolve = End;
}
impl ISingleForbiddenValue for End {
    type Push<O: Unsigned, T> = Array<O, T, Self>;
    type Or<T: ISingleForbiddenValue> = T;
    type And<T: ISingleForbiddenValue> = T;
    type Resolve = Self;
}
impl<Offset: Unsigned, T, Rest: ISingleForbiddenValue> ISingleForbiddenValue
    for Array<Offset, T, Rest>
{
    type Push<O: Unsigned, V> = Array<O, V, Self>;
    type Or<V: ISingleForbiddenValue> = Self;
    type And<V: ISingleForbiddenValue> = V;
    type Resolve = Self;
}
impl<Offset: Unsigned, T, Rest: IForbiddenValues> IForbiddenValues for Array<Offset, T, Rest> {
    type Shift<O: Unsigned> = Array<Offset::Add<O>, T, Rest::Shift<O>>;
    type Or<O: IForbiddenValues> = Or<O, Self>;
    type SelectFrom<Mask: IBitMask> =
        <<Mask::HasFreeByteAt<Offset> as IBitBase>::AsForbiddenValue as ISingleForbiddenValue>::And<
            <Rest::SelectFrom<Mask> as ISingleForbiddenValue>::Push<Offset, T>,
        >;
}
impl<A: IForbiddenValues, B: IForbiddenValues> IForbiddenValues for Or<A, B> {
    type Shift<O: Unsigned> = Or<A::Shift<O>, B::Shift<O>>;
    type Or<T: IForbiddenValues> = Or<T, Self>;
    type SelectFrom<Mask: IBitMask> =
        <A::SelectFrom<Mask> as ISingleForbiddenValue>::Or<B::SelectFrom<Mask>>;
}
impl<Offset: Unsigned, T, Rest: IBitMask> Default for Array<Offset, T, Rest> {
    fn default() -> Self {
        Self(Default::default())
    }
}
pub struct Or<A, B>(core::marker::PhantomData<(A, B)>);
pub trait IsEnd {
    type Output;
}
impl IsEnd for End {
    type Output = B1;
}
impl<O: Unsigned, T, R: IBitMask> IsEnd for Array<O, T, R> {
    type Output = B0;
}

unsafe impl<A: IStable, B: IStable> IStable for FieldPair<A, B>
where
    AlignedAfter<B, A::Size>: IStable,
{
    type ForbiddenValues =
        Or<A::ForbiddenValues, <AlignedAfter<B, A::Size> as IStable>::ForbiddenValues>;
    type UnusedBits =
        <A::UnusedBits as IBitMask>::BitOr<<AlignedAfter<B, A::Size> as IStable>::UnusedBits>;
    type Size = <AlignedAfter<B, A::Size> as IStable>::Size;
    type Align = <A::Align as PowerOf2>::Max<B::Align>;
    type HasExactlyOneNiche = <A::HasExactlyOneNiche as ISaturatingAdd>::SaturatingAdd<
        <AlignedAfter<B, A::Size> as IStable>::HasExactlyOneNiche,
    >;
}
pub trait ISaturatingAdd {
    type SaturatingAddB1: ISaturatingAdd;
    type SaturatingAdd<B: ISaturatingAdd>: ISaturatingAdd;
}
impl ISaturatingAdd for B0 {
    type SaturatingAdd<B: ISaturatingAdd> = B;
    type SaturatingAddB1 = B1;
}
impl ISaturatingAdd for B1 {
    type SaturatingAddB1 = Saturator;
    type SaturatingAdd<B: ISaturatingAdd> = B::SaturatingAddB1;
}
impl ISaturatingAdd for Saturator {
    type SaturatingAddB1 = Saturator;
    type SaturatingAdd<B: ISaturatingAdd> = Saturator;
}
pub struct Saturator;

pub trait Includes<SubSet> {
    type Output;
}
impl<T> Includes<End> for T {
    type Output = End;
}
impl<O: Unsigned, T, R: IBitMask> Includes<Array<O, T, R>> for End {
    type Output = End;
}
impl<O1: Unsigned, T1, R1: IBitMask, O2: Unsigned, T2, R2: IBitMask> Includes<Array<O1, T1, R1>>
    for Array<O2, T2, R2>
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
impl<O1: Unsigned, T1, Tail: IBitMask> Arrayify for ((O1, T1), Tail, B0, B0) {
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
impl<O1: Unsigned, T1, O2: Unsigned, T2, R2: IBitMask> IncludesComputer<(O1, T1)>
    for Array<O2, T2, R2>
where
    Self: IncludesComputer<(O1, T1, tyeval!(O1 == O2))>,
{
    type Output = <Self as IncludesComputer<(O1, T1, tyeval!(O1 == O2))>>::Output;
}
impl<O1: Unsigned, T1, O2: Unsigned, T2, R2: IBitMask> IncludesComputer<(O1, T1, B0)>
    for Array<O2, T2, R2>
where
    R2: IncludesComputer<(O1, T1)>,
{
    type Output = <R2 as IncludesComputer<(O1, T1)>>::Output;
}
impl<O1: Unsigned, T1, O2: Unsigned, T2: Unsigned, R2: IBitMask> IncludesComputer<(O1, T1, B1)>
    for Array<O2, T2, R2>
where
    Self: IncludesComputer<(O1, T1, B1, tyeval!(T2 == U255))>,
{
    type Output = <Self as IncludesComputer<(O1, T1, B1, tyeval!(T2 == U255))>>::Output;
}
impl<O1: Unsigned, T1, O2: Unsigned, T2, R2: IBitMask> IncludesComputer<(O1, T1, B1, B1)>
    for Array<O2, T2, R2>
{
    type Output = (O1, T1);
}
impl<O1: Unsigned, T1, O2: Unsigned, T2, R2: IBitMask> IncludesComputer<(O1, T1, B1, B0)>
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
    type ForbiddenValues = End;
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

unsafe impl<T: IStable, Start: Unsigned> IStable for (AlignedAfter<T, Start>, U1) {
    type Align = U1;
    type Size = tyeval!(Start + T::Size);
    type UnusedBits = <T::UnusedBits as IBitMask>::Shift<Start>;
    type ForbiddenValues = <T::ForbiddenValues as IForbiddenValues>::Shift<Start>;
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
{
    type Align = T::Align;
    type Size = tyeval!(Start + T::Size);
    type UnusedBits = <T::UnusedBits as IBitMask>::Shift<Start>;
    type ForbiddenValues = <T::ForbiddenValues as IForbiddenValues>::Shift<Start>;
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
// <tyeval!(T::Align - UInt<B, Int>) as Unsigned>::Padding: IStable,
{
    type Align = T::Align;
    type Size = tyeval!((Start + (T::Align - UInt<B, Int>)) + T::Size);
    type UnusedBits = <<<<tyeval!(T::Align - UInt<B, Int>) as Unsigned>::Padding as IStable>::UnusedBits as IBitMask>::Shift<Start> as IBitMask>::BitOr<
        <T::UnusedBits as IBitMask>::Shift<tyeval!(Start + (T::Align - UInt<B, Int>))>>;
    type ForbiddenValues =
        <T::ForbiddenValues as IForbiddenValues>::Shift<tyeval!(Start + (T::Align - UInt<B, Int>))>;
    type HasExactlyOneNiche = Saturator;
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
unsafe impl<T: IStable, Align, RemU: Unsigned, RemB: Bit> IStable
    for ((Struct<T>, Align), UInt<RemU, RemB>)
where
// <tyeval!(T::Align - UInt<RemU, RemB>) as Unsigned>::Padding: IStable,
{
    type Size = tyeval!(T::Size + (T::Align - UInt<RemU, RemB>));
    type Align = T::Align;
    type ForbiddenValues = T::ForbiddenValues;
    type UnusedBits = <T::UnusedBits as IBitMask>::BitOr<
        <<<tyeval!(T::Align - UInt<RemU, RemB>) as Unsigned>::Padding as IStable>::UnusedBits as IBitMask>::Shift<T::Size>>;
    type HasExactlyOneNiche = Saturator;
}
