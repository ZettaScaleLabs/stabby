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
    for field in &fields.named {
        let ty = &field.ty;
        layout = quote!(#st::Union<#layout, #ty>)
    }
    quote! {
        #(#attrs)*
        #[repr(C)]
        #vis union #ident #generics
            #fields

        #[automatically_derived]
        unsafe impl #generics #st::IStable for #ident <#unbound_generics> where #layout: #st::IStable {
            type ForbiddenValues = #st::End;
            type UnusedBits = #st::End;
            type Size = <#layout as #st::IStable>::Size;
            type Align = <#layout as #st::IStable>::Align;
            type HasExactlyOneNiche = #st::B0;
        }
    }
}
