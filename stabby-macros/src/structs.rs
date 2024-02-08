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

use proc_macro2::Ident;
use quote::quote;
use syn::{Attribute, DataStruct, Generics, Visibility};

pub fn stabby(
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    generics: Generics,
    DataStruct {
        fields, semi_token, ..
    }: DataStruct,
    stabby_attrs: &proc_macro::TokenStream,
) -> proc_macro2::TokenStream {
    let mut opt = match stabby_attrs.to_string().as_str() {
        "no_opt" => false,
        "" => true,
        _ => panic!("Unkown stabby attributes {stabby_attrs}"),
    };
    opt &= generics.params.is_empty();
    let st = crate::tl_mod();
    let unbound_generics = crate::utils::unbound_generics(&generics.params);
    let generics_without_defaults = crate::utils::generics_without_defaults(&generics.params);
    let where_clause = &generics.where_clause;
    let clauses = where_clause.as_ref().map(|w| &w.predicates);
    let mut layout = None;
    let mut report = Vec::new();
    let struct_code = match &fields {
        syn::Fields::Named(fields) => {
            let fields = &fields.named;
            for field in fields {
                let ty = &field.ty;
                layout = Some(layout.map_or_else(
                    || quote!(#ty),
                    |layout| quote!(#st::FieldPair<#layout, #ty>),
                ));
                report.push((field.ident.as_ref().unwrap().to_string(), ty));
            }
            quote! {
                #(#attrs)*
                #[repr(C)]
                #vis struct #ident #generics #where_clause {
                    #fields
                }
            }
        }
        syn::Fields::Unnamed(fields) => {
            let fields = &fields.unnamed;
            for (i, field) in fields.iter().enumerate() {
                let ty = &field.ty;
                layout = Some(layout.map_or_else(
                    || quote!(#ty),
                    |layout| quote!(#st::FieldPair<#layout, #ty>),
                ));
                report.push((i.to_string(), ty));
            }
            quote! {
                #(#attrs)*
                #[repr(C)]
                #vis struct #ident #generics #where_clause (#fields);
            }
        }
        syn::Fields::Unit => {
            quote! {
                #(#attrs)*
                #[repr(C)]
                #vis struct #ident #generics #where_clause;
            }
        }
    };
    let layout = layout.map_or_else(|| quote!(()), |layout| quote!(#st::Struct<#layout>));
    let opt_id = quote::format_ident!("OptimizedLayoutFor{ident}");
    let assertion = opt.then(|| {
        let sub_optimal_message = format!(
            "{ident}'s layout is sub-optimal, reorder fields or use `#[stabby::stabby(no_opt)]`"
        );
        let size_bug = format!(
            "{ident}'s size was mis-evaluated by stabby, this is a definitely a bug and may cause UB, please fill an issue"
        );
        let align_bug = format!(
            "{ident}'s align was mis-evaluated by stabby, this is a definitely a bug and may cause UB, please fill an issue"
        );
        quote! {
            const _: () = {
                if !<#ident>::has_optimal_layout() {
                    panic!(#sub_optimal_message)
                }
                if core::mem::size_of::<#ident>() != <<#ident as #st::IStable>::Size as #st::Unsigned>::USIZE {
                    panic!(#size_bug)
                }
                if core::mem::align_of::<#ident>() != <<#ident as #st::IStable>::Align as #st::Unsigned>::USIZE {
                    panic!(#align_bug)
                }
            };
        }
    });
    let (report, report_bounds) = crate::report(&report);
    let sident = format!("{ident}");
    let optdoc = format!("Returns true if the layout for [`{ident}`] is smaller or equal to that Rust would have generated for it.");
    quote! {
        #struct_code

        #[automatically_derived]
        unsafe impl <#generics_without_defaults> #st::IStable for #ident <#unbound_generics> where #layout: #st::IStable, #report_bounds #clauses {
            type ForbiddenValues = <#layout as #st::IStable>::ForbiddenValues;
            type UnusedBits =<#layout as #st::IStable>::UnusedBits;
            type Size = <#layout as #st::IStable>::Size;
            type Align = <#layout as #st::IStable>::Align;
            type HasExactlyOneNiche = <#layout as #st::IStable>::HasExactlyOneNiche;
            type ContainsIndirections = <#layout as #st::IStable>::ContainsIndirections;
            const REPORT: &'static #st::report::TypeReport = & #st::report::TypeReport {
                name: #st::str::Str::new(#sident),
                module: #st::str::Str::new(core::module_path!()),
                fields: unsafe{#st::StableLike::new(#report)},
                version: 0,
                tyty: #st::report::TyTy::Struct,
            };
            const ID: u64 = #st::report::gen_id(Self::REPORT);
        }
        #[allow(dead_code, missing_docs)]
        struct #opt_id #generics #where_clause #fields #semi_token
        #assertion
        impl < #generics_without_defaults > #ident <#unbound_generics> #where_clause {
            #[doc = #optdoc]
            pub const fn has_optimal_layout() -> bool {
                core::mem::size_of::<Self>() <= core::mem::size_of::<#opt_id<#unbound_generics>>()
            }
        }
    }
}
