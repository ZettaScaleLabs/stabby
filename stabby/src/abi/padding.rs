use crate::abi::*;
use crate::tuple::Tuple2;

#[crate::stabby]
#[derive(Clone, Copy)]
pub struct Padded<Left: IPadding, T> {
    pub lpad: Left::Padding,
    pub value: T,
}
impl<Left: IPadding, T> From<T> for Padded<Left, T> {
    fn from(value: T) -> Self {
        Self {
            lpad: Default::default(),
            value,
        }
    }
}
impl<Left: IPadding, T> Deref for Padded<Left, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl<Left: IPadding, T> DerefMut for Padded<Left, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

#[repr(transparent)]
#[derive(Debug, Default, Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PadByte(u8);
unsafe impl IStable for PadByte {
    type Size = U1;
    type Align = U1;
    type IllegalValues = End;
    type UnusedBits = Array<U0, U255, End>;
    type HasExactlyOneNiche = B0;
}

pub trait IPadding {
    type Padding: Default + Sized + Copy;
}
impl IPadding for UTerm {
    type Padding = ();
}
impl<A: IPadding> IPadding for UInt<A, B0> {
    type Padding = A::Padding;
}
impl IPadding for UInt<UTerm, B1> {
    type Padding = PadByte;
}
impl<A> IPadding for UInt<UInt<UTerm, A>, B1>
where
    UInt<UTerm, A>: IPadding,
{
    type Padding = Tuple2<
        <UInt<UTerm, A> as IPadding>::Padding,
        Tuple2<<U1 as IPadding>::Padding, <U1 as IPadding>::Padding>,
    >;
}
impl<A, B> IPadding for UInt<UInt<UInt<UTerm, A>, B>, B1>
where
    UInt<UInt<UTerm, A>, B>: IPadding,
{
    type Padding = Tuple2<
        <UInt<UInt<UTerm, A>, B> as IPadding>::Padding,
        Tuple2<<U2 as IPadding>::Padding, <U2 as IPadding>::Padding>,
    >;
}
impl<A, B, C> IPadding for UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, B1>
where
    UInt<UInt<UInt<UTerm, A>, B>, C>: IPadding,
{
    type Padding = Tuple2<
        <UInt<UInt<UInt<UTerm, A>, B>, C> as IPadding>::Padding,
        Tuple2<<U4 as IPadding>::Padding, <U4 as IPadding>::Padding>,
    >;
}
impl<A, B, C, D> IPadding for UInt<UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D>, B1>
where
    UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D>: IPadding,
{
    type Padding = Tuple2<
        <UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D> as IPadding>::Padding,
        Tuple2<<U8 as IPadding>::Padding, <U8 as IPadding>::Padding>,
    >;
}
impl<A, B, C, D, E> IPadding for UInt<UInt<UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D>, E>, B1>
where
    UInt<UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D>, E>: IPadding,
{
    type Padding = Tuple2<
        <UInt<UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D>, E> as IPadding>::Padding,
        Tuple2<<U16 as IPadding>::Padding, <U16 as IPadding>::Padding>,
    >;
}
impl<A, B, C, D, E, F> IPadding
    for UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D>, E>, F>, B1>
where
    UInt<UInt<UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D>, E>, F>: IPadding,
{
    type Padding = Tuple2<
        <UInt<UInt<UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D>, E>, F> as IPadding>::Padding,
        Tuple2<<U32 as IPadding>::Padding, <U32 as IPadding>::Padding>,
    >;
}
impl<A, B, C, D, E, F, G> IPadding
    for UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D>, E>, F>, G>, B1>
where
    UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D>, E>, F>, G>: IPadding,
{
    type Padding = Tuple2<
        <UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D>, E>, F>, G> as IPadding>::Padding,
        Tuple2<<U64 as IPadding>::Padding, <U64 as IPadding>::Padding>,
    >;
}
impl<A, B, C, D, E, F, G, H> IPadding
    for UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D>, E>, F>, G>, H>, B1>
where
    UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D>, E>, F>, G>, H>: IPadding,
{
    type Padding = Tuple2<
        <UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D>, E>, F>, G>, H> as IPadding>::Padding,
        Tuple2<<U128 as IPadding>::Padding, <U128 as IPadding>::Padding>,
    >;
}
impl<A, B, C, D, E, F, G, H, I> IPadding
    for UInt<
        UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D>, E>, F>, G>, H>, I>,
        B1,
    >
where
    UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D>, E>, F>, G>, H>, I>:
        IPadding,
{
    type Padding = Tuple2<
        <UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D>, E>, F>, G>, H>, I> as IPadding>::Padding,
        Tuple2<<U256 as IPadding>::Padding, <U256 as IPadding>::Padding>,
    >;
}
impl<A, B, C, D, E, F, G, H, I, J> IPadding
    for UInt<
        UInt<
            UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D>, E>, F>, G>, H>, I>,
            J,
        >,
        B1,
    >
where
    UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D>, E>, F>, G>, H>, I>, J>:
        IPadding,
{
    type Padding = Tuple2<
        <UInt<
            UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, A>, B>, C>, D>, E>, F>, G>, H>, I>,
            J,
        > as IPadding>::Padding,
        Tuple2<<U512 as IPadding>::Padding, <U512 as IPadding>::Padding>,
    >;
}
