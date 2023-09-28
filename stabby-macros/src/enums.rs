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

use std::ops::Deref;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, DataEnum, Generics, Ident, Visibility};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Repr {
    Stabby,
    C,
    U8,
    U16,
    U32,
    U64,
    Usize,
    I8,
    I16,
    I32,
    I64,
    Isize,
}
impl syn::parse::Parse for Repr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let id: syn::Ident = input.parse()?;
        match id.to_string().as_str() {
            "C" => Ok(Self::C),
            "u8" => Ok(Self::U8),
            "u16" => Ok(Self::U16),
            "u32" => Ok(Self::U32),
            "u64" => Ok(Self::U64),
            "usize" => Ok(Self::Usize),
            "i8" => Ok(Self::I8),
            "i16" => Ok(Self::I16),
            "i32" => Ok(Self::I32),
            "i64" => Ok(Self::I64),
            "isize" => Ok(Self::Isize),
            "stabby" => Ok(Self::Stabby),
            _ => Err(input.error("Unexpected repr, only `u*` and `stabby` are supported")),
        }
    }
}

pub fn stabby(
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    generics: Generics,
    data: DataEnum,
) -> TokenStream {
    let st = crate::tl_mod();
    let unbound_generics = &generics.params;
    let mut repr = None;
    let repr_ident = quote::format_ident!("repr");
    let mut new_attrs = Vec::with_capacity(attrs.len());
    for a in attrs {
        if a.path.is_ident(&repr_ident) {
            if repr.is_none() {
                repr = Some(a.parse_args().unwrap())
            } else {
                panic!("multiple reprs are forbidden")
            }
        } else {
            new_attrs.push(a)
        }
    }
    if data.variants.is_empty() {
        todo!("empty enums are not supported by stabby YET")
    }
    let mut layout = quote!(());
    let DataEnum { variants, .. } = &data;
    let mut has_non_empty_fields = false;
    let unit = syn::parse2(quote!(())).unwrap();
    let mut report = Vec::new();
    for variant in variants {
        match &variant.fields {
            syn::Fields::Named(_) => {
                panic!("stabby does not support named fields in enum variants")
            }
            syn::Fields::Unnamed(f) => {
                assert_eq!(
                    f.unnamed.len(),
                    1,
                    "stabby only supports one field per enum variant"
                );
                has_non_empty_fields = true;
                let f = f.unnamed.first().unwrap();
                let ty = &f.ty;
                layout = quote!(#st::Union<#layout, core::mem::ManuallyDrop<#ty>>);
                report.push((variant.ident.to_string(), ty));
            }
            syn::Fields::Unit => {
                report.push((variant.ident.to_string(), &unit));
            }
        }
    }
    let report = crate::report(&report);
    let repr = repr.unwrap_or(Repr::Stabby);
    let repr = match repr {
        Repr::Stabby => {
            if !has_non_empty_fields {
                panic!("Your enum doesn't have any field with values: use #[repr(C)] or #[repr(u*)] instead")
            }
            return repr_stabby(&new_attrs, &vis, &ident, &generics, data, report);
        }
        Repr::C => "u8",
        Repr::U8 => "u8",
        Repr::U16 => "u16",
        Repr::U32 => "u32",
        Repr::U64 => "u64",
        Repr::Usize => "usize",
        Repr::I8 => "i8",
        Repr::I16 => "i16",
        Repr::I32 => "i32",
        Repr::I64 => "i64",
        Repr::Isize => "isize",
    };
    let reprid = quote::format_ident!("{}", repr);
    layout = quote!(#st::Tuple<#reprid, #layout>);
    let sident = format!("{ident}");
    let (report, report_bounds) = report;
    quote! {
        #(#new_attrs)*
        #[repr(#reprid)]
        #vis enum #ident #generics {
            #variants
        }

        #[automatically_derived]
        unsafe impl #generics #st::IStable for #ident <#unbound_generics> where #report_bounds #layout: #st::IStable {
            type ForbiddenValues = <#layout as #st::IStable>::ForbiddenValues;
            type UnusedBits =<#layout as #st::IStable>::UnusedBits;
            type Size = <#layout as #st::IStable>::Size;
            type Align = <#layout as #st::IStable>::Align;
            type HasExactlyOneNiche = #st::B0;
            const REPORT: &'static #st::report::TypeReport = & #st::report::TypeReport {
                name: #st::str::Str::new(#sident),
                module: #st::str::Str::new(core::module_path!()),
                fields: unsafe {#st::StableLike::new(#report)},
                last_break: #st::report::Version::NEVER,
                tyty: #st::report::TyTy::Enum(#st::str::Str::new(#repr)),
            };
        }
    }
}

