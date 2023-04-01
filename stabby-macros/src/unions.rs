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

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, DataUnion, Generics, Ident, Visibility};
pub fn stabby(
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    generics: Generics,
    data: DataUnion,
) -> TokenStream {
    let st = crate::tl_mod();
    let DataUnion {
        union_token: _,
        fields,
    } = &data;
    let unbound_generics = &generics.params;
    let mut layout = quote!(());
    let mut report = Vec::new();
    for field in &fields.named {
        let ty = &field.ty;
        layout = quote!(#st::Union<#layout, #ty>);
        report.push((field.ident.as_ref().unwrap().to_string(), ty));
    }
    let sident = format!("{ident}");
    let (report, report_bounds) = crate::report(&report);
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
            const REPORT: &'static #st::report::TypeReport = & #st::report::TypeReport {
                name: #st::str::Str::new(#sident),
                module: #st::str::Str::new(core::module_path!()),
                fields: unsafe{#st::StableLike::new(#report)},
                last_break: #st::report::Version::NEVER,
                tyty: #st::report::TyTy::Struct,
            };
        }
    }
}
