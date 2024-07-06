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
use syn::{spanned::Spanned, Attribute, DataStruct, Generics, Visibility};

use crate::Unself;

struct Args {
    optimize: bool,
    version: u32,
    module: proc_macro2::TokenStream,
}
impl syn::parse::Parse for Args {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut this = Args {
            optimize: true,
            version: 0,
            module: quote!(),
        };
        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            match ident.to_string().as_str() {
                "no_opt" => this.optimize = false,
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
#[derive(Copy, Clone)]
enum AllowedRepr {
    C,
    Transparent,
    Align(usize),
}
impl syn::parse::Parse for AllowedRepr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut content = input.fork();
        if input.peek(syn::token::Paren) {
            syn::parenthesized!(content in input);
        }
        let ident: Ident = content.parse()?;
        Ok(match ident.to_string().as_str() {
            "C" => AllowedRepr::C,
            "transparent" => AllowedRepr::Transparent,
            "packed" => return Err(input.error("stabby does not support packed structs, though you may implement IStable manually if you're very comfident")),
            "align" => {
                let input = content;
                syn::parenthesized!(content in input);
                let lit: syn::LitInt = content.parse()?;
                AllowedRepr::Align(lit.base10_parse()?)
            }
            _ => {
                return Err(input.error(
                    "Only #[repr(C)] and #[repr(transparent)] are allowed for stabby structs",
                ))
            }
        })
    }
}
impl quote::ToTokens for AllowedRepr {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(match self {
            AllowedRepr::C => quote!(#[repr(C)]),
            AllowedRepr::Transparent => quote!(#[repr(transparent)]),
            AllowedRepr::Align(n) => {
                let n = syn::LitInt::new(&format!("{n}"), tokens.span());
                quote!(#[repr(align(#n))])
            }
        })
    }
}

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
    let Args {
        mut optimize,
        version,
        module,
    } = syn::parse(stabby_attrs.clone()).unwrap();
    optimize &= generics.params.is_empty();
    let st = crate::tl_mod();
    let unbound_generics = crate::utils::unbound_generics(&generics.params);
    let generics_without_defaults = crate::utils::generics_without_defaults(&generics.params);
    let where_clause = &generics.where_clause;
    let clauses = where_clause.as_ref().map(|w| &w.predicates);
    let mut layout = None;
    let mut report = crate::Report::r#struct(ident.to_string(), version, module);
    let repr = attrs.iter().find_map(|attr| {
        if attr.path.is_ident("repr") {
            syn::parse2::<AllowedRepr>(attr.tokens.clone()).ok()
        } else {
            None
        }
    });
    let repr_attr = repr.is_none().then(|| quote! {#[repr(C)]});
    optimize &= !matches!(repr, Some(AllowedRepr::Align(_)));
    let struct_code = match &fields {
        syn::Fields::Named(fields) => {
            let fields = &fields.named;
            for field in fields {
                let ty = field.ty.unself(&ident);
                layout = Some(layout.map_or_else(
                    || quote!(#ty),
                    |layout| quote!(#st::FieldPair<#layout, #ty>),
                ));
                report.add_field(field.ident.as_ref().unwrap().to_string(), ty);
            }
            quote! {
                #(#attrs)*
                #repr_attr
                #vis struct #ident #generics #where_clause {
                    #fields
                }
            }
        }
        syn::Fields::Unnamed(fields) => {
            let fields = &fields.unnamed;
            for (i, field) in fields.iter().enumerate() {
                let ty = field.ty.unself(&ident);
                layout = Some(layout.map_or_else(
                    || quote!(#ty),
                    |layout| quote!(#st::FieldPair<#layout, #ty>),
                ));
                report.add_field(i.to_string(), ty);
            }
            quote! {
                #(#attrs)*
                #repr_attr
                #vis struct #ident #generics #where_clause (#fields);
            }
        }
        syn::Fields::Unit => {
            quote! {
                #(#attrs)*
                #repr_attr
                #vis struct #ident #generics #where_clause;
            }
        }
    };
    let layout = layout.map_or_else(
        || quote!(()),
        |layout| {
            if let Some(AllowedRepr::Align(mut n)) = repr {
                let mut align = quote!(#st::U1);
                while n > 1 {
                    n /= 2;
                    align = quote!(#st::UInt<#align, #st::B0>);
                }
                quote!(#st::AlignedStruct<#layout, #align>)
            } else {
                quote!(#st::Struct<#layout>)
            }
        },
    );
    let opt_id = quote::format_ident!("OptimizedLayoutFor{ident}");
    let size_bug = format!(
        "{ident}'s size was mis-evaluated by stabby, this is definitely a bug and may cause UB, please file an issue"
    );
    let align_bug = format!(
        "{ident}'s align was mis-evaluated by stabby, this is definitely a bug and may cause UB, please file an issue"
    );
    let reprc_bug = format!(
        "{ident}'s CType was mis-evaluated by stabby, this is definitely a bug and may cause UB, please file an issue"
    );
    let assertion = optimize.then(|| {
        let sub_optimal_message = format!(
            "{ident}'s layout is sub-optimal, reorder fields or use `#[stabby::stabby(no_opt)]`"
        );
        quote! {
            const _: () = {
                if !<#ident>::has_optimal_layout() {
                    panic!(#sub_optimal_message)
                }
            };
        }
    });
    let report_bounds = report.bounds();
    let ctype = cfg!(feature = "experimental-ctypes").then(|| {
        let ctype = report.crepr();
        quote! {type CType = #ctype;}
    });
    let ctype_assert = cfg!(feature = "experimental-ctypes").then(|| {
        quote! {if core::mem::size_of::<Self>() != core::mem::size_of::<<Self as #st::IStable>::CType>() || core::mem::align_of::<Self>() != core::mem::align_of::<<Self as #st::IStable>::CType>() {
            panic!(#reprc_bug)
        }}
    });
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
            #ctype
            const REPORT: &'static #st::report::TypeReport = &#report;
            const ID: u64 = {
                #ctype_assert
                if core::mem::size_of::<Self>() != <<Self as #st::IStable>::Size as #st::Unsigned>::USIZE {
                    panic!(#size_bug)
                }
                if core::mem::align_of::<Self>() != <<Self as #st::IStable>::Align as #st::Unsigned>::USIZE {
                    panic!(#align_bug)
                }
                #st::report::gen_id(Self::REPORT)
            };
        }
        #[allow(dead_code, missing_docs)]
        struct #opt_id #generics #where_clause #fields #semi_token
        #assertion
        impl < #generics_without_defaults > #ident <#unbound_generics> where #layout: #st::IStable, #report_bounds #clauses {
            #[doc = #optdoc]
            pub const fn has_optimal_layout() -> bool {
                core::mem::size_of::<Self>() <= core::mem::size_of::<#opt_id<#unbound_generics>>()
            }
        }
    }
}