struct Variant {
    ident: Ident,
    field: Option<syn::Field>,
    attrs: Vec<Attribute>,
}
impl From<syn::Variant> for Variant {
    fn from(value: syn::Variant) -> Self {
        let syn::Variant {
            ident,
            fields,
            discriminant: None,
            attrs,
            ..
        } = value
        else {
            panic!("#[repr(stabby)] enums do not support explicit discriminants")
        };
        let field = match fields {
            syn::Fields::Unit => None,
            syn::Fields::Unnamed(mut f) => {
                let field = f.unnamed.pop().map(|f| f.into_value());
                assert!(f.unnamed.is_empty());
                field
            }
            syn::Fields::Named(_) => unreachable!(),
        };
        Variant {
            ident,
            field,
            attrs,
        }
    }
}
struct Variants {
    variants: Vec<Variant>,
}
impl Deref for Variants {
    type Target = Vec<Variant>;
    fn deref(&self) -> &Self::Target {
        &self.variants
    }
}
impl FromIterator<syn::Variant> for Variants {
    fn from_iter<T: IntoIterator<Item = syn::Variant>>(iter: T) -> Self {
        Self {
            variants: Vec::from_iter(iter.into_iter().map(Into::into)),
        }
    }
}
impl Variants {
    fn recursion<'a, U, LeafFn: FnMut(&'a Variant) -> U, JoinFn: FnMut(U, U) -> U>(
        variants: &'a [Variant],
        leaf: &mut LeafFn,
        join: &mut JoinFn,
    ) -> U {
        if variants.len() > 1 {
            let (left, right) = variants.split_at(variants.len() / 2);
            let left = Self::recursion(left, leaf, join);
            let right = Self::recursion(right, leaf, join);
            join(left, right)
        } else {
            leaf(&variants[0])
        }
    }
    fn map<'a, U, LeafFn: FnMut(&'a Variant) -> U, JoinFn: FnMut(U, U) -> U>(
        &'a self,
        mut leaf: LeafFn,
        mut join: JoinFn,
    ) -> U {
        Self::recursion(&self.variants, &mut leaf, &mut join)
    }
    fn map_with_finalizer<
        'a,
        U,
        V,
        LeafFn: FnMut(&'a Variant) -> U,
        JoinFn: FnMut(U, U) -> U,
        FinalJoinFn: FnOnce(U, U) -> V,
    >(
        &'a self,
        mut leaf: LeafFn,
        mut join: JoinFn,
        final_join: FinalJoinFn,
    ) -> V {
        let (left, right) = self.variants.split_at(self.variants.len() / 2);
        let left = Self::recursion(left, &mut leaf, &mut join);
        let right = Self::recursion(right, &mut leaf, &mut join);
        final_join(left, right)
    }
}

