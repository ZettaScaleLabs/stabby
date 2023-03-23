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
//   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
//

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
