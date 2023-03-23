pub use super::*;
/// Prevents the compiler from doing infinite recursion when evaluating `IDiscriminantProvider`
type DefaultRecursionBudget = T<T<T<T<T<T<T<T<H>>>>>>>>;
// ENTER LOOP ON Budget
impl<Ok: IStable, Err: IStable, EI: Unsigned, EB: Bit> IDiscriminantProviderInner
    for (Ok, Err, UInt<EI, EB>)
where
    (Ok, Err, UInt<EI, EB>, U0, DefaultRecursionBudget): IDiscriminantProviderInner,
{
    same_as!((Ok, Err, UInt<EI, EB>, U0, DefaultRecursionBudget));
}
// EXIT LOOP
impl<Ok: IStable, Err: IStable, ErrS: Unsigned, EI: Unsigned, EB: Bit> IDiscriminantProviderInner
    for (Ok, Err, UInt<EI, EB>, ErrS, H)
{
    same_as!((Ok, Err, UTerm, End, End));
}

/// Branch on whether some forbidden values for Err fit inside Ok's unused bits
impl<Ok: IStable, Err: IStable, ErrS: Unsigned, Budget, EI: Unsigned, EB: Bit>
    IDiscriminantProviderInner for (Ok, Err, UInt<EI, EB>, ErrS, T<Budget>)
where
    (
        Ok,
        Err,
        UInt<EI, EB>,
        ErrS,
        T<Budget>,
        <<ShiftedForbiddenValues<Err, ErrS> as IForbiddenValues>::SelectFrom<
            UnionMemberUnusedBits<Ok, Err, U0>,
        > as ISingleForbiddenValue>::Resolve,
    ): IDiscriminantProviderInner,
{
    same_as!((
        Ok,
        Err,
        UInt<EI, EB>,
        ErrS,
        T<Budget>,
        <<ShiftedForbiddenValues<Err, ErrS> as IForbiddenValues>::SelectFrom<
            UnionMemberUnusedBits<Ok, Err, U0>,
        > as ISingleForbiddenValue>::Resolve,
    ));
}

/// If some forbidden values for Err fit inside Ok's unused bits, exit the recursion
impl<
        Ok: IStable,
        Err: IStable,
        ErrS: Unsigned,
        Offset: Unsigned,
        V: Unsigned,
        Tail: IForbiddenValues + IntoValueIsErr,
        Budget,
        EI: Unsigned,
        EB: Bit,
    > IDiscriminantProviderInner
    for (
        Ok,
        Err,
        UInt<EI, EB>,
        ErrS,
        T<Budget>,
        Array<Offset, V, Tail>,
    )
{
    type ErrShift = ErrS;
    type Discriminant = Not<<Array<Offset, V, Tail> as IntoValueIsErr>::ValueIsErr>;
    type NicheExporter = NicheExporter<
        End,
        <UnionMemberUnusedBits<Ok, Err, U0> as IBitMask>::BitAnd<
            UnionMemberUnusedBits<Err, Ok, ErrS>,
        >,
        Saturator,
    >;
}

/// None of Err's forbidden values fit into Ok's unused bits, so branch on wherther
/// some of Ok's forbidden values fit into Err's forbidden value
impl<Ok: IStable, Err: IStable, ErrS: Unsigned, Budget, EI: Unsigned, EB: Bit>
    IDiscriminantProviderInner for (Ok, Err, UInt<EI, EB>, ErrS, T<Budget>, End)
where
    (
        Ok,
        Err,
        UInt<EI, EB>,
        ErrS,
        T<Budget>,
        End,
        <<Ok::ForbiddenValues as IForbiddenValues>::SelectFrom<
            UnionMemberUnusedBits<Err, Ok, ErrS>,
        > as ISingleForbiddenValue>::Resolve,
    ): IDiscriminantProviderInner,
{
    same_as!((
        Ok,
        Err,
        UInt<EI, EB>,
        ErrS,
        T<Budget>,
        End,
        <<Ok::ForbiddenValues as IForbiddenValues>::SelectFrom<
            UnionMemberUnusedBits<Err, Ok, ErrS>,
        > as ISingleForbiddenValue>::Resolve,
    ));
}

