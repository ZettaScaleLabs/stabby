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

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse::Parser, DeriveInput, TypeParamBound};

#[allow(dead_code)]
pub(crate) fn logfile() -> impl std::io::Write {
    use std::{fs::OpenOptions, io::BufWriter, path::PathBuf};
    let logfile = PathBuf::from("logfile.txt");
    let logfile = BufWriter::new(
        OpenOptions::new()
            .append(true)
            .create(true)
            .open(logfile)
            .unwrap(),
    );
    logfile
}

pub(crate) fn tl_mod() -> proc_macro2::TokenStream {
    match proc_macro_crate::crate_name("stabby-abi") {
        Ok(proc_macro_crate::FoundCrate::Itself) => return quote!(crate),
        Ok(proc_macro_crate::FoundCrate::Name(crate_name)) => {
            let crate_name = Ident::new(&crate_name, Span::call_site());
            return quote!(#crate_name);
        }
        _ => {}
    }
    match proc_macro_crate::crate_name("stabby")
        .expect("Couldn't find `stabby` in your dependencies")
    {
        proc_macro_crate::FoundCrate::Itself => quote!(crate::abi),
        proc_macro_crate::FoundCrate::Name(crate_name) => {
            let crate_name = Ident::new(&crate_name, Span::call_site());
            quote!(#crate_name::abi)
        }
    }
}

#[proc_macro_attribute]
pub fn stabby(stabby_attrs: TokenStream, tokens: TokenStream) -> TokenStream {
    if let Ok(DeriveInput {
        attrs,
        vis,
        ident,
        generics,
        data,
    }) = syn::parse(tokens.clone())
    {
        match data {
            syn::Data::Struct(data) => {
                structs::stabby(attrs, vis, ident, generics, data, &stabby_attrs)
            }
            syn::Data::Enum(data) => enums::stabby(attrs, vis, ident, generics, data),
            syn::Data::Union(data) => unions::stabby(attrs, vis, ident, generics, data),
        }
    } else if let Ok(fn_spec) = syn::parse(tokens.clone()) {
        functions::stabby(stabby_attrs, fn_spec)
    } else if let Ok(trait_spec) = syn::parse(tokens.clone()) {
        traits::stabby(trait_spec)
    } else if let Ok(async_block) = syn::parse::<syn::ExprAsync>(tokens) {
        quote!(Box::new(#async_block).into())
    } else {
        panic!("Expected a type declaration, a trait declaration or a function declaration")
    }
    .into()
}

#[proc_macro]
pub fn vtable(tokens: TokenStream) -> TokenStream {
    let st = tl_mod();
    let bounds =
        syn::punctuated::Punctuated::<TypeParamBound, syn::token::Add>::parse_separated_nonempty
            .parse(tokens)
            .unwrap();
    let mut vt = quote!(#st::vtable::VtDrop);
    for bound in bounds {
        match &bound {
            TypeParamBound::Trait(t) => vt = quote!(< dyn #t as #st::vtable::CompoundVt >::Vt<#vt>),
            TypeParamBound::Lifetime(_) => todo!(),
        }
    }
    vt.into()
}

mod enums;
mod functions;
mod structs;
mod traits;
mod unions;
pub(crate) mod utils;

mod tyops;
#[proc_macro]
pub fn tyeval(tokens: TokenStream) -> TokenStream {
    tyops::tyeval(&tokens.into()).into()
}
