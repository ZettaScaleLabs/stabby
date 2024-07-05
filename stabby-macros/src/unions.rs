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

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, DataUnion, Generics, Ident, Visibility};

use crate::Unself;

struct Args {
    version: u32,
    module: proc_macro2::TokenStream,
}
impl syn::parse::Parse for Args {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut this = Args {
            version: 0,
            module: quote!(),
        };
        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            match ident.to_string().as_str() {
                "version" => {
                    input.parse::<syn::Token!(=)>()?;
                    this.version = input.parse::<syn::LitInt>()?.to_string().parse().unwrap();
                }
                "module" => {
                    input.parse::<syn::Token!(=)>()?;
                    while !input.is_empty() {
                        if input.peek(syn::Token!(,)) {
                            break;
                        }
                        let token: proc_macro2::TokenTree = input.parse()?;
                        this.module.extend(Some(token))
                    }
                }
                _ => return Err(input.error("Unknown stabby attribute {ident}")),
            }
            _ = input.parse::<syn::Token!(,)>();
        }
        Ok(this)
    }
}
pub fn stabby(
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    generics: Generics,
    data: DataUnion,
    stabby_attrs: &proc_macro::TokenStream,
) -> TokenStream {
    let st = crate::tl_mod();
    let DataUnion {
        union_token: _,
        fields,
    } = &data;
    let Args { version, module } = syn::parse(stabby_attrs.clone()).unwrap();
    let unbound_generics = &generics.params;
    let mut layout = quote!(());
    let mut report = crate::Report::r#union(ident.to_string(), version, module);
    for field in &fields.named {
        let ty = field.ty.unself(&ident);
        layout = quote!(#st::Union<#layout, #ty>);
        report.add_field(field.ident.as_ref().unwrap().to_string(), ty);
    }
    let report_bounds = report.bounds();
    let ctype = cfg!(feature = "ctypes").then(|| {
        quote! {type CType = <#layout as #st::IStable>::CType;}
    });
    quote! {
        #(#attrs)*
        #[repr(C)]
        #vis union #ident #generics
            #fields

        #[automatically_derived]
        unsafe impl #generics #st::IStable for #ident <#unbound_generics> where #report_bounds #layout: #st::IStable {
            type ForbiddenValues = #st::End;
            type UnusedBits = #st::End;
            type Size = <#layout as #st::IStable>::Size;
            type Align = <#layout as #st::IStable>::Align;
            type HasExactlyOneNiche = #st::B0;
            type ContainsIndirections =  <#layout as #st::IStable>::ContainsIndirections;
            #ctype
            const REPORT: &'static #st::report::TypeReport = & #report;
            const ID: u64 = #st::report::gen_id(Self::REPORT);
        }
    }
}
