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

use core::borrow::Borrow;

use crate::{
    abi::{IDiscriminantProvider, IStable},
    str::Str,
    string::String,
};

#[crate::stabby]
pub enum Cow<'a, Borrowed: IStable + ToOwned>
where
    <Borrowed as ToOwned>::Owned: IStable,
{
    Borrowed(&'a Borrowed),
    Owned(<Borrowed as ToOwned>::Owned),
}

impl<Borrowed: IStable + ToOwned> Cow<'_, Borrowed>
where
    <Borrowed as ToOwned>::Owned: IStable,
    for<'a> &'a Borrowed: IDiscriminantProvider<<Borrowed as ToOwned>::Owned>,
{
    pub fn into_owned(self) -> <Borrowed as ToOwned>::Owned {
        self.match_owned(|b| b.to_owned(), |o| o)
    }
    pub fn to_owned(self) -> Cow<'static, Borrowed> {
        Cow::Owned(self.into_owned())
    }
}
impl<Borrowed: IStable + ToOwned> Borrow<Borrowed> for Cow<'_, Borrowed>
where
    <Borrowed as ToOwned>::Owned: IStable,
    for<'a> &'a Borrowed: IDiscriminantProvider<<Borrowed as ToOwned>::Owned>,
{
    fn borrow(&self) -> &Borrowed {
        self.match_ref(|&b| b, |o| o.borrow())
    }
}

#[crate::stabby]
pub enum CowStr<'a> {
    Borrowed(Str<'a>),
    Owned(String),
}
