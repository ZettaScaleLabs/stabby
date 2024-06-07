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

use quote::quote;
use syn::{ConstParam, GenericParam, Lifetime, LifetimeDef, TypeParam};

#[derive(Clone, Default)]
pub(crate) struct SeparatedGenerics {
    pub lifetimes: Vec<proc_macro2::TokenStream>,
    pub types: Vec<proc_macro2::TokenStream>,
    pub consts: Vec<proc_macro2::TokenStream>,
}
impl quote::ToTokens for SeparatedGenerics {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for l in &self.lifetimes {
            tokens.extend(quote!(#l,));
        }
        for l in &self.types {
            tokens.extend(quote!(#l,));
        }
        for l in &self.consts {
            tokens.extend(quote!(#l,));
        }
    }
}
pub(crate) fn unbound_generics<'a>(
    generics: impl IntoIterator<Item = &'a GenericParam>,
) -> SeparatedGenerics {
    let mut this = SeparatedGenerics::default();
    for g in generics {
        match g {
            GenericParam::Type(TypeParam { ident, .. }) => this.types.push(quote!(#ident)),
            GenericParam::Lifetime(LifetimeDef { lifetime, .. }) => {
                this.lifetimes.push(quote!(#lifetime))
            }
            GenericParam::Const(ConstParam { ident, .. }) => this.consts.push(quote!(#ident)),
        }
    }
    this
}
pub(crate) fn generics_without_defaults<'a>(
    generics: impl IntoIterator<Item = &'a GenericParam>,
) -> SeparatedGenerics {
    let mut this = SeparatedGenerics::default();
    for g in generics {
        match g {
            GenericParam::Type(TypeParam { ident, bounds, .. }) => {
                this.types.push(quote!(#ident: #bounds))
            }
            GenericParam::Lifetime(LifetimeDef {
                lifetime, bounds, ..
            }) => this.lifetimes.push(quote!(#lifetime: #bounds)),
            GenericParam::Const(ConstParam { ident, ty, .. }) => {
                this.consts.push(quote!(const #ident: #ty))
            }
        }
    }
    this
}

pub trait IGenerics<'a> {
    type Lifetimes: Iterator<Item = &'a LifetimeDef>;
    fn lifetimes(self) -> Self::Lifetimes;
    type Types: Iterator<Item = &'a TypeParam>;
    fn types(self) -> Self::Types;
    type Consts: Iterator<Item = &'a ConstParam>;
    fn consts(self) -> Self::Consts;
}
impl<'a, T: IntoIterator<Item = &'a GenericParam>> IGenerics<'a> for T {
    type Lifetimes =
        core::iter::FilterMap<T::IntoIter, fn(&'a GenericParam) -> Option<&'a LifetimeDef>>;
    fn lifetimes(self) -> Self::Lifetimes {
        self.into_iter().filter_map(|g| {
            if let GenericParam::Lifetime(l) = g {
                Some(l)
            } else {
                None
            }
        })
    }
    type Types = core::iter::FilterMap<T::IntoIter, fn(&'a GenericParam) -> Option<&'a TypeParam>>;
    fn types(self) -> Self::Types {
        self.into_iter().filter_map(|g| {
            if let GenericParam::Type(l) = g {
                Some(l)
            } else {
                None
            }
        })
    }
    type Consts =
        core::iter::FilterMap<T::IntoIter, fn(&'a GenericParam) -> Option<&'a ConstParam>>;
    fn consts(self) -> Self::Consts {
        self.into_iter().filter_map(|g| {
            if let GenericParam::Const(l) = g {
                Some(l)
            } else {
                None
            }
        })
    }
}
pub trait Unbound {
    type Unbound;
    fn unbound(self) -> Self::Unbound;
}
impl<'a> Unbound for &'a LifetimeDef {
    type Unbound = &'a Lifetime;
    fn unbound(self) -> Self::Unbound {
        &self.lifetime
    }
}

impl<'a> Unbound for &'a TypeParam {
    type Unbound = &'a syn::Ident;
    fn unbound(self) -> Self::Unbound {
        &self.ident
    }
}

impl<'a> Unbound for &'a ConstParam {
    type Unbound = &'a syn::Ident;
    fn unbound(self) -> Self::Unbound {
        &self.ident
    }
}
impl<'a, T: Iterator<Item = &'a GenericParam>, I: Unbound> Unbound
    for core::iter::FilterMap<T, fn(&'a GenericParam) -> Option<I>>
{
    type Unbound = core::iter::Map<
        core::iter::FilterMap<T, fn(&'a GenericParam) -> Option<I>>,
        fn(I) -> I::Unbound,
    >;
    fn unbound(self) -> Self::Unbound {
        self.map(I::unbound)
    }
}
