//
// Copyright (c) 2023 ZettaScale Technology
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   Pierre Avital, <pierre.avital@me.com>
//

/// A tuple of 2 elements.
#[crate::stabby]
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Tuple2<A, B>(pub A, pub B);

/// A tuple of 3 elements.
#[crate::stabby]
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Tuple3<A, B, C>(pub A, pub B, pub C);

/// A tuple of 4 elements.
#[crate::stabby]
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Tuple4<A, B, C, D>(pub A, pub B, pub C, pub D);

/// A tuple of 5 elements.
#[crate::stabby]
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Tuple5<A, B, C, D, E>(pub A, pub B, pub C, pub D, pub E);

/// A tuple of 6 elements.
#[crate::stabby]
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Tuple6<A, B, C, D, E, F>(pub A, pub B, pub C, pub D, pub E, pub F);

/// A tuple of 7 elements.
#[crate::stabby]
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Tuple7<A, B, C, D, E, F, G>(pub A, pub B, pub C, pub D, pub E, pub F, pub G);

/// A tuple of 8 elements.
#[crate::stabby]
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Tuple8<A, B, C, D, E, F, G, H>(pub A, pub B, pub C, pub D, pub E, pub F, pub G, pub H);

/// A tuple of 9 elements.
#[crate::stabby]
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Tuple9<A, B, C, D, E, F, G, H, I>(
    pub A,
    pub B,
    pub C,
    pub D,
    pub E,
    pub F,
    pub G,
    pub H,
    pub I,
);
