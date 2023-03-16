#[crate::stabby]
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Tuple2<A, B>(pub A, pub B);

#[crate::stabby]
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Tuple3<A, B, C>(pub A, pub B, pub C);

#[crate::stabby]
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Tuple4<A, B, C, D>(pub A, pub B, pub C, pub D);

#[crate::stabby]
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Tuple5<A, B, C, D, E>(pub A, pub B, pub C, pub D, pub E);

#[crate::stabby]
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Tuple6<A, B, C, D, E, F>(pub A, pub B, pub C, pub D, pub E, pub F);