/// If some forbidden values for Ok fit inside Err's unused bits, exit the recursion
impl<
        Ok: IStable,
        Err: IStable,
        ErrS: Unsigned,
        Offset: Unsigned,
        V: Unsigned,
        Tail: IForbiddenValues + IntoValueIsErr,
        Budget,
        EI: Unsigned,
        EB: Bit,
    > IDiscriminantProviderInner
    for (
        Ok,
        Err,
        UInt<EI, EB>,
        ErrS,
        T<Budget>,
        End,
        Array<Offset, V, Tail>,
    )
{
    type ErrShift = ErrS;
    type Discriminant = <Array<Offset, V, Tail> as IntoValueIsErr>::ValueIsErr;
    type NicheExporter = NicheExporter<
        End,
        <UnionMemberUnusedBits<Ok, Err, U0> as IBitMask>::BitAnd<
            UnionMemberUnusedBits<Err, Ok, ErrS>,
        >,
        Saturator,
    >;
}

/// If neither Err nor Ok's unused bits can fit any of the other's forbidden value,
/// check if their unused bits have an intersection
impl<Ok: IStable, Err: IStable, ErrS: Unsigned, Budget, EI: Unsigned, EB: Bit>
    IDiscriminantProviderInner for (Ok, Err, UInt<EI, EB>, ErrS, T<Budget>, End, End)
where
    (
        Ok,
        Err,
        UInt<EI, EB>,
        ErrS,
        T<Budget>,
        End,
        End,
        <UnionMemberUnusedBits<Ok, Err, U0> as IBitMask>::BitAnd<
            UnionMemberUnusedBits<Err, Ok, ErrS>,
        >,
    ): IDiscriminantProviderInner,
{
    same_as!((
        Ok,
        Err,
        UInt<EI, EB>,
        ErrS,
        T<Budget>,
        End,
        End,
        <UnionMemberUnusedBits<Ok, Err, U0> as IBitMask>::BitAnd<
            UnionMemberUnusedBits<Err, Ok, ErrS>,
        >,
    ));
}
/// If Ok and Err's unused bits have an intersection, use it.
impl<
        Ok: IStable,
        Err: IStable,
        ErrS: Unsigned,
        Offset: Unsigned,
        V: NonZero,
        Rest: IBitMask,
        Budget,
        EI: Unsigned,
        EB: Bit,
    > IDiscriminantProviderInner
    for (
        Ok,
        Err,
        UInt<EI, EB>,
        ErrS,
        T<Budget>,
        End,
        End,
        Array<Offset, V, Rest>,
    )
{
    type ErrShift = ErrS;
    type Discriminant = BitIsErr<
        <Array<Offset, V, Rest> as IBitMask>::ExtractedBitByteOffset,
        <Array<Offset, V, Rest> as IBitMask>::ExtractedBitMask,
    >;
    type NicheExporter =
        NicheExporter<End, <Array<Offset, V, Rest> as IBitMask>::ExtractBit, Saturator>;
}
/// If no niche was found, compare Ok and Err's sizes to push the smallest to the right
impl<Ok: IStable, Err: IStable, ErrS: Unsigned, Budget, EI: Unsigned, EB: Bit>
    IDiscriminantProviderInner for (Ok, Err, UInt<EI, EB>, ErrS, T<Budget>, End, End, End)
where
    (
        Ok,
        Err,
        UInt<EI, EB>,
        ErrS,
        T<Budget>,
        End,
        End,
        End,
        <tyeval!((Err::Size + Err::Align) + ErrS) as Unsigned>::SmallerOrEq<
            UnionSize<Ok, Err, U0, U0>,
        >,
    ): IDiscriminantProviderInner,
{
    same_as!((
        Ok,
        Err,
        UInt<EI, EB>,
        ErrS,
        T<Budget>,
        End,
        End,
        End,
        <tyeval!((Err::Size + Err::Align) + ErrS) as Unsigned>::SmallerOrEq<
            UnionSize<Ok, Err, U0, U0>,
        >
    ));
}
impl<Ok: IStable, Err: IStable, ErrS: Unsigned, Budget, EI: Unsigned, EB: Bit>
    IDiscriminantProviderInner for (Ok, Err, UInt<EI, EB>, ErrS, T<Budget>, End, End, End, B0)
{
    type Discriminant = BitDiscriminant;
    type ErrShift = U0;
    type NicheExporter = ();
}

impl<Ok: IStable, Err: IStable, ErrS: Unsigned, Budget, EI: Unsigned, EB: Bit>
    IDiscriminantProviderInner for (Ok, Err, UInt<EI, EB>, ErrS, T<Budget>, End, End, End, B1)
where
    (Ok, Err, UInt<EI, EB>, tyeval!(ErrS + Err::Align), Budget): IDiscriminantProviderInner,
{
    same_as!((Ok, Err, UInt<EI, EB>, tyeval!(ErrS + Err::Align), Budget));
}
