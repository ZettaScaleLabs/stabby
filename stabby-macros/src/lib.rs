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
pub(crate) fn logfile(logfile: std::path::PathBuf) -> impl std::io::Write {
    use std::{fs::OpenOptions, io::BufWriter};
    let logfile = BufWriter::new(
        OpenOptions::new()
            .append(true)
            .create(true)
            .open(logfile)
            .unwrap(),
    );
    logfile
}

#[allow(unused_macros)]
macro_rules! log {
    ($path: literal, $e: expr) => {{
        let logfile = std::path::PathBuf::from($path);
        use std::io::Write;
        let e = $e;
        writeln!(crate::logfile(logfile), "{e}");
        e
    }};
    ($e: expr) => {
        log!("logfile.txt", $e)
    };
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
            TypeParamBound::Lifetime(lt) => panic!("Cannot give lifetimes to vtables, use `Dyn<{lt}, P, Vt>` or `DynRef<{lt}, Vt> instead`"),
        }
    }
    vt.into()
}

enum PtrType {
    Path(proc_macro2::TokenStream),
    Ref,
    RefMut,
}
struct DynPtr {
    ptr: PtrType,
    bounds: Vec<syn::TraitBound>,
    lifetime: Option<syn::Lifetime>,
}
impl syn::parse::Parse for DynPtr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let (mut this, elem) = match input.parse::<syn::Type>()? {
            syn::Type::Path(syn::TypePath {
                path:
                    syn::Path {
                        leading_colon,
                        mut segments,
                    },
                ..
            }) => {
                let syn::PathSegment { ident, arguments: syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments { colon2_token: None, mut args, ..}) } = segments.pop().unwrap().into_value() else {panic!()};
                if args.len() != 1 {
                    panic!("Pointer-type must have exactly one generic argument containing `dyn Bounds`")
                }
                let arg = args.pop().unwrap().into_value();
                let syn::GenericArgument::Type(ty) = arg else {panic!()};
                (
                    DynPtr {
                        ptr: PtrType::Path(quote!(#leading_colon #segments #ident)),
                        lifetime: None,
                        bounds: Vec::new(),
                    },
                    ty,
                )
            }
            syn::Type::Reference(syn::TypeReference {
                lifetime,
                mutability,
                elem,
                ..
            }) => (
                DynPtr {
                    ptr: if mutability.is_some() {
                        PtrType::RefMut
                    } else {
                        PtrType::Ref
                    },
                    lifetime,
                    bounds: Vec::new(),
                },
                *elem,
            ),
            _ => panic!("Only references and paths are supported by this macro"),
        };
        let syn::Type::TraitObject(syn::TypeTraitObject {  bounds, .. }) = elem else {panic!("expected `dyn` not found")};
        for bound in bounds {
            match bound {
                TypeParamBound::Trait(t) => this.bounds.push(t),
                TypeParamBound::Lifetime(lt) => {
                    if this.lifetime.is_some() {
                        panic!("Only a single lifetime is supported in this macro")
                    } else {
                        this.lifetime = Some(lt)
                    }
                }
            }
        }
        Ok(this)
    }
}

#[proc_macro]
pub fn dynptr(tokens: TokenStream) -> TokenStream {
    let st = tl_mod();
    let DynPtr {
        ptr,
        bounds,
        lifetime,
    } = syn::parse(tokens).unwrap();
    let mut vt = quote!(#st::vtable::VtDrop);
    for bound in bounds {
        vt = quote!(< dyn #bound as #st::vtable::CompoundVt >::Vt<#vt>);
    }
    let lifetime = lifetime.unwrap_or(syn::Lifetime::new("'static", Span::call_site()));
    match ptr {
        PtrType::Path(path) => quote!(#st::Dyn<#lifetime, #path<()>, #vt>),
        PtrType::RefMut => quote!(#st::Dyn<#lifetime, &#lifetime mut (), #vt>),
        PtrType::Ref => quote!(#st::DynRef<#lifetime, #vt>),
    }
    .into()
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

mod gen_closures;
#[proc_macro]
pub fn gen_closures_impl(_: TokenStream) -> TokenStream {
    gen_closures::gen_closures().into()
}

pub(crate) fn report(
    fields: &[(String, &syn::Type)],
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let st = crate::tl_mod();
    let mut report_bounds = quote!();
    let mut report = quote!(None);
    for (name, ty) in fields.iter().rev() {
        report_bounds = quote!(#ty: #st::IStable, #report_bounds);
        report = quote! {
            Some(& #st::report::FieldReport {
                name: #st::str::Str::new(#name),
                ty: <#ty as #st::IStable>::REPORT,
                next_field: #st::StableLike::new(#report)
            })
        };
    }
    (report, report_bounds)
}