pub fn repr_stabby(
    attrs: &Vec<Attribute>,
    vis: &Visibility,
    ident: &Ident,
    generics: &Generics,
    data: DataEnum,
    report: (TokenStream, TokenStream),
) -> TokenStream {
    let st = crate::tl_mod();
    let unbound_generics = crate::utils::unbound_generics(&generics.params);
    let variants = data.variants;
    if variants.len() < 2 {
        panic!("#[repr(stabby)] doesn't support single-member enums");
    }
    let variants = variants.into_iter().collect::<Variants>();
    let vty = variants
        .iter()
        .map(|v| v.field.as_ref().map(|f| &f.ty))
        .collect::<Vec<_>>();
    let vtyref = vty
        .iter()
        .map(|v| v.map(|ty| quote!(&'st_lt #ty)))
        .collect::<Vec<_>>();
    let vtymut = vty
        .iter()
        .map(|v| v.map(|ty| quote!(&'st_lt mut #ty)))
        .collect::<Vec<_>>();
    let vid = variants.iter().map(|v| &v.ident).collect::<Vec<_>>();
    let vattrs = variants.iter().map(|v| &v.attrs);
    let fnvid = vid
        .iter()
        .map(|i| quote::format_ident!("{i}Fn"))
        .collect::<Vec<_>>();
    let (result, bounds) = variants.map(
        |Variant { field, .. }| match field.as_ref() {
            Some(syn::Field { ty, .. }) => (quote!(#ty), quote!()),
            None => (quote!(()), quote!()),
        },
        |(aty, abound), (bty, bbound)| {
            (
                quote!(#st::Result<#aty, #bty>),
                quote!(#aty: #st::IDiscriminantProvider<#bty>, #bty: #st::IStable, #abound #bbound),
            )
        },
    );
    let mut cparams = Vec::new();
    let constructors = variants.map(
        |v| {
            let ovid = match &v.field {
                Some(syn::Field { ty, .. }) => {
                    cparams.push(quote!(value: #ty));
                    quote!(value)
                }
                None => {
                    cparams.push(quote!());
                    quote!(())
                }
            };
            vec![ovid]
        },
        |a, b| {
            let mut r = Vec::with_capacity(a.len() + b.len());
            for v in a {
                r.push(quote!(#st::Result::Ok(#v)))
            }
            for v in b {
                r.push(quote!(#st::Result::Err(#v)))
            }
            r
        },
    );
    let matcher = |matcher| {
        variants.map_with_finalizer(
            |Variant { ident, field, .. }| match field {
                Some(_) => quote!(#ident),
                None => quote!(|_| #ident()),
            },
            |a, b| quote!(move |this| this.#matcher(#a, #b)),
            |a, b| quote!(self.0.#matcher(#a, #b)),
        )
    };
    let matcher_ctx = |matcher| {
        variants.map_with_finalizer(
            |Variant { ident, field, .. }| match field {
                Some(_) => quote!(#ident),
                None => quote!(|stabby_ctx, _| #ident(stabby_ctx)),
            },
            |a, b| quote!(move |stabby_ctx, this| this.#matcher(stabby_ctx, #a, #b)),
            |a, b| quote!(self.0.#matcher(stabby_ctx, #a, #b)),
        )
    };
    let owned_matcher = matcher(quote!(match_owned));
    let ref_matcher = matcher(quote!(match_ref));
    let mut_matcher = matcher(quote!(match_mut));
    let owned_matcher_ctx = matcher_ctx(quote!(match_owned_ctx));
    let ref_matcher_ctx = matcher_ctx(quote!(match_ref_ctx));
    let mut_matcher_ctx = matcher_ctx(quote!(match_mut_ctx));
    let layout = &result;

    let bounds2 = generics.where_clause.as_ref().map(|c| &c.predicates);
    let bounds = quote!(#bounds #bounds2);

    let sident = format!("{ident}");
    let (report, report_bounds) = report;
    let enum_as_struct = quote! {
        #(#attrs)*
        #vis struct #ident #generics (#result) where #report_bounds #bounds;
    };
    quote! {
        #enum_as_struct
        #[automatically_derived]
        unsafe impl #generics #st::IStable for #ident < #unbound_generics > where #report_bounds #bounds #layout: #st::IStable {
            type ForbiddenValues = <#layout as #st::IStable>::ForbiddenValues;
            type UnusedBits =<#layout as #st::IStable>::UnusedBits;
            type Size = <#layout as #st::IStable>::Size;
            type Align = <#layout as #st::IStable>::Align;
            type HasExactlyOneNiche = #st::B0;
            const REPORT: &'static #st::report::TypeReport = & #st::report::TypeReport {
                name: #st::str::Str::new(#sident),
                module: #st::str::Str::new(core::module_path!()),
                fields: unsafe {#st::StableLike::new(#report)},
                last_break: #st::report::Version::NEVER,
                tyty: #st::report::TyTy::Enum(#st::str::Str::new("stabby")),
            };
        }
        #[automatically_derived]
        impl #generics #ident < #unbound_generics > where #report_bounds #bounds {
            #(
                #[allow(non_snake_case)]
                #(#vattrs)*
                pub fn #vid(#cparams) -> Self {
                    Self (#constructors)
                }
            )*
            #[allow(non_snake_case)]
            /// Equivalent to `match self`.
            pub fn match_owned<StabbyOut, #(#fnvid: FnOnce(#vty) -> StabbyOut,)*>(self, #(#vid: #fnvid,)*) -> StabbyOut {
                #owned_matcher
            }
            #[allow(non_snake_case)]
            /// Equivalent to `match &self`.
            pub fn match_ref<'st_lt, StabbyOut, #(#fnvid: FnOnce(#vtyref) -> StabbyOut,)*>(&'st_lt self, #(#vid: #fnvid,)*) -> StabbyOut {
                #ref_matcher
            }
            #[allow(non_snake_case)]
            /// Equivalent to `match &mut self`.
            pub fn match_mut<'st_lt, StabbyOut, #(#fnvid: FnOnce(#vtymut) -> StabbyOut,)*>(&'st_lt mut self, #(#vid: #fnvid,)*) -> StabbyOut {
                #mut_matcher
            }
            #[allow(non_snake_case)]
            /// Equivalent to `match self`, but allows you to pass common arguments to all closures to make the borrow checker happy.
            pub fn match_owned_ctx<StabbyOut, StabbyCtx, #(#fnvid: FnOnce(StabbyCtx, #vty) -> StabbyOut,)*>(self, stabby_ctx: StabbyCtx, #(#vid: #fnvid,)*) -> StabbyOut {
                #owned_matcher_ctx
            }
            #[allow(non_snake_case)]
            /// Equivalent to `match &self`, but allows you to pass common arguments to all closures to make the borrow checker happy.
            pub fn match_ref_ctx<'st_lt, StabbyCtx, StabbyOut, #(#fnvid: FnOnce(StabbyCtx, #vtyref) -> StabbyOut,)*>(&'st_lt self, stabby_ctx: StabbyCtx, #(#vid: #fnvid,)*) -> StabbyOut {
                #ref_matcher_ctx
            }
            #[allow(non_snake_case)]
            /// Equivalent to `match &mut self`, but allows you to pass common arguments to all closures to make the borrow checker happy.
            pub fn match_mut_ctx<'st_lt, StabbyCtx, StabbyOut, #(#fnvid: FnOnce(StabbyCtx, #vtymut) -> StabbyOut,)*>(&'st_lt mut self, stabby_ctx: StabbyCtx, #(#vid: #fnvid,)*) -> StabbyOut {
                #mut_matcher_ctx
            }
        }
    }
}
